use bevy::prelude::*;
use lightyear::connection::netcode::PRIVATE_KEY_BYTES;
use lightyear::prelude::*;

use self::client::ComponentSyncMode;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, || println!("Game has begun"));
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

#[derive(Component, Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
pub struct PlayerId(ClientId);

#[derive(Component, Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
pub struct PlayerPosition(Vec2);

#[derive(Component, Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
pub struct PlayerColor(pub(crate) Color);

pub const PROTOCOL_ID: u64 = 0;
pub const KEY: [u8; PRIVATE_KEY_BYTES] = [0; PRIVATE_KEY_BYTES];

pub struct ProtocolPlugin;

impl Plugin for ProtocolPlugin {
    fn build(&self, app: &mut App) {
        app.add_channel::<Channel1>(ChannelSettings {
            mode: ChannelMode::OrderedReliable(default()),
            ..default()
        });

        app.add_plugins(InputPlugin::<Inputs>::default());

        app.register_component::<PlayerId>(ChannelDirection::ServerToClient)
            .add_prediction(ComponentSyncMode::Once)
            .add_interpolation(ComponentSyncMode::Once);
        app.register_component::<PlayerPosition>(ChannelDirection::ServerToClient)
            .add_prediction(ComponentSyncMode::Once)
            .add_interpolation(ComponentSyncMode::Once);
        app.register_component::<PlayerColor>(ChannelDirection::ServerToClient)
            .add_prediction(ComponentSyncMode::Once)
            .add_interpolation(ComponentSyncMode::Once);
    }
}
