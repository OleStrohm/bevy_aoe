use std::net::Ipv4Addr;
use std::net::SocketAddr;
use std::str::FromStr;
use std::time::Duration;

use bevy::prelude::*;
use lightyear::client::input::native::InputSystemSet;
use lightyear::prelude::*;
use lightyear::shared::events::components::InputEvent;

use crate::game::shared_config;
use crate::game::shared_movement_behaviour;
use crate::game::PlayerPosition;
use crate::game::KEY;
use crate::game::PROTOCOL_ID;
use crate::game::{Direction, Inputs};

use self::client::ClientCommands;
use self::client::InputManager;
use self::client::Predicted;
use self::client::{Authentication, ClientTransport, IoConfig, NetConfig};

pub struct HostClientPlugin;
//pub enum ClientPlugin {
//    HostClient,
//    NetworkClient {
//        server_port: u16,
//        client_id: u64,
//    }
//}

impl Plugin for HostClientPlugin {
    fn build(&self, app: &mut App) {
        let net_config = NetConfig::Local {
            id: 0,
        };

        let client_config = client::ClientConfig {
            shared: shared_config(Mode::HostServer),
            net: net_config,
            ..default()
        };

        app.add_plugins(client::ClientPlugins::new(client_config));

        app.add_systems(Startup, |mut commands: Commands| {
            commands.connect_client();
        });
        app.add_systems(
            FixedPreUpdate,
            buffer_input.in_set(InputSystemSet::BufferInputs),
        );
        app.add_systems(FixedUpdate, player_movement);
    }
}

pub struct ClientPlugin {
    pub server_port: u16,
    pub client_id: u64,
}

impl Plugin for ClientPlugin {
    fn build(&self, app: &mut App) {
        let link_conditioner = LinkConditionerConfig {
            incoming_latency: Duration::from_millis(200),
            incoming_jitter: Duration::from_millis(0),
            incoming_loss: 0.0,
        };
        let io_config = IoConfig::from_transport(ClientTransport::UdpSocket(
            SocketAddr::from_str("0.0.0.0:0").unwrap(),
        ))
        .with_conditioner(link_conditioner);
        let server_addr = SocketAddr::new(Ipv4Addr::LOCALHOST.into(), self.server_port);
        let auth = Authentication::Manual {
            server_addr,
            client_id: self.client_id,
            private_key: KEY,
            protocol_id: PROTOCOL_ID,
        };

        let net_config = NetConfig::Netcode {
            auth,
            io: io_config,
            config: default(),
        };

        let client_config = client::ClientConfig {
            shared: shared_config(Mode::Separate),
            net: net_config,
            ..default()
        };

        app.add_plugins(client::ClientPlugins::new(client_config));

        app.add_systems(Startup, |mut commands: Commands| {
            commands.connect_client();
        });
        app.add_systems(
            FixedPreUpdate,
            buffer_input.in_set(InputSystemSet::BufferInputs),
        );
        app.add_systems(FixedUpdate, player_movement);
    }
}

fn buffer_input(
    tick_manager: Res<TickManager>,
    mut input_manager: ResMut<InputManager<Inputs>>,
    keypress: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
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

    if mouse.just_pressed(MouseButton::Right) {
        input_manager.add_input(Inputs::Target(Vec2::new(-2.0, -2.0)), tick);
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
