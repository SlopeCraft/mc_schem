use std::collections::HashMap;
use ndarray::Array3;
use crate::block::Block;
use crate::error::Error;

#[derive(Debug, Clone)]
pub struct Entity {
    pub tags: HashMap<String, fastnbt::Value>,
    pub position: [f64; 3],
    pub block_pos: [i32; 3],
}

#[derive(Debug, Clone)]
pub struct BlockEntity {
    pub tags: HashMap<String, fastnbt::Value>,
}


#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum PendingTickInfo {
    Fluid { id: String },
    Block { id: String },
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct PendingTick {
    pub priority: i32,
    pub sub_tick: i64,
    pub time: i32,
    pub info: PendingTickInfo,
}

#[derive(Debug, Clone)]
pub struct Region {
    pub name: String,
    //XYZ
    pub array: Array3<u16>,
    pub palette: Vec<Block>,
    pub block_entities: HashMap<[i32; 3], BlockEntity>,
    pub pending_ticks: HashMap<[i32; 3], PendingTick>,
    pub entities: Vec<Entity>,

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

#[allow(dead_code)]
impl Region {
    pub fn new() -> Region {
        let mut result = Region {
            name: String::from("NewRegion"),
            array: Array3::zeros([1, 1, 1]),
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

    pub fn i32_to_usize(pos: &[i32; 3]) -> [usize; 3] {
        let x = pos[0] as usize;
        let y = pos[1] as usize;
        let z = pos[2] as usize;

        return [x, y, z];
    }

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
        self.array[pos_usize] = blkid;

        return Ok(());
    }

    pub fn set_block_id(&mut self, r_pos: [i32; 3], block_id: u16) -> Result<(), ()> {
        if !self.contains_coord(r_pos) {
            return Err(());
        }
        if block_id as usize >= self.palette.len() {
            return Err(());
        }
        let pos_usize = Self::i32_to_usize(&r_pos);
        self.array[pos_usize] = block_id;
        return Ok(());
    }

    pub fn reshape(&mut self, size: &[i32; 3]) {
        let mut usz: [usize; 3] = [0, 0, 0];
        for idx in 0..3 {
            let sz = size[idx];
            if sz < 0 {
                panic!("Try resizing with negative size [{},{},{}]", size[0], size[1], size[2]);
            }
            usz[idx] = sz as usize;
        }
        self.array = Array3::zeros(usz);
    }
    pub fn shape(&self) -> [i32; 3] {
        let shape = self.array.shape();
        if shape.len() != 3 {
            panic!("Invalid array dimensions: should be 3 but now it is {}", shape.len());
        }
        return [shape[0] as i32, shape[1] as i32, shape[2] as i32];
    }

    pub fn volume(&self) -> u64 {
        return self.array.shape()[0] as u64 * self.array.shape()[1] as u64 * self.array.shape()[2] as u64;
    }

    pub fn block_index_of_air(&self) -> Option<u16> {
        for (idx, blk) in self.palette.iter().enumerate() {
            if blk.is_air() {
                return Some(idx as u16);
            }
        }
        return None;
    }

    pub fn block_index_of_structure_void(&self) -> Option<u16> {
        for (idx, blk) in self.palette.iter().enumerate() {
            if blk.is_structure_void() {
                return Some(idx as u16);
            }
        }
        return None;
    }

    pub fn total_blocks(&self, include_air: bool) -> u64 {
        let mut counter = 0;

        for blk_id in &self.array {
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
    pub fn contains_coord(&self, r_pos: [i32; 3]) -> bool {
        for dim in 0..3 {
            if r_pos[dim] >= 0 && r_pos[dim] <= self.shape()[dim] {
                continue;
            }
            return false;
        }
        return true;
    }
    pub fn block_at(&self, r_pos: [i32; 3]) -> Option<&Block> {
        return if let Some(pid) = self.block_index_at(r_pos) {
            Some(&self.palette[pid as usize])
        } else {
            None
        };
    }

    pub fn block_index_at(&self, r_pos: [i32; 3]) -> Option<u16> {
        if !self.contains_coord(r_pos) {
            return None;
        }

        let x = r_pos[0] as usize;
        let y = r_pos[1] as usize;
        let z = r_pos[2] as usize;

        let pid = self.array[[x, y, z]] as usize;
        return Some(pid as u16);
    }

    pub fn block_info_at(&self, r_pos: [i32; 3]) -> Option<(u16, &Block, Option<&BlockEntity>, Option<&PendingTick>)> {
        return if let Some(pid) = self.block_index_at(r_pos) {
            Some((pid, &self.palette[pid as usize],
                  self.block_entities.get(&r_pos),
                  self.pending_ticks.get(&r_pos)))
        } else {
            None
        };
    }

    pub fn global_pos_to_relative_pos(&self, g_pos: [i32; 3]) -> [i32; 3] {
        return [
            g_pos[0] - self.offset[0],
            g_pos[1] - self.offset[1],
            g_pos[2] - self.offset[2],
        ];
    }

    pub fn relative_pos_to_global_pos(&self, r_pos: [i32; 3]) -> [i32; 3] {
        return [
            r_pos[0] + self.offset[0],
            r_pos[1] + self.offset[1],
            r_pos[2] + self.offset[2],
        ];
    }

    pub fn shrink_palette(&mut self) -> Result<(), Error> {
        let mut block_counter: Vec<u64> = Vec::new();
        block_counter.resize(self.palette.len(), 0);

        for x in 0..self.shape()[0] {
            for y in 0..self.shape()[1] {
                for z in 0..self.shape()[2] {
                    let idx = self.array[[x as usize, y as usize, z as usize]];
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
        for blkid in &mut self.array {
            let new_id = id_map[*blkid as usize];
            assert!((new_id as usize) < self.palette.len());
            *blkid = new_id;
        }

        return Ok(());
    }

    pub fn find_in_palette(&self, block: &Block) -> Option<u16> {
        for (idx, blk) in self.palette.iter().enumerate() {
            if blk == block {
                return Some(idx as u16);
            }
        }
        return None;
    }

    pub fn find_or_append_to_palette(&mut self, block: &Block) -> u16 {
        let blk_idx = self.palette.len();
        for (idx, blk) in self.palette.iter().enumerate() {
            if blk == block {
                return idx as u16;
            }
        }
        if blk_idx >= self.palette.len() {
            self.palette.push(block.clone());
        }
        return self.palette.len() as u16;
    }

    pub fn fill_with(&mut self, block: &Block) {
        let blk_id = self.find_or_append_to_palette(block);
        self.array.fill(blk_id);
    }

    pub fn block_entity_at(&self, r_pos: [i32; 3]) -> Option<&BlockEntity> {
        return self.block_entities.get(&r_pos);
    }
    pub fn block_entity_at_mut(&mut self, r_pos: [i32; 3]) -> Option<&mut BlockEntity> {
        return self.block_entities.get_mut(&r_pos);
    }

    pub fn set_block_entity_at(&mut self, r_pos: [i32; 3], be: BlockEntity) -> Option<BlockEntity> {
        return self.block_entities.insert(r_pos, be);
    }

    pub fn pending_tick_at(&self, r_pos: [i32; 3]) -> Option<&PendingTick> {
        return self.pending_ticks.get(&r_pos);
    }
    pub fn pending_tick_at_mut(&mut self, r_pos: [i32; 3]) -> Option<&mut PendingTick> {
        return self.pending_ticks.get_mut(&r_pos);
    }
    pub fn set_pending_tick_at(&mut self, r_pos: [i32; 3], value: PendingTick) -> Option<PendingTick> {
        return self.pending_ticks.insert(r_pos, value);
    }
}