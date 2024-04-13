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

#[derive(Event, Debug)]
pub struct TerrainCellEvent {
    world_pos: IVec3,
    value: f32,
}

impl TerrainPlugin {
    fn create_terrain(settings: Res<TerrainSettings>, mut commands: Commands) {
        // test chunks
        for x in -5..6 {
            for y in -5..6 {
                for z in -5..6 {
                    commands.spawn(Chunk::new(
                        &settings.fbm,
                        settings.cell_noise_scale,
                        x,
                        y,
                        z,
                    ));
                }
            }
        }
    }

    fn read_terrain_events(
        mut events: EventReader<TerrainCellEvent>,
        mut q_chunks: Query<&mut Chunk>,
    ) {
        for event in events.read() {
            info!("{:?}", event);
            for mut chunk in &mut q_chunks {
                if chunk.is_in_chunk(event.world_pos) {
                    chunk.edit(event.world_pos, event.value);
                }
            }
        }
    }

    fn update_chunks(
        mut q_chunks: Query<(Entity, &mut Chunk, &Children)>,
        mut commands: Commands,
        mut meshes: ResMut<Assets<Mesh>>,
        mut materials: ResMut<Assets<StandardMaterial>>,
    ) {
        for (chunk_id, mut chunk, children) in &mut q_chunks {
            let mut mesh_updated = false;
            if chunk.is_dirty {
                chunk.mesh = chunk.polygonize();
                mesh_updated = true;
            }
            let mesh_valid = chunk.mesh.is_some();

            let should_destroy = mesh_updated || !mesh_valid;
            let should_spawn = mesh_updated && mesh_valid;

            if should_destroy {
                if let Some(handle) = &chunk.mesh_handle {
                    meshes.remove(handle);
                }
                if let Some(handle) = &chunk.material_handle {
                    materials.remove(handle);
                }

                for child in children {
                    commands.entity(*child).despawn_recursive();
                }

                chunk.mesh = None;
                chunk.mesh_handle = None;
                chunk.material_handle = None;
                chunk.collider = None;
            }

            if should_spawn {
                chunk.mesh_handle = Some(meshes.add(chunk.mesh.unwrap()));
                let shape = ComputedColliderShape::TriMesh;
                chunk.collider = Collider::from_bevy_mesh(&chunk.mesh.unwrap(), &shape);

                let pbr_id = commands
                    .spawn(PbrBundle {
                        mesh: chunk.mesh_handle.unwrap(),
                        material: materials.add(StandardMaterial {
                            base_color: Color::GRAY,
                            metallic: 0.0,
                            perceptual_roughness: 0.6,
                            ..default()
                        }),
                        ..default()
                    })
                    .id();

                let col_id = commands.spawn(chunk.collider.unwrap()).id();

                commands.entity(chunk_id).add_child(pbr_id);
                commands.entity(chunk_id).add_child(col_id);
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
        .add_systems(Startup, Self::create_terrain)
        .add_systems(Update, Self::read_terrain_events)
        .add_systems(Update, Self::update_chunks);
    }
}
