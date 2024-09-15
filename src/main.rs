use bevy::log::Level;
use bevy::log::LogPlugin;
use bevy::prelude::*;
use bevy::winit::WinitPlugin;
use fork::{fork, waitpid, Fork};
use os_pipe::pipe;
use tracing_subscriber::filter::LevelFilter;
use std::io::Read;
use std::io::Write;
use std::sync::Mutex;
use tracing_subscriber::Layer;

fn main() {
    let (mut reader, writer) = pipe().expect("Failed to create pipe");

    let monitor_width = 2560;
    let monitor_height = 1440;

    match fork() {
        Ok(Fork::Parent(child)) => {
            App::new()
                .add_plugins(DefaultPlugins.set(WindowPlugin {
                    primary_window: Some(Window {
                        resolution:
                            ((monitor_width / 2) as f32, (monitor_height / 2) as f32).into(),
                        title: "Bevy AOE".into(),
                        resizable: false,
                        focused: true,
                        ..default()
                    }),
                    ..default()
                }))
                .add_systems(Startup, move |mut windows: Query<&mut Window>| {
                    for mut window in &mut windows {
                        window.position = WindowPosition::At(
                            (monitor_width / 4, monitor_height / 2 - 100).into(),
                        );
                    }
                })
                .run();

            let mut writer = writer;
            if writeln!(writer).is_ok() {
                waitpid(child).expect("Failed to wait for child");
            }
        }
        Ok(Fork::Child) => {
            let (app_exit_tx, app_exit_rx) = std::sync::mpsc::channel::<()>();
            let app_exit_rx = Mutex::new(app_exit_rx);

            std::thread::spawn(move || {
                let mut should_stop = [0; 1];
                reader
                    .read_exact(&mut should_stop)
                    .expect("Failed to read from pipe");
                app_exit_tx.send(()).expect("Could not notify of read");
            });

            App::new()
                .add_plugins(
                    DefaultPlugins
                        .set(WindowPlugin {
                            primary_window: Some(Window {
                                position: WindowPosition::At((1800, 100).into()),
                                resolution: (400.0, 400.0).into(),
                                title: "Bevy AOE - client".into(),
                                resizable: false,
                                focused: false,
                                ..default()
                            }),
                            ..default()
                        })
                        .set(LogPlugin {
                            custom_layer: |_| {
                                Some(
                                    tracing_subscriber::fmt::layer()
                                        .pretty()
                                        .with_writer(std::io::stdout)
                                        .with_filter(LevelFilter::INFO)
                                        .boxed(),
                                )
                            },
                            filter: String::new(),
                            level: Level::ERROR,
                        }),
                )
                .add_systems(Update, move |mut event_writer: EventWriter<AppExit>| {
                    if app_exit_rx.lock().unwrap().try_recv().is_ok() {
                        event_writer.send(AppExit::Success);
                    }
                })
                .run();

            std::process::exit(0);
        }
        Err(e) => println!("Fork failed: {e}"),
    }
}
