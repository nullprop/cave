use bevy::prelude::*;
use bevy_rapier3d::geometry::{Collider, ComputedColliderShape};
use noise::{Fbm, Perlin};

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
                    // TODO: all this should be in a ChunkBundle or something...
                    if let Some(mesh) = chunk.polygonize() {
                        let shape = ComputedColliderShape::TriMesh;
                        let collider = Collider::from_bevy_mesh(&mesh, &shape);
                        let mesh_handle = meshes.add(mesh);
                        let pbr = PbrBundle {
                            mesh: mesh_handle,
                            material: materials.add(StandardMaterial {
                                base_color: Color::GRAY,
                                metallic: 0.0,
                                perceptual_roughness: 0.6,
                                ..default()
                            }),
                            ..default()
                        };
                        commands.spawn((chunk, pbr, collider.unwrap()));
                    }
                }
            }
        }
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
