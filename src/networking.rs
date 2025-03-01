use bevy::prelude::*;
use bevy_egui::EguiContexts;
use bevy_egui::egui::{Align, Align2, Layout, Style};
use std::net::SocketAddr;

pub fn show_networking_menu(
    mut locals: Local<Option<String>>,
    mut contexts: EguiContexts,
    mut next_network_state: ResMut<NextState<NetworkState>>,
) {
    let addr = locals.get_or_insert("127.0.0.1:5000".into());
    bevy_egui::egui::Window::new("Network menu")
        .anchor(Align2::CENTER_CENTER, (0.0, 0.0))
        .resizable(false)
        .collapsible(false)
        .show(contexts.ctx_mut(), |ui| {
            let style = Style {
                override_text_style: Some(bevy_egui::egui::TextStyle::Heading),
                ..default()
            };
            ui.set_style(style);
            ui.set_max_size((ui.available_width(), 100.0).into());

            ui.horizontal(|ui| {
                ui.label("Address");
                ui.text_edit_singleline(addr);
            });

            ui.with_layout(Layout::right_to_left(Align::RIGHT), |ui| {
                match addr.parse::<SocketAddr>() {
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
                    Err(e) => {
                        ui.label(e.to_string());
                    }
                }
            });
        });
}

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
