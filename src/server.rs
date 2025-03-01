use std::net::{Ipv4Addr, SocketAddr};
use std::time::Duration;

use bevy::prelude::*;
use bevy::utils::HashMap;
use lightyear::prelude::server::{
    ControlledBy, IoConfig, NetConfig, NetcodeConfig, Replicate, ServerCommands, ServerConfig,
    ServerTransport, SyncTarget,
};
use lightyear::prelude::*;
use lightyear::shared::events::components::InputEvent;

use crate::game::{
    ClientMessage, InputHandling, KEY, PROTOCOL_ID,
    minion::MinionTarget,
    player::{Inputs, PlayerColor, PlayerId, PlayerPosition, shared_movement_behaviour},
    resource::{Item, ItemPos, Scoreboard},
    shared_config,
};
use crate::networking::{IsServer, NetworkState};

pub struct ServerPlugin;

impl Plugin for ServerPlugin {
    fn build(&self, app: &mut App) {
        let link_conditioner = LinkConditionerConfig {
            incoming_latency: Duration::from_millis(200),
            incoming_jitter: Duration::from_millis(0),
            incoming_loss: 0.0,
        };
        let server_addr = SocketAddr::new(Ipv4Addr::LOCALHOST.into(), 0);
        let io_config = IoConfig::from_transport(ServerTransport::UdpSocket(server_addr))
            .with_conditioner(link_conditioner);
        let netcode_config = NetcodeConfig::default()
            .with_protocol_id(PROTOCOL_ID)
            .with_key(KEY);

        let net_config = NetConfig::Netcode {
            config: netcode_config,
            io: io_config,
        };

        let server_config = ServerConfig {
            shared: shared_config(Mode::HostServer),
            net: vec![net_config],
            replication: ReplicationConfig {
                send_interval: Duration::from_millis(40),
                ..default()
            },
            ..default()
        };

        app.add_plugins(server::ServerPlugins::new(server_config))
            .init_resource::<Global>()
            .add_computed_state::<IsServer>()
            .add_systems(
                FixedUpdate,
                (handle_connections, handle_inputs.in_set(InputHandling)).chain(),
            )
            .add_systems(OnEnter(IsServer), start_server);
    }
}

fn start_server(
    mut commands: Commands,
    network_state: Res<State<NetworkState>>,
    mut server_config: ResMut<ServerConfig>,
) {
    // Start server
    match network_state.get() {
        &NetworkState::Host(addr) | &NetworkState::Server(addr) => {
            let link_conditioner = LinkConditionerConfig {
                incoming_latency: Duration::from_millis(200),
                incoming_jitter: Duration::from_millis(0),
                incoming_loss: 0.0,
            };
            let io_config = IoConfig::from_transport(ServerTransport::UdpSocket(addr))
                .with_conditioner(link_conditioner);
            let netcode_config = NetcodeConfig::default()
                .with_protocol_id(PROTOCOL_ID)
                .with_key(KEY);

            let net_config = NetConfig::Netcode {
                config: netcode_config,
                io: io_config,
            };

            *server_config = ServerConfig {
                shared: shared_config(Mode::HostServer),
                net: vec![net_config],
                replication: ReplicationConfig {
                    send_interval: Duration::from_millis(40),
                    ..default()
                },
                ..default()
            };
        }
        _ => return,
    };
    commands.start_server();

    // Set up game world
    commands.spawn((
        Item::Apple,
        ItemPos(Vec2::new(2.0, 2.0)),
        Replicate::default(),
    ));
    commands.spawn((Scoreboard(HashMap::new()), Replicate::default()));
}

#[derive(Resource, Default)]
struct Global {
    pub client_id_to_entity_id: HashMap<ClientId, Entity>,
}

fn handle_connections(
    mut commands: Commands,
    mut connections: EventReader<ServerConnectEvent>,
    mut global: ResMut<Global>,
    mut scoreboard: Query<&mut Scoreboard>,
) {
    for connection in connections.read() {
        let client_id = connection.client_id;
        scoreboard.single_mut().insert(client_id, 0);

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

fn handle_inputs(
    mut commands: Commands,
    mut positions: Query<&mut PlayerPosition>,
    mut input_reader: EventReader<InputEvent<Inputs, ClientId>>,
    mut message_reader: EventReader<ServerMessageEvent<ClientMessage>>,
    mut minion_targets: Query<&mut MinionTarget>,
    global: Res<Global>,
    time: Res<Time<Fixed>>,
) {
    for input in input_reader.read() {
        let client_id = input.from();
        if let Some(input) = input.input() {
            if let Some(player_entity) = global.client_id_to_entity_id.get(&client_id) {
                if let Ok(mut position) = positions.get_mut(*player_entity) {
                    shared_movement_behaviour(
                        input,
                        &mut commands,
                        &mut position,
                        &time,
                        client_id,
                        (
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
                            PreSpawnedPlayerObject::default(),
                        ),
                    );
                    //&Inputs::Spawn(pos, color) => {
                    //    commands.spawn((
                    //        Name::new(format!("Minion - {client_id}")),
                    //        MinionPosition(pos),
                    //        MinionTarget(Vec2::new(4.0, 4.0)),
                    //        PlayerColor(color),
                    //        Replicate {
                    //            sync: SyncTarget {
                    //                prediction: NetworkTarget::Single(client_id),
                    //                interpolation: NetworkTarget::AllExceptSingle(client_id),
                    //            },
                    //            controlled_by: ControlledBy {
                    //                target: NetworkTarget::Single(client_id),
                    //                ..default()
                    //            },
                    //            ..default()
                    //        },
                    //        OwnedBy(client_id),
                    //        PreSpawnedPlayerObject::default(),
                    //    ));
                    //}
                    //Inputs::None => {}
                }
            }
        }
    }

    for event in message_reader.read() {
        match &event.message {
            ClientMessage::Target(minions, target) => {
                for &minion in minions {
                    if let Ok(mut minion_target) = minion_targets.get_mut(minion) {
                        minion_target.0 = *target;
                    }
                }
            }
        }
    }
}
