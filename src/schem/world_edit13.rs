use std::collections::HashMap;
use std::fs::File;
use fastnbt::Value;
use flate2::{GzBuilder};
use flate2::read::GzDecoder;
use ndarray::Array3;
use crate::block::Block;
use crate::error::{Error};
use crate::region::{BlockEntity, Region};
use crate::schem::{common, MetaDataIR, RawMetaData, Schematic, WE13MetaData, WE13MetaDataV3Extra, WorldEdit13LoadOption, WorldEdit13SaveOption};
use crate::{unwrap_opt_tag, unwrap_tag};
use crate::schem::id_of_nbt_tag;

#[allow(dead_code)]
impl Schematic {
    pub fn from_world_edit_13_file(filename: &str, option: &WorldEdit13LoadOption) -> Result<Schematic, Error> {
        let mut file;
        match File::open(filename) {
            Ok(f) => file = f,
            Err(e) => return Err(Error::FileOpenError(e)),
        }

        let decoder = GzDecoder::new(&mut file);
        let nbt =
            match fastnbt::from_reader(decoder) {
                Ok(nbt_) => nbt_,
                Err(e) => return Err(Error::NBTReadError(e)),
            };
        return Self::from_world_edit_13_nbt(&nbt, option);
    }

    fn parse_v2(root: &HashMap<String, Value>, option: &WorldEdit13LoadOption) -> Result<Schematic, Error> {
        let mut schem = Schematic::new();
        // metadata
        {
            let we13;
            match parse_metadata(&root, "", option) {
                Ok(we13_) => we13 = we13_,
                Err(e) => return Err(e),
            }

            let ir = MetaDataIR::from_world_edit13(&we13);
            schem.original_metadata = Some(RawMetaData::WE13(we13));
            schem.metadata = ir;
        }
        match Region::from_world_edit_13_v2(&root, option) {
            Ok(reg) => schem.regions.push(reg),
            Err(e) => return Err(e),
        }
        return Ok(schem);
    }

    fn parse_v3(root: &HashMap<String, Value>, option: &WorldEdit13LoadOption) -> Result<Schematic, Error> {
        let tag_schem = unwrap_opt_tag!(root.get("Schematic"),Compound,HashMap::new(),"/Schematic");
        let mut schem = Schematic::new();
        // metadata
        {
            let we13 = parse_metadata(tag_schem, "/Schematic", option)?;
            let ir = MetaDataIR::from_world_edit13(&we13);
            schem.original_metadata = Some(RawMetaData::WE13(we13));
            schem.metadata = ir;
        }
        let region = Region::from_world_edit_13_v3(tag_schem, option)?;
        schem.regions.push(region);

        return Ok(schem);
    }
    pub fn from_world_edit_13_nbt(root: &HashMap<String, Value>, option: &WorldEdit13LoadOption) -> Result<Schematic, Error> {
        return if root.contains_key("Schematic") {//v3
            Self::parse_v3(root, option)
        } else {
            Self::parse_v2(root, option)
        }
    }

    pub fn from_world_edit_13_reader(src: &mut dyn std::io::Read, option: &WorldEdit13LoadOption) -> Result<Schematic, Error> {
        let root_opt: Result<HashMap<String, Value>, fastnbt::error::Error> = fastnbt::from_reader(src);
        let root = match root_opt {
            Ok(nbt_) => nbt_,
            Err(e) => return Err(Error::NBTReadError(e)),
        };
        return Self::from_world_edit_13_nbt(&root, option);
    }
}


impl MetaDataIR {
    pub fn from_world_edit13(src: &WE13MetaData) -> MetaDataIR {
        let mut result = MetaDataIR::default();
        result.mc_data_version = src.data_version;
        result.schem_version = src.version;
        result.schem_offset = src.offset;
        result.schem_we_offset = Some(src.we_offset);
        result.date = src.date;

        if let Some(date) = src.date {
            result.time_created = date;
            result.time_modified = date;
        }


        if let Some(extra) = &src.v3_extra {
            result.schem_editing_platform = Some(extra.editing_platform.clone());
            result.schem_world_edit_version = Some(extra.world_edit_version.clone());
            result.schem_origin = Some(extra.origin);
            result.schem_material = "Alpha".to_string();
        }

        return result;
    }
}

fn parse_metadata(nbt: &HashMap<String, Value>, tag_path: &str, _option: &WorldEdit13LoadOption)
    -> Result<WE13MetaData, Error> {
    let mut we13 = WE13MetaData::default();

    we13.version = *unwrap_opt_tag!(nbt.get("Version"),Int,0,format!("{tag_path}/Version"));
    we13.data_version = *unwrap_opt_tag!(nbt.get("DataVersion"),Int,0,format!("{tag_path}/DataVersion"));
    let schem_version = we13.version;

    // offset
    {
        let offset_list =
            unwrap_opt_tag!(nbt.get("Offset"),IntArray,fastnbt::IntArray::new(vec![]),format!("{tag_path}/Offset"));
        match common::parse_size_list(offset_list.as_ref(), "Offset", true) {
            Ok(offset) => we13.offset = offset,
            Err(e) => return Err(e),
        }
    }

    let tag_md = unwrap_opt_tag!(nbt.get("Metadata"),Compound,HashMap::new(),format!("{tag_path}/Metadata"));
    if schem_version == 2 {
        // we offset
        {
            let keys = ["WEOffsetX", "WEOffsetY", "WEOffsetZ"];
            for dim in 0..3 {
                we13.we_offset[dim] = *unwrap_opt_tag!(tag_md.get(keys[dim]),Int,0,format!("{tag_path}/Metadata/{}",keys[dim]));
            }
        }
        return Ok(we13);
    }
    if schem_version == 3 {
        if let Some(tag_date) = tag_md.get("Date") {
            we13.date = Some(*unwrap_tag!(tag_date,Long,0,format!("{tag_path}/Metadata/Date")));
        } else {
            we13.date = None;
        }
        if let Some(tag_we) = tag_md.get("WorldEdit") {
            let we_tag = unwrap_tag!(tag_we,Compound,HashMap::new(),format!("{tag_path}/Metadata/WorldEdit"));
            let mut v3extra = WE13MetaDataV3Extra::default();
            v3extra.world_edit_version =
                unwrap_opt_tag!(we_tag.get("Version"),String,"".to_string(),format!("{tag_path}/Metadata/WorldEdit/Version")).to_string();
            v3extra.editing_platform =
                unwrap_opt_tag!(we_tag.get("EditingPlatform"),String,"".to_string(),format!("{tag_path}/Metadata/WorldEdit/EditingPlatform")).to_string();
            we13.v3_extra = Some(v3extra);
        } else {
            we13.v3_extra = None;
        }

        return Ok(we13);
    }

    return Ok(we13);
}

fn parse_single_block(src: &[i8]) -> i32 {
    debug_assert!(src.len() > 0);
    if src.len() == 1 {
        debug_assert!(src[0] >= 0);
        return src[0] as i32;
    }

    let mut result = 0;
    for (idx, value) in src.iter().enumerate() {
        if idx == src.len() - 1 {
            debug_assert!(*value >= 1);
        } else {
            debug_assert!(*value < 0);
        }
        let value_fixed;
        if idx != 0 {
            value_fixed = *value as i32 + 1;
        } else {
            value_fixed = *value as i32;
        }
        result += value_fixed * (1 << (idx * 7));
    }

    return result;
}

#[allow(dead_code)]
impl Region {
    fn parse_palette_v2(nbt: &HashMap<String, Value, >, tag_path: &str, _option: &WorldEdit13LoadOption) -> Result<Vec<Block>, Error> {
        let palette_max = *unwrap_opt_tag!(nbt.get("PaletteMax"),Int,0,format!("{tag_path}/PaletteMax"));
        let palette_comp = unwrap_opt_tag!(nbt.get("Palette"),Compound,HashMap::new(),format!("{tag_path}/Palette"));
        if palette_max != palette_comp.len() as i32 {
            return Err(Error::InvalidValue {
                tag_path: format!("{tag_path}/Palette"),
                error: format!("PaletteMax should equal to the size of Palette ({}), but found {}", palette_comp.len(), palette_max),
            });
        }

        return parse_palette(palette_comp, tag_path);
    }

    fn parse_size_v2(nbt: &HashMap<String, Value, >, tag_path: &str, _option: &WorldEdit13LoadOption) -> Result<[i32; 3], Error> {
        let mut sz = [0, 0, 0];
        let keys = ["Width", "Height", "Length"];
        for dim in 0..3 {
            let tag_path = format!("{tag_path}/{}", keys[dim]);
            let val = *unwrap_opt_tag!(nbt.get(keys[dim]),Short,0,tag_path);
            if val < 0 {
                return Err(Error::InvalidValue {
                    tag_path,
                    error: format!("Schem size should be non-negative, but found {}", val),
                });
            }
            sz[dim] = val as i32;
        }
        return Ok(sz);
    }

    fn parse_3d_array_v2(block_data: &[i8], tag_path: &str, _option: &WorldEdit13LoadOption, size: [i32; 3], palette: &[Block]) -> Result<Array3<u16>, Error> {
        let mut array: Array3<u16> = Array3::default([size[0] as usize, size[1] as usize, size[2] as usize]);

        let total_blocks = size[1] as usize * size[2] as usize * size[0] as usize;
        let mut decoded_blocks = 0;
        let mut idx = 0;
        for y in 0..size[1] as usize {
            for z in 0..size[2] as usize {
                for x in 0..size[0] as usize {
                    if idx >= block_data.len() {
                        return Err(Error::BlockDataIncomplete {
                            tag_path: tag_path.to_string(),
                            index: idx,
                            detail: format!("{} blocks decoded, {} blocks missing, {} blocks in total.", decoded_blocks, total_blocks - decoded_blocks, total_blocks),
                        });
                    }

                    let cur_block_first_byte_index = idx;

                    let mut cur_block_bytes = usize::MAX;
                    for iidx in cur_block_first_byte_index..block_data.len() {
                        if block_data[iidx] >= 0 {
                            cur_block_bytes = iidx - cur_block_first_byte_index + 1;
                            break;
                        }
                    }
                    if cur_block_bytes >= block_data.len() {
                        let first_byte = block_data[cur_block_first_byte_index];
                        return Err(Error::BlockDataIncomplete {
                            tag_path: tag_path.to_string(),
                            index: idx,
                            detail: format!("BlockData[{}] is {}, which expects one or more elements to represent a block, but the data ends; {} blocks decoded, {} blocks missing, {} blocks in total.", idx - 1, first_byte, decoded_blocks, total_blocks - decoded_blocks, total_blocks),
                        });
                    }
                    idx += cur_block_bytes;
                    let decoded_block_index = parse_single_block(&block_data[cur_block_first_byte_index..(cur_block_first_byte_index + cur_block_bytes)]);

                    assert!(decoded_block_index >= 0);
                    if decoded_block_index as usize >= palette.len() {
                        return Err(Error::BlockIndexOutOfRange {
                            tag_path: format!("{tag_path}[{}]", cur_block_first_byte_index),
                            index: decoded_block_index,
                            range: [0, palette.len() as i32],
                        });
                    }
                    decoded_blocks += 1;
                    array[[x, y, z]] = decoded_block_index as u16;
                }
            }
        }
        debug_assert!(idx == block_data.len());
        return Ok(array);
    }

    fn parse_block_entities_v2(block_entities: &[Value], tag_path: &str, _option: &WorldEdit13LoadOption, size: [i32; 3])
        -> Result<HashMap<[i32; 3], BlockEntity>, Error> {
        let mut result = HashMap::with_capacity(block_entities.len());
        for (idx, nbt) in block_entities.iter().enumerate() {
            let cur_tag_path = format!("{tag_path}[{}]", idx);
            let nbt = unwrap_tag!(nbt,Compound,HashMap::new(),cur_tag_path);
            let (be, pos) = parse_block_entity(nbt, &cur_tag_path, &size)?;

            if result.contains_key(&pos) {
                return Err(Error::MultipleBlockEntityInOnePos {
                    pos,
                    latter_tag_path: cur_tag_path,
                });
            }
            result.insert(pos, be);
        }
        return Ok(result);
    }

    pub fn from_world_edit_13_v2(root: &HashMap<String, Value>, option: &WorldEdit13LoadOption) -> Result<Region, Error> {
        let mut region = Region::new();
        let tag_path = "";
        // palette
        region.palette = Self::parse_palette_v2(root, tag_path, option)?;

        // offset
        region.offset = [0, 0, 0];

        let size: [i32; 3] = Self::parse_size_v2(root, tag_path, option)?;

        // parse 3d array
        {
            let block_data_tag_path = format!("{tag_path}/BlockData");
            let block_data = unwrap_opt_tag!(root.get("BlockData"),ByteArray,fastnbt::ByteArray::new(vec![]),block_data_tag_path);
            region.array = Self::parse_3d_array_v2(block_data.as_ref(), &block_data_tag_path, option, size, &region.palette)?;
        }


        // parse block entities
        {
            let be_tag_path = format!("{tag_path}/BlockEntities");
            let block_entities = unwrap_opt_tag!(root.get("BlockEntities"),List,vec![],be_tag_path);
            region.block_entities = Self::parse_block_entities_v2(&block_entities, &be_tag_path, option, size)?;
        }
        return Ok(region);
    }

    pub fn from_world_edit_13_v3(tag_schem: &HashMap<String, Value>, option: &WorldEdit13LoadOption) -> Result<Region, Error> {
        let tag_schem_path = "/Schematic";
        let mut region = Region::new();
        //size
        let size = Self::parse_size_v2(tag_schem, tag_schem_path, option)?;

        let tag_blocks_path = "/Schematic/Blocks";
        let tag_blocks = unwrap_opt_tag!(tag_schem.get("Blocks"),Compound,HashMap::new(),tag_blocks_path);
        //palette
        {
            let tag_palette_path = "/Schematic/Blocks/Palette";
            let tag_palette = unwrap_opt_tag!(tag_blocks.get("Palette"),Compound,HashMap::new(),tag_palette_path);
            region.palette = parse_palette(tag_palette, tag_palette_path)?;
        }
        //3d array
        {
            let tag_data_path = "/Schematic/Blocks/Data";
            let tag_data = unwrap_opt_tag!(tag_blocks.get("Data"),ByteArray,fastnbt::ByteArray::new(vec![]),tag_data_path);
            region.array = Self::parse_3d_array_v2(&tag_data, tag_data_path, option, size, &region.palette)?;
        }
        //block entities
        {
            let tag_be_path = "/Schematic/Blocks/BlockEntities";
            let tag_be = unwrap_opt_tag!(tag_blocks.get("BlockEntities"),List,vec![],tag_be_path);
            region.block_entities = Self::parse_block_entities_v2(&tag_be, tag_be_path, option, size)?;
        }


        return Ok(region);
    }

}


fn parse_palette(pal: &HashMap<String, Value>, tag_path: &str) -> Result<Vec<Block>, Error> {
    if pal.len() >= 65536 {
        return Err(Error::PaletteTooLong(pal.len()));
    }

    let mut is_set: Vec<Option<&str>> = Vec::new();
    is_set.resize(pal.len(), None);
    let mut result = Vec::new();
    result.resize(pal.len(), Block::air());

    for (key, val) in pal {
        let block;
        match Block::from_id(key) {
            Ok(blk) => block = blk,
            Err(e) => return Err(Error::InvalidBlockId { id: key.clone(), reason: e }),
        }

        let cur_tag_path = format!("{tag_path}/Palette/{}", key);
        let idx = *unwrap_tag!(val,Int,0,cur_tag_path);
        if idx < 0 || idx >= pal.len() as i32 {
            return Err(Error::InvalidValue {
                tag_path: cur_tag_path,
                error: format!("Block index {} in palette is out of range [0,{})", idx, pal.len()),
            });
        }
        if let Some(prev_blk_id) = is_set[idx as usize] {
            return Err(Error::ConflictingIndexInPalette {
                index: idx as u16,
                former_block_id: prev_blk_id.to_string(),
                latter_block_id: key.clone(),
            });
        }

        result[idx as usize] = block;
        is_set[idx as usize] = Some(&key);
    }
    return Ok(result);
}

fn parse_block_entity(nbt: &HashMap<String, Value>, tag_path: &str, region_size: &[i32; 3])
    -> Result<(BlockEntity, [i32; 3]), Error> {
    let pos;
    let pos_tag_path = format!("{}/Pos", tag_path);
    // parse pos
    {
        let pos_tag = unwrap_opt_tag!(nbt.get("Pos"),IntArray,fastnbt::IntArray::new(vec![]),pos_tag_path);
        match common::parse_size_list(pos_tag.as_ref(), &pos_tag_path, false) {
            Ok(pos_) => pos = pos_,
            Err(e) => return Err(e),
        }
    }
    for dim in 0..3 {
        if pos[dim] < 0 || pos[dim] >= region_size[dim] {
            return Err(Error::BlockPosOutOfRange {
                tag_path: pos_tag_path,
                pos,
                range: region_size.clone(),
            });
        }
    }

    let mut be = BlockEntity::new();
    be.tags.reserve(nbt.len() - 1);
    for (key, val) in nbt {
        if key == "Pos" {
            continue;
        }
        be.tags.insert(key.clone(), val.clone());
    }

    return Ok((be, pos));
}

#[allow(dead_code)]
impl Schematic {
    pub fn supported_world_edit_13_versions() -> Vec<i32> {
        return vec![2, 3];
    }

    pub fn metadata_world_edit_13(&self) -> Result<WE13MetaData, Error> {
        let mut result = WE13MetaData::from_data_version_i32(self.metadata.mc_data_version)?;
        // if let Some(raw_md) = &self.raw_metadata {
        //     if let RawMetaData::WE13(raw) = &raw_md {
        //         result = raw.clone();
        //     }
        // }

        result.data_version = self.metadata.mc_data_version;
        result.offset = [0, 0, 0];

        return Ok(result);
    }

    fn write_metadata_v2(dest: &mut HashMap<String, Value>, md: &WE13MetaData) {
        let mut md_nbt = HashMap::new();
        let pos_letter = ['X', 'Y', 'Z'];
        for dim in 0..3 {
            md_nbt.insert(format!("WEOffset{}", pos_letter[dim]), Value::Int(md.we_offset[dim]));
        }
        dest.insert("Metadata".to_string(), Value::Compound(md_nbt));
        dest.insert("Offset".to_string(), Value::IntArray(fastnbt::IntArray::new(Vec::from(&md.offset))));
        dest.insert("DataVersion".to_string(), Value::Int(md.data_version));
        dest.insert("Version".to_string(), Value::Int(md.version));
    }

    fn write_metadata_v3(dest: &mut HashMap<String, Value>, md: &WE13MetaData) {
        dest.insert("Offset".to_string(), Value::IntArray(fastnbt::IntArray::new(Vec::from(&md.offset))));
        dest.insert("DataVersion".to_string(), Value::Int(md.data_version));
        dest.insert("Version".to_string(), Value::Int(md.version));

        let mut md_nbt = HashMap::new();
        if let Some(date) = md.date {
            md_nbt.insert("Date".to_string(), Value::Long(date));
        }
        if let Some(extra) = &md.v3_extra {
            let mut we = HashMap::new();
            we.insert("Version".to_string(), Value::String(extra.world_edit_version.to_string()));
            we.insert("EditingPlatform".to_string(), Value::String(extra.editing_platform.to_string()));
            we.insert("Origin".to_string(), Value::IntArray(fastnbt::IntArray::new(extra.origin.to_vec())));
            md_nbt.insert("WorldEdit".to_string(), Value::Compound(we));
        }
        dest.insert("Metadata".to_string(), Value::Compound(md_nbt));
    }

    fn write_shape_v2(dest: &mut HashMap<String, Value>, shape: [i32; 3]) -> Result<(), Error> {
        for sz in shape {
            if sz < 0 {
                return Err(Error::NegativeSize { size: shape, region_name: "all regions".to_string() });
            }
            if sz >= 16384 {
                return Err(Error::SizeTooLarge {
                    size: [shape[0] as u64, shape[1] as u64, shape[2] as u64],
                    max_size: [16383, 16383, 16383],
                });
            }
        }
        let keys = ["Width", "Height", "Length"];
        for dim in 0..3 {
            dest.insert(keys[dim].to_string(), Value::Short(shape[dim] as i16));
        }
        return Ok(());
    }
    fn save_palette_v2(full_palette: &Vec<(&Block, u64)>, option: &WorldEdit13SaveOption) -> Result<(HashMap<String, Value>, u16), Error> {
        let mut pal = HashMap::with_capacity(full_palette.len());
        for (index, (blk, _)) in full_palette.iter().enumerate() {
            let id = blk.full_id();
            debug_assert!(!pal.contains_key(&id));
            pal.insert(id, Value::Int(index as i32));
        }
        let background_blk_index: u16;
        // find/insert background block
        {
            let bk_id = option.background_block.to_block().full_id();
            match pal.get(&bk_id) {
                Some(id) => {
                    let id = id.as_i64();
                    debug_assert!(id.is_some());
                    let id = id.unwrap();
                    background_blk_index = id as u16;
                }
                None => {
                    background_blk_index = pal.len() as u16;
                    pal.insert(bk_id, Value::Int(background_blk_index as i32));
                }
            }
        }
        return Ok((pal, background_blk_index));
    }

    fn save_block_data_v2(&self, shape: [i32; 3], luts_of_block_idx: &[Vec<usize>], background_blk_index: u16) -> Result<Vec<i8>, Error> {
        let mut block_data = Vec::with_capacity(self.volume() as usize * 2);
        for y in 0..shape[1] {
            for z in 0..shape[2] {
                for x in 0..shape[0] {
                    let mut cur_block_gindex: Option<u16> = None;
                    for (reg_idx, reg) in self.regions.iter().enumerate() {
                        let offset = reg.offset;
                        match reg.block_index_at([x - offset[0], y - offset[1], z - offset[2]]) {
                            Some(cur_idx) => {
                                cur_block_gindex = Some(luts_of_block_idx[reg_idx][cur_idx as usize] as u16);
                                break;
                            }
                            None => {},
                        }
                    }
                    let cur_block_gindex = cur_block_gindex.unwrap_or_else(|| background_blk_index);

                    let encoded_index = encode_single_block(cur_block_gindex);
                    for value in &encoded_index {
                        block_data.push(*value);
                        if *value >= 0 {
                            break;
                        }
                    }
                }
            }
        }
        return Ok(block_data);
    }

    fn save_block_entities_v2(&self, shape: [i32; 3]) -> Result<Vec<Value>, Error> {
        let mut be_list;
        {
            let mut counter = 0usize;
            for reg in &self.regions {
                counter += reg.block_entities.len();
            }
            be_list = Vec::with_capacity(counter);
        }

        for y in 0..shape[1] {
            for z in 0..shape[2] {
                for x in 0..shape[0] {
                    let be_ = self.first_block_entity_at([x, y, z]);
                    let be;
                    if let Some(b) = be_ {
                        be = b;
                    } else {
                        continue;
                    }

                    let mut nbt = HashMap::new();
                    nbt.insert("Pos".to_string(), Value::IntArray(fastnbt::IntArray::new(
                        vec![x, y, z])));
                    for (key, val) in &be.tags {
                        if key == "Pos" {
                            continue;
                        }
                        nbt.insert(key.clone(), val.clone());
                    }
                    be_list.push(Value::Compound(nbt));
                }
            }
        }
        return Ok(be_list);
    }

    pub fn to_nbt_world_edit_13_v2(&self, md: WE13MetaData, option: &WorldEdit13SaveOption) -> Result<HashMap<String, Value>, Error> {
        let mut root = HashMap::new();
        // metadata
        Self::write_metadata_v2(&mut root, &md);


        let (full_palette, luts_of_block_idx) = self.full_palette();
        let background_blk_index: u16;
        // palette
        {
            let (pal, bbi) = Self::save_palette_v2(&full_palette, option)?;
            background_blk_index = bbi;
            root.insert("PaletteMax".to_string(), Value::Int(pal.len() as i32));
            root.insert("Palette".to_string(), Value::Compound(pal));
        }


        // shape
        let shape = self.shape();
        Self::write_shape_v2(&mut root, shape)?;

        // block data
        {
            let block_data = self.save_block_data_v2(shape, &luts_of_block_idx, background_blk_index)?;
            root.insert("BlockData".to_string(), Value::ByteArray(fastnbt::ByteArray::new(block_data)));
        }

        // block entities
        {
            let be_list = self.save_block_entities_v2(shape)?;
            root.insert("BlockEntities".to_string(), Value::List(be_list));
        }

        return Ok(root);
    }

    pub fn to_nbt_world_edit_13_v3(&self, md: WE13MetaData, option: &WorldEdit13SaveOption) -> Result<HashMap<String, Value>, Error> {
        let mut tag_schem = HashMap::new();
        // metadata
        Self::write_metadata_v3(&mut tag_schem, &md);

        let mut tag_blocks = HashMap::new();
        let (full_palette, luts_of_block_idx) = self.full_palette();
        let background_blk_index: u16;
        // palette
        {
            let (pal, bbi) = Self::save_palette_v2(&full_palette, option)?;
            background_blk_index = bbi;
            tag_blocks.insert("Palette".to_string(), Value::Compound(pal));
        }


        // shape
        let shape = self.shape();
        Self::write_shape_v2(&mut tag_schem, shape)?;

        // block data
        {
            let block_data = self.save_block_data_v2(shape, &luts_of_block_idx, background_blk_index)?;
            tag_blocks.insert("Data".to_string(), Value::ByteArray(fastnbt::ByteArray::new(block_data)));
        }

        // block entities
        {
            let be_list = self.save_block_entities_v2(shape)?;
            tag_blocks.insert("BlockEntities".to_string(), Value::List(be_list));
        }
        tag_schem.insert("Blocks".to_string(), Value::Compound(tag_blocks));
        let mut root = HashMap::new();
        root.insert("Schematic".to_string(), Value::Compound(tag_schem));

        return Ok(root);
    }

    pub fn to_nbt_world_edit_13(&self, option: &WorldEdit13SaveOption) -> Result<HashMap<String, Value>, Error> {
        let md = self.metadata_world_edit_13()?;
        let schem_version = md.version;

        return match schem_version {
            2 => self.to_nbt_world_edit_13_v2(md, option),
            3 => self.to_nbt_world_edit_13_v3(md, option),
            _ => Err(Error::UnsupportedWorldEdit13Version {
                version: schem_version,
                supported_versions: Self::supported_world_edit_13_versions(),
            })
            ,
        }

        //Self::write_metadata_v3(&mut root, &md)
    }

    pub fn save_world_edit_13_file(&self, filename: &str, option: &WorldEdit13SaveOption) -> Result<(), Error> {
        let nbt;
        match self.to_nbt_world_edit_13(option) {
            Ok(n) => nbt = n,
            Err(e) => return Err(e),
        }

        let file;
        match File::create(filename) {
            Ok(f) => file = f,
            Err(e) => return Err(Error::FileCreateError(e)),
        }

        let mut encoder = GzBuilder::new()
            .filename(filename)
            .comment("Generated by mc_schem")
            .write(file, option.compress_level);

        let res: Result<(), fastnbt::error::Error> = fastnbt::to_writer(&mut encoder, &nbt);
        if let Err(e) = res {
            return Err(Error::NBTWriteError(e));
        }
        if let Err(e) = encoder.finish() {
            return Err(Error::NBTWriteError(e.into()));
        }

        return Ok(());
    }
}


fn encode_single_block(value: u16) -> [i8; 8] {
    // let index = index as i32;
    //
    // let first_byte = index % 128 - 128;
    // let second_byte = (index - first_byte) / 128 - 1;
    // debug_assert!(first_byte < 0);
    // debug_assert!(second_byte >= 1);
    //
    // return [first_byte as i8, second_byte as i8];
    let mut result = [0; 8];
    if value <= 127 {
        result[0] = value as i8;
        return result;
    }

    let mut value = value as i32;
    let mut byte_idx = 0;
    loop {
        let cur_byte_val = value % 128;
        value = value / 128;
        let encoded_byte: i8;
        if value > 0 {// not the last byte
            encoded_byte = (cur_byte_val - 128) as i8;
            debug_assert!(encoded_byte < 0);
        } else {// the last byte of this block
            encoded_byte = cur_byte_val as i8;
            debug_assert!(encoded_byte > 0);
        }
        result[byte_idx] = encoded_byte;
        byte_idx += 1;
        if value <= 0 {
            break;
        }
    }
    return result;
}

#[test]
fn test_schem_encode_decoding() {
    for id in 0..65536 {
        let code = encode_single_block(id as u16);
        let mut code_length = 0;
        for val in code {
            code_length += 1;
            if val >= 0 {
                break;
            }
        }
        let decoded_id = parse_single_block(&code[0..code_length]);
        assert_eq!(id, decoded_id);
    }
}