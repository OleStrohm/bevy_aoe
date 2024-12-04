use std::ops::Add;
use std::ops::Mul;

use bevy::prelude::*;
use lightyear::prelude::client::{Interpolated, Predicted};
use lightyear::prelude::*;

use crate::client::Selected;

use super::PlayerColor;

pub struct MinionPlugin;

impl Plugin for MinionPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            (show_minions, show_selected_minions, move_minions),
        );
    }
}

#[derive(
    Component, Reflect, Deref, DerefMut, Serialize, Deserialize, Clone, Copy, Debug, PartialEq,
)]
pub struct MinionPosition(pub Vec2);

impl Add for MinionPosition {
    type Output = MinionPosition;

    #[inline]
    fn add(self, rhs: MinionPosition) -> MinionPosition {
        MinionPosition(self.0.add(rhs.0))
    }
}

impl Mul<f32> for &MinionPosition {
    type Output = MinionPosition;

    #[inline]
    fn mul(self, rhs: f32) -> MinionPosition {
        MinionPosition(self.0 * rhs)
    }
}

#[derive(
    Component, Reflect, Deref, DerefMut, Serialize, Deserialize, Clone, Copy, Debug, PartialEq,
)]
pub struct MinionTarget(pub Vec2);

fn move_minions(mut minions: Query<(&MinionPosition, &mut Transform)>) {
    for (pos, mut tf) in &mut minions {
        tf.translation = pos.extend(0.0);
    }
}

fn show_minions(
    mut commands: Commands,
    players: Query<
        (
            Entity,
            &MinionPosition,
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
                transform: Transform {
                    translation: pos.extend(0.0),
                    scale: Vec3::splat(0.5),
                    ..default()
                },
                ..default()
            });
        }
    }
}

fn show_selected_minions(
    mut commands: Commands,
    selected_minions: Query<(Entity, Option<&Selected>)>,
) {
    for (minion, selected) in &selected_minions {
        if selected.is_some() {
            commands.entity(minion).with_children(|commands| {
                commands.spawn(SpriteBundle {
                    sprite: Sprite {
                        color: Color::srgb(1.0, 0.0, 1.0),
                        ..default()
                    },
                    transform: Transform {
                        translation: Vec3::new(0.0, 0.0, -0.1),
                        scale: Vec3::splat(1.1),
                        ..default()
                    },
                    ..default()
                });
            });
        } else {
            commands.entity(minion).despawn_descendants();
        }
    }
}
