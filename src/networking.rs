use bevy::prelude::*;
use std::net::SocketAddr;

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum NetworkState {
    #[default]
    Disconnected,
    Host(SocketAddr),
    #[expect(unused)]
    Server(SocketAddr),
    Client {
        server_addr: SocketAddr,
        client_id: u64,
    },
}

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

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub struct IsServer;

impl ComputedStates for IsServer {
    type SourceStates = NetworkState;

    fn compute(network_state: NetworkState) -> Option<Self> {
        match network_state {
            NetworkState::Host { .. } | NetworkState::Server { .. } => Some(Self),
            _ => None,
        }
    }
}
