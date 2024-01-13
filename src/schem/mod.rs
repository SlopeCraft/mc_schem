mod world_edit12;
mod world_edit13;
pub(crate) mod litematica;

mod vanilla_structure;
mod mc_version;
pub(crate) mod common;

use std::cmp::max;
use std::collections::hash_map::DefaultHasher;
//mod schem {
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use ndarray::Array3;
use crate::block::{Block, CommonBlock};
use fastnbt;
use crate::error::WriteError;
//use schem::mc_version;
use crate::schem;


#[derive(Debug, Clone)]
pub struct BlockEntity {
    pub tags: HashMap<String, fastnbt::Value>,
}

impl BlockEntity {
    pub fn new() -> BlockEntity {
        return BlockEntity {
            tags: HashMap::new(),
        };
    }
}

#[derive(Debug, Clone)]
pub struct Entity {
    pub tags: HashMap<String, fastnbt::Value>,
    pub position: [f64; 3],
    pub block_pos: [i32; 3],
}

impl Entity {
    pub fn new() -> Entity {
        return Entity {
            tags: HashMap::new(),
            position: [0.0, 0.0, 0.0],
            block_pos: [0, 0, 0],
        };
    }
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum PendingTickInfo {
    Fluid { id: String },
    Block { id: String },
}

impl PendingTickInfo {
    pub fn default() -> PendingTickInfo {
        return PendingTickInfo::Block { id: "".to_string() };
    }
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct PendingTick {
    pub priority: i32,
    pub sub_tick: i64,
    pub time: i32,
    pub info: PendingTickInfo,
}

#[derive(Debug)]
pub struct Region {
    pub name: String,
    //XYZ
    pub array: Array3<u16>,
    pub palette: Vec<Block>,
    pub block_entities: HashMap<[i32; 3], BlockEntity>,
    pub pending_ticks: HashMap<[i32; 3], PendingTick>,
    pub entities: Vec<Entity>,

    pub offset: [i32; 3],
}

impl Region {
    pub fn new() -> Region {
        return Region {
            name: String::from("NewRegion"),
            array: Array3::zeros([1, 1, 1]),
            palette: Vec::new(),
            block_entities: HashMap::new(),
            pending_ticks: HashMap::new(),
            entities: Vec::new(),
            offset: [0, 0, 0],
        };
    }

    pub fn reshape(&mut self, size: [i32; 3]) {
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
    pub fn contains_coord(&self, coord: [i32; 3]) -> bool {
        for dim in 0..3 {
            if coord[dim] >= 0 && coord[dim] <= self.shape()[dim] {
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
        }
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

    pub fn block_info_at(&self, r_pos: [i32; 3]) -> Option<(u16, &Block, Option<&BlockEntity>)> {
        return if let Some(pid) = self.block_index_at(r_pos) {
            Some((pid, &self.palette[pid as usize], self.block_entities.get(&r_pos)))
        } else {
            None
        }
    }

    pub fn global_pos_to_relative_pos(&self, g_pos: [i32; 3]) -> [i32; 3] {
        return [
            g_pos[0] - self.offset[0],
            g_pos[1] - self.offset[1],
            g_pos[2] - self.offset[2],
        ]
    }

    pub fn shrink_palette(&mut self) -> Result<(), WriteError> {
        let mut block_counter: Vec<u64> = Vec::new();
        block_counter.resize(self.palette.len(), 0);

        for x in 0..self.shape()[0] {
            for y in 0..self.shape()[1] {
                for z in 0..self.shape()[2] {
                    let idx = self.array[[x as usize, y as usize, z as usize]];
                    if idx as usize >= self.palette.len() {
                        return Err(WriteError::BlockIndexOfOfRange {
                            r_pos: [x, y, z],
                            block_index: idx,
                            max_index: self.palette.len() as u16 - 1,
                        })
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
}

#[derive(Debug, Clone)]
pub struct LitematicaMetaData {
    pub data_version: i32,

    pub version: i32,
    pub sub_version: Option<i32>,
    pub time_created: i64,
    pub time_modified: i64,
    pub author: String,
    pub name: String,
    pub description: String,
    //pub total_volume: i64,
}

impl LitematicaMetaData {
    pub fn new() -> LitematicaMetaData {
        return LitematicaMetaData {
            data_version: mc_version::DataVersion::new() as i32,
            version: 5,
            sub_version: None,
            time_created: 0,
            time_modified: 0,
            author: String::from("mc_schem.rs"),
            name: String::from("Default litematica"),
            description: String::from("Default litematica generated by mc_schem.rs"),
            //total_volume: 0,
        };
    }
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct WE12MetaData {}

#[allow(dead_code)]
impl WE12MetaData {
    pub fn new() -> WE12MetaData {
        return WE12MetaData {};
    }
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct WE13MetaData {
    pub data_version: i32,
    pub version: i32,
    pub we_offset: [i32; 3],
    pub offset: [i32; 3],
}

#[allow(dead_code)]
impl WE13MetaData {
    pub fn new() -> WE13MetaData {
        return WE13MetaData {
            data_version: mc_version::DataVersion::new() as i32,
            version: 5,
            we_offset: [0, 0, 0],
            offset: [0, 0, 0],
        };
    }
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct VanillaStructureMetaData {
    pub data_version: i32,
}

impl VanillaStructureMetaData {
    pub fn new() -> VanillaStructureMetaData {
        return VanillaStructureMetaData {
            data_version: mc_version::DataVersion::new() as i32,
        };
    }
}


#[derive(Debug)]
#[allow(dead_code)]
pub enum RawMetaData {
    Litematica(LitematicaMetaData),
    WE12(WE12MetaData),
    WE13(WE13MetaData),
    VanillaStructure(VanillaStructureMetaData),
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct MetaDataIR {
    pub mc_data_version: i32,

    pub time_created: i64,
    pub time_modified: i64,
    pub author: String,
    pub name: String,
    pub description: String,

    //pub raw_metadata: Option<MetaData>,
}

#[allow(dead_code)]
impl MetaDataIR {
    pub fn default() -> MetaDataIR {
        use std::time::{SystemTime, UNIX_EPOCH};
        let time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as i64;
        return MetaDataIR {
            mc_data_version: mc_version::DataVersion::new() as i32,
            time_created: time,
            time_modified: time,
            author: String::from("mc_schem"),
            name: String::from("DefaultMetaDataIR"),
            description: String::from("Default metadata generated by mc_schem"),
        }
    }
}

#[derive(Debug)]
pub struct Schematic {
    pub metadata: MetaDataIR,
    raw_metadata: Option<RawMetaData>,

    pub regions: Vec<Region>,
    pub enclosing_size: [i64; 3],

}


// enum SchemFormat {
//     Litematica,
//     WorldEdit12,
//     WorldEdit13,
//     VanillaStructure,
// }
#[allow(dead_code)]
impl Schematic {
    pub fn new() -> Schematic {
        return Schematic {
            //data_version: mc_version::DataVersion::new() as i32,
            metadata: MetaDataIR::default(),
            raw_metadata: None,
            regions: Vec::new(),
            enclosing_size: [1, 1, 1],

        };
    }

    pub fn block_indices_at(&self, g_pos: [i32; 3]) -> Vec<u16> {
        let mut result = Vec::with_capacity(self.regions.len());
        for reg in &self.regions {
            let cur_pos = reg.global_pos_to_relative_pos(g_pos);
            if let Some(blk) = reg.block_index_at(cur_pos) {
                result.push(blk);
            }
        }
        return result;
    }

    pub fn blocks_at(&self, pos: [i32; 3]) -> Vec<&Block> {
        let mut result = Vec::with_capacity(self.regions.len());
        for reg in &self.regions {
            let cur_pos = reg.global_pos_to_relative_pos(pos);
            if let Some(blk) = reg.block_at(cur_pos) {
                result.push(blk);
            }
        }
        return result;
    }

    pub fn block_entities_at(&self, pos: [i32; 3]) -> Vec<&BlockEntity> {
        let mut result = Vec::with_capacity(self.regions.len());
        for reg in &self.regions {
            let cur_pos = reg.global_pos_to_relative_pos(pos);
            if let Some(blk) = reg.block_entities.get(&cur_pos) {
                result.push(blk);
            }
        }
        return result;
    }


    pub fn first_block_index_at(&self, pos: [i32; 3]) -> Option<u16> {
        if self.regions.is_empty() {
            return None;
        }
        let reg = &self.regions[0];
        return reg.block_index_at(
            reg.global_pos_to_relative_pos(pos));
    }
    pub fn first_block_at(&self, pos: [i32; 3]) -> Option<&Block> {
        if self.regions.is_empty() {
            return None;
        }
        let reg = &self.regions[0];
        return reg.block_at(reg.global_pos_to_relative_pos(pos));
    }
    pub fn first_block_entity_at(&self, pos: [i32; 3]) -> Option<&BlockEntity> {
        if self.regions.is_empty() {
            return None;
        }
        let reg = &self.regions[0];
        return reg.block_entities.get(&reg.global_pos_to_relative_pos(pos));
    }

    pub fn shape(&self) -> [i32; 3] {
        let mut result = [0, 0, 0];
        for reg in &self.regions {
            for dim in 0..3 {
                result[dim] = max(result[dim], reg.offset[dim] + reg.shape()[dim]);
            }
        }
        return result;
    }

    pub fn volume(&self) -> u64 {
        let mut result: u64 = 1;
        for sz in self.shape() {
            result *= sz as u64;
        }
        return result;
    }

    pub fn total_blocks(&self, include_air: bool) -> u64 {
        let mut counter = 0;
        for reg in &self.regions {
            counter += reg.total_blocks(include_air);
        }
        return counter;
    }


    pub fn full_palette(&self) -> (Vec<(&Block, u64)>, Vec<Vec<usize>>) {
        let possible_max_palette_size;
        {
            let mut pmps: usize = 0;
            for reg in &self.regions {
                pmps = max(pmps, reg.palette.len());
            }
            possible_max_palette_size = pmps;
        }

        let mut palette: Vec<(&Block, u64)> = Vec::with_capacity(possible_max_palette_size);
        let mut lut_lut: Vec<Vec<usize>> = Vec::with_capacity(self.regions.len());
        for reg in &self.regions {
            let mut lut: Vec<usize> = Vec::with_capacity(reg.palette.len());

            for cur_blk in &reg.palette {
                let mut hasher = DefaultHasher::new();
                cur_blk.hash(&mut hasher);
                let cur_hash = hasher.finish();

                let mut cur_block_index_in_full_palette = palette.len();
                for (idx, (blk, hash)) in palette.iter().enumerate() {
                    if *hash != cur_hash {
                        continue;
                    }
                    if *blk != cur_blk {
                        continue;
                    }
                    cur_block_index_in_full_palette = idx;
                    break;
                }

                if cur_block_index_in_full_palette >= palette.len() {
                    palette.push((cur_blk, cur_hash));
                }
                lut.push(cur_block_index_in_full_palette);
            }
            lut_lut.push(lut);
        }
        return (palette, lut_lut);
    }
}

pub fn id_of_nbt_tag(tag: &fastnbt::Value) -> u8 {
    return match tag {
        fastnbt::Value::Byte(_) => 1,
        fastnbt::Value::Short(_) => 2,
        fastnbt::Value::Int(_) => 3,
        fastnbt::Value::Long(_) => 4,
        fastnbt::Value::Float(_) => 5,
        fastnbt::Value::Double(_) => 6,
        fastnbt::Value::ByteArray(_) => 7,
        fastnbt::Value::String(_) => 8,
        fastnbt::Value::List(_) => 9,
        fastnbt::Value::Compound(_) => 10,
        fastnbt::Value::IntArray(_) => 11,
        fastnbt::Value::LongArray(_) => 12,
    }
}

#[derive(Debug)]
pub struct VanillaStructureLoadOption {
    pub background_block: CommonBlock,
}

impl VanillaStructureLoadOption {
    pub fn default() -> VanillaStructureLoadOption {
        return VanillaStructureLoadOption {
            background_block: CommonBlock::StructureVoid
        }
    }
}

#[derive(Debug)]
pub struct VanillaStructureSaveOption {
    pub keep_air: bool,
}

impl VanillaStructureSaveOption {
    pub fn default() -> VanillaStructureSaveOption {
        return VanillaStructureSaveOption {
            keep_air: true
        }
    }
}

#[derive(Debug)]
pub struct LitematicaLoadOption {}

impl LitematicaLoadOption {
    pub fn default() -> LitematicaLoadOption {
        return LitematicaLoadOption {};
    }
}


#[derive(Debug)]
pub struct LitematicaSaveOption {
    rename_duplicated_regions: bool,
}

impl LitematicaSaveOption {
    pub fn default() -> LitematicaSaveOption {
        return LitematicaSaveOption {
            rename_duplicated_regions: true,
        };
    }
}