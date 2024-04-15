use std::borrow::BorrowMut;

use bevy::prelude::*;
use bevy_rapier3d::geometry::{Collider, ComputedColliderShape};
use bevy_rapier3d::pipeline::QueryFilter;
use bevy_rapier3d::plugin::RapierContext;
use noise::{Fbm, Perlin};

use super::super::player::plugin::Player;
use super::chunk::*;

pub const RENDER_DISTANCE_CHUNKS: u32 = 5;
const CHUNK_SPAWN_DISTANCE: i32 = RENDER_DISTANCE_CHUNKS as i32 + 1;

pub struct TerrainPlugin {
    pub seed: u32,
}

#[derive(Resource)]
pub struct TerrainSettings {
    fbm: Fbm<Perlin>,
    fbm_scale: f64,
    type_noise: Perlin,
    type_noise_scale: f64,
}

#[derive(Debug)]
pub enum TerrainEditShape {
    Sphere(f32),
}

#[derive(Event, Debug)]
pub struct TerrainCellEvent {
    pub origin: Vec3,
    pub dir: Vec3,
    pub value: f32,
    pub shape: TerrainEditShape,
    pub cell_type: Option<CellType>,
}

#[derive(Component)]
pub struct Empty {}

impl TerrainPlugin {
    fn spawn_chunk(commands: &mut Commands, settings: &Res<TerrainSettings>, pos: IVec3) {
        commands
            .spawn((
                Chunk::new(
                    &settings.fbm,
                    settings.fbm_scale,
                    settings.type_noise,
                    settings.type_noise_scale,
                    pos.x,
                    pos.y,
                    pos.z,
                ),
                SpatialBundle { ..default() }, // required for children
            ))
            // FIXME
            // need at least 1 child for the update query to work...
            .with_children(|parent| {
                parent.spawn(Empty {});
            });
    }

    fn spawn_around_player(
        mut commands: Commands,
        mut q_chunk: Query<&mut Chunk>,
        // FIXME: should not require player plugin?,
        // add some internal position indicator...
        q_player: Query<&Transform, With<Player>>,
        settings: Res<TerrainSettings>,
    ) {
        for player_trans in &q_player {
            let player_chunk = IVec3 {
                x: f32::floor(player_trans.translation.x / CHUNK_CUBE_SIZE as f32) as i32,
                y: f32::floor(player_trans.translation.y / CHUNK_CUBE_SIZE as f32) as i32,
                z: f32::floor(player_trans.translation.z / CHUNK_CUBE_SIZE as f32) as i32,
            };
            let mut wanted_chunks: Vec<IVec3> = Vec::new();
            for x in
                (player_chunk.x - CHUNK_SPAWN_DISTANCE)..(player_chunk.x + CHUNK_SPAWN_DISTANCE)
            {
                for y in
                    (player_chunk.y - CHUNK_SPAWN_DISTANCE)..(player_chunk.y + CHUNK_SPAWN_DISTANCE)
                {
                    for z in (player_chunk.z - CHUNK_SPAWN_DISTANCE)
                        ..(player_chunk.z + CHUNK_SPAWN_DISTANCE)
                    {
                        wanted_chunks.push(IVec3 { x, y, z });
                    }
                }
            }

            for mut chunk in q_chunk.iter_mut() {
                let mut found: i32 = -1;
                for i in 0..wanted_chunks.len() {
                    if wanted_chunks[i] == chunk.position {
                        found = i as i32;
                        break;
                    }
                }
                if found < 0 {
                    chunk.should_destroy = true;
                } else {
                    wanted_chunks.remove(found as usize);
                }
            }

            let max_spawn_per_frame = 1;
            wanted_chunks.sort_by(|a, b| {
                return (*a - player_chunk)
                    .length_squared()
                    .cmp(&(*b - player_chunk).length_squared());
            });

            for i in 0..usize::min(wanted_chunks.len(), max_spawn_per_frame) {
                Self::spawn_chunk(commands.borrow_mut(), &settings, wanted_chunks[i]);
            }
        }
    }

    fn read_terrain_events(
        mut events: EventReader<TerrainCellEvent>,
        mut q_chunks: Query<&mut Chunk>,
        q_colliders: Query<&Parent, With<Collider>>,
        q_player: Query<Entity, With<Player>>,
        rapier_context: Res<RapierContext>,
    ) {
        for event in events.read() {
            let max_dist = 10.0;
            let query_filter = QueryFilter {
                exclude_collider: Some(q_player.get_single().unwrap()),
                ..default()
            };
            if let Some((collider_id, toi)) =
                rapier_context.cast_ray(event.origin, event.dir, max_dist, true, query_filter)
            {
                let end_pos = event.origin + event.dir * toi;
                let mut hit_chunk = None;

                if let Ok(chunk_id) = q_colliders.get(collider_id) {
                    if let Ok(mut chunk) = q_chunks.get_mut(chunk_id.get()) {
                        chunk.edit(end_pos, event);
                        hit_chunk = Some(chunk);
                    }
                }

                // edit neighboring chunks if near edge
                if let Some(hit) = hit_chunk {
                    // todo:
                    // filter to chunks actually in range.
                    // no need to check all 26 if in one corner.
                    // also disallows radiuses larger than 1 chunk...
                    let neighbors = hit.get_neighbors();
                    for mut chunk in q_chunks.iter_mut() {
                        for nb in neighbors {
                            if chunk.position == nb {
                                chunk.edit(end_pos, event);
                            }
                        }
                    }
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
            if chunk.should_destroy {
                commands.entity(chunk_id).despawn_recursive();
                continue;
            }

            let mut mesh_updated = false;
            if chunk.is_dirty {
                chunk.mesh = chunk.polygonize();
                mesh_updated = true;
            }
            let mesh_valid = chunk.mesh.is_some();

            // TODO: Can we update the mesh in place instead?
            let destroy_children = mesh_updated;
            let spawn_children = mesh_updated && mesh_valid;

            if destroy_children {
                if let Some(handle) = &chunk.mesh_handle {
                    meshes.remove(handle);
                }
                if let Some(handle) = &chunk.material_handle {
                    materials.remove(handle);
                }

                for child in children {
                    commands.entity(*child).despawn_recursive();
                }

                chunk.mesh_handle = None;
                chunk.material_handle = None;
                chunk.collider = None;
            }

            if spawn_children {
                // TODO: fix all the clones
                if let Some(mesh) = chunk.mesh.clone() {
                    let mesh_handle = meshes.add(mesh.clone());
                    chunk.mesh_handle = Some(mesh_handle.clone());
                    let shape = ComputedColliderShape::TriMesh;
                    chunk.collider = Collider::from_bevy_mesh(&mesh, &shape);

                    let pbr_id = commands
                        .spawn(PbrBundle {
                            mesh: mesh_handle,
                            material: materials.add(StandardMaterial {
                                metallic: 0.0,
                                perceptual_roughness: 0.8,
                                ..default()
                            }),
                            ..default()
                        })
                        .id();

                    let col_id = commands.spawn(chunk.collider.clone().unwrap()).id();

                    commands.entity(chunk_id).add_child(pbr_id);
                    commands.entity(chunk_id).add_child(col_id);
                }
            }
        }
    }
}

impl Plugin for TerrainPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(TerrainSettings {
            fbm: Fbm::<Perlin>::new(self.seed),
            fbm_scale: 0.02,
            type_noise: Perlin::new(self.seed),
            type_noise_scale: 0.05,
        })
        .add_event::<TerrainCellEvent>()
        .add_systems(Update, Self::spawn_around_player)
        .add_systems(Update, Self::read_terrain_events)
        .add_systems(Update, Self::update_chunks);
    }
}
