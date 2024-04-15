use bevy::{
    prelude::*,
    render::{mesh::Indices, render_asset::RenderAssetUsages},
};
use bevy_rapier3d::geometry::Collider;
use noise::{Fbm, NoiseFn, Perlin};

use super::{
    marching_cube::*,
    plugin::{TerrainCellEvent, TerrainEditShape},
};

pub const CHUNK_CUBE_SIZE: usize = 16;

const CELL_GRID_SIZE: usize = CHUNK_CUBE_SIZE + 1;
const CELL_GRID_SIZE_2: usize = CELL_GRID_SIZE * CELL_GRID_SIZE;
const CELL_GRID_SIZE_3: usize = CELL_GRID_SIZE_2 * CELL_GRID_SIZE;

#[derive(Copy, Clone, Default, Debug)]
pub enum CellType {
    #[default]
    Stone,
    Granite,
    Dirt,
    Iron,
    Gold,
    Ruby,
}

#[derive(Copy, Clone, Default)]
pub struct Cell {
    pub value: f32,
    pub cell_type: CellType,
}

#[derive(Component)]
pub struct Chunk {
    pub position: IVec3,
    pub cells: [Cell; CELL_GRID_SIZE_3],
    pub is_dirty: bool,
    pub should_destroy: bool,
    pub mesh: Option<Mesh>,
    pub mesh_handle: Option<Handle<Mesh>>,
    pub material_handle: Option<Handle<StandardMaterial>>,
    pub collider: Option<Collider>,
}

impl Chunk {
    pub fn new(
        fbm: &Fbm<Perlin>,
        cell_noise_scale: f64,
        type_noise: Perlin,
        type_noise_scale: f64,
        x: i32,
        y: i32,
        z: i32,
    ) -> Self {
        let mut chunk = Chunk {
            position: IVec3 { x, y, z },
            cells: [Cell::default(); CELL_GRID_SIZE_3],
            is_dirty: true,
            should_destroy: false,
            mesh: None,
            mesh_handle: None,
            material_handle: None,
            collider: None,
        };

        chunk.generate_noise(fbm, cell_noise_scale, type_noise, type_noise_scale);

        return chunk;
    }

    pub fn is_in_chunk(&self, world_pos: Vec3) -> bool {
        let min = self.position * CHUNK_CUBE_SIZE as i32;
        let max = min + IVec3::ONE * CHUNK_CUBE_SIZE as i32; // inclusive

        if world_pos.x < min.x as f32 || world_pos.y < min.y as f32 || world_pos.z < min.z as f32 {
            return false;
        }
        if world_pos.x > max.x as f32 || world_pos.y > max.y as f32 || world_pos.z > max.z as f32 {
            return false;
        }

        return true;
    }

    pub fn edit(&mut self, end_pos: Vec3, event: &TerrainCellEvent) {
        let mut cells_to_edit = Vec::new();

        match event.shape {
            TerrainEditShape::Sphere(radius) => {
                let min = end_pos - radius;
                let max = end_pos + radius;

                let mut int_min = IVec3 {
                    x: min.x.round() as i32,
                    y: min.y.round() as i32,
                    z: min.z.round() as i32,
                };
                let mut int_max = IVec3 {
                    x: max.x.round() as i32,
                    y: max.y.round() as i32,
                    z: max.z.round() as i32,
                };

                let chunk_min = self.position * CHUNK_CUBE_SIZE as i32; // inclusive
                let chunk_max = chunk_min + IVec3::ONE * CHUNK_CUBE_SIZE as i32; // inclusive

                if int_min.x > chunk_max.x || int_min.y > chunk_max.y || int_min.z > chunk_max.z {
                    return;
                }
                if int_max.x < chunk_min.x || int_max.y < chunk_min.y || int_max.z < chunk_min.z {
                    return;
                }

                int_min = int_min.clamp(chunk_min, chunk_max);
                int_max = int_max.clamp(chunk_min, chunk_max);

                for x in int_min.x..(int_max.x + 1) {
                    for y in int_min.y..(int_max.y + 1) {
                        for z in int_min.z..(int_max.z + 1) {
                            let cell_pos = self.world_to_cell(IVec3 { x, y, z });
                            let index = Self::cell_to_index(
                                cell_pos.x as usize,
                                cell_pos.y as usize,
                                cell_pos.z as usize,
                            );
                            cells_to_edit.push(index);
                        }
                    }
                }
            }
        }

        for index in cells_to_edit {
            self.cells[index].value = event.value;
            if let Some(t) = event.cell_type {
                self.cells[index].cell_type = t;
            }
            self.is_dirty = true;
        }
    }

    // Gets all neighboring chunk positions,
    // including diagonals.
    pub fn get_neighbors(&self) -> [IVec3; 26] {
        let mut neighbors = [self.position; 26];
        let mut i = 0;
        for x in -1..2 {
            for y in -1..2 {
                for z in -1..2 {
                    if x == 0 && y == 0 && z == 0 {
                        continue;
                    }
                    neighbors[i].x += x;
                    neighbors[i].y += y;
                    neighbors[i].z += z;
                    i += 1;
                }
            }
        }
        return neighbors;
    }

    fn generate_noise(
        &mut self,
        fbm: &Fbm<Perlin>,
        cell_noise_scale: f64,
        type_noise: Perlin,
        type_noise_scale: f64,
    ) {
        for cell_x in 0..CELL_GRID_SIZE {
            for cell_y in 0..CELL_GRID_SIZE {
                for cell_z in 0..CELL_GRID_SIZE {
                    let index = Self::cell_to_index(cell_x, cell_y, cell_z);
                    let cell_world = self.cell_to_world(cell_x, cell_y, cell_z);
                    let cell_world_f = [
                        cell_world.x as f64 * cell_noise_scale,
                        cell_world.y as f64 * cell_noise_scale,
                        cell_world.z as f64 * cell_noise_scale,
                    ];
                    let type_world_f = [
                        cell_world.x as f64 * type_noise_scale,
                        cell_world.y as f64 * type_noise_scale,
                        cell_world.z as f64 * type_noise_scale,
                    ];
                    self.cells[index].value = (fbm.get(cell_world_f) as f32) * 0.5 + 0.5;
                    self.cells[index].cell_type =
                        Self::get_cell_type(type_noise.get(type_world_f) as f32 * 0.5 + 0.5);
                }
            }
        }
    }

    // TODO: just pass cell type to shader and have a list of colors, etc.
    fn get_cell_color(cell_type: CellType) -> Vec4 {
        match cell_type {
            CellType::Stone => Vec4::new(0.4, 0.4, 0.4, 1.0),
            CellType::Granite => Vec4::new(1.0, 1.0, 1.0, 1.0),
            CellType::Dirt => Vec4::new(0.3, 0.15, 0.1, 1.0),
            CellType::Iron => Vec4::new(0.6, 0.3, 0.0, 1.0),
            CellType::Gold => Vec4::new(1.0, 0.8, 0.1, 1.0),
            CellType::Ruby => Vec4::new(1.0, 0.0, 0.0, 1.0),
        }
    }

    fn get_cell_type(value: f32) -> CellType {
        if value < 0.2 {
            return CellType::Dirt;
        }

        if value > 0.3 && value < 0.4 {
            return CellType::Granite;
        }

        if value > 0.6 && value < 0.64 {
            return CellType::Iron;
        }

        if value > 0.8 && value < 0.82 {
            return CellType::Gold;
        }

        if value > 0.9 && value < 0.91 {
            return CellType::Ruby;
        }

        return CellType::Stone;
    }

    fn cell_to_world(&self, cell_x: usize, cell_y: usize, cell_z: usize) -> IVec3 {
        return self.position * CHUNK_CUBE_SIZE as i32
            + IVec3 {
                x: cell_x as i32,
                y: cell_y as i32,
                z: cell_z as i32,
            };
    }

    fn world_to_cell(&self, world_pos: IVec3) -> IVec3 {
        return world_pos - self.position * CHUNK_CUBE_SIZE as i32;
    }

    fn cell_index_to_world(&self, index: usize) -> IVec3 {
        let cells = Self::index_to_cell(index);
        return self.cell_to_world(cells[0], cells[1], cells[2]);
    }

    fn cell_to_index(cell_x: usize, cell_y: usize, cell_z: usize) -> usize {
        return cell_x * CELL_GRID_SIZE_2 + cell_y * CELL_GRID_SIZE + cell_z;
    }

    fn index_to_cell(index: usize) -> [usize; 3] {
        let mut coords: [usize; 3] = [0; 3];
        coords[2] = index % CELL_GRID_SIZE;
        coords[1] = ((index - coords[2]) % CELL_GRID_SIZE_2) / CELL_GRID_SIZE;
        coords[0] = (index - coords[2] - coords[1] * CELL_GRID_SIZE) / CELL_GRID_SIZE_2;
        return coords;
    }

    fn cube_to_cell_indices(cube_x: usize, cube_y: usize, cube_z: usize) -> [usize; 8] {
        return [
            // bottom
            Self::cell_to_index(cube_x, cube_y, cube_z),
            Self::cell_to_index(cube_x + 1, cube_y, cube_z),
            Self::cell_to_index(cube_x + 1, cube_y, cube_z + 1),
            Self::cell_to_index(cube_x, cube_y, cube_z + 1),
            // top
            Self::cell_to_index(cube_x, cube_y + 1, cube_z),
            Self::cell_to_index(cube_x + 1, cube_y + 1, cube_z),
            Self::cell_to_index(cube_x + 1, cube_y + 1, cube_z + 1),
            Self::cell_to_index(cube_x, cube_y + 1, cube_z + 1),
        ];
    }

    pub fn polygonize(&mut self) -> Option<Mesh> {
        self.is_dirty = false;
        let mut mesh_verts = Vec::new();
        let mut mesh_colors: Vec<Vec4> = Vec::new();

        for cube_x in 0..CHUNK_CUBE_SIZE {
            for cube_y in 0..CHUNK_CUBE_SIZE {
                for cube_z in 0..CHUNK_CUBE_SIZE {
                    let iso_level: f32 = 0.5;
                    let corner_indices = Self::cube_to_cell_indices(cube_x, cube_y, cube_z);
                    let corner_cells = corner_indices.map(|i| self.cells[i]);

                    // Determine the index into the edge table, which
                    // tells us which vertices are inside of the surface.
                    let mut cube_index = 0;
                    for i in 0..8 {
                        if corner_cells[i].value < iso_level {
                            cube_index |= 1 << i;
                        }
                    }

                    let edge = MC_EDGE_TABLE[cube_index];

                    // Is the cube entirely in/out of the surface?
                    if edge == 0 {
                        continue;
                    }

                    let corners = corner_indices
                        .map(|i| self.cell_index_to_world(i))
                        .map(|v| Vec3 {
                            x: v.x as f32,
                            y: v.y as f32,
                            z: v.z as f32,
                        });

                    let mut vertices = [Vec3::default(); 12];
                    let mut types = [CellType::default(); 12];

                    // Find the vertices where the surface intersects the cube.
                    if (edge & 1) == 1 {
                        (vertices[0], types[0]) = mc_interpolate_vertex(
                            iso_level,
                            corners[0],
                            corners[1],
                            corner_cells[0],
                            corner_cells[1],
                        );
                    }
                    if (edge & 2) == 2 {
                        (vertices[1], types[1]) = mc_interpolate_vertex(
                            iso_level,
                            corners[1],
                            corners[2],
                            corner_cells[1],
                            corner_cells[2],
                        );
                    }
                    if (edge & 4) == 4 {
                        (vertices[2], types[2]) = mc_interpolate_vertex(
                            iso_level,
                            corners[2],
                            corners[3],
                            corner_cells[2],
                            corner_cells[3],
                        );
                    }
                    if (edge & 8) == 8 {
                        (vertices[3], types[3]) = mc_interpolate_vertex(
                            iso_level,
                            corners[3],
                            corners[0],
                            corner_cells[3],
                            corner_cells[0],
                        );
                    }
                    if (edge & 16) == 16 {
                        (vertices[4], types[4]) = mc_interpolate_vertex(
                            iso_level,
                            corners[4],
                            corners[5],
                            corner_cells[4],
                            corner_cells[5],
                        );
                    }
                    if (edge & 32) == 32 {
                        (vertices[5], types[5]) = mc_interpolate_vertex(
                            iso_level,
                            corners[5],
                            corners[6],
                            corner_cells[5],
                            corner_cells[6],
                        );
                    }
                    if (edge & 64) == 64 {
                        (vertices[6], types[6]) = mc_interpolate_vertex(
                            iso_level,
                            corners[6],
                            corners[7],
                            corner_cells[6],
                            corner_cells[7],
                        );
                    }
                    if (edge & 128) == 128 {
                        (vertices[7], types[7]) = mc_interpolate_vertex(
                            iso_level,
                            corners[7],
                            corners[4],
                            corner_cells[7],
                            corner_cells[4],
                        );
                    }
                    if (edge & 256) == 256 {
                        (vertices[8], types[8]) = mc_interpolate_vertex(
                            iso_level,
                            corners[0],
                            corners[4],
                            corner_cells[0],
                            corner_cells[4],
                        );
                    }
                    if (edge & 512) == 512 {
                        (vertices[9], types[9]) = mc_interpolate_vertex(
                            iso_level,
                            corners[1],
                            corners[5],
                            corner_cells[1],
                            corner_cells[5],
                        );
                    }
                    if (edge & 1024) == 1024 {
                        (vertices[10], types[10]) = mc_interpolate_vertex(
                            iso_level,
                            corners[2],
                            corners[6],
                            corner_cells[2],
                            corner_cells[6],
                        );
                    }
                    if (edge & 2048) == 2048 {
                        (vertices[11], types[11]) = mc_interpolate_vertex(
                            iso_level,
                            corners[3],
                            corners[7],
                            corner_cells[3],
                            corner_cells[7],
                        );
                    }

                    // Create the triangle.
                    let mut idx = 0;
                    while MC_TRI_TABLE[cube_index][idx] != -1 {
                        let i1 = MC_TRI_TABLE[cube_index][idx] as usize;
                        let i2 = MC_TRI_TABLE[cube_index][idx + 1] as usize;
                        let i3 = MC_TRI_TABLE[cube_index][idx + 2] as usize;

                        mesh_verts.push(vertices[i1]);
                        mesh_verts.push(vertices[i2]);
                        mesh_verts.push(vertices[i3]);

                        mesh_colors.push(Self::get_cell_color(types[i1]));
                        mesh_colors.push(Self::get_cell_color(types[i2]));
                        mesh_colors.push(Self::get_cell_color(types[i3]));

                        idx += 3;
                    }
                }
            }
        }

        if mesh_verts.len() == 0 {
            return None;
        }

        let mesh_indices: Vec<u32> = (0u32..mesh_verts.len() as u32).collect();

        return Some(
            Mesh::new(
                bevy::render::mesh::PrimitiveTopology::TriangleList,
                RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
            )
            .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, mesh_verts)
            .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, mesh_colors)
            .with_computed_flat_normals()
            .with_inserted_indices(Indices::U32(mesh_indices)),
        );
    }
}
