use std::collections::HashMap;
use fastnbt::Value;
use crate::error::LoadError;
use crate::{unwrap_opt_tag, schem::{id_of_nbt_tag}, unwrap_tag};
use crate::block::Block;

pub fn size_to_compound<T>(size: &[T; 3]) -> HashMap<String, Value>
    where T: Copy, Value: From<T>
{
    return HashMap::from([("x".to_string(), Value::from(size[0])),
        ("y".to_string(), Value::from(size[1])),
        ("z".to_string(), Value::from(size[2]))]);
}

pub fn size_to_list<T>(size: &[T; 3]) -> Vec<Value>
    where T: Copy, Value: From<T> {
    return vec![Value::from(size[0]), Value::from(size[1]), Value::from(size[2])];
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


pub fn parse_size_list(data: &[i32], tag_path: &str, allow_negative: bool) -> Result<[i32; 3], LoadError> {
    let mut result: [i32; 3] = [0, 0, 0];
    if data.len() != 3 {
        return Err(LoadError::InvalidValue {
            tag_path: tag_path.to_string(),
            error: format!("Expected a list with 3 elements, but found {}", data.len()),
        });
    }

    for dim in 0..3 {
        let val = data[dim];
        if (!allow_negative) && (val < 0) {
            return Err(LoadError::InvalidValue {
                tag_path: format!("{}[{}]", tag_path, dim),
                error: format!("Expected non-negative value, but found {}", val),
            });
        }
        result[dim] = val;
    }

    return Ok(result);
}

pub fn ceil_up_to(a: isize, b: isize) -> isize {
    assert!(b > 0);
    if (a % b) == 0 {
        return a;
    }
    return ((a / b) + 1) * b;
}


pub fn parse_block(nbt: &HashMap<String, Value>, tag_path: &str) -> Result<Block, LoadError> {
    let mut blk;

    let id = unwrap_opt_tag!(nbt.get("Name"),String,String::new(),&*format!("{}/Name", tag_path));
    let id_parse = Block::from_id(id);

    match id_parse {
        Ok(blk_temp) => blk = blk_temp,
        Err(e) => return Err(LoadError::InvalidBlockId { id: id.clone(), reason: e }),
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
