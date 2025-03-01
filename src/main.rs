#![allow(clippy::type_complexity)]

use std::fmt::Display;
use std::net::{Ipv4Addr, SocketAddr};
use std::process::Stdio;

use bevy::input::common_conditions::input_toggle_active;
use bevy::prelude::*;
use bevy::window::WindowResolution;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use owo_colors::OwoColorize;

use client::ClientPlugin;
use game::GamePlugin;
use networking::NetworkState;
use server::ServerPlugin;

use self::networking::show_networking_menu;

mod client;
mod game;
mod networking;
mod server;

fn main() {
    match std::env::args().nth(1).as_deref() {
        _ | Some("normal") => {
            create_app(
                "Bevy AoE".into(),
                WindowPosition::Centered(MonitorSelection::Primary),
                default(),
                true,
            )
            .run();
        }
        Some("client") => client(
            std::env::args()
                .nth(2)
                .expect("Client needs a second argument")
                .parse::<i32>()
                .expect("Second argument must be a number"),
        ),
        Some("server") => server(),
        Some("host") | None => {
            start_client(1, "[C1]".green());
            start_client(2, "[C2]".yellow());

            server();
        }
        _ => panic!("The first argument must be in {{server,client,host}}"),
    }
}

fn start_client(index: usize, prefix: impl Display) {
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

    let _ = child;
}

pub fn create_app(
    title: String,
    position: WindowPosition,
    resolution: WindowResolution,
    focused: bool,
) -> App {
    let mut app = App::new();
    app.add_plugins((
        DefaultPlugins
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title,
                    position,
                    resolution: resolution.clone(),
                    //resizable: false,
                    decorations: false,
                    focused,
                    ..default()
                }),
                ..default()
            })
            .set(ImagePlugin::default_nearest()),
        WorldInspectorPlugin::new().run_if(input_toggle_active(false, KeyCode::F3)),
        ServerPlugin,
        ClientPlugin,
        GamePlugin,
    ))
    .add_systems(
        Update,
        (
            move |mut windows: Query<&mut Window>, time: Res<Time>| {
                if time.elapsed_secs_f64() < 1.0 {
                    for mut window in &mut windows {
                        window.position = position;
                        window.resolution = resolution.clone();
                        window.focused = focused;
                    }
                }
            },
            show_networking_menu.run_if(in_state(NetworkState::Disconnected)),
        ),
    )
    .init_state::<NetworkState>();
    app
}

pub fn server() {
    println!("Starting host server/client!");

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

    create_app("Bevy AoE - Server".into(), position, resolution, true)
        .add_systems(
            Update,
            move |mut windows: Query<&mut Window>, time: Res<Time>| {
                if time.elapsed_secs_f64() < 1.0 {
                    for mut window in &mut windows {
                        window.focused = true;
                    }
                }
            },
        )
        .insert_state(NetworkState::Host(SocketAddr::new(
            Ipv4Addr::LOCALHOST.into(),
            5000,
        )))
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

    create_app("Bevy AoE - client".into(), position, resolution, false)
        .insert_state(NetworkState::Client {
            server_addr: SocketAddr::new(Ipv4Addr::LOCALHOST.into(), 5000),
            client_id: index as u64,
        })
        .run();
}
