use std::collections::HashMap;
use std::fs::File;
use std::mem;
use fastnbt::Value;
use flate2::read::GzDecoder;
use ndarray::Array3;
use crate::block::Block;
use crate::error::Error;
use crate::old_block::OldBlockParseError;
use crate::region::{BlockEntity, Region};
use crate::schem::{common, id_of_nbt_tag, MetaDataIR, Schematic, WE12MetaData, WorldEdit12LoadOption};
use crate::{unwrap_opt_tag, unwrap_tag};


fn i8_to_u8(a: i8) -> u8 {
    return if a >= 0 {
        a as u8
    } else {
        (256 + a as i16) as u8
    }
}

impl Schematic {
    pub fn parse_number_id_from_we12(nbt: &HashMap<String, Value>) -> Result<Array3<(u8, u8)>, Error> {
        let x_size = *unwrap_opt_tag!(nbt.get("Width"),Short,0,"/Width".to_string()) as usize;
        let y_size = *unwrap_opt_tag!(nbt.get("Height"),Short,0,"/Height".to_string()) as usize;
        let z_size = *unwrap_opt_tag!(nbt.get("Length"),Short,0,"/Length".to_string()) as usize;
        let mut array = Array3::default([x_size, y_size, z_size]);

        let blocks = unwrap_opt_tag!(nbt.get("Blocks"),ByteArray,fastnbt::ByteArray::new(vec![]),"/Blocks".to_string());
        let data = unwrap_opt_tag!(nbt.get("Data"),ByteArray,fastnbt::ByteArray::new(vec![]),"/Data".to_string());

        {
            let expected_elements = x_size * y_size * z_size;
            if blocks.len() != expected_elements {
                return Err(Error::InvalidValue {
                    tag_path: "/Blocks".to_string(),
                    error: format!("Expected to contain {expected_elements} elements but found {}.", blocks.len()),
                });
            }
            if data.len() != blocks.len() {
                return Err(Error::InvalidValue {
                    tag_path: "/Data".to_string(),
                    error: format!("Expected to contain {expected_elements} elements but found {}.", data.len()),
                });
            }
        }
        array.fill((0, 0));
        let mut counter = 0;
        for y in 0..y_size {
            //let y = y_size - 1 - y;
            for z in 0..z_size {
                for x in 0..x_size {
                    let id = i8_to_u8(blocks[counter]);
                    let damage = i8_to_u8(data[counter]);
                    counter += 1;
                    array[[x, y, z]] = (id, damage);
                }
            }
        }


        return Ok(array);
    }

    fn parse_metadata(nbt: &mut HashMap<String, Value>, option: &WorldEdit12LoadOption) -> Result<(MetaDataIR, WE12MetaData), Error> {
        let mut raw = WE12MetaData::default();

        mem::swap(&mut raw.materials, unwrap_opt_tag!(nbt.get_mut("Materials"),String,"".to_string(),"/Materials"));

        for (dim, letter) in ['X', 'Y', 'Z'].iter().enumerate() {
            let key_offset = format!("WEOffset{}", letter);
            let key_origin = format!("WEOrigin{}", letter);
            raw.we_offset[dim] = *unwrap_opt_tag!(nbt.get(&key_offset),Int,0,format!("/{key_offset}"));
            raw.we_origin[dim] = *unwrap_opt_tag!(nbt.get(&key_origin),Int,0,format!("/{key_origin}"));
        }

        let mut md = MetaDataIR::default();
        md.mc_data_version = option.data_version as i32;
        md.schem_offset = raw.we_offset;
        md.schem_origin = Some(raw.we_origin);
        md.schem_material = raw.materials.clone();
        md.schem_version = 1;
        return Ok((md, raw));
    }

    pub fn from_world_edit_12_file(filename: &str, option: &WorldEdit12LoadOption) -> Result<(Schematic, WE12MetaData, Array3<(u8, u8)>), Error> {
        let file = match File::open(filename) {
            Ok(f) => f,
            Err(e) => return Err(Error::FileOpenError(e)),
        };
        let decoder = GzDecoder::new(file);
        let nbt: HashMap<String, Value> = match fastnbt::from_reader(decoder) {
            Ok(n) => n,
            Err(e) => return Err(Error::NBTReadError(e)),
        };
        return Self::from_world_edit_12_nbt(nbt, option);
    }

    pub fn from_world_edit_12_reader(src: &mut dyn std::io::Read, option: &WorldEdit12LoadOption) -> Result<(Schematic, WE12MetaData, Array3<(u8, u8)>), Error> {
        let nbt: HashMap<String, Value> = match fastnbt::from_reader(src) {
            Ok(n) => n,
            Err(e) => return Err(Error::NBTReadError(e)),
        };
        return Self::from_world_edit_12_nbt(nbt, option);
    }

    pub fn from_world_edit_12_nbt(mut nbt: HashMap<String, Value>, option: &WorldEdit12LoadOption) -> Result<(Schematic, WE12MetaData, Array3<(u8, u8)>), Error> {
        let mut schem = Schematic::new();
        // metadata

        let (md, raw) = Self::parse_metadata(&mut nbt, option)?;
        schem.metadata = md;


        let (region, number_id) = Region::from_world_edit_12(&mut nbt, option)?;
        schem.regions.push(region);

        return Ok((schem, raw, number_id));
    }
}

#[derive(Clone, Copy)]
struct BlockStats {
    pub count: u64,
    pub id: u16,
    pub first_occur_index: u32,
}

impl Default for BlockStats {
    fn default() -> Self {
        return BlockStats {
            count: 0,
            id: u16::MAX,
            first_occur_index: u32::MAX,
        };
    }
}

impl Region {
    pub fn from_world_edit_12(nbt: &mut HashMap<String, Value>, option: &WorldEdit12LoadOption)
        -> Result<(Region, Array3<(u8, u8)>), Error> {
        let data_version = option.data_version;
        let id_damage_array = Schematic::parse_number_id_from_we12(&nbt)?;
        let mut region = Region::new();

        let mut id_damage_counter = [[BlockStats::default(); 16]; 256];
        for (idx, (id, damage)) in id_damage_array.iter().enumerate() {
            if *damage >= 16 {
                return Err(Error::InvalidBlockNumberId {
                    tag_path: format!("/Data[{idx}]"),
                    detail: OldBlockParseError::DamageMoreThan15 { damage: *damage },
                });
            }
            let stat = &mut id_damage_counter[*id as usize][*damage as usize];
            stat.count += 1;
            if stat.first_occur_index == u32::MAX {
                stat.first_occur_index = idx as u32;
            }
        }

        region.palette.clear();
        region.palette.reserve(256);
        for id in 0..256 {
            for damage in 0..16 {
                let stat = &mut id_damage_counter[id as usize][damage as usize];
                if stat.count <= 0 {
                    continue;
                }
                let block = match Block::from_old(id as u8, damage, data_version) {
                    Ok(b) => b,
                    Err(detail) => return Err(Error::InvalidBlockNumberId {
                        tag_path: format!("/Data[{}]", stat.first_occur_index),
                        detail,
                    }),
                };
                stat.id = region.palette.len() as u16;
                region.palette.push(block);
            }
        }

        let shape_usize = id_damage_array.shape();
        let shape: [i32; 3] = [shape_usize[0] as i32, shape_usize[1] as i32, shape_usize[2] as i32];
        region.reshape(&shape);

        for x in 0..shape[0] {
            for y in 0..shape[1] {
                for z in 0..shape[2] {
                    let pos = [x as usize, y as usize, z as usize];
                    let (id, damage) = id_damage_array[pos];
                    let stat = &id_damage_counter[id as usize][damage as usize];
                    debug_assert!((stat.id as usize) < region.palette.len());
                    region.array[pos] = stat.id;
                }
            }
        }

        //tile entities
        let tile_entities = unwrap_opt_tag!(nbt.get_mut("TileEntities"),List,vec![],"/TileEntities");
        region.block_entities.reserve(tile_entities.len());
        for (idx, te) in tile_entities.iter_mut().enumerate() {
            let tag_path = format!("/TileEntities[{idx}]");
            let te = unwrap_tag!(te,Compound,HashMap::new(),&tag_path);
            let pos = common::parse_size_compound(te, &tag_path, false)?;
            //check pos
            for dim in 0..3 {
                if pos[dim] < 0 || pos[dim] >= shape[dim] {
                    return Err(Error::BlockPosOutOfRange {
                        tag_path,
                        pos,
                        range: shape,
                    });
                }
            }
            let mut block_entity = BlockEntity::new();
            mem::swap(&mut block_entity.tags, te);
            for key in ["x", "y", "z"] {
                if block_entity.tags.contains_key(key) {
                    block_entity.tags.remove(key);
                }
            }
            region.block_entities.insert(pos, block_entity);
        }

        if option.fix_string_id_with_block_entity_data {
            let mut block_to_index: HashMap<Block, u16> = HashMap::new();
            block_to_index.reserve(region.palette.len() + region.block_entities.len());
            for (idx, blk) in region.palette.iter().enumerate() {
                block_to_index.insert(blk.clone(), idx as u16);
            }

            for (pos, be) in &region.block_entities {
                let pos_usize = [pos[0] as usize, pos[1] as usize, pos[2] as usize];
                let original_blk = region.block_at(*pos);
                debug_assert!(original_blk.is_some());
                let original_blk = original_blk.unwrap();
                let (id, damage) = id_damage_array[pos_usize];
                let fixed_block = match original_blk.fix_block_property_with_block_entity(id, damage, be) {
                    Ok(b) => b,
                    Err(e) => return Err(Error::InvalidBlockNumberId {
                        tag_path: "(unknown)".to_string(),
                        detail: e,
                    }),
                };

                if let Some(fixed_block) = fixed_block {
                    debug_assert!(fixed_block != *original_blk);
                    let fixed_id: u16;
                    if block_to_index.contains_key(&fixed_block) {
                        fixed_id = block_to_index[&fixed_block];
                    } else {
                        fixed_id = block_to_index.len() as u16;
                        block_to_index.insert(fixed_block, fixed_id);
                    }
                    debug_assert!(region.array[pos_usize] != fixed_id);
                    region.array[pos_usize] = fixed_id;
                }
            }
            let original_pal_len = region.palette.len();
            let full_pal_len = block_to_index.len();
            region.palette.reserve(full_pal_len);
            while region.palette.len() < full_pal_len {
                region.palette.push(Block::empty_block());
            }
            for (block, index) in block_to_index.into_iter() {
                let index = index as usize;
                if index < original_pal_len {
                    debug_assert!(region.palette[index] == block);
                    continue;
                }
                region.palette[index] = block;
            }

            for blk in &region.palette {
                debug_assert!(!blk.id.is_empty());
            }
        }

        // if option.discard_number_id_array {
        //     id_damage_array.
        // }

        return Ok((region, id_damage_array));
    }
}