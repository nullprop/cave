use std::borrow::BorrowMut;

use bevy::prelude::*;
use bevy_rapier3d::geometry::{Collider, ComputedColliderShape};
use bevy_rapier3d::pipeline::QueryFilter;
use bevy_rapier3d::plugin::RapierContext;
use noise::{Fbm, Perlin};

use super::super::player::plugin::Player;
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
    pub origin: Vec3,
    pub dir: Vec3,
    pub value: f32,
    pub radius: f32,
}

#[derive(Component)]
pub struct Empty {}

impl TerrainPlugin {
    fn spawn_chunk(commands: &mut Commands, settings: &Res<TerrainSettings>, pos: IVec3) {
        commands
            .spawn((
                Chunk::new(
                    &settings.fbm,
                    settings.cell_noise_scale,
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
        // FIXME: should not require player plugin,
        // add some internal position indicator...
        q_player: Query<&Transform, With<Player>>,
        settings: Res<TerrainSettings>,
    ) {
        for player_trans in &q_player {
            let chunk_spawn_range = 4;
            let player_chunk = IVec3 {
                x: f32::floor(player_trans.translation.x / CHUNK_CUBE_SIZE as f32) as i32,
                y: f32::floor(player_trans.translation.y / CHUNK_CUBE_SIZE as f32) as i32,
                z: f32::floor(player_trans.translation.z / CHUNK_CUBE_SIZE as f32) as i32,
            };
            let mut wanted_chunks: Vec<IVec3> = Vec::new();
            for x in (player_chunk.x - chunk_spawn_range)..(player_chunk.x + chunk_spawn_range) {
                for y in (player_chunk.y - chunk_spawn_range)..(player_chunk.y + chunk_spawn_range)
                {
                    for z in
                        (player_chunk.z - chunk_spawn_range)..(player_chunk.z + chunk_spawn_range)
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

                if let Ok(chunk_id) = q_colliders.get(collider_id) {
                    if let Ok(mut chunk) = q_chunks.get_mut(chunk_id.get()) {
                        // the chunk we hit
                        chunk.edit(end_pos, event);

                        // neighboring chunks if near edge
                        // FIXME this sucks
                        // FIXME also causes gaps between chunks
                        let neg_x = (end_pos.x % CHUNK_CUBE_SIZE as f32) < event.radius;
                        let pos_x = (end_pos.x % CHUNK_CUBE_SIZE as f32)
                            > CHUNK_CUBE_SIZE as f32 - event.radius;
                        let neg_y = (end_pos.y % CHUNK_CUBE_SIZE as f32) < event.radius;
                        let pos_y = (end_pos.y % CHUNK_CUBE_SIZE as f32)
                            > CHUNK_CUBE_SIZE as f32 - event.radius;
                        let neg_z = (end_pos.z % CHUNK_CUBE_SIZE as f32) < event.radius;
                        let pos_z = (end_pos.z % CHUNK_CUBE_SIZE as f32)
                            > CHUNK_CUBE_SIZE as f32 - event.radius;

                        let mut other_chunks: Vec<IVec3> = Vec::new();
                        if neg_x {
                            other_chunks.push(chunk.position + IVec3 { x: -1, y: 0, z: 0 });
                        }
                        if pos_x {
                            other_chunks.push(chunk.position + IVec3 { x: 1, y: 0, z: 0 });
                        }
                        if neg_y {
                            other_chunks.push(chunk.position + IVec3 { x: 0, y: -1, z: 0 });
                        }
                        if pos_y {
                            other_chunks.push(chunk.position + IVec3 { x: 0, y: 1, z: 0 });
                        }
                        if neg_z {
                            other_chunks.push(chunk.position + IVec3 { x: 0, y: 0, z: -1 });
                        }
                        if pos_z {
                            other_chunks.push(chunk.position + IVec3 { x: 0, y: 0, z: 1 });
                        }

                        for mut chunk in q_chunks.iter_mut() {
                            for v in &other_chunks {
                                if chunk.position == *v {
                                    chunk.edit(end_pos, event);
                                }
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
                                base_color: Color::GRAY,
                                metallic: 0.0,
                                perceptual_roughness: 0.6,
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
            cell_noise_scale: 0.02,
        })
        .add_event::<TerrainCellEvent>()
        .add_systems(Update, Self::spawn_around_player)
        .add_systems(Update, Self::read_terrain_events)
        .add_systems(Update, Self::update_chunks);
    }
}
