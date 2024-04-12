mod player;
mod terrain;

use bevy::prelude::*;
use bevy_rapier3d::{
    plugin::{NoUserData, RapierPhysicsPlugin},
    render::{DebugRenderContext, RapierDebugRenderPlugin},
};
use player::plugin::PlayerPlugin;
use terrain::plugin::TerrainPlugin;

fn debug_input(kb_input: Res<ButtonInput<KeyCode>>, mut debug_render: ResMut<DebugRenderContext>) {
    if kb_input.just_pressed(KeyCode::F1) {
        debug_render.enabled = !debug_render.enabled;
    }
}

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::BLACK))
        .add_plugins(DefaultPlugins)
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugins(RapierDebugRenderPlugin {
            enabled: false,
            ..default()
        })
        .add_plugins(TerrainPlugin { seed: 1337 })
        .add_plugins(PlayerPlugin {})
        .add_systems(Update, debug_input)
        .run();
}
