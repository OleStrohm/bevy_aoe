use std::ops::Add;
use std::ops::Mul;

use bevy::prelude::*;
use lightyear::prelude::*;

use super::{InputHandling, PlayerColor, Relevant};

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
                .chain()
                .after(InputHandling),
        );
        app.add_observer(unselect_minions);
    }
}

#[derive(Debug, Component)]
pub struct Selected;

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

pub fn minion_movement(
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
        commands.entity(player).insert((
            Sprite::from_color(color, Vec2::splat(1.0)),
            Transform {
                translation: pos.extend(0.0),
                scale: Vec3::splat(0.5),
                ..default()
            },
        ));
    }
}

fn show_selected_minions(mut commands: Commands, selected_minions: Query<Entity, Added<Selected>>) {
    for minion in &selected_minions {
        commands.entity(minion).with_children(|commands| {
            commands.spawn((
                Sprite::from_color(Color::srgb(1.0, 0.0, 1.0), Vec2::splat(1.1)),
                Transform::from_xyz(0.0, 0.0, -0.1),
            ));
        });
    }
}

fn unselect_minions(trigger: Trigger<OnRemove, Selected>, mut commands: Commands) {
    let entity = trigger.entity();
    commands.queue(move |world: &mut World| {
        if let Ok(mut entity) = world.get_entity_mut(entity) {
            entity.despawn_descendants();
        }
    });
}
