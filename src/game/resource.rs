use std::time::Duration;

use bevy::input::common_conditions::input_pressed;
use bevy::prelude::*;
use bevy::time::common_conditions::on_timer;
use bevy::utils::HashMap;
use bevy_egui::EguiContexts;
use bevy_egui::egui::Align2;
use lightyear::prelude::{ClientId, is_server};
use serde::{Deserialize, Serialize};

use super::OwnedBy;
use super::minion::MinionPosition;

#[expect(non_snake_case)]
pub fn ResourcePlugin(app: &mut App) {
    app.add_systems(
        FixedUpdate,
        (
            show_items,
            update_scoreboard
                .run_if(is_server)
                .run_if(on_timer(Duration::from_secs(1))),
        ),
    )
    .add_systems(Update, show_scoreboard.run_if(input_pressed(KeyCode::Tab)));
}

#[derive(Debug, Component, Reflect, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
pub enum Item {
    Apple,
}

#[derive(
    Component, Reflect, Deref, DerefMut, Serialize, Deserialize, Clone, Copy, Debug, PartialEq,
)]
pub struct ItemPos(pub Vec2);

fn show_items(
    mut commands: Commands,
    items: Query<(Entity, &Item, &ItemPos), Added<Item>>,
    assets: ResMut<AssetServer>,
) {
    for (entity, item, pos) in &items {
        match item {
            Item::Apple => commands.entity(entity).insert((
                Sprite {
                    image: assets.load("apple.png"),
                    custom_size: Some(Vec2::splat(1.0)),
                    ..default()
                },
                Transform::from_translation(pos.0.extend(-0.2)),
            )),
        };
    }
}

#[derive(Component, Reflect, Deref, DerefMut, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Scoreboard(pub HashMap<ClientId, u64>);

fn show_scoreboard(mut contexts: EguiContexts, scoreboard: Query<&Scoreboard>) {
    let Ok(scoreboard) = scoreboard.get_single() else {
        return;
    };

    bevy_egui::egui::Window::new("Scoreboard")
        .anchor(Align2::RIGHT_TOP, (0.0, 0.0))
        .resizable([false, false])
        .collapsible(false)
        .show(contexts.ctx_mut(), |ui| {
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.label("Player");
                    for player in scoreboard.keys() {
                        ui.label(format!("{player}"));
                    }
                });
                ui.separator();
                ui.vertical(|ui| {
                    ui.label("Points");
                    for points in scoreboard.values() {
                        ui.label(format!("{points}"));
                    }
                });
            });
        });
}

fn update_scoreboard(
    mut scoreboard: Query<&mut Scoreboard>,
    minions: Query<(&MinionPosition, &OwnedBy)>,
    items: Query<&ItemPos>,
) {
    let mut scoreboard = scoreboard.single_mut();
    for (&minion_pos, owner) in &minions {
        for &item_pos in &items {
            if (minion_pos.0 - item_pos.0).length() < 1.0 {
                *scoreboard.0.entry(owner.0).or_default() += 1;
            }
        }
    }
}
