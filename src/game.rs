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
use self::player::{Inputs, PlayerColor, PlayerId, PlayerPlugin, PlayerPosition};
use self::resource::Item;
use self::resource::ItemPos;
use self::resource::ResourcePlugin;
use self::resource::Scoreboard;

pub mod minion;
pub mod player;
pub mod resource;

pub type Relevant = Or<(
    With<Predicted>,
    With<Interpolated>,
    With<Replicating>,
    With<PreSpawnedPlayerObject>,
)>;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((ProtocolPlugin, PlayerPlugin, MinionPlugin, ResourcePlugin))
            .add_systems(Startup, spawn_camera);
    }
}

#[derive(
    Component, Reflect, Deref, DerefMut, Serialize, Deserialize, Clone, Copy, Debug, PartialEq,
)]
pub struct OwnedBy(pub ClientId);

#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct InputHandling;

fn spawn_camera(mut commands: Commands) {
    commands.spawn((
        Camera2d,
        OrthographicProjection {
            scaling_mode: ScalingMode::FixedVertical {
                viewport_height: 10.0,
            },
            far: 1000.0,
            near: -1000.0,
            ..OrthographicProjection::default_2d()
        },
    ));
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
            .add_prediction(ComponentSyncMode::Once)
            .add_interpolation(ComponentSyncMode::Once);
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
        app.register_type::<Item>()
            .register_component::<Item>(ChannelDirection::ServerToClient)
            .add_prediction(ComponentSyncMode::Once)
            .add_interpolation(ComponentSyncMode::Once);
        app.register_type::<ItemPos>()
            .register_component::<ItemPos>(ChannelDirection::ServerToClient)
            .add_prediction(ComponentSyncMode::Once)
            .add_interpolation(ComponentSyncMode::Once);
        app.register_type::<Scoreboard>()
            .register_component::<Scoreboard>(ChannelDirection::ServerToClient);

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
