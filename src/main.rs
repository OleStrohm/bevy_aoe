#![allow(clippy::type_complexity)]

use std::fmt::Display;
use std::process::{Child, Stdio};

use bevy::prelude::*;
use bevy::window::WindowResolution;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use owo_colors::OwoColorize;

use game::GamePlugin;

use self::client::ClientPlugin;
use self::server::ServerPlugin;

mod client;
mod game;
mod server;

fn main() {
    match std::env::args().nth(1).as_deref() {
        Some("client") => client(
            std::env::args()
                .nth(2)
                .expect("Client needs a second argument")
                .parse::<i32>()
                .expect("Second argument must be a number"),
        ),
        Some("server") => {
            server(vec![]);
        }
        Some("host") | None => {
            let client1 = start_client(1, "[C1]".green());
            let client2 = start_client(2, "[C2]".yellow());

            server(vec![client1, client2]);
        }
        _ => panic!("The first argument is nonsensical"),
    }
}

fn start_client(index: usize, prefix: impl Display) -> std::process::Child {
    let mut child = std::process::Command::new(std::env::args().next().unwrap())
        .args(["client", &format!("{index}")])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();

    let prefix = format!("{{ print \"{} \" $0}}", prefix);

    #[expect(clippy::zombie_processes)]
    std::process::Command::new("awk")
        .arg(prefix.clone())
        .stdin(child.stdout.take().unwrap())
        .spawn()
        .unwrap();
    #[expect(clippy::zombie_processes)]
    std::process::Command::new("awk")
        .arg(prefix)
        .stdin(child.stderr.take().unwrap())
        .spawn()
        .unwrap();

    child
}

pub fn server(clients: Vec<Child>) {
    println!("Starting server!");

    let monitor_width = 2560.0;
    let monitor_height = 1440.0;
    let window_width = monitor_width / 2.0;
    let window_height = monitor_height / 2.0;
    let position = WindowPosition::At(IVec2::new(
        (monitor_width - window_width) as i32 / 2,
        (monitor_height - window_height) as i32 / 2,
    ));
    let resolution =
        WindowResolution::new(window_width, window_height).with_scale_factor_override(1.0);

    App::new()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Bevy AoE".to_string(),
                    position,
                    resolution: resolution.clone(),
                    resizable: false,
                    decorations: false,
                    focused: true,
                    ..default()
                }),
                ..default()
            }),
            //.disable::<AudioPlugin>(/* Disabled due to audio bug with pipewire */),
            //WorldInspectorPlugin::default(),
            ServerPlugin {
                server_port: 5000,
                clients: std::sync::Arc::new(std::sync::Mutex::new(clients)),
            },
            ClientPlugin::HostClient,
            GamePlugin,
        ))
        .add_systems(
            Update,
            move |mut windows: Query<&mut Window>, time: Res<Time>| {
                if time.elapsed_secs_f64() < 1.0 {
                    for mut window in &mut windows {
                        window.position = position;
                        window.resolution = resolution.clone();
                        window.focused = true;
                    }
                }
            },
        )
        .run();
}

pub fn client(index: i32) {
    println!("Starting client!");

    let monitor_width = 2560.0;
    let monitor_height = 1440.0;
    let window_width = monitor_width / 4.0;
    let window_height = monitor_height / 4.0;
    let position = WindowPosition::At(
        (
            monitor_width as i32 / 2 - window_width as i32 * (index - 1),
            0,
        )
            .into(),
    );
    let resolution =
        WindowResolution::new(window_width, window_height).with_scale_factor_override(1.0);

    App::new()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Bevy AoE - client".to_string(),
                    position,
                    resolution: resolution.clone(),
                    resizable: false,
                    decorations: false,
                    focused: false,
                    ..default()
                }),
                ..default()
            }),
            //.disable::<AudioPlugin>(/* Disabled due to audio bug with pipewire */),
            WorldInspectorPlugin::default(),
            ClientPlugin::NetworkClient {
                server_port: 5000,
                client_id: index as u64,
            },
            GamePlugin,
        ))
        .add_systems(
            Update,
            move |mut windows: Query<&mut Window>, time: Res<Time>| {
                if time.elapsed_secs_f64() < 1.0 {
                    for mut window in &mut windows {
                        window.position = position;
                        window.resolution = resolution.clone();
                    }
                }
            },
        )
        .run();
}
