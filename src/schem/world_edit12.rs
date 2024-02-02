use std::collections::HashMap;
use std::mem;
use fastnbt::Value;
use ndarray::Array3;
use crate::block::Block;
use crate::error::LoadError;
use crate::old_block::OldBlockParseError;
use crate::region::{BlockEntity, Region};
use crate::schem::{common, id_of_nbt_tag, MetaDataIR, RawMetaData, Schematic, WE12MetaData, WorldEdit12LoadOption};
use crate::{unwrap_opt_tag, unwrap_tag};


fn i8_to_u8(a: i8) -> u8 {
    return if a >= 0 {
        a as u8
    } else {
        (256 + a as i16) as u8
    }
}

impl Schematic {
    pub fn parse_number_id_from_we12(nbt: &HashMap<String, Value>) -> Result<Array3<(u8, u8)>, LoadError> {
        let x_size = *unwrap_opt_tag!(nbt.get("Width"),Short,0,"/Width".to_string()) as usize;
        let y_size = *unwrap_opt_tag!(nbt.get("Height"),Short,0,"/Height".to_string()) as usize;
        let z_size = *unwrap_opt_tag!(nbt.get("Length"),Short,0,"/Length".to_string()) as usize;
        let mut array = Array3::default([x_size, y_size, z_size]);

        let blocks = unwrap_opt_tag!(nbt.get("Blocks"),ByteArray,fastnbt::ByteArray::new(vec![]),"/Blocks".to_string());
        let data = unwrap_opt_tag!(nbt.get("Data"),ByteArray,fastnbt::ByteArray::new(vec![]),"/Data".to_string());

        {
            let expected_elements = x_size * y_size * z_size;
            if blocks.len() != expected_elements {
                return Err(LoadError::InvalidValue {
                    tag_path: "/Blocks".to_string(),
                    error: format!("Expected to contain {expected_elements} elements but found {}.", blocks.len()),
                });
            }
            if data.len() != blocks.len() {
                return Err(LoadError::InvalidValue {
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

    fn parse_metadata(nbt: &mut HashMap<String, Value>, option: &WorldEdit12LoadOption) -> Result<(MetaDataIR, WE12MetaData), LoadError> {
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
        return Ok((md, raw));
    }

    pub fn from_world_edit_12(mut nbt: HashMap<String, Value>, option: &WorldEdit12LoadOption) -> Result<Schematic, LoadError> {
        let mut schem = Schematic::new();
        // metadata
        {
            let (md, raw) = Self::parse_metadata(&mut nbt, option)?;
            schem.metadata = md;
            schem.raw_metadata = Some(RawMetaData::WE12(raw));
        }

        let region = Region::from_world_edit_12(&mut nbt, option)?;
        schem.regions.push(region);

        return Ok(schem);
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
                              -> Result<Region, LoadError> {
        let data_version = option.data_version;
        let id_damage_array = Schematic::parse_number_id_from_we12(&nbt)?;
        let mut region = Region::new();

        let mut id_damage_counter = [[BlockStats::default(); 16]; 256];
        for (idx, (id, damage)) in id_damage_array.iter().enumerate() {
            if *damage >= 16 {
                return Err(LoadError::InvalidBlockNumberId {
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
                    Err(detail) => return Err(LoadError::InvalidBlockNumberId {
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
                    return Err(LoadError::BlockPosOutOfRange {
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

        return Ok(region);
    }
}