use ndarray::{ArrayView2, ArrayView3};
use crate::biome::Biome;
use crate::region::{Light, Region};
use crate::world::SubChunk;


impl SubChunk {
    pub fn new() -> SubChunk {
        let result = SubChunk {
            region: Region::with_shape([16, 16, 16]),
            sky_block_light_array: [Light::new(15, 15); 4096],
            biome_array: [Biome::the_void; 64],
            // sky_block_light: Array3::default(shape_yzx),
            // biome: Array2::default(shape_zx),
        };
        return result;
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