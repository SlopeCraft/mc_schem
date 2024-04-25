use std::collections::{BTreeMap, HashMap};
use std::fmt::{Display, Formatter};
use std::ops::Range;
use std::time;

use fastnbt::Value;
use math::round::{ceil, floor};

use crate::{unwrap_opt_tag, unwrap_tag};
use crate::biome::Biome;
use crate::error::Error;
use crate::region::{Light, PendingTick, PendingTickInfo, WorldSlice};
use crate::schem::common;
use crate::schem::common::ceil_up_to;
use crate::schem::id_of_nbt_tag;
use crate::world::{Chunk, ChunkPos, ChunkRefAbsolutePos, ChunkRefRelativePos, ChunkStatus, NBTWithSource, SubChunk};

impl Display for ChunkStatus {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        return write!(f, "minecraft:{}", self.name_without_namespace());
    }
}

impl ChunkStatus {
    pub fn name_without_namespace(&self) -> &'static str {
        return match self {
            ChunkStatus::Empty => "empty",
            ChunkStatus::StructureStarts => "structure_starts",
            ChunkStatus::StructureReferences => "structure_references",
            ChunkStatus::Biomes => "biomes",
            ChunkStatus::Noise => "noise",
            ChunkStatus::Surface => "surface",
            ChunkStatus::Carvers => "carvers",
            ChunkStatus::Features => "features",
            ChunkStatus::InitializeLight => "initialize_light",
            ChunkStatus::Light => "light",
            ChunkStatus::Spawn => "spawn",
            ChunkStatus::Full => "full",
        };
    }

    fn all() -> &'static [ChunkStatus] {
        return &[
            ChunkStatus::Empty,
            ChunkStatus::StructureStarts,
            ChunkStatus::StructureReferences,
            ChunkStatus::Biomes,
            ChunkStatus::Noise,
            ChunkStatus::Surface,
            ChunkStatus::Carvers,
            ChunkStatus::Features,
            ChunkStatus::InitializeLight,
            ChunkStatus::Light,
            ChunkStatus::Spawn,
            ChunkStatus::Full, ];
    }

    pub fn from_str(str: &str) -> Option<ChunkStatus> {
        if str.starts_with("minecraft:") {
            return Self::from_str(&str[10..str.len()]);
        }

        for cs in Self::all() {
            if str == cs.name_without_namespace() {
                return Some(*cs);
            }
        }
        return None;
    }
}

fn parse_pending_tick(nbt: &HashMap<String, Value>, is_block_tick: bool, tag_path: &str) -> Result<([i32; 3], PendingTick), Error> {
    let pos = common::parse_size_compound(nbt, tag_path, true)?;
    let id = unwrap_opt_tag!(nbt.get("i"),String,"".to_string(),format!("{tag_path}/i"));
    let p = *unwrap_opt_tag!(nbt.get("p"),Int,0,format!("{tag_path}/p"));
    let t = *unwrap_opt_tag!(nbt.get("t"),Int,0,format!("{tag_path}/t"));

    let info = if is_block_tick {
        PendingTickInfo::Block { id: id.clone() }
    } else {
        PendingTickInfo::Fluid { id: id.clone() }
    };
    let tick = PendingTick {
        priority: p,
        time: t,
        sub_tick: 0,
        info,
    };

    return Ok((pos, tick));
}

impl Chunk {
    pub fn new() -> Chunk {
        return Chunk {
            time_stamp: time::SystemTime::now().duration_since(time::UNIX_EPOCH).unwrap().as_secs() as u32,
            status: ChunkStatus::Empty,
            last_update: 0,
            inhabited_time: 0,
            is_light_on: true,
            sub_chunks: BTreeMap::new(),
            entities: vec![],
            block_entities: HashMap::new(),
            pending_ticks: HashMap::new(),
            file_region: "NoFile".to_string(),
            file_entities: "NoFile".to_string(),
        };
    }

    pub fn height(&self) -> i32 {
        debug_assert!(self.missing_sub_chunks().is_empty());
        return self.sub_chunks.len() as i32 * 16;
    }

    pub fn from_nbt(region_nbt_data: NBTWithSource, entity_nbt_data: Option<NBTWithSource>, chunk_pos: &ChunkPos) -> Result<Chunk, Error> {
        let region_source_filename = region_nbt_data.source;
        let mut region_nbt = region_nbt_data.nbt;

        let path_in_saves = format!("{region_source_filename}/[{},{}]",
                                    chunk_pos.local_coordinate().x,
                                    chunk_pos.local_coordinate().z);
        let mut result = Chunk::new();
        result.file_region = region_source_filename.to_string();
        // chunk status
        {
            let status: ChunkStatus;
            let str = unwrap_opt_tag!(region_nbt.get("Status"),String,"".to_string(),format!("{path_in_saves}/Status"));
            match ChunkStatus::from_str(str) {
                Some(s) => status = s,
                None => {
                    return Err(Error::InvalidChunkStatus {
                        tag_path: format!("{path_in_saves}/Status"),
                        chunk_status: str.to_string(),
                    });
                }
            };
            result.status = status;
        }
        result.last_update = *unwrap_opt_tag!(region_nbt.get("LastUpdate"),Long,0,format!("{path_in_saves}/LastUpdate"));
        result.inhabited_time = *unwrap_opt_tag!(region_nbt.get("InhabitedTime"),Long,0,format!("{path_in_saves}/InhabitedTime"));
        if let Some(tag) = region_nbt.get("isLightOn") {
            result.is_light_on = *unwrap_tag!(tag,Byte,1,format!("{path_in_saves}/isLightOn")) != 0;
        }

        let sections = unwrap_opt_tag!(region_nbt.get_mut("sections"),List,vec![],format!("{path_in_saves}/sections"));

        for (idx, nbt) in sections.iter_mut().enumerate() {
            let path = format!("{path_in_saves}/sections[{idx}]");
            let sect_nbt = unwrap_tag!(nbt,Compound,HashMap::new(),path);
            let opt = parse_section(sect_nbt, &path)?;
            if let Some((sub_chunk, y)) = opt {
                result.sub_chunks.insert(y, sub_chunk);
            }
        }
        // check for missing sub chunks
        {
            let missing = result.missing_sub_chunks();
            if missing.len() > 0 {
                return Err(Error::MissingSubChunk {
                    tag_path: format!("{path_in_saves}"),
                    sub_chunk_y: missing,
                });
            }
        }

        let pos_lb = chunk_pos.block_pos_lower_bound();
        let pos_ub = chunk_pos.block_pos_upper_bound();
        // block entities
        {
            let be_list_tag = format!("{path_in_saves}/block_entities");
            let mut be_list = unwrap_opt_tag!(region_nbt.remove("block_entities"),List,vec![],be_list_tag);

            let pos_lb = [pos_lb[0], result.y_range().start, pos_lb[1]];
            let pos_ub = [pos_ub[0], result.y_range().end - 1, pos_ub[1]];

            result.block_entities.reserve(be_list.len());

            for (idx, nbt) in be_list.iter_mut().enumerate() {
                let mut temp = Value::Byte(0);
                std::mem::swap(&mut temp, nbt);
                let be_nbt_tag = format!("{path_in_saves}/block_entities/[{idx}]");
                let be_nbt = unwrap_tag!(temp,Compound,HashMap::new(),be_nbt_tag);
                let (pos, be) = common::parse_block_entity_nocheck(be_nbt, &be_nbt_tag, true)?;
                if !common::check_pos_in_range(pos, pos_lb, pos_ub) {
                    return Err(Error::BlockPosOutOfRange {
                        tag_path: be_nbt_tag,
                        pos,
                        lower_bound: pos_lb,
                        upper_bound: pos_ub,
                    });
                }
                if result.block_entities.contains_key(&pos) {
                    return Err(Error::MultipleBlockEntityInOnePos {
                        pos,
                        latter_tag_path: be_nbt_tag,
                    });
                }
                result.block_entities.insert(pos, be);
            }
        }
        // pending ticks
        {
            //let mut tick_tag_record = HashMap::new();
            let tag_path_block_ticks = format!("{path_in_saves}/block_ticks");
            let tag_block_ticks = unwrap_opt_tag!(region_nbt.get("block_ticks"),List,vec![],tag_path_block_ticks);


            let tag_path_fluid_ticks = format!("{path_in_saves}/fluid_ticks");
            let tag_fluid_ticks = unwrap_opt_tag!(region_nbt.get("fluid_ticks"),List,vec![],tag_path_fluid_ticks);
            result.pending_ticks.reserve(tag_block_ticks.len() + tag_fluid_ticks.len());
            for (is_block, (list, tag_path)) in [(tag_block_ticks, tag_path_block_ticks),
                (tag_fluid_ticks, tag_path_fluid_ticks)].iter().enumerate() {
                let is_block = is_block == 0;
                for (idx, nbt) in list.iter().enumerate() {
                    let cur_tag_path = format!("{tag_path}[{idx}]");
                    let nbt = unwrap_tag!(nbt,Compound,HashMap::new(),cur_tag_path);
                    let (pos, tick) = parse_pending_tick(nbt, is_block, &cur_tag_path)?;

                    //if result.pending_ticks.contains_key(&pos) {
                    // let former = &result.pending_ticks[&pos];
                    // // If the pending tick is repeated, ignore the latter
                    // if &tick == former {
                    //     continue;
                    // }
                    // debug_assert!(tick_tag_record.contains_key(&pos));
                    // return Err(Error::MultiplePendingTickInOnePos {
                    //     pos,
                    //     former_tag_path: tick_tag_record.remove(&pos).unwrap(),
                    //     latter_tag_path: cur_tag_path,
                    // });
                    //}
                    if let Some(dst) = result.pending_ticks.get_mut(&pos) {
                        dst.push(tick);
                    } else {
                        result.pending_ticks.insert(pos, vec![tick]);
                    }
                    //tick_tag_record.insert(pos, cur_tag_path);
                }
            }
        }


        // entities
        if let Some(entity_nbt_data) = entity_nbt_data {
            let entity_source_file = entity_nbt_data.source;
            let mut entity_nbt = entity_nbt_data.nbt;

            result.file_entities = entity_source_file.to_string();
            let entity_path = format!("{entity_source_file}/[{},{}]/Entities",
                                      chunk_pos.local_coordinate().x,
                                      chunk_pos.local_coordinate().z);
            let mut entity_list = unwrap_opt_tag!(entity_nbt.remove("Entities"),List,vec![],entity_path);
            result.entities.reserve(entity_list.len());
            for (idx, entity) in entity_list.iter_mut().enumerate() {
                let cur_path = format!("{entity_path}/[{idx}]");
                let temp = unwrap_tag!(entity,Compound,HashMap::new(),cur_path);
                let mut entity = HashMap::new();
                std::mem::swap(&mut entity, temp);

                let entity = common::parse_entity_litematica(entity, &cur_path)?;
                // check for position
                let entity_pos_xz = [entity.block_pos[0], entity.block_pos[2]];
                for dim in 0..2 {
                    if entity_pos_xz[dim] < pos_lb[dim] || entity_pos_xz[dim] > pos_ub[dim] {
                        return Err(Error::BlockPosOutOfRange {
                            tag_path: format!("{cur_path}/Pos"),
                            pos: entity.block_pos,
                            lower_bound: [pos_lb[0], i32::MIN, pos_lb[1]],
                            upper_bound: [pos_ub[0], i32::MAX, pos_ub[1]],
                        });
                    }
                }

                result.entities.push(entity);
            }
        }

        return Ok(result);
    }

    fn missing_sub_chunks(&self) -> Vec<i8> {
        let mut max = i8::MIN;
        let mut min = i8::MAX;
        for (y, _) in &self.sub_chunks {
            max = max.max(*y);
            min = min.min(*y);
        }

        if max <= min { // zero or 1 elements
            return vec![];
        }
        if (max - min + 1) as usize == self.sub_chunks.len() {
            return vec![];
        }
        let mut missing = Vec::with_capacity((max - min + 1) as usize);
        for y in min..(max + 1) {
            if !self.sub_chunks.contains_key(&y) {
                missing.push(y);
            }
        }
        return missing;
    }

    pub fn y_range(&self) -> Range<i32> {
        debug_assert!(self.missing_sub_chunks().is_empty());

        let mut min = i8::MAX;
        for (y, _) in &self.sub_chunks {
            min = min.min(*y);
        }
        let y_min = min as i32 * 16;
        return y_min..(y_min + self.height());
    }
    pub fn y_offset(&self) -> i32 {
        return self.y_range().start;
    }


    pub fn shape(&self) -> [i32; 3] {
        return [16, self.height(), 16];
    }

    pub fn total_blocks(&self, include_air: bool) -> u64 {
        let mut num_blocks = 0;
        for (_, subchunk) in &self.sub_chunks {
            num_blocks += subchunk.total_blocks(include_air);
        }
        return num_blocks;
    }

    pub fn as_relative_pos(&self, chunk_pos: &ChunkPos) -> ChunkRefRelativePos {
        return ChunkRefRelativePos {
            chunk: self,
            chunk_pos: *chunk_pos,
        };
    }

    pub fn as_absolute_pos(&self, chunk_pos: &ChunkPos) -> ChunkRefAbsolutePos {
        return ChunkRefAbsolutePos {
            chunk: self,
            chunk_pos: *chunk_pos,
        };
    }

}

pub fn bits_per_block(block_types: usize, min_value: u8) -> u8 {
    return (ceil((block_types as f64).log2(), 0) as u8).max(min_value);
}

fn parse_blocks(reg: &mut SubChunk, sect: &HashMap<String, Value>, path: &str) -> Result<(), Error> {
    let block_states = unwrap_opt_tag!(sect.get("block_states"),Compound,HashMap::new(),format!("{path}/block_states"));

    {
        let palette = unwrap_opt_tag!(block_states.get("palette"),List,vec![],format!("{path}/block_states/palette"));
        let mut pal = Vec::with_capacity(palette.len());
        for (idx, blk) in palette.iter().enumerate() {
            let path = format!("{path}/block_states/palette[{idx}]");
            let blk = unwrap_tag!(blk,Compound,HashMap::new(),path);
            let blk = common::parse_block(blk, &path)?;
            pal.push(blk);
        }
        reg.palette = pal;
    }
    if reg.palette.len() <= 0 {
        return Err(Error::PaletteIsEmpty { tag_path: format!("{path}/block_states/palette") });
    }
    if reg.palette.len() > 65535 {
        return Err(Error::PaletteTooLong(reg.palette.len()));
    }
    // blocks
    if reg.palette.len() == 1 {
        reg.block_id_array.fill(0);
    } else {
        let path = format!("{path}/block_states/data");
        let array_i64 = unwrap_opt_tag!(block_states.get("data"),LongArray,fastnbt::LongArray::new(vec![]),path);

        let block_id_max = reg.palette.len() - 1;
        let bits_per_block = bits_per_block(reg.palette.len(), 4);
        let mut mbs = MultiBitSet::new(4096, bits_per_block);

        if array_i64.len() != mbs.num_u64() {
            return Err(Error::InvalidValue {
                tag_path: path,
                error: format!("This subchunk has 4096 blocks of {} types, required {} i64 element to store them, but found {}",
                               reg.palette.len(), mbs.num_u64(), array_i64.len()),
            });
        }
        mbs.set_array_from_nbt(&array_i64);

        for idx in 0..4096 {
            let blk_id = mbs.get(idx) as u16;
            if blk_id > block_id_max as u16 {
                return Err(Error::BlockIndexOutOfRange {
                    tag_path: format!("{path}/block_states/data"),
                    index: blk_id as i32,
                    range: [0, block_id_max as i32],
                });
            }
            reg.block_id_array[idx] = blk_id;
        }

    }
    return Ok(());
}

fn parse_biomes(reg: &mut SubChunk, sect: &HashMap<String, Value>, path: &str) -> Result<(), Error> {
    let biomes = unwrap_opt_tag!(sect.get("biomes"),Compound,HashMap::new(),format!("{path}/biomes"));
    // parse biome palette
    let mut biome_pal = Vec::new();
    {
        let tag_pal = unwrap_opt_tag!(biomes.get("palette"),List,vec![],format!("{path}/biomes/palette"));
        biome_pal.reserve(tag_pal.len());
        for (idx, biome_str) in tag_pal.iter().enumerate() {
            let tag_path = format!("{path}/biomes/palette[{idx}]");
            let biome_str = unwrap_tag!(biome_str,String,"".to_string(),tag_path);
            if let Some(b) = Biome::from_str(biome_str) {
                biome_pal.push(b);
            } else {
                return Err(Error::InvalidBiome { tag_path, biome: biome_str.to_string() });
            }
        }
    }

    if biome_pal.is_empty() {
        return Err(Error::PaletteIsEmpty { tag_path: format!("{path}/biomes/palette") });
    }
    if biome_pal.len() == 1 {
        reg.biome_array.fill(biome_pal[0]);
        return Ok(());
    }
    //parse 3d
    {
        let path = format!("{path}/biomes/data");
        let array_i64 = unwrap_opt_tag!(biomes.get("data"),LongArray,fastnbt::LongArray::new(vec![]),path);

        let block_id_max = biome_pal.len() - 1;
        let bits_per_block = bits_per_block(biome_pal.len(), 1);
        let mut mbs = MultiBitSet::new(64, bits_per_block);

        if array_i64.len() != mbs.num_u64() {
            // // If the biome is not initialized, this error can be processed
            // if chunk_status <= ChunkStatus::Biomes {
            //     reg.biome.fill(Biome::the_void);
            //     return Ok(());
            // }

            return Err(Error::InvalidValue {
                tag_path: path,
                error: format!("This subchunk has {} types of biome, required {} i64 element to store them, but found {}",
                               biome_pal.len(), mbs.num_u64(), array_i64.len()),
            });
        }
        mbs.set_array_from_nbt(&array_i64);

        for counter in 0..64 {
            let biome_idx = mbs.get(counter) as usize;
            if biome_idx > block_id_max {
                return Err(Error::BlockIndexOutOfRange {
                    tag_path: path,
                    index: biome_idx as i32,
                    range: [0, block_id_max as i32],
                });
            }
            reg.biome_array[counter] = biome_pal[biome_idx];
        }
    }
    return Ok(());
}

fn parse_section(sect: &HashMap<String, Value>, path: &str) -> Result<Option<(SubChunk, i8)>, Error> {
    let mut subchunk = SubChunk::new();
    // let reg = &mut subchunk.region;

    // y
    let y_pos = *unwrap_opt_tag!(sect.get("Y"),Byte,0,format!("{path}/Y"));

    if y_pos >= 20 || y_pos <= -5 {
        return Ok(None);
    }
    // block entities
    if let Some(be) = sect.get("block_entities") {
        let be = unwrap_tag!(be,List,vec![],format!("{path}/Entities"));
        if !be.is_empty() {
            println!("{} block entities in {path}", be.len());
        }
    }


    // palette
    parse_blocks(&mut subchunk, sect, path)?;

    // skylight and block light
    {
        let sky_light = if let Some(s) = sect.get("SkyLight") {
            let tag_path = format!("{path}/SkyLight");
            let arr = unwrap_tag!(s,ByteArray,fastnbt::ByteArray::new(vec![]),tag_path).as_ref();
            if arr.len() != 2048 {
                return Err(Error::InvalidValue { tag_path, error: format!("The length should be 2048, but found {}", arr.len()) });
            }
            arr
        } else {
            &[]
        };
        let block_light = if let Some(s) = sect.get("BlockLight") {
            let tag_path = format!("{path}/BlockLight");
            let arr = unwrap_tag!(s,ByteArray,fastnbt::ByteArray::new(vec![]),tag_path).as_ref();
            if arr.len() != 2048 {
                return Err(Error::InvalidValue { tag_path, error: format!("The length should be 2048, but found {}", arr.len()) });
            }
            arr
        } else {
            &[]
        };

        for counter in 0..4096 {
            let sl: u8 = if sky_light.is_empty() {
                15
            } else {
                let b = u8::from_ne_bytes(sky_light[counter / 2].to_ne_bytes());
                (b >> (4 * (counter % 2))) & 0xF
            };
            debug_assert!(sl <= 15);
            let bl: u8 = if block_light.is_empty() {
                15
            } else {
                let b = u8::from_ne_bytes(block_light[counter / 2].to_ne_bytes());
                (b >> (4 * (counter % 2))) & 0xF
            };
            debug_assert!(bl <= 15);

            let light = Light::new(sl, bl);
            subchunk.sky_block_light_array[counter] = light;
        }
    }

    //biomes
    parse_biomes(&mut subchunk, sect, path)?;

    return Ok(Some((subchunk, y_pos)));
}

// MultiBitSet in chunk.rs and litematic.rs is different. MC doesn't allow to separate an element
// into 2 u64, but litematica does
struct MultiBitSet {
    array: Vec<u64>,
    num_elements: usize,
    element_bits: u8,
}

#[allow(dead_code)]
impl MultiBitSet {
    pub fn num_element_per_u64(element_bits: u8) -> u8 {
        return floor(64.0 / element_bits as f64, 0) as u8;
    }

    pub fn required_num_u64(num_elements: usize, element_bits: u8) -> usize {
        let num_per_u64 = Self::num_element_per_u64(element_bits) as isize;
        (ceil_up_to(num_elements as isize, num_per_u64) / num_per_u64) as usize
    }
    pub fn new(len: usize, element_bits: u8) -> Self {
        assert!(element_bits < 64);
        let mut result = Self {
            array: Vec::new(),
            num_elements: len,
            element_bits,
        };
        result.reset(len, element_bits);
        return result;
    }
    pub fn reset(&mut self, num_elements: usize, element_bits: u8) {
        assert!(element_bits < 64);
        self.element_bits = element_bits;
        self.num_elements = num_elements;
        self.array.resize(Self::required_num_u64(num_elements, element_bits), 0);
    }

    pub fn index_of_element(&self, ele_idx: usize) -> (usize, u8) {
        let num_per_u64 = Self::num_element_per_u64(self.element_bits) as usize;
        let u64_idx = ele_idx / num_per_u64;
        let bit_index_beg: u8 = ((ele_idx % num_per_u64) * self.element_bits as usize) as u8;
        debug_assert!(bit_index_beg + self.element_bits <= 64);
        return (u64_idx, bit_index_beg);
    }

    pub fn mask(element_bits: u8, bit_index_beg: u8) -> u64 {
        assert!(element_bits < 64);
        let mask = (1u64 << element_bits) - 1;
        return mask << bit_index_beg;
    }

    pub fn get(&self, ele_idx: usize) -> u64 {
        let (u64_idx, bit_index_beg) = self.index_of_element(ele_idx);
        let mask = Self::mask(self.element_bits, bit_index_beg);
        return (self.array[u64_idx] & mask) >> bit_index_beg;
    }

    pub fn set(&mut self, ele_idx: usize, value: u64) {
        debug_assert!(value <= Self::mask(self.element_bits, 0));
        let value = value & Self::mask(self.element_bits, 0);

        let (u64_idx, bit_index_beg) = self.index_of_element(ele_idx);
        let inv_mask = !Self::mask(self.element_bits, bit_index_beg);
        self.array[u64_idx] &= inv_mask;
        self.array[u64_idx] |= value << bit_index_beg;

        debug_assert!(self.get(ele_idx) == value);
    }

    pub fn num_u64(&self) -> usize {
        return self.array.len();
    }

    pub fn set_array_from_nbt(&mut self, i64_ne: &[i64]) {
        self.array.clear();
        self.array.reserve(i64_ne.len());

        for val in i64_ne {
            let val = u64::from_be_bytes(val.to_be_bytes());
            self.array.push(val);
        }
    }
}

#[test]
fn test_multi_bit_set() {
    let bits = 63;
    let len = 8192;

    let mut vec = MultiBitSet::new(len, bits);
    for i in 0..len {
        vec.set(i, (i as u64) & (1u64 << bits - 1));
    }
    for i in 0..len {
        let expected = (i as u64) & (1u64 << bits - 1);
        assert_eq!(vec.get(i), expected);
    }
}

