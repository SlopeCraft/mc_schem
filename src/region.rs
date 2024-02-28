/*
mc_schem is a rust library to generate, load, manipulate and save minecraft schematic files.
Copyright (C) 2024  joseph

This program is free software: you can redistribute it and/or modify
it under the terms of the GNU General Public License as published by
the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU General Public License for more details.

You should have received a copy of the GNU General Public License
along with this program.  If not, see <https://www.gnu.org/licenses/>.
*/

use std::collections::HashMap;
use ndarray::Array3;
use crate::block::Block;
use crate::error::Error;

/// An entity in MC, like zombie, minecart, etc.
#[derive(Debug, Clone)]
pub struct Entity {
    /// nbt tags of entity
    pub tags: HashMap<String, fastnbt::Value>,
    /// Position in double precision float
    pub position: [f64; 3],
    /// Position in integer
    pub block_pos: [i32; 3],
}

/// Block entity(also known as tile entity) in MC, like chest, furnace, etc.
#[derive(Debug, Clone)]
pub struct BlockEntity {
    /// nbt tags of block entity
    pub tags: HashMap<String, fastnbt::Value>,
}


#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum PendingTickInfo {
    Fluid { id: String },
    Block { id: String },
}

/// A tick waiting to be processed
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct PendingTick {
    pub priority: i32,
    pub sub_tick: i64,
    pub time: i32,
    pub info: PendingTickInfo,
}

/// Part of a Minecraft world
pub trait WorldSlice {
    /// Offset of this region
    fn offset(&self) -> [i32; 3];
    /// Shape in x, y, z
    fn shape(&self) -> [i32; 3];
    /// If `r_pos` is inside the region
    fn contains_coord(&self, r_pos: [i32; 3]) -> bool {
        for dim in 0..3 {
            if r_pos[dim] >= 0 && r_pos[dim] < self.shape()[dim] {
                continue;
            }
            return false;
        }
        return true;
    }
    /// Returns the volume
    fn volume(&self) -> u64 {
        return self.shape()[0] as u64 * self.shape()[1] as u64 * self.shape()[2] as u64;
    }
    ///Returns the count of blocks in region. Air will be counted if `include_air` is true, structure
    /// void is never counted.
    fn total_blocks(&self, include_air: bool) -> u64;
    /// Returns detailed block infos at `r_pos`, including block index, block, block entity and pending tick.
    /// Returns `None` if the block is outside the region
    fn block_info_at(&self, r_pos: [i32; 3]) -> Option<(u16, &Block, Option<&BlockEntity>, Option<&PendingTick>)>;
    /// Get block index at `r_pos`, returns `None` if the block is outside the region
    fn block_index_at(&self, r_pos: [i32; 3]) -> Option<u16> {
        return Some(self.block_info_at(r_pos)?.0);
    }
    /// Get block at `r_pos`, returns `None` if the block is outside the region
    fn block_at(&self, r_pos: [i32; 3]) -> Option<&Block> {
        return Some(self.block_info_at(r_pos)?.1);
    }
    /// Get block entity at `r_pos`
    fn block_entity_at(&self, r_pos: [i32; 3]) -> Option<&BlockEntity> {
        return self.block_info_at(r_pos)?.2;
    }
    /// Get pending tick at `r_pos`
    fn pending_tick_at(&self, r_pos: [i32; 3]) -> Option<&PendingTick> {
        return self.block_info_at(r_pos)?.3;
    }
    // /// Returns detailed block infos at `r_pos`, including block index, block, block entity(mutable) and pending tick(mutable).
    // /// Returns `None` if the block is outside the region
    // fn block_info_at_mut(&mut self, r_pos: [i32; 3]) -> Option<(u16, &Block, Option<&mut BlockEntity>, Option<&mut PendingTick>)>;
    // /// Get mutable block entity at `r_pos`
    // fn block_entity_at_mut(&mut self, r_pos: [i32; 3]) -> Option<&mut BlockEntity> {
    //     return self.block_info_at_mut(r_pos)?.2;
    // }
    // /// Get mutable pending tick at `r_pos`
    // fn pending_tick_at_mut(&mut self, r_pos: [i32; 3]) -> Option<&mut PendingTick> {
    //     return self.block_info_at_mut(r_pos)?.3;
    // }
}

/// Region is a 3d area in Minecraft, containing blocks and entities. \
/// Litematica files can have multiple regions, but vanilla structure, world edit schematics can have only one. \
/// Blocks in a region are stored as continuous 3d index array. A palette(Vec of blocks) records all
/// kinds of blocks in this region, so each block can be represented by an index(u16). Indices of
/// every block are stored in 3d array, indexed by y, z, x. YZX is applied because all schematic
/// formats store blocks in this order, by following this custom, our library can have better performance in loading and saving.
#[derive(Debug, Clone)]
pub struct Region {
    /// Name of this region, only useful in litematica
    pub name: String,
    /// Array of block indices, stored in y,z,x
    pub array_yzx: Array3<u16>,
    /// All kinds of blocks
    pub palette: Vec<Block>,
    /// All block entities. The key is position (xyz)
    pub block_entities: HashMap<[i32; 3], BlockEntity>,
    /// All pending ticks. The key is position (xyz)
    pub pending_ticks: HashMap<[i32; 3], PendingTick>,
    /// All entities
    pub entities: Vec<Entity>,
    /// Offset of this region
    pub offset: [i32; 3],

    //pub array_number_id_damage: Option<Array3<(u8, u8)>>
}

impl Entity {
    pub fn new() -> Entity {
        return Entity {
            tags: HashMap::new(),
            position: [0.0, 0.0, 0.0],
            block_pos: [0, 0, 0],
        };
    }

    /// Add adder to position and block_pos
    pub fn pos_shift(&mut self, adder: [i32; 3]) {
        for dim in 0..3 {
            self.block_pos[dim] += adder[dim];
            self.position[dim] += adder[dim] as f64;
        }
    }
}

impl BlockEntity {
    pub fn new() -> BlockEntity {
        return BlockEntity {
            tags: HashMap::new(),
        };
    }
}


impl PendingTickInfo {
    pub fn default() -> PendingTickInfo {
        return PendingTickInfo::Block { id: "".to_string() };
    }
}

impl WorldSlice for Region {
    fn offset(&self) -> [i32; 3] {
        return self.offset;
    }

    /// Shape in x, y, z
    fn shape(&self) -> [i32; 3] {
        let shape = self.array_yzx.shape();
        if shape.len() != 3 {
            panic!("Invalid array dimensions: should be 3 but now it is {}", shape.len());
        }
        return Self::pos_yzx_to_xyz(&[shape[0] as i32, shape[1] as i32, shape[2] as i32]);
    }
    ///Returns the count of blocks in region. Air will be counted if `include_air` is true, structure
    /// void is never counted.
    fn total_blocks(&self, include_air: bool) -> u64 {
        let mut counter = 0;

        for blk_id in &self.array_yzx {
            if let Some(air_idx) = self.block_index_of_air() {
                if *blk_id == air_idx {
                    if include_air {
                        counter += 1;
                    }
                    continue;
                }
            }

            if let Some(sv_idx) = self.block_index_of_structure_void() {
                if *blk_id == sv_idx {
                    counter += 1;
                    continue;
                }
            }

            counter += 1;
        }
        return counter;
    }

    /// Returns detailed block infos at `r_pos`, including block index, block, block entity and pending tick.
    /// Returns `None` if the block is outside the region
    fn block_info_at(&self, r_pos: [i32; 3]) -> Option<(u16, &Block, Option<&BlockEntity>, Option<&PendingTick>)> {
        return if let Some(pid) = self.block_index_at(r_pos) {
            Some((pid, &self.palette[pid as usize],
                  self.block_entities.get(&r_pos),
                  self.pending_ticks.get(&r_pos)))
        } else {
            None
        };
    }
    /// Get block index at `r_pos`, returns `None` if the block is outside the region
    fn block_index_at(&self, r_pos: [i32; 3]) -> Option<u16> {
        if !self.contains_coord(r_pos) {
            return None;
        }

        let x = r_pos[0] as usize;
        let y = r_pos[1] as usize;
        let z = r_pos[2] as usize;

        let pid = self.array_yzx[[y, z, x]] as usize;
        return Some(pid as u16);
    }
    /// Get block at `r_pos`, returns `None` if the block is outside the region
    fn block_at(&self, r_pos: [i32; 3]) -> Option<&Block> {
        return if let Some(pid) = self.block_index_at(r_pos) {
            Some(&self.palette[pid as usize])
        } else {
            None
        };
    }
    /// Get block entity at `r_pos`
    fn block_entity_at(&self, r_pos: [i32; 3]) -> Option<&BlockEntity> {
        return self.block_entities.get(&r_pos);
    }

    /// Get pending tick at `r_pos`
    fn pending_tick_at(&self, r_pos: [i32; 3]) -> Option<&PendingTick> {
        return self.pending_ticks.get(&r_pos);
    }
}

#[allow(dead_code)]
impl Region {
    /// Convert pos from xyz to yzx
    pub fn pos_xyz_to_yzx<T>(pos: &[T; 3]) -> [T; 3]
        where T: Copy {
        return [pos[1], pos[2], pos[0]];
    }

    /// Convert pos from yzx to xyz
    pub fn pos_yzx_to_xyz<T>(yzx: &[T; 3]) -> [T; 3]
        where T: Copy {
        return [yzx[2], yzx[0], yzx[1]];
    }

    /// Create a new region with size \[1,1,1\], filled with air
    pub fn new() -> Region {
        let mut result = Region {
            name: String::from("NewRegion"),
            array_yzx: Array3::zeros([1, 1, 1]),
            palette: Vec::new(),
            block_entities: HashMap::new(),
            pending_ticks: HashMap::new(),
            entities: Vec::new(),
            offset: [0, 0, 0],
        };
        result.find_or_append_to_palette(&Block::air());
        return result;
    }
    // pub fn array(&self) -> &Array3<u16> {
    //     return &self.array;
    // }
    // pub fn palette(&self) -> &[Block] {
    //     return &self.palette;
    // }
    // pub fn block_entities(&self) -> &HashMap<[i32; 3], BlockEntity> {
    //     return &self.block_entities;
    // }
    // pub fn pending_ticks(&self) -> &HashMap<[i32; 3], PendingTick> {
    //     return &self.pending_ticks;
    // }
    // pub fn entities(&self) -> &[Entity] {
    //     return &self.entities;
    // }
    // pub fn offset(&self) -> &[i32; 3] {
    //     return &self.offset;
    // }
    //
    // pub fn set_offset(&mut self, new_offset: [i32; 3]) {
    //     self.offset = new_offset;
    // }

    /// Convert pos in `[i32;3]` to `[usize;3]`
    pub fn i32_to_usize(pos: &[i32; 3]) -> [usize; 3] {
        let x = pos[0] as usize;
        let y = pos[1] as usize;
        let z = pos[2] as usize;

        return [x, y, z];
    }

    /// Set block as assigned position. `r_pos` is a relative pos in xyz. \
    /// If there's block in palette same as `block`, the palette won't change, otherwise `block`
    /// will be cloned and pushed to palette. \
    /// This function returns `Err(())` if block palette exceeds 65535, which seldom happens.
    pub fn set_block(&mut self, r_pos: [i32; 3], block: &Block) -> Result<(), ()> {
        if !self.contains_coord(r_pos) {
            return Err(());
        }
        let mut blkid = self.palette.len();
        for (idx, blk) in self.palette.iter().enumerate() {
            if blk == block {
                blkid = idx;
                break;
            }
        }
        if blkid >= self.palette.len() {
            self.palette.push(block.clone());
        }
        if blkid >= 65536 {
            return Err(());
        }
        let blkid = blkid as u16;

        let pos_usize = Self::i32_to_usize(&r_pos);
        self.array_yzx[Self::pos_xyz_to_yzx(&pos_usize)] = blkid;

        return Ok(());
    }

    /// Set block index as `r_pos`. If `block_id` >= length of palette, returns `Err(())`
    pub fn set_block_id(&mut self, r_pos: [i32; 3], block_id: u16) -> Result<(), ()> {
        if !self.contains_coord(r_pos) {
            return Err(());
        }
        if block_id as usize >= self.palette.len() {
            return Err(());
        }
        let pos_usize = Self::i32_to_usize(&r_pos);
        self.array_yzx[Self::pos_xyz_to_yzx(&pos_usize)] = block_id;
        return Ok(());
    }

    /// Reshape the region and fill `array_yzx` with 0
    pub fn reshape(&mut self, size_xyz: &[i32; 3]) {
        let mut usz: [usize; 3] = [0, 0, 0];
        for idx in 0..3 {
            let sz = size_xyz[idx];
            if sz < 0 {
                panic!("Try resizing with negative size [{},{},{}]", size_xyz[0], size_xyz[1], size_xyz[2]);
            }
            usz[idx] = sz as usize;
        }
        let shape_yzx = Self::pos_xyz_to_yzx(&usz);
        self.array_yzx = Array3::zeros(shape_yzx);
    }

    /// Shape in y, z, x
    pub fn shape_yzx(&self) -> [i32; 3] {
        let shape = self.array_yzx.shape();
        if shape.len() != 3 {
            panic!("Invalid array dimensions: should be 3 but now it is {}", shape.len());
        }
        return [shape[0] as i32, shape[1] as i32, shape[2] as i32];
    }

    /// Returns the block index of air in this region
    pub fn block_index_of_air(&self) -> Option<u16> {
        for (idx, blk) in self.palette.iter().enumerate() {
            if blk.is_air() {
                return Some(idx as u16);
            }
        }
        return None;
    }

    /// Returns the block index of structure void in this region
    pub fn block_index_of_structure_void(&self) -> Option<u16> {
        for (idx, blk) in self.palette.iter().enumerate() {
            if blk.is_structure_void() {
                return Some(idx as u16);
            }
        }
        return None;
    }
    /// Convert global position to relative position. `r_pos` = `g_pos` - `self.offset`
    pub fn global_pos_to_relative_pos(&self, g_pos: [i32; 3]) -> [i32; 3] {
        return [
            g_pos[0] - self.offset[0],
            g_pos[1] - self.offset[1],
            g_pos[2] - self.offset[2],
        ];
    }

    /// Convert relative position to global position. `g_pos` = `r_pos` + `self.offset`
    pub fn relative_pos_to_global_pos(&self, r_pos: [i32; 3]) -> [i32; 3] {
        return [
            r_pos[0] + self.offset[0],
            r_pos[1] + self.offset[1],
            r_pos[2] + self.offset[2],
        ];
    }
    /// Remove non-existing blocks from palette. Returns error if there is any block index that is
    /// equal or greater than length of palette
    pub fn shrink_palette(&mut self) -> Result<(), Error> {
        let mut block_counter: Vec<u64> = Vec::new();
        block_counter.resize(self.palette.len(), 0);

        for x in 0..self.shape()[0] {
            for y in 0..self.shape()[1] {
                for z in 0..self.shape()[2] {
                    let idx = self.array_yzx[[y as usize, z as usize, x as usize]];
                    if idx as usize >= self.palette.len() {
                        return Err(Error::BlockIndexOfOfRange {
                            r_pos: [x, y, z],
                            block_index: idx,
                            max_index: self.palette.len() as u16 - 1,
                        });
                    }
                    block_counter[idx as usize] += 1;
                }
            }
        }

        let mut id_map: Vec<u16> = Vec::new();
        id_map.resize(self.palette.len(), 65535);
        {
            let mut counter: u16 = 0;
            for id in 0..self.palette.len() {
                if block_counter[id] <= 0 {
                    continue;
                }
                id_map[id] = counter;
                counter += 1;
            }
            for id in (0..block_counter.len()).rev() {
                if block_counter[id] <= 0 {
                    self.palette.remove(id);
                }
            }
        }
        for blkid in &mut self.array_yzx {
            let new_id = id_map[*blkid as usize];
            assert!((new_id as usize) < self.palette.len());
            *blkid = new_id;
        }

        return Ok(());
    }

    /// Find the block index of a block in palette
    pub fn find_in_palette(&self, block: &Block) -> Option<u16> {
        for (idx, blk) in self.palette.iter().enumerate() {
            if blk == block {
                return Some(idx as u16);
            }
        }
        return None;
    }

    /// Find the block in palette, if not exist, append it to the palette.
    pub fn find_or_append_to_palette(&mut self, block: &Block) -> u16 {
        return match self.find_in_palette(block) {
            Some(idx) => idx,
            None => {
                self.palette.push(block.clone());
                (self.palette.len() - 1) as u16
            }
        }
    }
    /// Fill the region with block
    pub fn fill_with(&mut self, block: &Block) {
        let blk_id = self.find_or_append_to_palette(block);
        self.array_yzx.fill(blk_id);
    }

    /// Set block entity at `r_pos`
    pub fn set_block_entity_at(&mut self, r_pos: [i32; 3], be: BlockEntity) -> Option<BlockEntity> {
        return self.block_entities.insert(r_pos, be);
    }
    /// Set pending tick at `r_pos`
    pub fn set_pending_tick_at(&mut self, r_pos: [i32; 3], value: PendingTick) -> Option<PendingTick> {
        return self.pending_ticks.insert(r_pos, value);
    }

    /// Returns detailed block infos at `r_pos`, including block index, block, block entity(mutable) and pending tick(mutable).
    /// Returns `None` if the block is outside the region
    pub fn block_info_at_mut(&mut self, r_pos: [i32; 3]) -> Option<(u16, &Block, Option<&mut BlockEntity>, Option<&mut PendingTick>)> {
        return if let Some(pid) = self.block_index_at(r_pos) {
            Some((pid, &self.palette[pid as usize],
                  self.block_entities.get_mut(&r_pos),
                  self.pending_ticks.get_mut(&r_pos)))
        } else {
            None
        };
    }
    /// Get mutable block entity at `r_pos`
    pub fn block_entity_at_mut(&mut self, r_pos: [i32; 3]) -> Option<&mut BlockEntity> {
        return self.block_entities.get_mut(&r_pos);
    }

    /// Get mutable pending tick at `r_pos`
    pub fn pending_tick_at_mut(&mut self, r_pos: [i32; 3]) -> Option<&mut PendingTick> {
        return self.pending_ticks.get_mut(&r_pos);
    }
}