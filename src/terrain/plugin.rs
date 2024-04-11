use bevy::prelude::*;
use noise::{Fbm, MultiFractal, Perlin};

use super::chunk::*;

pub struct TerrainPlugin {
    pub seed: u32,
}

#[derive(Resource)]
pub struct TerrainSettings {
    fbm: Fbm<Perlin>,
    cell_noise_scale: f64,
}

impl TerrainPlugin {
    fn create_terrain(
        settings: Res<TerrainSettings>,
        mut commands: Commands,
        mut meshes: ResMut<Assets<Mesh>>,
        mut materials: ResMut<Assets<StandardMaterial>>,
    ) {
        // test chunks
        for x in -5..6 {
            for y in -5..6 {
                for z in -5..6 {
                    let chunk = Chunk::new(&settings.fbm, settings.cell_noise_scale, x, y, z);
                    let mesh_handle = meshes.add(chunk.polygonize());
                    let pbr = PbrBundle {
                        mesh: mesh_handle,
                        material: materials.add(Color::GRAY),
                        transform: Transform::from_xyz(0.0, 0.0, 0.0),
                        ..default()
                    };
                    commands.spawn((chunk, pbr));
                }
            }
        }

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
            cell_noise_scale: 0.02,
        })
        .add_systems(Startup, Self::create_terrain);
    }
}
