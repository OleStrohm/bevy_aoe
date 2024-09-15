use std::time::Duration;

use bevy::prelude::*;
use bevy::render::camera::ScalingMode;
use lightyear::connection::netcode::PRIVATE_KEY_BYTES;
use lightyear::prelude::*;

use self::client::ComponentSyncMode;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ProtocolPlugin);
        app.add_systems(Startup, || println!("Game has begun"));
        app.add_systems(Startup, spawn_camera);
        app.add_systems(FixedUpdate, show_players);
        app.add_systems(FixedUpdate, move_players);
    }
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle {
        projection: OrthographicProjection {
            scaling_mode: ScalingMode::FixedVertical(10.0),
            far: 1000.0,
            near: -1000.0,
            ..Default::default()
        },
        ..Default::default()
    });
}

fn move_players(mut players: Query<(&PlayerPosition, &mut Transform)>) {
    for (pos, mut tf) in &mut players {
        tf.translation = pos.extend(0.0);
    }
}

//fn show_players(mut gizmos: Gizmos, players: Query<(&PlayerPosition, &PlayerColor)>) {
//    for (position, color) in &players {
//        gizmos.rect(
//            position.extend(0.0),
//            Quat::IDENTITY,
//            Vec2::ONE * 1.0,
//            color.0,
//        );
//    }
//}

fn show_players(
    mut commands: Commands,
    players: Query<(Entity, &PlayerPosition, &PlayerColor), Without<Sprite>>,
) {
    for (player, pos, &PlayerColor(color)) in &players {
        commands.entity(player).insert(SpriteBundle {
            sprite: Sprite { color, ..default() },
            transform: Transform::from_xyz(pos.x, pos.y, 0.0),
            ..default()
        });
    }
}

#[derive(Component, Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub struct Direction {
    pub(crate) up: bool,
    pub(crate) down: bool,
    pub(crate) left: bool,
    pub(crate) right: bool,
}

#[derive(Component, Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Inputs {
    Direction(Direction),
    Delete,
    Spawn,
    None,
}

#[derive(Channel)]
pub struct Channel1;

#[derive(
    Component, Reflect, Deref, DerefMut, Serialize, Deserialize, Clone, Copy, Debug, PartialEq,
)]
pub struct PlayerId(pub ClientId);

#[derive(
    Component, Reflect, Deref, DerefMut, Serialize, Deserialize, Clone, Copy, Debug, PartialEq,
)]
pub struct PlayerPosition(pub Vec2);

#[derive(
    Component, Reflect, Deref, DerefMut, Serialize, Deserialize, Clone, Copy, Debug, PartialEq,
)]
pub struct PlayerColor(pub Color);

pub const PROTOCOL_ID: u64 = 0;
pub const KEY: [u8; PRIVATE_KEY_BYTES] = [0; PRIVATE_KEY_BYTES];

pub struct ProtocolPlugin;

impl Plugin for ProtocolPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(InputPlugin::<Inputs>::default());

        app.register_component::<PlayerId>(ChannelDirection::ServerToClient)
            .add_prediction(ComponentSyncMode::Once)
            .add_interpolation(ComponentSyncMode::Once);
        app.register_component::<PlayerPosition>(ChannelDirection::ServerToClient)
            .add_prediction(ComponentSyncMode::Full)
            .add_interpolation(ComponentSyncMode::Once);
        app.register_component::<PlayerColor>(ChannelDirection::ServerToClient)
            .add_prediction(ComponentSyncMode::Once)
            .add_interpolation(ComponentSyncMode::Once);

        app.add_channel::<Channel1>(ChannelSettings {
            mode: ChannelMode::OrderedReliable(default()),
            ..default()
        });
    }
}

pub fn shared_config(mode: Mode) -> SharedConfig {
    SharedConfig {
        server_replication_send_interval: Duration::from_millis(40),
        tick: TickConfig {
            tick_duration: Duration::from_secs_f64(1.0 / 64.0),
        },
        mode,
    }
}

pub fn shared_movement_behaviour(mut position: Mut<PlayerPosition>, input: &Inputs, time: &Time) {
    const MOVE_SPEED: f32 = 10.0;
    let move_speed = MOVE_SPEED * time.delta_seconds();
    if let Inputs::Direction(direction) = input {
        if direction.up {
            position.y += move_speed;
        }
        if direction.down {
            position.y -= move_speed;
        }
        if direction.left {
            position.x -= move_speed;
        }
        if direction.right {
            position.x += move_speed;
        }
    }
}
