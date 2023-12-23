mod vanilla_structure {
    use std::fmt::format;
    use crate::schem::schem;
    //use compress::zlib;
    use crate::schem::schem::{MetaData, Schematic, VanillaStructureMetaData};
    use nbt;
    use nbt::{Blob, Map, Value};
    use nbt::Value::{Compound, Int};
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
    pub enum VanillaStructureLoadError {
        NBTReadError(nbt::Error),
        TagTypeMismatch(TagTypeMismatchDetail),
        InvalidValue(TagValueInvalidDetail),
        TagMissing(String),
        InvalidBlockId(String),
        InvalidBlockProperty(TagValueInvalidDetail)
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
            ))
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
                ))
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
            }
            {
                if let Some(dv) = nbt.get("/DataVersion") {
                    if let Value::Int(dv) = dv {
                        schem.data_version = *dv;
                    } else {
                        return Err(VanillaStructureLoadError::TagTypeMismatch(TagTypeMismatchDetail::new("/DataVersion", Int(0), dv)));
                    }
                } else {
                    return Err(VanillaStructureLoadError::TagMissing(String::from("/DataVersion")));
                }
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
                            TagTypeMismatchDetail::new(&tag_path, Compound(Map::new()), blk_tag)
                        ));
                    }
                }
            }


            schem.regions.push(region);
            return Ok(schem);
        }
    }
}
