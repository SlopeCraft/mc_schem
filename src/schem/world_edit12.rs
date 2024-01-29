use std::collections::HashMap;
use fastnbt::Value;
use ndarray::Array3;
use crate::error::LoadError;
use crate::schem::{id_of_nbt_tag, Schematic};
use crate::unwrap_opt_tag;


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
}