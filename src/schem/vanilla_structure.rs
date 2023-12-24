mod vanilla_structure {
    use crate::schem::schem;
    //use compress::zlib;
    use crate::schem::schem::{BlockEntity, Entity, MetaData, Schematic, VanillaStructureMetaData};
    use nbt;
    use nbt::{Blob, Map, Value};
    use crate::block::Block;

    #[derive(Debug)]
    pub struct TagTypeMismatchDetail {
        pub tag_path: String,
        pub expected_type: u8,
        pub found_type: u8,
    }

    impl TagTypeMismatchDetail {
        pub fn new(tag_path: &str, expected: Value, found: &Value) -> TagTypeMismatchDetail {
            let mut result = TagTypeMismatchDetail {
                tag_path: String::from(tag_path),
                expected_type: 255,
                found_type: 255,
            };

            result.expected_type = expected.id();
            result.found_type = found.id();


            assert_ne!(result.expected_type, result.found_type);

            return result;
        }
    }

    #[derive(Debug)]
    pub struct TagValueInvalidDetail {
        pub tag_path: String,
        pub error: String,
    }

    #[derive(Debug)]
    pub struct BlockIndexOutOfRangeDetail {
        pub tag_path: String,
        pub index: i32,
        pub range: [i32; 2],
    }

    #[derive(Debug)]
    pub struct BlockPosOutOfRangeDetail {
        pub tag_path: String,
        pub pos: [i32; 3],
        pub range: [i32; 3],
    }

    #[derive(Debug)]
    pub enum VanillaStructureLoadError {
        NBTReadError(nbt::Error),
        TagTypeMismatch(TagTypeMismatchDetail),
        InvalidValue(TagValueInvalidDetail),
        TagMissing(String),
        InvalidBlockId(String),
        InvalidBlockProperty(TagValueInvalidDetail),
        PaletteTooLong(usize),
        BlockIndexOutOfRange(BlockIndexOutOfRangeDetail),
        BlockPosOutOfRange(BlockPosOutOfRangeDetail),
    }

    struct VanillaStructureLoadOption {
        pub keep_structure_void: bool,
    }

    impl VanillaStructureLoadOption {
        pub fn default() -> VanillaStructureLoadOption {
            return VanillaStructureLoadOption {
                keep_structure_void: false
            }
        }
    }

    macro_rules! unwrap_opt_tag {
        ($value_opt:expr,$expected_type:ident,$expected_default_ctor:expr,$tag_path:expr) => {
            if let Some(value)=$value_opt {
                if let Value::$expected_type(unwrapped)=value {
                    unwrapped
                }else {
                    return Err(VanillaStructureLoadError::TagTypeMismatch(
                        TagTypeMismatchDetail::new($tag_path,Value::$expected_type($expected_default_ctor),value)
                    ));
                }
            } else {
                return Err(VanillaStructureLoadError::TagMissing(String::from($tag_path)));
            }
        };
    }


    macro_rules! unwrap_tag {
        ($value:expr,$expected_type:ident,$expected_default_ctor:expr,$tag_path:expr) => {
                if let Value::$expected_type(unwrapped)=$value {
                    unwrapped
                }else {
                    return Err(VanillaStructureLoadError::TagTypeMismatch(
                        TagTypeMismatchDetail::new($tag_path,Value::$expected_type($expected_default_ctor),$value)
                    ));
                }
        };
    }


    fn parse_size_tag(nbt: &Blob) -> Result<[i64; 3], VanillaStructureLoadError> {
        let size_list;
        if let Some(size_tag) = nbt.get("/size") {
            if let Value::List(size_list_) = size_tag {
                size_list = size_list_;
            } else {
                return Err(VanillaStructureLoadError::TagTypeMismatch(
                    TagTypeMismatchDetail::new("/size", Value::List(vec![Value::Int(0)]), size_tag)
                ));
            }
        } else {
            return Err(VanillaStructureLoadError::TagMissing(String::from("/size")));
        }

        if size_list.len() != 3 {
            return Err(VanillaStructureLoadError::InvalidValue(
                TagValueInvalidDetail {
                    tag_path: String::from("/size"),
                    error: format!("The length should be 3, but found {}", size_list.len()),
                }
            ));
        }
        let mut size: [i64; 3] = [0, 0, 0];
        for idx in 0..3 {
            let size_i = &size_list[idx];
            if let Value::Int(sz) = size_i {
                if *sz <= 0 {
                    return Err(VanillaStructureLoadError::InvalidValue(
                        TagValueInvalidDetail {
                            tag_path: format!("/size[{}]", idx),
                            error: format!("Expected non-negative number, but found {}", sz),
                        }
                    ));
                }
                size[idx] = *sz as i64;
            } else {
                return Err(VanillaStructureLoadError::TagTypeMismatch(
                    TagTypeMismatchDetail {
                        tag_path: format!("/size[{}]", idx),
                        expected_type: Value::Int(0).id(),
                        found_type: size_i.id(),
                    }
                ));
            }
        }
        return Ok(size);
    }

    fn parse_block(nbt: &Map<String, Value>, tag_path: &str) -> Result<Block, VanillaStructureLoadError> {
        let mut blk = Block::new();
        if let Some(name_tag) = nbt.get("Name") {
            if let Value::String(id) = name_tag {
                let id_parse = Block::from_id(id);

                match id_parse {
                    Ok(blk_temp) => blk = blk_temp,
                    _ => return Err(VanillaStructureLoadError::InvalidBlockId(String::from(id))),
                }
            } else {
                return Err(VanillaStructureLoadError::TagTypeMismatch(
                    TagTypeMismatchDetail::new(&*format!("{}/Name", tag_path),
                                               Value::String(String::new()), name_tag)
                ));
            }
        } else {
            return Err(VanillaStructureLoadError::TagMissing(format!("{}/Name", tag_path)));
        }

        let prop_comp;
        // unwrap the properties map
        if let Some(prop_tag) = nbt.get("Properties") {
            if let Value::Compound(prop_list_temp) = prop_tag {
                prop_comp = prop_list_temp;
            } else {
                return Err(VanillaStructureLoadError::TagTypeMismatch(
                    TagTypeMismatchDetail::new(
                        &*format!("{}/Properties", tag_path),
                        Value::Compound(Map::new()),
                        prop_tag,
                    )
                ));
            }
        } else {
            return Ok(blk);
        }

        // parse properties
        for (key, tag) in prop_comp {
            if let Value::String(value) = tag {
                blk.attributes.insert(key.to_string(), value.to_string());
            } else {
                return Err(VanillaStructureLoadError::TagTypeMismatch(
                    TagTypeMismatchDetail::new(
                        &*format!("{}/Properties/{}", tag_path, key),
                        Value::String(String::new()),
                        tag,
                    )));
            }
        }

        return Ok(blk);
    }

    fn parse_array_item(item: &Value, tag_path: &str, palette_size: i32, region_size: [i32; 3]) -> Result<(i32, [i32; 3], Option<BlockEntity>), VanillaStructureLoadError> {
        let map;
        if let Value::Compound(map_) = item {
            map = map_;
        } else {
            return Err(VanillaStructureLoadError::TagTypeMismatch(
                TagTypeMismatchDetail::new(
                    tag_path, Value::Compound(Map::new()), item,
                )));
        }

        // parse state
        let state: i32;
        if let Some(state_tag) = map.get("state") {
            if let Value::Int(state_val) = state_tag {
                state = *state_val;
            } else {
                return Err(VanillaStructureLoadError::TagTypeMismatch(TagTypeMismatchDetail::new(
                    &*format!("{}/state", tag_path), Value::Int(0), state_tag,
                )));
            }
        } else {
            return Err(VanillaStructureLoadError::TagMissing(format!("{}/state", tag_path)));
        }
        if state < 0 || state >= palette_size {
            return Err(VanillaStructureLoadError::BlockIndexOutOfRange(
                BlockIndexOutOfRangeDetail {
                    tag_path: format!("{}/state", tag_path),
                    index: state,
                    range: [0, palette_size],
                }));
        }

        let pos_list;
        if let Some(pos_tag) = map.get("pos") {
            if let Value::List(pos_list_temp) = pos_tag {
                pos_list = pos_list_temp;
            } else {
                return Err(VanillaStructureLoadError::TagTypeMismatch(TagTypeMismatchDetail::new(
                    &*format!("{}/pos", tag_path),
                    Value::List(vec![]),
                    pos_tag,
                )));
            }
        } else {
            return Err(VanillaStructureLoadError::TagMissing(format!("{}/pos", tag_path)));
        }

        if pos_list.len() != 3 {
            return Err(VanillaStructureLoadError::InvalidValue(TagValueInvalidDetail {
                tag_path: format!("{}/pos", tag_path),
                error: format!("The length of pos should be 3, but found {}", pos_list.len()),
            }));
        }

        let mut pos: [i32; 3] = [0, 0, 0];
        for idx in 0..3 {
            if let Value::Int(coord) = pos_list[idx] {
                pos[idx] = coord;
            } else {
                return Err(VanillaStructureLoadError::TagTypeMismatch(
                    TagTypeMismatchDetail::new(
                        &*format!("{}/pos[{}]", tag_path, idx),
                        Value::Int(0),
                        &pos_list[idx],
                    )));
            }
        }
        for idx in 0..3 {
            if pos[idx] < 0 || pos[idx] >= region_size[idx] {
                return Err(VanillaStructureLoadError::BlockPosOutOfRange(
                    BlockPosOutOfRangeDetail {
                        tag_path: format!("{}/pos[{}]", tag_path, idx),
                        pos,
                        range: region_size,
                    }));
            }
        }

        let nbt_comp;
        match map.get("nbt") {
            Some(nbt_comp_tmp) => nbt_comp = nbt_comp_tmp,
            None => return Ok((state, pos, None)),
        }

        let nbt_comp = unwrap_tag!(nbt_comp,Compound,Map::new(),&*format!("{}/nbt",tag_path));
        let block_entity = BlockEntity {
            tags: nbt_comp.clone(),
        };

        return Ok((state, pos, Some(block_entity)));
    }

    fn parse_entity(tag: &Value, tag_path: &str) -> Result<Entity, VanillaStructureLoadError> {
        let compound = unwrap_tag!(tag,Compound,Map::new(),tag_path);

        let mut entity = Entity::new();
        // parse blockPos
        {
            let block_pos = unwrap_opt_tag!(compound.get("blockPos"),List,vec![],&*format!("{}/blockPos",tag_path));
            if block_pos.len() != 3 {
                return Err(VanillaStructureLoadError::InvalidValue(
                    TagValueInvalidDetail {
                        tag_path: format!("{}/blockPos", tag_path),
                        error: format!("blockPos should have 3 elements, but found {}", block_pos.len()),
                    }
                ));
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
                return Err(VanillaStructureLoadError::InvalidValue(
                    TagValueInvalidDetail {
                        tag_path: format!("{}/pos", tag_path),
                        error: format!("blockPos should have 3 elements, but found {}", pos.len()),
                    }
                ));
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
                Compound,Map::new(),&*format!("{}/nbt",tag_path));
            entity.tags = nbt.clone();
        }
        return Ok(entity);
    }


    impl Schematic {
        pub fn from_vanilla_structure(src: &mut dyn std::io::Read) -> Result<Schematic, VanillaStructureLoadError> {
            let loaded_opt: Result<Blob, nbt::Error> = nbt::from_gzip_reader(src);
            let nbt: Blob;
            match loaded_opt {
                Ok(loaded_nbt) => nbt = loaded_nbt,
                Err(err) => return Err(VanillaStructureLoadError::NBTReadError(err)),
            }

            let mut schem = Schematic::new();

            {
                let md = VanillaStructureMetaData::new();
                schem.metadata = MetaData::VanillaStructure(md);
            }

            let mut region = schem::Region::new();
            //setup basic info for region
            {
                region.offset = [0, 0, 0];
                region.name.clear();

                schem.data_version = *unwrap_opt_tag!(nbt.get("DataVersion"),Int,0,"/DataVersion");
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
                let palette_list;
                if let Some(palette_tag) = nbt.get("properties") {
                    if let Value::List(plist_tmp) = palette_tag {
                        palette_list = plist_tmp;
                    } else {
                        return Err(VanillaStructureLoadError::TagTypeMismatch(
                            TagTypeMismatchDetail::new("/palette", Value::List(vec![]), palette_tag)
                        ));
                    }
                } else {
                    return Err(VanillaStructureLoadError::TagMissing(
                        String::from("/palette")
                    ));
                }

                region.palette.reserve(palette_list.len());

                for (idx, blk_tag) in palette_list.iter().enumerate() {
                    let tag_path = format!("/palette[{}]", idx);
                    if let Value::Compound(blk_comp) = blk_tag {
                        let blk = parse_block(blk_comp, &tag_path);
                        match blk {
                            Err(err) => return Err(err),
                            Ok(blk) => region.palette.push(blk),
                        }
                    } else {
                        return Err(VanillaStructureLoadError::TagTypeMismatch(
                            TagTypeMismatchDetail::new(&tag_path, Value::Compound(Map::new()), blk_tag)
                        ));
                    }
                }
            }

            if region.palette.len() >= 65536 {
                return Err(VanillaStructureLoadError::PaletteTooLong(region.palette.len()));
            }
            // adding structure void and compute structure void index
            let structure_void_idx: u16;
            {
                let mut svi = region.palette.len() as u16;
                for (idx, blk) in region.palette.iter().enumerate() {
                    if blk.is_structure_void() {
                        svi = idx as u16;
                        break;
                    }
                }
                if svi as usize >= region.palette.len() {
                    region.palette.push(Block::structure_void());
                }
                structure_void_idx = svi;
            }

            // fill region with structure void
            region.array.fill(structure_void_idx);

            // fill in blocks
            {
                let blocks_tag;
                if let Some(tag) = nbt.get("blocks") {
                    blocks_tag = tag;
                } else {
                    return Err(VanillaStructureLoadError::TagMissing(String::from("/blocks")));
                }
                let blocks_list;
                if let Value::List(list_temp) = blocks_tag {
                    blocks_list = list_temp;
                } else {
                    return Err(VanillaStructureLoadError::TagTypeMismatch(TagTypeMismatchDetail::new(
                        "/blocks",
                        Value::List(vec![]),
                        blocks_tag,
                    )));
                }

                for (idx, blk_item) in blocks_list.iter().enumerate() {
                    let blk_item = parse_array_item(blk_item,
                                                    &*format!("/blocks[{}]", idx),
                                                    region.palette.len() as i32,
                                                    [region_size[0] as i32, region_size[1] as i32, region_size[2] as i32]);
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
                        region.block_entities.insert([pos[0] as i64, pos[1] as i64, pos[2] as i64], block_entity);
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

            schem.regions.push(region);
            return Ok(schem);
        }
    }
}
