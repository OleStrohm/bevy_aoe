use std::ops::Add;
use std::ops::Mul;
use std::time::Duration;

use bevy::ecs::entity::MapEntities;
use bevy::prelude::*;
use bevy::render::camera::ScalingMode;
use lightyear::connection::netcode::PRIVATE_KEY_BYTES;
use lightyear::prelude::client::{ComponentSyncMode, Interpolated, Predicted};
use lightyear::prelude::*;

use self::minion::MinionPlugin;
use self::minion::MinionPosition;
use self::minion::MinionTarget;

pub mod minion;

pub type Relevant = Or<(
    With<Predicted>,
    With<Interpolated>,
    With<Replicating>,
    With<PreSpawnedPlayerObject>,
)>;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((ProtocolPlugin, MinionPlugin))
            .add_systems(Startup, spawn_camera)
            .add_systems(FixedUpdate, (show_players, move_players));
    }
}

#[derive(
    Component, Reflect, Deref, DerefMut, Serialize, Deserialize, Clone, Copy, Debug, PartialEq,
)]
pub struct OwnedBy(pub ClientId);

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

fn show_players(
    mut commands: Commands,
    players: Query<
        (
            Entity,
            &PlayerPosition,
            &PlayerColor,
            Option<&Predicted>,
            Option<&Interpolated>,
        ),
        Without<Sprite>,
    >,
) {
    for (player, pos, &PlayerColor(color), predicted, interpolated) in &players {
        if predicted.is_some() || interpolated.is_some() {
            commands.entity(player).insert(SpriteBundle {
                sprite: Sprite { color, ..default() },
                transform: Transform::from_xyz(pos.x, pos.y, 0.0),
                ..default()
            });
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub struct Direction {
    pub(crate) up: bool,
    pub(crate) down: bool,
    pub(crate) left: bool,
    pub(crate) right: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum Inputs {
    Direction(Direction),
    Spawn(Vec2, Color),
    None,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum ClientMessage {
    Target(Vec<Entity>, Vec2),
}

impl MapEntities for ClientMessage {
    fn map_entities<M: EntityMapper>(&mut self, entity_mapper: &mut M) {
        match self {
            ClientMessage::Target(entities, _) => {
                for entity in entities {
                    *entity = entity_mapper.map_entity(*entity);
                }
            }
        }
    }
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

impl Add for PlayerPosition {
    type Output = PlayerPosition;

    #[inline]
    fn add(self, rhs: PlayerPosition) -> PlayerPosition {
        PlayerPosition(self.0.add(rhs.0))
    }
}

impl Mul<f32> for &PlayerPosition {
    type Output = PlayerPosition;

    #[inline]
    fn mul(self, rhs: f32) -> PlayerPosition {
        PlayerPosition(self.0 * rhs)
    }
}

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

        app.register_message::<ClientMessage>(ChannelDirection::ClientToServer)
            .add_map_entities();

        app.register_type::<PlayerId>()
            .register_component::<PlayerId>(ChannelDirection::ServerToClient)
            .add_prediction(ComponentSyncMode::Once)
            .add_interpolation(ComponentSyncMode::Once);
        app.register_type::<PlayerPosition>()
            .register_component::<PlayerPosition>(ChannelDirection::ServerToClient)
            .add_prediction(ComponentSyncMode::Full)
            .add_interpolation(ComponentSyncMode::Full)
            .add_linear_interpolation_fn();
        app.register_type::<PlayerColor>()
            .register_component::<PlayerColor>(ChannelDirection::ServerToClient)
            .add_prediction(ComponentSyncMode::Simple)
            .add_interpolation(ComponentSyncMode::Simple);
        app.register_type::<MinionPosition>()
            .register_component::<MinionPosition>(ChannelDirection::ServerToClient)
            .add_prediction(ComponentSyncMode::Full)
            .add_interpolation(ComponentSyncMode::Full)
            .add_linear_interpolation_fn();
        app.register_type::<MinionTarget>()
            .register_component::<MinionTarget>(ChannelDirection::ServerToClient)
            .add_prediction(ComponentSyncMode::Simple)
            .add_interpolation(ComponentSyncMode::Simple);
        app.register_type::<OwnedBy>()
            .register_component::<OwnedBy>(ChannelDirection::ServerToClient)
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

pub fn shared_movement_behaviour(
    mut position: Mut<PlayerPosition>,
    direction: &Direction,
    time: &Time,
) {
    const MOVE_SPEED: f32 = 10.0;
    let move_speed = MOVE_SPEED * time.delta_seconds();
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
