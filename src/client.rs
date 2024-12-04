use std::net::Ipv4Addr;
use std::net::SocketAddr;
use std::str::FromStr;
use std::time::Duration;

use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use lightyear::client::input::native::InputSystemSet;
use lightyear::packet::message_manager::MessageManager;
use lightyear::prelude::client::NetClient;
use lightyear::prelude::*;
use lightyear::shared::events::components::InputEvent;

use crate::game::minion::MinionPosition;
use crate::game::shared_config;
use crate::game::shared_movement_behaviour;
use crate::game::Channel1;
use crate::game::ClientMessage;
use crate::game::OwnedBy;
use crate::game::PlayerPosition;
use crate::game::KEY;
use crate::game::PROTOCOL_ID;
use crate::game::{Direction, Inputs};

use self::client::ClientCommands;
use self::client::ClientConfig;
use self::client::ClientConnection;
use self::client::InputManager;
use self::client::Interpolated;
use self::client::Predicted;
use self::client::{Authentication, ClientTransport, IoConfig, NetConfig};

#[derive(Debug, Resource)]
pub struct StartDrag(Vec2);

#[derive(Debug, Resource)]
pub struct SelectedMinions(Vec<Entity>);

#[derive(Debug, Component)]
pub struct Selected;

pub enum ClientPlugin {
    HostClient,
    NetworkClient { server_port: u16, client_id: u64 },
}

impl Plugin for ClientPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(client::ClientPlugins::new(self.client_config()));

        app.insert_resource(SelectedMinions(vec![]))
            .add_systems(Startup, |mut commands: Commands| {
                commands.connect_client();
            })
            .add_systems(
                FixedPreUpdate,
                (
                    buffer_input.in_set(InputSystemSet::BufferInputs),
                    player_movement,
                ),
            );
    }
}

#[allow(clippy::too_many_arguments)]
fn buffer_input(
    mut commands: Commands,
    tick_manager: Res<TickManager>,
    mut input_manager: ResMut<InputManager<Inputs>>,
    keypress: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    camera: Query<(&Camera, &GlobalTransform)>,
    selected_minions: Res<SelectedMinions>,
    start_drag: Option<Res<StartDrag>>,
    mut gizmos: Gizmos,
    my_minions: Query<(Entity, &MinionPosition, &OwnedBy), With<Interpolated>>,
    interpolated: Query<&Interpolated>,
    currently_selected_minions: Query<(Entity, &OwnedBy), (With<Selected>, With<MinionPosition>)>,
    connection: Res<ClientConnection>,
    mut message_manager: ResMut<ClientConnectionManager>,
) {
    let tick = tick_manager.tick();
    let mut input = Inputs::None;
    let direction = Direction {
        up: keypress.pressed(KeyCode::KeyW),
        down: keypress.pressed(KeyCode::KeyS),
        left: keypress.pressed(KeyCode::KeyA),
        right: keypress.pressed(KeyCode::KeyD),
    };

    if direction.up || direction.down || direction.left || direction.right {
        input = Inputs::Direction(direction);
    }

    input_manager.add_input(input, tick);

    if keypress.just_pressed(KeyCode::Space) {
        input_manager.add_input(Inputs::Spawn, tick);
    }

    let window = windows.single();
    let (camera, camera_transform) = camera.single();
    if let Some(mouse_pos) = window
        .cursor_position()
        .and_then(|cursor| camera.viewport_to_world_2d(camera_transform, cursor))
    {
        if mouse.just_pressed(MouseButton::Right) {
            message_manager
                .send_message::<Channel1, _>(&mut ClientMessage::Target(
                    selected_minions
                        .0
                        .iter()
                        .filter_map(|&s| Some(interpolated.get(s).ok()?.confirmed_entity))
                        .collect(),
                    mouse_pos,
                ))
                .unwrap();
        }

        if mouse.just_pressed(MouseButton::Left) {
            commands.insert_resource(StartDrag(mouse_pos));
        } else if let Some(start_drag) = start_drag {
            let top_left = mouse_pos.min(start_drag.0);
            let size = mouse_pos.max(start_drag.0) - top_left;

            if mouse.pressed(MouseButton::Left) {
                gizmos.rect_2d(top_left + size / 2.0, 0.0, size, Color::BLACK);
            } else if mouse.just_released(MouseButton::Left) {
                let selrect = Rect::from_corners(top_left, top_left + size);
                let selected_minions = my_minions
                    .iter()
                    .filter(|&(_, pos, owned_by)| {
                        selrect.contains(pos.0) && owned_by.0 == connection.id()
                    })
                    .map(|(e, ..)| e)
                    .collect::<Vec<_>>();

                for (minion, _) in &currently_selected_minions {
                    commands.entity(minion).remove::<Selected>();
                }
                for &minion in &selected_minions {
                    commands.entity(minion).insert(Selected);
                }
                println!("Selected {} minions", selected_minions.len());
                commands.insert_resource(SelectedMinions(selected_minions));
            }
        }
    }
}

fn player_movement(
    mut position_query: Query<&mut PlayerPosition, With<Predicted>>,
    mut input_reader: EventReader<InputEvent<Inputs>>,
    time: Res<Time>,
) {
    for input in input_reader.read() {
        if let Some(Inputs::Direction(dir)) = input.input() {
            for position in position_query.iter_mut() {
                shared_movement_behaviour(position, dir, &time);
            }
        }
    }
}

impl ClientPlugin {
    fn client_config(&self) -> ClientConfig {
        match self {
            ClientPlugin::HostClient => {
                let net_config = NetConfig::Local { id: 0 };

                client::ClientConfig {
                    shared: shared_config(Mode::HostServer),
                    net: net_config,
                    ..default()
                }
            }
            &ClientPlugin::NetworkClient {
                server_port,
                client_id,
            } => {
                let link_conditioner = LinkConditionerConfig {
                    incoming_latency: Duration::from_millis(200),
                    incoming_jitter: Duration::from_millis(0),
                    incoming_loss: 0.0,
                };
                let io_config = IoConfig::from_transport(ClientTransport::UdpSocket(
                    SocketAddr::from_str("0.0.0.0:0").unwrap(),
                ))
                .with_conditioner(link_conditioner);
                let server_addr = SocketAddr::new(Ipv4Addr::LOCALHOST.into(), server_port);
                let auth = Authentication::Manual {
                    server_addr,
                    client_id,
                    private_key: KEY,
                    protocol_id: PROTOCOL_ID,
                };

                let net_config = NetConfig::Netcode {
                    auth,
                    io: io_config,
                    config: default(),
                };

                client::ClientConfig {
                    shared: shared_config(Mode::Separate),
                    net: net_config,
                    ..default()
                }
            }
        }
    }
}
