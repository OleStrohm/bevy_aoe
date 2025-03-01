use bevy::prelude::*;
use bevy_egui::EguiContexts;
use bevy_egui::egui::{Align2, Style, TextEdit};
use std::net::SocketAddr;
use steamworks::{FriendFlags, FriendGame, LobbyId};

use crate::SteamClient;

pub fn show_networking_menu(
    mut locals: Local<Option<String>>,
    mut contexts: EguiContexts,
    steam_client: Res<SteamClient>,
    mut next_network_state: ResMut<NextState<NetworkState>>,
) {
    let addr = locals.get_or_insert("127.0.0.1:5000".into());
    bevy_egui::egui::Window::new("Network menu")
        .anchor(Align2::CENTER_CENTER, (0.0, 0.0))
        .default_size((400.0, 300.0))
        .resizable(false)
        .collapsible(false)
        .show(contexts.ctx_mut(), |ui| {
            let style = Style {
                override_text_style: Some(bevy_egui::egui::TextStyle::Heading),
                ..default()
            };
            ui.set_style(style);

            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        ui.label("Address");
                        ui.add_sized((150.0, 20.0), TextEdit::singleline(addr));
                    });

                    ui.horizontal(|ui| match addr.parse::<SocketAddr>() {
                        Ok(addr) => {
                            if ui.button("Host").clicked() {
                                next_network_state.set(NetworkState::Host(addr));
                            }
                            if ui.button("Connect").clicked() {
                                next_network_state.set(NetworkState::Client {
                                    server_addr: addr,
                                    client_id: rand::random(),
                                });
                            }
                        }
                        Err(_) => {
                            ui.label("Invalid IP address");
                        }
                    });
                });

                ui.separator();

                ui.vertical(|ui| {
                    ui.label("Steam friends");
                    for friend in steam_client
                        .read()
                        .get_client()
                        .friends()
                        .get_friends(FriendFlags::all())
                    {
                        if let Some(game_info) = friend.game_played() {
                            if game_info.game.app_id().0 == 480
                                && ui.button(friend.name()).clicked()
                            {
                                next_network_state.set(NetworkState::ClientSteam {
                                    server_addr: SocketAddr::new(
                                        game_info.game_address.into(),
                                        game_info.game_port,
                                    ),
                                });
                            }
                        }
                    }
                });
            });
        });
}

#[derive(Debug, Clone, Default, Eq, PartialEq, Hash, States)]
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
    ClientSteam {
        server_addr: SocketAddr,
    },
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub struct IsClient;

impl ComputedStates for IsClient {
    type SourceStates = NetworkState;

    fn compute(network_state: NetworkState) -> Option<Self> {
        use NetworkState::*;
        match network_state {
            Host { .. } | Client { .. } | ClientSteam { .. } => Some(Self),
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
