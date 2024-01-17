use std::collections::HashMap;
use std::fs::File;
use fastnbt::Value;
use flate2::read::GzDecoder;
use crate::block::Block;
use crate::error::LoadError;
use crate::region::{BlockEntity, Region};
use crate::schem::{common, MetaDataIR, RawMetaData, Schematic, WE13MetaData, WorldEdit13LoadOption};
use crate::{unwrap_opt_tag, unwrap_tag};
use crate::schem::id_of_nbt_tag;

#[allow(dead_code)]
impl Schematic {
    pub fn from_world_edit_13_file(filename: &str, option: &WorldEdit13LoadOption) -> Result<Schematic, LoadError> {
        let mut file;
        match File::open(filename) {
            Ok(f) => file = f,
            Err(e) => return Err(LoadError::FileOpenError(e)),
        }

        let mut decoder = GzDecoder::new(&mut file);
        return Self::from_world_edit_13(&mut decoder, option);
    }

    pub fn from_world_edit_13(src: &mut dyn std::io::Read, option: &WorldEdit13LoadOption) -> Result<Schematic, LoadError> {
        let nbt: HashMap<String, Value>;
        match fastnbt::from_reader(src) {
            Ok(nbt_) => nbt = nbt_,
            Err(e) => return Err(LoadError::NBTReadError(e)),
        }

        let mut schem = Schematic::new();

        // metadata
        {
            let we13;
            match parse_metadata(&nbt, option) {
                Ok(we13_) => we13 = we13_,
                Err(e) => return Err(e),
            }

            let ir = MetaDataIR::from_world_edit13(&we13);
            schem.raw_metadata = Some(RawMetaData::WE13(we13));
            schem.metadata = ir;
        }

        match Region::from_world_edit_13(&nbt, option) {
            Ok(reg) => schem.regions.push(reg),
            Err(e) => return Err(e),
        }
        return Ok(schem);
    }
}


impl MetaDataIR {
    pub fn from_world_edit13(src: &WE13MetaData) -> MetaDataIR {
        let mut result = MetaDataIR::default();
        result.mc_data_version = src.data_version;

        return result;
    }
}

fn parse_metadata(root: &HashMap<String, Value>, _option: &WorldEdit13LoadOption)
                  -> Result<WE13MetaData, LoadError> {
    let mut we13 = WE13MetaData::new();

    we13.version = *unwrap_opt_tag!(root.get("Version"),Int,0,"/Version".to_string());
    we13.data_version = *unwrap_opt_tag!(root.get("DataVersion"),Int,0,"/DataVersion".to_string());

    // offset
    {
        let offset_list =
            unwrap_opt_tag!(root.get("Offset"),IntArray,fastnbt::IntArray::new(vec![]),"Offset".to_string());
        match common::parse_size_list(offset_list.as_ref(), "Offset", true) {
            Ok(offset) => we13.offset = offset,
            Err(e) => return Err(e),
        }
    }

    let tag_md = unwrap_opt_tag!(root.get("Metadata"),Compound,HashMap::new(),"/Metadata".to_string());
    // we offset
    {
        let keys = ["WEOffsetX", "WEOffsetY", "WEOffsetZ"];
        for dim in 0..3 {
            we13.we_offset[dim] = *unwrap_opt_tag!(tag_md.get(keys[dim]),Int,0,format!("/Metadata/{}",keys[dim]));
        }
    }

    return Ok(we13);
}

#[allow(dead_code)]
impl Region {
    pub fn from_world_edit_13(nbt: &HashMap<String, Value>, _option: &WorldEdit13LoadOption) -> Result<Region, LoadError> {
        let mut region = Region::new();

        // palette
        {
            let palette_max = *unwrap_opt_tag!(nbt.get("PaletteMax"),Int,0,"/PaletteMax".to_string());
            let palette_comp = unwrap_opt_tag!(nbt.get("Palette"),Compound,HashMap::new(),"/Palette".to_string());
            if palette_max != palette_comp.len() as i32 {
                return Err(LoadError::InvalidValue {
                    tag_path: "/PaletteMax".to_string(),
                    error: format!("PaletteMax should equal to the size of Palette ({}), but found {}", palette_comp.len(), palette_max),
                });
            }

            match parse_palette(palette_comp) {
                Err(e) => return Err(e),
                Ok(pal) => region.palette = pal,
            }
        }

        // offset
        region.offset = [0, 0, 0];

        let size: [i32; 3];
        // size
        {
            let mut sz = [0, 0, 0];
            let keys = ["Width", "Height", "Length"];
            for dim in 0..3 {
                let tag_path = format!("/{}", keys[dim]);
                let val = *unwrap_opt_tag!(nbt.get(keys[dim]),Short,0,tag_path);
                if val < 0 {
                    return Err(LoadError::InvalidValue {
                        tag_path,
                        error: format!("Schem size should be non-negative, but found {}", val),
                    });
                }
                sz[dim] = val as i32;
            }
            size = sz;
        }
        region.reshape(&size);

        // parse 3d array
        {
            let block_data = unwrap_opt_tag!(nbt.get("BlockData"),ByteArray,fastnbt::ByteArray::new(vec![]),"/BlockData");

            let total_blocks = size[1] as usize * size[2] as usize * size[0] as usize;
            let mut decoded_blocks = 0;
            let mut idx = 0;
            for y in 0..size[1] as usize {
                for z in 0..size[2] as usize {
                    for x in 0..size[0] as usize {
                        if idx >= block_data.len() {
                            return Err(LoadError::BlockDataIncomplete {
                                tag_path: "/BlockData".to_string(),
                                index: idx,
                                detail: format!("{} blocks decoded, {} blocks missing, {} blocks in total.", decoded_blocks, total_blocks - decoded_blocks, total_blocks),
                            });
                        }

                        let cur_block_first_byte_index = idx;

                        let decoded_block_index: i32;
                        let first_byte = block_data[idx];
                        idx += 1;

                        if first_byte >= 0 {
                            decoded_block_index = first_byte as i32;
                        } else {
                            if idx >= block_data.len() {
                                return Err(LoadError::BlockDataIncomplete {
                                    tag_path: "/BlockData".to_string(),
                                    index: idx,
                                    detail: format!("BlockData[{}] is {}, which expects one more element to represent a block, but the data ends; {} blocks decoded, {} blocks missing, {} blocks in total.", idx - 1, first_byte, decoded_blocks, total_blocks - decoded_blocks, total_blocks),
                                });
                            }

                            let second_byte = block_data[idx];
                            idx += 1;
                            decoded_block_index = 128 + 128 * second_byte as i32 + first_byte as i32;
                        }
                        assert!(decoded_block_index >= 0);
                        if decoded_block_index as usize >= region.palette.len() {
                            return Err(LoadError::BlockIndexOutOfRange {
                                tag_path: format!("/BlockData[{}]", cur_block_first_byte_index),
                                index: decoded_block_index,
                                range: [0, region.palette.len() as i32],
                            });
                        }
                        decoded_blocks += 1;
                        region.array[[x, y, z]] = decoded_block_index as u16;
                    }
                }
            }
        }


        // parse block entities
        {
            let block_entities = unwrap_opt_tag!(nbt.get("BlockEntities"),List,vec![],"/BlockEntities".to_string());
            for (idx, nbt) in block_entities.iter().enumerate() {
                let cur_tag_path = format!("/BlockEntities[{}]", idx);
                let nbt = unwrap_tag!(nbt,Compound,HashMap::new(),cur_tag_path);
                let pos;
                let be;
                match parse_block_entity(nbt, &cur_tag_path, &size) {
                    Ok((b_, p_)) => {
                        be = b_;
                        pos = p_;
                    },
                    Err(e) => return Err(e),
                }

                if region.block_entities.contains_key(&pos) {
                    return Err(LoadError::MultipleBlockEntityInOnePos {
                        pos,
                        latter_tag_path: cur_tag_path,
                    });
                }
                region.block_entities.insert(pos, be);
            }
        }
        return Ok(region);
    }
}


fn parse_palette(pal: &HashMap<String, Value>) -> Result<Vec<Block>, LoadError> {
    if pal.len() >= 65536 {
        return Err(LoadError::PaletteTooLong(pal.len()));
    }

    let mut is_set: Vec<Option<&str>> = Vec::new();
    is_set.resize(pal.len(), None);
    let mut result = Vec::new();
    result.resize(pal.len(), Block::air());

    for (key, val) in pal {
        let block;
        match Block::from_id(key) {
            Ok(blk) => block = blk,
            Err(e) => return Err(LoadError::InvalidBlockId { id: key.clone(), reason: e }),
        }

        let cur_tag_path = format!("/Palette/{}", key);
        let idx = *unwrap_tag!(val,Int,0,cur_tag_path);
        if idx < 0 || idx >= pal.len() as i32 {
            return Err(LoadError::InvalidValue {
                tag_path: cur_tag_path,
                error: format!("Block index {} in palette is out of range [0,{})", idx, pal.len()),
            });
        }
        if let Some(prev_blk_id) = is_set[idx as usize] {
            return Err(LoadError::ConflictingIndexInPalette {
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
                      -> Result<(BlockEntity, [i32; 3]), LoadError> {
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
            return Err(LoadError::BlockPosOutOfRange {
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