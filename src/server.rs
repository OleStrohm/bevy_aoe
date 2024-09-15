use std::net::Ipv4Addr;
use std::net::SocketAddr;
use std::time::Duration;

use bevy::prelude::*;
use lightyear::prelude::*;

use crate::game::KEY;
use crate::game::PROTOCOL_ID;

use self::server::NetcodeConfig;
use self::server::ServerCommands;
use self::server::{IoConfig, NetConfig, ServerTransport};

pub fn shared_config(mode: Mode) -> SharedConfig {
    SharedConfig {
        server_replication_send_interval: Duration::from_millis(40),
        tick: TickConfig {
            tick_duration: Duration::from_secs_f64(1.0 / 64.0),
        },
        mode,
    }
}

pub struct ServerPlugin {
    pub server_port: u16,
}

impl Plugin for ServerPlugin {
    fn build(&self, app: &mut App) {
        let link_conditioner = LinkConditionerConfig {
            incoming_latency: Duration::from_millis(100),
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

        let client_config = server::ServerConfig {
            shared: shared_config(Mode::Separate),
            net: vec![net_config],
            ..default()
        };

        app.add_plugins(server::ServerPlugins::new(client_config));

        app.init_resource::<Global>();
        app.add_systems(Startup, |mut commands: Commands| {
            commands.start_server();
        })
        .add_systems(FixedUpdate, handle_connections);
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

}
