use std::collections::HashMap;
use fastnbt::Value;
use crate::schem::{LitematicaMetaData, Schematic, id_of_nbt_tag, RawMetaData, MetaDataIR, Region};
use crate::error::LoadError;
use crate::{schem, unwrap_opt_tag, unwrap_tag};

impl MetaDataIR {
    pub fn from_litematica(src: &LitematicaMetaData) -> MetaDataIR {
        return MetaDataIR {
            mc_data_version: src.data_version,
            time_created: src.time_created,
            time_modified: src.time_modified,
            author: src.author.clone(),
            name: src.name.clone(),
            description: src.description.clone(),
        }
    }
}


impl Schematic {
    pub fn from_litematica(src: &mut dyn std::io::Read) -> Result<Schematic, LoadError> {
        let parse_res: Result<HashMap<String, Value>, fastnbt::error::Error> = fastnbt::from_reader(src);
        let parsed;
        match parse_res {
            Ok(nbt) => parsed = nbt,
            Err(e) => return Err(LoadError::NBTReadError(e)),
        }

        let mut schem = Schematic::new();
        match parse_metadata(&parsed) {
            Ok(md) => {
                schem.metadata = MetaDataIR::from_litematica(&md);
                schem.raw_metadata = Some(RawMetaData::Litematica(md));
            }
            Err(e) => return Err(e)
        }

        let regions = unwrap_opt_tag!(parsed.get("Regions"),Compound,HashMap::new(),"/Regions".to_string());
        schem.regions.reserve(regions.len());
        for (key, val) in regions {
            let reg = unwrap_tag!(val,Compound,HashMap::new(),format!("/Regions/{}",key));
            match parse_region(reg, &*format!("/Regions/{}", key)) {
                Ok(mut reg) => {
                    reg.name = key.clone();
                    schem.regions.push(reg);
                },
                Err(e) => return Err(e),
            }
        }


        return Ok(schem);
    }
}


fn parse_metadata(root: &HashMap<String, Value>) -> Result<LitematicaMetaData, LoadError> {
    let mut result = LitematicaMetaData::new();

    result.data_version = *unwrap_opt_tag!(root.get("MinecraftDataVersion"),Int,0,"/MinecraftDataVersion");
    result.version = *unwrap_opt_tag!(root.get("Version"),Int,0,"/Version");

    let md = unwrap_opt_tag!(root.get("Metadata"),Compound,HashMap::new(),"/Metadata".to_string());

    result.time_created = *unwrap_opt_tag!(md.get("TimeCreated"),Long,0,"/Metadata/TimeCreated".to_string());
    result.time_modified = *unwrap_opt_tag!(md.get("TimeModified"),Long,0,"/Metadata/TimeModified".to_string());
    {
        let enclosing_size = unwrap_opt_tag!(md.get("EnclosingSize"),List,vec![],"/Metadata/EnclosingSize".to_string());
        if enclosing_size.len() != 3 {
            return Err(LoadError::InvalidValue {
                tag_path: "/Metadata/EnclosingSize".to_string(),
                error: format!("Expected a list containing 3 elements, but found {}", enclosing_size.len()),
            });
        }
        for dim in 0..3 {
            let tag_path = format!("/Metadata/EnclosingSize[{}]", dim);
            let val = unwrap_tag!(enclosing_size[dim],Int,0,tag_path);
            if val < 0 {
                return Err(LoadError::InvalidValue {
                    tag_path,
                    error: format!("Negative number {} in size", val),
                });
            }
        }
    }

    result.description
        = unwrap_opt_tag!(md.get("Description"),String,"".to_string(),"/Metadata/Description".to_string()).clone();
    //result.total_volume = *unwrap_opt_tag!(md.get("TotalVolume"),Int,0,"/Metadata/TotalVolume".to_string()) as i64;
    result.author = unwrap_opt_tag!(md.get("Author"),String,"".to_string(),"/Metadata/Author".to_string()).clone();
    result.name = unwrap_opt_tag!(md.get("Name"),String,"".to_string(),"/Metadata/Name".to_string()).clone();

    return Ok(result);
}

pub fn parse_size_compound(nbt: &HashMap<String, Value>, tag_path: &str, allow_negative: bool) -> Result<[i32; 3], LoadError> {
    // let x = *unwrap_opt_tag!(nbt.get("x"),Int,0,format!("{}/x",tag_path));
    // let y = *unwrap_opt_tag!(nbt.get("y"),Int,0,format!("{}/y",tag_path));
    // let z = *unwrap_opt_tag!(nbt.get("z"),Int,0,format!("{}/z",tag_path));
    let mut result: [i32; 3] = [0, 0, 0];
    for (idx, key) in ["x", "y", "z"].iter().enumerate() {
        let cur_tag_path = format!("{}/{}", tag_path, key);
        let val = *unwrap_opt_tag!(nbt.get(*key),Int,0,cur_tag_path);
        if (!allow_negative) && (val < 0) {
            return Err(LoadError::InvalidValue {
                tag_path: cur_tag_path,
                error: format!("Expected non-negative value, but found {}", val),
            });
        }
        result[idx] = val;
    }

    return Ok(result);
}

fn parse_region(nbt: &HashMap<String, Value>, tag_path: &str) -> Result<Region, LoadError> {
    let mut region = Region::new();

    // parse position(offset)
    {
        let cur_tag_path = format!("{}/Position", tag_path);
        let position = unwrap_opt_tag!(nbt.get("Position"),Compound,HashMap::new(),cur_tag_path);
        match parse_size_compound(position, &cur_tag_path, false) {
            Ok(pos) => region.offset = pos,
            Err(e) => return Err(e),
        }
    }

    // parse palette
    {
        let palette = unwrap_opt_tag!(nbt.get("BlockStatePalette"),List,vec![],format!("{}/BlockStatePalette",tag_path));
        region.palette.reserve(palette.len());
        for (idx, blk_nbt) in palette.iter().enumerate() {
            let cur_tag_path = format!("{}/BlockStatePalette[{}]", tag_path, idx);
            let blk_nbt = unwrap_tag!(blk_nbt,Compound,HashMap::new(),&cur_tag_path);
            let block = schem::vanilla_structure::parse_block(blk_nbt, &cur_tag_path);
            match block {
                Ok(blk) => region.palette.push(blk),
                Err(e) => return Err(e),
            }
        }
    }

    // parse size
    let region_size;
    {
        let cur_tag_path = format!("{}/Size", tag_path);
        let size = unwrap_opt_tag!(nbt.get("Size"),Compound,HashMap::new(),cur_tag_path);
        match parse_size_compound(size, &cur_tag_path, false) {
            Ok(size) => {
                region.reshape(size);
                region_size = size;
            },
            Err(e) => return Err(e),
        }
    }

    
    return Ok(region);
}
