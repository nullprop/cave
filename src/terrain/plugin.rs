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
        mut meshes: ResMut<Assets<Mesh>>,
        mut materials: ResMut<Assets<StandardMaterial>>,
    ) {
        // test chunk
        let chunk = Chunk::new(&settings.fbm, 0, 0, 0);
        let mesh_handle = meshes.add(chunk.polygonize());
        let pbr = PbrBundle {
            mesh: mesh_handle,
            material: materials.add(Color::GRAY),
            transform: Transform::from_xyz(0.0, 0.0, 0.0),
            ..default()
        };
        commands.spawn((chunk, pbr));

        // test cube
        commands.spawn(PbrBundle {
            mesh: meshes.add(Cuboid::new(1.0, 1.0, 1.0)),
            material: materials.add(Color::RED),
            transform: Transform::from_xyz(0.0, 0.0, 0.0),
            ..default()
        });
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
