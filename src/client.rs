use std::net::Ipv4Addr;
use std::net::SocketAddr;
use std::str::FromStr;
use std::time::Duration;

use bevy::prelude::*;
use lightyear::prelude::*;

use crate::game::KEY;
use crate::game::PROTOCOL_ID;

use self::client::ClientCommands;
use self::client::{Authentication, ClientTransport, IoConfig, NetConfig};

pub fn shared_config(mode: Mode) -> SharedConfig {
    SharedConfig {
        server_replication_send_interval: Duration::from_millis(40),
        tick: TickConfig {
            tick_duration: Duration::from_secs_f64(1.0 / 64.0),
        },
        mode,
    }
}

pub struct ClientPlugin {
    pub server_port: u16,
    pub client_id: u64,
}

impl Plugin for ClientPlugin {
    fn build(&self, app: &mut App) {
        let link_conditioner = LinkConditionerConfig {
            incoming_latency: Duration::from_millis(100),
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
    }
}
