/*
mc_schem is a rust library to generate, load, manipulate and save minecraft schematic files.
Copyright (C) 2024  joseph

This program is free software: you can redistribute it and/or modify
it under the terms of the GNU General Public License as published by
the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU General Public License for more details.

You should have received a copy of the GNU General Public License
along with this program.  If not, see <https://www.gnu.org/licenses/>.
*/

use crate::block::Block;
use crate::error::Error;
use crate::region::{BlockEntity, Entity};
use crate::{schem::id_of_nbt_tag, unwrap_opt_tag, unwrap_tag};
use fastnbt::Value;
use std::collections::HashMap;
use std::fmt::Display;
use std::ops::{Add, Range};
use std::time;
use std::time::SystemTime;

pub fn size_to_compound<T>(size: &[T; 3]) -> HashMap<String, Value>
where
    T: Copy,
    Value: From<T>,
{
    return HashMap::from([
        ("x".to_string(), Value::from(size[0])),
        ("y".to_string(), Value::from(size[1])),
        ("z".to_string(), Value::from(size[2])),
    ]);
}

pub fn size_to_list<T>(size: &[T; 3]) -> Vec<Value>
where
    T: Copy,
    Value: From<T>,
{
    return vec![
        Value::from(size[0]),
        Value::from(size[1]),
        Value::from(size[2]),
    ];
}

pub fn size_i32_abs(pos: [i32; 3]) -> [i32; 3] {
    return [pos[0].abs(), pos[1].abs(), pos[2].abs()];
}

pub fn parse_size_compound(
    nbt: &HashMap<String, Value>,
    tag_path: &str,
    allow_negative: bool,
) -> Result<[i32; 3], Error> {
    // let x = *unwrap_opt_tag!(nbt.get("x"),Int,0,format!("{}/x",tag_path));
    // let y = *unwrap_opt_tag!(nbt.get("y"),Int,0,format!("{}/y",tag_path));
    // let z = *unwrap_opt_tag!(nbt.get("z"),Int,0,format!("{}/z",tag_path));
    let mut result: [i32; 3] = [0, 0, 0];
    for (idx, key) in ["x", "y", "z"].iter().enumerate() {
        let cur_tag_path = format!("{}/{}", tag_path, key);
        let val = *unwrap_opt_tag!(nbt.get(*key), Int, 0, cur_tag_path);
        if (!allow_negative) && (val < 0) {
            return Err(Error::InvalidValue {
                tag_path: cur_tag_path,
                error: format!("Expected non-negative value, but found {}", val),
            });
        }
        result[idx] = val;
    }

    return Ok(result);
}
pub fn parse_size_list(
    data: &[i32],
    tag_path: &str,
    allow_negative: bool,
) -> Result<[i32; 3], Error> {
    let mut result: [i32; 3] = [0, 0, 0];
    if data.len() != 3 {
        return Err(Error::InvalidValue {
            tag_path: tag_path.to_string(),
            error: format!("Expected a list with 3 elements, but found {}", data.len()),
        });
    }

    for dim in 0..3 {
        let val = data[dim];
        if (!allow_negative) && (val < 0) {
            return Err(Error::InvalidValue {
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

pub fn parse_block(nbt: &HashMap<String, Value>, tag_path: &str) -> Result<Block, Error> {
    let mut blk;

    let id = unwrap_opt_tag!(
        nbt.get("Name"),
        String,
        String::new(),
        &*format!("{}/Name", tag_path)
    );
    let id_parse = Block::from_id(id);

    match id_parse {
        Ok(blk_temp) => blk = blk_temp,
        Err(e) => {
            return Err(Error::InvalidBlockId {
                id: id.clone(),
                reason: e,
            })
        }
    }

    let prop_comp;
    // unwrap the properties map
    if let Some(prop_tag) = nbt.get("Properties") {
        prop_comp = unwrap_tag!(
            prop_tag,
            Compound,
            HashMap::new(),
            &*format!("{}/Properties", tag_path)
        );
    } else {
        return Ok(blk);
    }

    // parse properties
    for (key, tag) in prop_comp {
        let value = unwrap_tag!(
            tag,
            String,
            String::new(),
            &*format!("{}/Properties/{}", tag_path, key)
        );
        blk.attributes.insert(key.to_string(), value.to_string());
    }

    return Ok(blk);
}

pub fn format_size<T>(size: &[T; 3]) -> String
where
    T: Display,
{
    return format!("[{}, {}, {}]", size[0], size[1], size[2]);
}

pub fn i64_ms_timestamp_to_system_time(timestamp: i64) -> SystemTime {
    let time = time::UNIX_EPOCH.add(time::Duration::from_millis(timestamp as u64));
    return time;
}

pub fn parse_entity_litematica(
    nbt: HashMap<String, Value>,
    tag_path: &str,
) -> Result<Entity, Error> {
    let mut entity = Entity::new();
    {
        let tag_pos_path = format!("{}/Pos", tag_path);
        let pos = unwrap_opt_tag!(nbt.get("Pos"), List, vec![], tag_pos_path);
        if pos.len() != 3 {
            return Err(Error::InvalidValue {
                tag_path: tag_pos_path,
                error: format!(
                    "Pos field for an entity should contain 3 doubles, but found {}",
                    pos.len()
                ),
            });
        }

        let mut pos_d = [0.0, 0.0, 0.0];
        for dim in 0..3 {
            let cur_tag_path = format!("{}/Pos[{}]", tag_path, dim);
            pos_d[dim] = unwrap_tag!(pos[dim], Double, 0.0, cur_tag_path);
            entity.block_pos[dim] = pos_d[dim] as i32;
        }

        entity.position = pos_d;
    }

    entity.tags = nbt;
    return Ok(entity);
}

// Checks if pos >= lower_bound && pos <= upper_bound
pub fn check_pos_in_range(pos: [i32; 3], lower_bound: [i32; 3], upper_bound: [i32; 3]) -> bool {
    for dim in 0..3 {
        if pos[dim] < lower_bound[dim] || pos[dim] > upper_bound[dim] {
            return false;
        }
    }
    return true;
}

pub fn format_range<T: Display>(range: &Range<T>) -> String {
    return format!("[{}, {})", range.start, range.end);
}
pub fn parse_block_entity_nocheck(
    mut nbt: HashMap<String, Value>,
    tag_path: &str,
    allow_negative: bool,
) -> Result<([i32; 3], BlockEntity), Error> {
    let mut be = BlockEntity::new();

    let pos: [i32; 3];
    let pos_res = parse_size_compound(&nbt, tag_path, allow_negative);
    match pos_res {
        Ok(pos_) => pos = pos_,
        Err(e) => return Err(e),
    }

    nbt.remove("x");
    nbt.remove("y");
    nbt.remove("z");
    be.tags = nbt;

    return Ok((pos, be));
}
