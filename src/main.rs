use bevy::prelude::*;

mod player;
mod terrain;

use player::plugin::PlayerPlugin;
use terrain::plugin::TerrainPlugin;

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::BLACK))
        .add_plugins(DefaultPlugins)
        .add_plugins(TerrainPlugin { seed: 1337 })
        .add_plugins(PlayerPlugin {})
        .run();
}
