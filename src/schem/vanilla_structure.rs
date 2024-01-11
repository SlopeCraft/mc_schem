use std::collections::HashMap;
use std::fs::File;
use crate::schem::{id_of_nbt_tag, MetaDataIR, schem, VanillaStructureLoadOption, VanillaStructureSaveOption};
//use compress::zlib;
use crate::schem::schem::{BlockEntity, Entity, Schematic, VanillaStructureMetaData};
use fastnbt;
use fastnbt::{Value};
use flate2::read::GzDecoder;
use crate::block::{Block};
use crate::error::{LoadError, WriteError};
use crate::{unwrap_tag, unwrap_opt_tag};
use crate::error::LoadError::FileOpenError;
use crate::schem::RawMetaData::VanillaStructure;


impl MetaDataIR {
    pub fn from_vanilla_structure(src: &VanillaStructureMetaData) -> MetaDataIR {
        let mut result = MetaDataIR::default();
        result.mc_data_version = src.data_version;
        return result;
    }

    pub fn to_vanilla_structure(&self) -> VanillaStructureMetaData {
        let mut result = VanillaStructureMetaData::new();
        result.data_version = self.mc_data_version;
        return result;
    }
}

fn parse_size_tag(nbt: &HashMap<String, Value>) -> Result<[i32; 3], LoadError> {
    let size_list = unwrap_opt_tag!(nbt.get("size"),List,vec![],"/size");

    if size_list.len() != 3 {
        return Err(LoadError::InvalidValue {
            tag_path: "/size".to_string(),
            error: format!("The length should be 3, but found {}", size_list.len()),
        }
        );
    }
    let mut size: [i32; 3] = [0, 0, 0];
    for idx in 0..3 {
        let sz = *unwrap_tag!(&size_list[idx],Int,0,&*format!("/size[{}]", idx));
        if sz <= 0 {
            return Err(LoadError::InvalidValue {
                tag_path: format!("/size[{}]", idx),
                error: format!("Expected non-negative number, but found {}", sz),
            }
            );
        }
        size[idx] = sz;
    }
    return Ok(size);
}

pub fn parse_block(nbt: &HashMap<String, Value>, tag_path: &str) -> Result<Block, LoadError> {
    let mut blk = Block::new();

    let id = unwrap_opt_tag!(nbt.get("Name"),String,String::new(),&*format!("{}/Name", tag_path));
    let id_parse = Block::from_id(id);

    match id_parse {
        Ok(blk_temp) => blk = blk_temp,
        _ => return Err(LoadError::InvalidBlockId(String::from(id))),
    }

    let prop_comp;
    // unwrap the properties map
    if let Some(prop_tag) = nbt.get("Properties") {
        prop_comp = unwrap_tag!(prop_tag,Compound,HashMap::new(),&*format!("{}/Properties", tag_path));
    } else {
        return Ok(blk);
    }

    // parse properties
    for (key, tag) in prop_comp {
        let value = unwrap_tag!(tag,String,String::new(),&*format!("{}/Properties/{}", tag_path, key));
        blk.attributes.insert(key.to_string(), value.to_string());
    }

    return Ok(blk);
}

fn parse_array_item(item: &Value, tag_path: &str, palette_size: i32, region_size: [i32; 3]) -> Result<(i32, [i32; 3], Option<BlockEntity>), LoadError> {
    let map = unwrap_tag!(item,Compound,HashMap::new(),tag_path);

    // parse state
    let state: i32 = *unwrap_opt_tag!(map.get("state"),Int,0,&*format!("{}/state", tag_path));
    if state < 0 || state >= palette_size {
        return Err(LoadError::BlockIndexOutOfRange {
            tag_path: format!("{}/state", tag_path),
            index: state,
            range: [0, palette_size],
        });
    }

    let pos_list = unwrap_opt_tag!(map.get("pos"),List,vec![],&*format!("{}/pos", tag_path));

    if pos_list.len() != 3 {
        return Err(LoadError::InvalidValue {
            tag_path: format!("{}/pos", tag_path),
            error: format!("The length of pos should be 3, but found {}", pos_list.len()),
        });
    }

    let mut pos: [i32; 3] = [0, 0, 0];
    for idx in 0..3 {
        pos[idx] = *unwrap_tag!(&pos_list[idx],Int,0,&*format!("{}/pos[{}]", tag_path, idx));
    }
    for idx in 0..3 {
        if pos[idx] < 0 || pos[idx] >= region_size[idx] {
            return Err(LoadError::BlockPosOutOfRange {
                tag_path: format!("{}/pos[{}]", tag_path, idx),
                pos,
                range: region_size,
            });
        }
    }

    let nbt_comp;
    match map.get("nbt") {
        Some(nbt_comp_tmp) => nbt_comp = nbt_comp_tmp,
        None => return Ok((state, pos, None)),
    }

    let nbt_comp = unwrap_tag!(nbt_comp,Compound,HashMap::new(),&*format!("{}/nbt",tag_path));
    let block_entity = BlockEntity {
        tags: nbt_comp.clone(),
    };

    return Ok((state, pos, Some(block_entity)));
}

fn parse_entity(tag: &Value, tag_path: &str) -> Result<Entity, LoadError> {
    let compound = unwrap_tag!(tag,Compound,HashMap::new(),tag_path);

    let mut entity = Entity::new();
    // parse blockPos
    {
        let block_pos = unwrap_opt_tag!(compound.get("blockPos"),List,vec![],&*format!("{}/blockPos",tag_path));
        if block_pos.len() != 3 {
            return Err(LoadError::InvalidValue {
                tag_path: format!("{}/blockPos", tag_path),
                error: format!("blockPos should have 3 elements, but found {}", block_pos.len()),
            }
            );
        }

        for idx in 0..3 {
            entity.block_pos[idx] = *unwrap_opt_tag!(block_pos.get(idx),
                Int,0,
                &*format!("{}/blockPos[{}]",tag_path,idx));
        }
    }
    // parse pos
    {
        let pos = unwrap_opt_tag!(compound.get("pos"),List,vec![],&*format!("{}/pos",tag_path));
        if pos.len() != 3 {
            return Err(LoadError::InvalidValue {
                tag_path: format!("{}/pos", tag_path),
                error: format!("blockPos should have 3 elements, but found {}", pos.len()),
            });
        }

        for idx in 0..3 {
            entity.position[idx] = *unwrap_opt_tag!(pos.get(idx),
                Double,0.0,
                &*format!("{}/pos[{}]",tag_path,idx));
        }
    }

    // parse nbt
    {
        let nbt = unwrap_opt_tag!(compound.get("nbt"),
            Compound,HashMap::new(),&*format!("{}/nbt",tag_path));
        entity.tags = nbt.clone();
    }
    return Ok(entity);
}


impl Schematic {
    pub fn from_vanilla_structure_file(filename: &str, option: &VanillaStructureLoadOption) -> Result<Schematic, LoadError> {
        let mut file_res = File::open(filename);
        let mut file;
        match file_res {
            Ok(f) => file = f,
            Err(e) => return Err(LoadError::FileOpenError(e)),
        }

        let mut decoder = GzDecoder::new(&mut file);
        return Self::from_vanilla_structure(&mut decoder, option);
    }
    pub fn from_vanilla_structure(src: &mut dyn std::io::Read, option: &VanillaStructureLoadOption)
                                  -> Result<Schematic, LoadError> {
        let loaded_opt: Result<HashMap<String, Value>, fastnbt::error::Error> = fastnbt::from_reader(src);
        let nbt;
        match loaded_opt {
            Ok(loaded_nbt) => nbt = loaded_nbt,
            Err(err) => return Err(LoadError::NBTReadError(err)),
        }

        let mut schem = Schematic::new();

        {
            let mut md = VanillaStructureMetaData::new();
            md.data_version = *unwrap_opt_tag!(nbt.get("DataVersion"),Int,0,"/DataVersion");
            schem.metadata = MetaDataIR::from_vanilla_structure(&md);
            schem.raw_metadata = Some(VanillaStructure(md));
        }

        let mut region = schem::Region::new();
        //setup basic info for region
        {
            region.offset = [0, 0, 0];
            region.name.clear();
        }

        // set up size
        let region_size = parse_size_tag(&nbt);
        if let Err(err) = region_size {
            return Err(err);
        }
        let region_size = region_size.unwrap();
        region.reshape(region_size);

        //parse block palette
        {
            let palette_list = unwrap_opt_tag!(nbt.get("palette"),List,vec![],"/palette");

            region.palette.reserve(palette_list.len());

            for (idx, blk_tag) in palette_list.iter().enumerate() {
                let tag_path = format!("/palette[{}]", idx);

                let blk_comp = unwrap_tag!(blk_tag,Compound,HashMap::new(),&tag_path);
                let blk = parse_block(blk_comp, &tag_path);
                match blk {
                    Err(err) => return Err(err),
                    Ok(blk) => region.palette.push(blk),
                }
            }
        }

        if region.palette.len() >= 65536 {
            return Err(LoadError::PaletteTooLong(region.palette.len()));
        }
        let default_blk_idx: u16;
        {
            let mut di = region.palette.len();
            let default_blk = option.background_block.to_block();
            for (idx, blk) in region.palette.iter().enumerate() {
                if blk == &default_blk {
                    di = idx;
                    break;
                }
            }
            if di == region.palette.len() {
                region.palette.push(default_blk);
            }
            default_blk_idx = di as u16;
        }

        // fill region with structure void
        region.array.fill(default_blk_idx);

        // fill in blocks
        {
            let blocks_list = unwrap_opt_tag!(nbt.get("blocks"),List,vec![],"/blocks");

            for (idx, blk_item) in blocks_list.iter().enumerate() {
                let blk_item = parse_array_item(blk_item,
                                                &*format!("/blocks[{}]", idx),
                                                region.palette.len() as i32,
                                                [region_size[0], region_size[1], region_size[2]]);
                let state;
                let pos;
                let block_entity_opt;
                match blk_item {
                    Ok(unwrapped_tmp) => (state, pos, block_entity_opt) = unwrapped_tmp,
                    Err(e) => return Err(e),
                }

                let pos_ndarr = [pos[0] as usize, pos[1] as usize, pos[2] as usize];
                region.array[pos_ndarr] = state as u16;

                if let Some(block_entity) = block_entity_opt {
                    region.block_entities.insert([pos[0], pos[1], pos[2]], block_entity);
                }
            }
        }

        // fill in entities
        {
            // unwrap the list
            let entity_list = unwrap_opt_tag!(nbt.get("entities"),List,vec![],"/entities");
            for (idx, entity_tag) in entity_list.iter().enumerate() {
                let tag_path = format!("/entities[{}]", idx);
                let parsed_entity = parse_entity(entity_tag, &tag_path);
                match parsed_entity {
                    Ok(e) => region.entities.push(e),
                    Err(e) => return Err(e),
                }
            }
        }

        {
            let shrink_err = region.shrink_palette();
            assert!(shrink_err.is_ok());
        }

        schem.regions.push(region);
        return Ok(schem);
    }
}

fn block_entity_to_nbt(be: &BlockEntity) -> HashMap<String, Value> {
    return be.tags.clone();
}

fn block_to_nbt(pos: [i32; 3], state: i32, be: &Option<&BlockEntity>) -> HashMap<String, Value> {
    let mut result: HashMap<String, Value> = HashMap::new();
    result.insert(String::from("state"), Value::Int(state));
    result.insert(String::from("pos"), pos_to_nbt(&pos));


    if let Some(be) = be {
        result.insert(String::from("nbt"), Value::Compound(block_entity_to_nbt(be)));
    }

    return result;
}

fn pos_to_nbt(pos: &[i32; 3]) -> Value {
    let mut pos_list = Vec::with_capacity(3);
    for p in pos {
        pos_list.push(Value::Int(*p));
    }
    return Value::List(pos_list);
}

impl Schematic {
    pub fn to_nbt_vanilla_structure(&self, option: &VanillaStructureSaveOption) -> Result<HashMap<String, Value>, WriteError> {
        let mut nbt: HashMap<String, Value> = HashMap::new();

        {
            let mut size = Vec::with_capacity(3);
            for dim in 0..3 {
                size.push(Value::Int(self.regions[0].shape()[dim]));
            }
            nbt.insert(String::from("size"), Value::List(size));
        }

        let (full_palette, luts_of_block_idx) = self.full_palette();
        {
            let mut nbt_palette = Vec::with_capacity(full_palette.len());
            for (blk, _) in full_palette {
                let mut cur_blk_nbt: HashMap<String, Value> = HashMap::new();
                cur_blk_nbt.insert(String::from("Name"),
                                   Value::String(format!("{}:{}", blk.namespace, blk.id)));
                if !blk.attributes.is_empty() {
                    let mut props: HashMap<String, Value> = HashMap::new();
                    for (key, val) in &blk.attributes {
                        props.insert(key.clone(), Value::String(val.clone()));
                    }
                    cur_blk_nbt.insert(String::from("Properties"), Value::Compound(props));
                }
                nbt_palette.push(Value::Compound(cur_blk_nbt));
            }
            nbt.insert(String::from("palette"), Value::List(nbt_palette));
        }

        let shape = self.shape();

        {
            let mut blocks: Vec<Value> = Vec::with_capacity(self.volume() as usize);
            for x in 0..shape[0] {
                for y in 0..shape[1] {
                    for z in 0..shape[2] {
                        let g_pos = [x, y, z];

                        let mut first_region_idx = None;
                        let mut first_r_blk_info = None;

                        for (reg_idx, reg) in self.regions.iter().enumerate() {
                            let r_pos = reg.global_pos_to_relative_pos(g_pos);
                            let r_blk_info = reg.block_info_at(r_pos);
                            if let Some(r_blk_info) = r_blk_info {
                                first_region_idx = Some(reg_idx);
                                first_r_blk_info = Some(r_blk_info);
                                break;
                            } else {
                                continue;
                            }
                        }

                        if let None = first_region_idx {
                            // there is no block through out all regions
                            continue;
                        }

                        assert_eq!(first_r_blk_info.is_some(), first_region_idx.is_some());
                        let first_region_idx: usize = first_region_idx.unwrap();
                        let first_r_blk_info = first_r_blk_info.unwrap();
                        let g_blk_id = luts_of_block_idx[first_region_idx][first_r_blk_info.0 as usize];

                        if first_r_blk_info.1.is_structure_void() {
                            continue;
                        }

                        if (!option.keep_air) && (first_r_blk_info.1.id == "air") {
                            continue;
                        }

                        let mut cur_nbt: HashMap<String, Value> = HashMap::new();
                        cur_nbt.insert(String::from("state"), Value::Int(g_blk_id as i32));
                        {
                            cur_nbt.insert(String::from("pos"), pos_to_nbt(&g_pos));
                        }
                        if let Some(be) = first_r_blk_info.2 {
                            cur_nbt.insert(String::from("nbt"), Value::Compound(be.tags.clone()));
                        }
                        blocks.push(Value::Compound(cur_nbt));
                    }
                }
            }
            nbt.insert(String::from("blocks"), Value::List(blocks));
        }

        {
            let mut entities: Vec<Value> = Vec::new();
            for reg in &self.regions {
                for entity in &reg.entities {
                    let mut nbt = HashMap::new();
                    let mut block_pos = Vec::with_capacity(3);
                    let mut pos = Vec::with_capacity(3);
                    for dim in 0..3 {
                        block_pos.push(Value::Int(entity.block_pos[dim]));
                        pos.push(Value::Double(entity.position[dim]));
                    }
                    nbt.insert(String::from("blockPos"), Value::List(block_pos));
                    nbt.insert(String::from("pos"), Value::List(pos));
                    nbt.insert(String::from("nbt"), Value::Compound(entity.tags.clone()));

                    entities.push(Value::Compound(nbt));
                }
            }
            nbt.insert(String::from("entities"), Value::List(entities));
        }

        nbt.insert(String::from("DataVersion"), Value::Int(self.metadata.mc_data_version));

        return Ok(nbt);
    }

    pub fn save_vanilla_structure(&self, dst: &mut dyn std::io::Write, option: &VanillaStructureSaveOption) -> Result<(), WriteError> {
        let nbt;
        match self.to_nbt_vanilla_structure(option) {
            Ok(nbt_) => nbt = nbt_,
            Err(e) => return Err(e),
        }


        let res: Result<(), fastnbt::error::Error> = fastnbt::to_writer(dst, &nbt);
        return match res {
            Err(err) => Err(WriteError::NBTWriteError(err)),
            _ => Ok(())
        }
    }
}

