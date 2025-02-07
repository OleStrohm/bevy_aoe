use std::collections::HashMap;
use std::net::Ipv4Addr;
use std::net::SocketAddr;
use std::process::Child;
use std::time::Duration;

use bevy::prelude::*;
use lightyear::prelude::*;
use lightyear::shared::events::components::InputEvent;

use crate::game::minion::MinionPosition;
use crate::game::minion::MinionTarget;
use crate::game::shared_config;
use crate::game::shared_movement_behaviour;
use crate::game::ClientMessage;
use crate::game::Inputs;
use crate::game::OwnedBy;
use crate::game::PlayerColor;
use crate::game::PlayerId;
use crate::game::PlayerPosition;
use crate::game::KEY;
use crate::game::PROTOCOL_ID;

use self::server::ControlledBy;
use self::server::NetcodeConfig;
use self::server::Replicate;
use self::server::ServerCommands;
use self::server::SyncTarget;
use self::server::{IoConfig, NetConfig, ServerTransport};

pub struct ServerPlugin {
    pub server_port: u16,
    pub clients: std::sync::Arc<std::sync::Mutex<Vec<Child>>>,
}

impl Plugin for ServerPlugin {
    fn build(&self, app: &mut App) {
        let link_conditioner = LinkConditionerConfig {
            incoming_latency: Duration::from_millis(200),
            incoming_jitter: Duration::from_millis(0),
            incoming_loss: 0.0,
        };
        let server_addr = SocketAddr::new(Ipv4Addr::LOCALHOST.into(), self.server_port);
        let io_config = IoConfig::from_transport(ServerTransport::UdpSocket(server_addr))
            .with_conditioner(link_conditioner);
        let netcode_config = NetcodeConfig::default()
            .with_protocol_id(PROTOCOL_ID)
            .with_key(KEY);

        let net_config = NetConfig::Netcode {
            config: netcode_config,
            io: io_config,
        };

        let server_config = server::ServerConfig {
            shared: shared_config(Mode::HostServer),
            net: vec![net_config],
            replication: ReplicationConfig {
                send_interval: Duration::from_millis(40),
                ..default()
            },
            ..default()
        };

        app.add_plugins(server::ServerPlugins::new(server_config));
        let mut clients = std::mem::take(&mut *self.clients.lock().unwrap());
        app.add_systems(Last, move |app_exit: EventReader<AppExit>| {
            if !app_exit.is_empty() {
                for client in &mut clients {
                    client.kill().unwrap();
                }
            }
        });

        app.init_resource::<Global>();
        app.add_systems(Startup, |mut commands: Commands| {
            commands.start_server();
        })
        .add_systems(FixedUpdate, (handle_connections, movement));
    }
}

#[derive(Resource, Default)]
struct Global {
    pub client_id_to_entity_id: HashMap<ClientId, Entity>,
}

fn handle_connections(
    mut commands: Commands,
    mut connections: EventReader<ServerConnectEvent>,
    mut global: ResMut<Global>,
) {
    for connection in connections.read() {
        let client_id = connection.client_id;

        let entity = commands.spawn((
            Name::new(format!("Player - {client_id}")),
            PlayerId(client_id),
            PlayerPosition(Vec2::ZERO),
            PlayerColor(Color::linear_rgb(
                rand::random(),
                rand::random(),
                rand::random(),
            )),
            Replicate {
                sync: SyncTarget {
                    prediction: NetworkTarget::Single(client_id),
                    interpolation: NetworkTarget::AllExceptSingle(client_id),
                },
                controlled_by: ControlledBy {
                    target: NetworkTarget::Single(client_id),
                    ..default()
                },
                ..default()
            },
        ));

        global.client_id_to_entity_id.insert(client_id, entity.id());
    }
}

fn movement(
    mut commands: Commands,
    mut positions: Query<&mut PlayerPosition>,
    mut input_reader: EventReader<InputEvent<Inputs, ClientId>>,
    mut message_reader: EventReader<ServerMessageEvent<ClientMessage>>,
    mut minion_targets: Query<&mut MinionTarget>,
    global: Res<Global>,
    time: Res<Time>,
) {
    for input in input_reader.read() {
        let client_id = *input.context();
        if let Some(input) = input.input() {
            match input {
                Inputs::Direction(dir) => {
                    if let Some(player_entity) = global.client_id_to_entity_id.get(&client_id) {
                        if let Ok(position) = positions.get_mut(*player_entity) {
                            shared_movement_behaviour(position, dir, &time);
                        }
                    }
                }
                &Inputs::Spawn(pos, color) => {
                    commands.spawn((
                        Name::new(format!("Minion - {client_id}")),
                        MinionPosition(pos),
                        MinionTarget(Vec2::new(4.0, 4.0)),
                        PlayerColor(color),
                        Replicate {
                            sync: SyncTarget {
                                prediction: NetworkTarget::Single(client_id),
                                interpolation: NetworkTarget::AllExceptSingle(client_id),
                            },
                            controlled_by: ControlledBy {
                                target: NetworkTarget::Single(client_id),
                                ..default()
                            },
                            ..default()
                        },
                        OwnedBy(client_id),
                        PreSpawnedPlayerObject::default(),
                    ));
                }
                Inputs::None => {}
            }
        }
    }

    for event in message_reader.read() {
        match &event.message {
            ClientMessage::Target(minions, target) => {
                println!("targeting something!");
                for &minion in minions {
                    if let Ok(mut minion_target) = minion_targets.get_mut(minion) {
                        minion_target.0 = *target;
                    }
                }
            }
        }
    }
}
