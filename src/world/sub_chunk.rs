use ndarray::{ArrayView2, ArrayView3};
use crate::biome::Biome;
use crate::block::Block;
use crate::region::{BlockEntity, HasPalette, Light, PendingTick, WorldSlice};
use crate::world::SubChunk;


impl SubChunk {
    pub fn new() -> SubChunk {
        let result = SubChunk {
            // region: Region::with_shape([16, 16, 16]),
            palette: Vec::new(),
            block_id_array: [0; 4096],
            sky_block_light_array: [Light::new(15, 15); 4096],
            biome_array: [Biome::the_void; 64],
            // sky_block_light: Array3::default(shape_yzx),
            // biome: Array2::default(shape_zx),
        };
        return result;
    }

    pub fn block_id(&self) -> ArrayView3<u16> {
        return ArrayView3::from_shape([16, 16, 16], &self.block_id_array).unwrap();
    }

    pub fn sky_block_light(&self) -> ArrayView3<Light> {
        // this will always succeed
        return ArrayView3::from_shape([16, 16, 16], &self.sky_block_light_array).unwrap();
    }

    pub fn biome(&self) -> ArrayView2<Biome> {
        // this will always succeed
        return ArrayView2::from_shape([8, 8], &self.biome_array).unwrap();
    }

    pub fn biome_at(&self, r_pos: [i32; 3]) -> Biome {
        return self.biome()[[(r_pos[2] / 2) as usize, (r_pos[0] / 2) as usize]];
    }
}

impl HasPalette for SubChunk {
    fn palette(&self) -> &[Block] {
        return &self.palette;
    }
}

impl WorldSlice for SubChunk {
    fn shape(&self) -> [i32; 3] {
        return [16, 16, 16];
    }


    fn total_blocks(&self, include_air: bool) -> u64 {
        let air_index = self.block_index_of_air();
        let void_index = self.block_index_of_structure_void();
        let mut counter = 0;
        for idx in self.block_id_array {
            if let Some(void_index) = void_index {
                if idx == void_index {
                    continue;
                }
            }
            if let Some(air_index) = air_index {
                if include_air && air_index == idx {
                    counter += 1;
                    continue;
                }
            }
            counter += 1;
        }
        return counter;
    }

    fn block_index_at(&self, r_pos: [i32; 3]) -> Option<u16> {
        if self.contains_coord(r_pos) {
            let r_pos = [r_pos[0] as usize, r_pos[1] as usize, r_pos[2] as usize];
            return Some(self.block_id()[r_pos]);
        }
        return None;
    }

    fn block_at(&self, r_pos: [i32; 3]) -> Option<&Block> {
        if self.contains_coord(r_pos) {
            let r_pos = [r_pos[0] as usize, r_pos[1] as usize, r_pos[2] as usize];
            let id = self.block_id()[r_pos];
            return Some(&self.palette[id as usize]);
        }
        return None;
    }

    fn block_entity_at(&self, _r_pos: [i32; 3]) -> Option<&BlockEntity> {
        return None;
    }

    fn pending_tick_at(&self, _r_pos: [i32; 3]) -> Option<&PendingTick> {
        return None;
    }
}