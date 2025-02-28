use std::net::Ipv4Addr;
use std::net::SocketAddr;
use std::str::FromStr;
use std::thread::sleep;
use std::time::Duration;

use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use lightyear::client::input::native::InputSystemSet;
use lightyear::prelude::client::NetClient;
use lightyear::prelude::*;
use lightyear::shared::events::components::InputEvent;

use crate::game::minion::Selected;
use crate::game::{
    minion::{MinionPosition, MinionTarget},
    shared_config, shared_movement_behaviour, Channel1, ClientMessage, Direction, InputHandling,
    Inputs, OwnedBy, PlayerColor, PlayerPosition, KEY, PROTOCOL_ID,
};
use crate::NetworkState;

use self::client::{
    Authentication, ClientCommands, ClientConfig, ClientConnection, ClientTransport, InputManager,
    IoConfig, NetConfig, Predicted,
};

#[derive(Debug, Resource)]
pub struct StartDrag(Vec2);

#[derive(Debug, Resource)]
pub struct SelectedMinions(Vec<Entity>);

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub struct IsClient;

impl ComputedStates for IsClient {
    type SourceStates = NetworkState;

    fn compute(network_state: NetworkState) -> Option<Self> {
        match network_state {
            NetworkState::Host { .. } | NetworkState::Client { .. } => Some(Self),
            _ => None,
        }
    }
}

pub struct ClientPlugin;

impl Plugin for ClientPlugin {
    fn build(&self, app: &mut App) {
        let net_config = NetConfig::Local { id: 0 };
        let client_config = ClientConfig {
            shared: shared_config(Mode::HostServer),
            net: net_config,
            ..default()
        };

        app.add_plugins(client::ClientPlugins::new(client_config));

        app.insert_resource(SelectedMinions(vec![]))
            .add_computed_state::<IsClient>()
            .add_systems(
                FixedPreUpdate,
                (
                    buffer_input.in_set(InputSystemSet::BufferInputs),
                    player_movement.in_set(InputHandling),
                )
                    .chain(),
            )
            .add_systems(OnEnter(IsClient), start_client);
    }
}

fn start_client(
    mut commands: Commands,
    network_state: Res<State<NetworkState>>,
    mut client_config: ResMut<ClientConfig>,
) {
    *client_config = match network_state.get() {
        NetworkState::Host { .. } => {
            let net_config = NetConfig::Local { id: 0 };

            ClientConfig {
                shared: shared_config(Mode::HostServer),
                net: net_config,
                ..default()
            }
        }
        &NetworkState::Client {
            server_addr,
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

            ClientConfig {
                shared: shared_config(Mode::Separate),
                net: net_config,
                ..default()
            }
        }
        _ => return,
    };

    commands.connect_client();
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
    players: Query<&PlayerColor, (With<PlayerPosition>, With<Predicted>, Without<OwnedBy>)>,
    mut my_minions: Query<
        (Entity, &MinionPosition, &mut MinionTarget, &OwnedBy),
        Or<(With<Predicted>, With<PreSpawnedPlayerObject>)>,
    >,
    predicted: Query<&Predicted>,
    currently_selected_minions: Query<(Entity, &OwnedBy), (With<Selected>, With<MinionPosition>)>,
    connection: Res<ClientConnection>,
    mut message_manager: ResMut<ClientConnectionManager>,
) {
    let tick = tick_manager.tick();
    let player_color = players.get_single().copied().map(|c| c.0).ok();

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

    let window = windows.single();
    let (camera, camera_transform) = camera.single();
    if let Some(mouse_pos) = window
        .cursor_position()
        .and_then(|cursor| camera.viewport_to_world_2d(camera_transform, cursor).ok())
    {
        if let Some(player_color) = player_color {
            if keypress.just_pressed(KeyCode::Space) {
                input_manager.add_input(Inputs::Spawn(mouse_pos, player_color), tick);
            }
        }

        if mouse.just_pressed(MouseButton::Right) {
            message_manager
                .send_message::<Channel1, _>(&ClientMessage::Target(
                    selected_minions
                        .0
                        .iter()
                        .filter_map(|&s| predicted.get(s).ok()?.confirmed_entity)
                        .collect(),
                    mouse_pos,
                ))
                .unwrap();
            for &minion in &selected_minions.0 {
                if let Ok((.., mut target, _)) = my_minions.get_mut(minion) {
                    *target = MinionTarget(mouse_pos);
                }
            }
        }

        if mouse.just_pressed(MouseButton::Left) {
            commands.insert_resource(StartDrag(mouse_pos));
        } else if let Some(start_drag) = start_drag {
            let top_left = mouse_pos.min(start_drag.0);
            let size = mouse_pos.max(start_drag.0) - top_left;

            if mouse.pressed(MouseButton::Left) {
                let position = Isometry2d::from_translation(top_left + size / 2.0);
                gizmos.rect_2d(position, size, Color::BLACK);
            } else if mouse.just_released(MouseButton::Left) {
                let selrect = Rect::from_corners(top_left, top_left + size);
                let selected_minions = my_minions
                    .iter()
                    .filter(|&(_, pos, _, owned_by)| {
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
    mut commands: Commands,
    mut position_query: Query<&mut PlayerPosition, With<Predicted>>,
    mut input_reader: EventReader<InputEvent<Inputs>>,
    time: Res<Time<Fixed>>,
    connection: Res<ClientConnection>,
) {
    for input in input_reader.read() {
        if let Some(input) = input.input() {
            match input {
                Inputs::Direction(dir) => {
                    for position in position_query.iter_mut() {
                        shared_movement_behaviour(position, dir, &time);
                    }
                }
                &Inputs::Spawn(pos, color) => {
                    println!("Spawn minion");
                    commands.spawn((
                        Name::new(format!("Minion - {}", connection.id())),
                        MinionPosition(pos),
                        MinionTarget(Vec2::new(4.0, 4.0)),
                        PlayerColor(color),
                        OwnedBy(connection.id()),
                        PreSpawnedPlayerObject::default(),
                    ));
                }
                Inputs::None => (),
            }
        }
    }
}
