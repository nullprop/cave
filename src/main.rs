mod player;
mod terrain;

use bevy::prelude::*;
use bevy_rapier3d::{
    plugin::{NoUserData, RapierPhysicsPlugin},
    render::RapierDebugRenderPlugin,
};
use player::plugin::PlayerPlugin;
use terrain::plugin::TerrainPlugin;

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::BLACK))
        .add_plugins(DefaultPlugins)
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugins(RapierDebugRenderPlugin::default())
        .add_plugins(TerrainPlugin { seed: 1337 })
        .add_plugins(PlayerPlugin {})
        .run();
}
