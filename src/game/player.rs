use std::ops::Add;
use std::ops::Mul;

use bevy::prelude::*;
use lightyear::prelude::*;
use lightyear::shared::events::components::InputEvent;

use crate::game::OwnedBy;
use crate::game::minion::MinionPosition;
use crate::game::minion::MinionTarget;

use self::client::ClientConnection;
use self::client::NetClient;
use self::client::Predicted;

use super::InputHandling;
use super::Relevant;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            (
                show_players,
                (player_movement.after(InputHandling), move_players).chain(),
            ),
        );
    }
}

fn move_players(mut players: Query<(&PlayerPosition, &mut Transform)>) {
    for (pos, mut tf) in &mut players {
        tf.translation = pos.extend(0.0);
    }
}

fn show_players(
    mut commands: Commands,
    players: Query<(Entity, &PlayerPosition, &PlayerColor), (Without<Sprite>, Relevant)>,
) {
    for (player, pos, &PlayerColor(color)) in &players {
        commands.entity(player).insert((
            Sprite { color, ..default() },
            Transform::from_xyz(pos.x, pos.y, 0.0),
        ));
    }
}

pub fn shared_movement_behaviour(
    input: &Inputs,
    commands: &mut Commands,
    position: &mut PlayerPosition,
    time: &Time<Fixed>,
    client_id: ClientId,
    spawn_bundle: impl Bundle,
) {
    match input {
        Inputs::Direction(dir) => {
            const MOVE_SPEED: f32 = 10.0;
            let move_speed = MOVE_SPEED * time.delta_secs();
            if dir.up {
                position.y += move_speed;
            }
            if dir.down {
                position.y -= move_speed;
            }
            if dir.left {
                position.x -= move_speed;
            }
            if dir.right {
                position.x += move_speed;
            }
        }
        &Inputs::Spawn(pos, color) => {
            println!("Spawn minion");
            commands.spawn((
                Name::new(format!("Minion - {client_id}")),
                MinionPosition(pos),
                MinionTarget(Vec2::new(4.0, 4.0)),
                PlayerColor(color),
                OwnedBy(client_id),
                spawn_bundle,
            ));
        }
        Inputs::None => (),
    }
}

fn player_movement(
    mut commands: Commands,
    mut position_query: Query<&mut PlayerPosition, With<Predicted>>,
    mut input_reader: EventReader<InputEvent<Inputs>>,
    time: Res<Time<Fixed>>,
    connection: Res<ClientConnection>,
) {
    for input in input_reader.read() {
        if let Some(input) = input.input() {
            if let Ok(mut position) = position_query.get_single_mut() {
                shared_movement_behaviour(
                    input,
                    &mut commands,
                    &mut position,
                    &time,
                    connection.id(),
                    PreSpawnedPlayerObject::default(),
                );
            }
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
