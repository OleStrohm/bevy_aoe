use std::ops::Add;
use std::ops::Mul;

use bevy::prelude::*;
use lightyear::prelude::*;

use crate::client::Selected;

use super::PlayerColor;
use super::Relevant;

pub struct MinionPlugin;

impl Plugin for MinionPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            (
                show_minions,
                (minion_movement, move_minions).chain(),
                (show_selected_minions, apply_deferred).chain(),
            )
                .chain(),
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

fn minion_movement(
    mut minions: Query<(&mut MinionPosition, &MinionTarget), Relevant>,
    time: Res<Time<Fixed>>,
) {
    for (mut pos, target) in &mut minions {
        let diff = target.0 - pos.0;
        if diff.length_squared() < 0.001 {
            pos.0 = target.0;
        } else {
            pos.0 += diff.clamp_length(0.0, 1.0 * time.delta_secs());
        }
    }
}

fn move_minions(mut minions: Query<(&MinionPosition, &mut Transform), Relevant>) {
    for (pos, mut tf) in &mut minions {
        tf.translation = pos.extend(0.0);
    }
}

fn show_minions(
    mut commands: Commands,
    players: Query<(Entity, &MinionPosition, &PlayerColor), (Without<Sprite>, Relevant)>,
) {
    for (player, pos, &PlayerColor(color)) in &players {
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

fn show_selected_minions(
    mut commands: Commands,
    selected_minions: Query<
        (Entity, Option<&Selected>),
        Or<(Changed<Selected>, Without<Selected>)>,
    >,
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
