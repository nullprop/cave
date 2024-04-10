use bevy::prelude::*;
use noise::{Fbm, Perlin};

use super::chunk::*;

pub struct TerrainPlugin {
    pub seed: u32,
}

#[derive(Resource)]
pub struct TerrainSettings {
    fbm: Fbm<Perlin>,
}

impl TerrainPlugin {
    fn create_terrain(
        settings: Res<TerrainSettings>,
        mut commands: Commands,
        mut mesh_assets: ResMut<Assets<Mesh>>,
    ) {
        let chunk = Chunk::new(&settings.fbm, 0, 0, 0);
        let mesh_handle = mesh_assets.add(chunk.polygonize());
        let pbr = PbrBundle {
            mesh: mesh_handle,
            ..default()
        };
        commands.spawn((chunk, pbr));
    }
}

impl Plugin for TerrainPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(TerrainSettings {
            fbm: Fbm::<Perlin>::new(self.seed),
        })
        .add_systems(Startup, Self::create_terrain);
    }
}
