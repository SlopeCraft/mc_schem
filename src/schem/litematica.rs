use std::cmp::max;
use std::collections::HashMap;
use std::convert::From;
use std::ops::Index;
use fastnbt::{LongArray, Value};
use math::round::{ceil, floor};
use crate::schem::{LitematicaMetaData, Schematic, id_of_nbt_tag, RawMetaData, MetaDataIR, Region};
use crate::error::LoadError;
use crate::{schem, unwrap_opt_tag, unwrap_tag};
use crate::block::Block;

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

pub fn block_required_bits(palette_size: usize) -> usize {
    let palette_size = max(palette_size, 1);
    let mut bits = 0;
    while (1 << bits) < palette_size {
        bits += 1;
    }
    return bits;
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


    let array =
        unwrap_opt_tag!(nbt.get("BlockStates"),LongArray,LongArray::new(vec![]),format!("{}/BlockStates",tag_path));

    return Ok(region);
}

#[derive(Debug)]
pub struct MultiBitSet {
    arr: Vec<u64>,
    length: usize,
    element_bits: u8,

}

pub fn ceil_up_to(a: isize, b: isize) -> isize
{
    assert!(b > 0);
    if ((a % b) == 0) {
        return a;
    }
    return ((a / b) + 1) * b;
}

impl MultiBitSet {
    pub fn new() -> MultiBitSet {
        return MultiBitSet {
            arr: Vec::new(),
            length: 0,
            element_bits: 1,
        }
    }

    pub fn from_data(data: &[u64], length: usize, ele_bits: u8) -> Option<MultiBitSet> {
        if ele_bits <= 0 || ele_bits > 64 {
            return None;
        }

        if (length * ele_bits as usize) > (data.len() * 64) {
            return None;
        }

        let result = MultiBitSet {
            arr: Vec::from(data),
            length,
            element_bits: ele_bits,
        };
        return Some(result);
    }
    pub fn element_bits(&self) -> u8 {
        return self.element_bits;
    }
    pub fn len(&self) -> usize {
        return self.length;
    }
    pub fn total_bits(&self) -> usize {
        return self.length * (self.element_bits as usize);
    }
    fn required_u64_num(&self) -> usize {
        let total_bits = self.total_bits();
        if total_bits % 64 == 0 {
            return total_bits / 64;
        }
        return total_bits / 64 + 1;
    }
    pub fn reset(&mut self, element_bits: u8, len: usize) {
        assert!(element_bits > 0);
        assert!(element_bits <= 64);
        self.length = len;
        self.element_bits = element_bits;
        self.arr.resize(self.required_u64_num(), 0);
    }


    fn global_bit_index_to_u64_index(&self, gbit_index: usize) -> usize {
        return gbit_index / 64;
    }
    fn global_bit_index_to_local_bit_index(&self, gbit_index: usize) -> usize {
        return gbit_index % 64;
    }

    pub fn mask_by_bits(bits: u8) -> u64 {
        if bits <= 63 {
            return (1 << (bits)) - 1;
        }
        return 0xFFFFFFFFFFFFFFFF;
    }
    pub fn mask_on_top_by_bits(bits: u8) -> u64 {
        assert!(bits <= 64);
        let shift_bits = 64 - bits;
        return Self::mask_by_bits(bits) << shift_bits;
    }
    pub fn basic_mask(&self) -> u64 {
        return Self::mask_by_bits(self.element_bits());
    }

    pub fn logic_bit_index_to_global_bit_index(logic_bit_index: isize) -> usize {
        assert!(logic_bit_index < 64);
        if logic_bit_index >= 0 {
            return logic_bit_index as usize;
        }
        let addon = ceil_up_to(-logic_bit_index, 64) * 2;
        //println!("logic_bit_index = {}, addon = {}", logic_bit_index, addon);
        return (logic_bit_index + addon) as usize;
    }

    fn first_global_bit_index_of(&self, ele_index: usize) -> usize {
        let logic_bit_index = 63 - ((ele_index + 1) * (self.element_bits as usize) - 1) as isize;
        return Self::logic_bit_index_to_global_bit_index(logic_bit_index);
    }
    fn last_global_bit_index_of(&self, ele_index: usize) -> usize {
        let logic_bit_index = 63 - (ele_index * (self.element_bits() as usize)) as isize;
        return Self::logic_bit_index_to_global_bit_index(logic_bit_index);
    }


    fn is_element_on_single_block(&self, ele_index: usize) -> bool {
        let fgbi = self.first_global_bit_index_of(ele_index);
        let lgbi = self.last_global_bit_index_of(ele_index);
        assert_ne!(fgbi, lgbi);
        if fgbi > lgbi {
            return false;
        }
        return true;
    }

    pub fn get(&self, ele_index: usize) -> u64 {
        assert!(ele_index < self.length);

        let fgbi = self.first_global_bit_index_of(ele_index);//first global bit index
        let lgbi = self.last_global_bit_index_of(ele_index);//last global bit index

        return if self.is_element_on_single_block(ele_index) {
            let u64_idx = self.global_bit_index_to_u64_index(fgbi);
            assert_eq!(u64_idx, self.global_bit_index_to_u64_index(lgbi));
            let llbi = self.global_bit_index_to_local_bit_index(lgbi);//last local bit index
            assert!(llbi < 64);
            let shifts = 63 - (llbi as isize);
            assert!(shifts >= 0);
            assert!(shifts + self.element_bits as isize <= 64);
            let mask = self.basic_mask() << shifts;

            let taken_val = (self.arr[u64_idx] & mask) >> shifts;

            taken_val
        } else {
            let u64idx_f = self.global_bit_index_to_u64_index(fgbi);
            let u64idx_l = self.global_bit_index_to_u64_index(lgbi);
            assert_eq!(u64idx_f, u64idx_l + 1);

            let l_part_bits = lgbi - u64idx_l * 64 + 1;
            let f_part_bits = ((u64idx_f + 1) * 64) - fgbi;
            assert!(l_part_bits > 0);
            assert!(f_part_bits > 0);
            assert_eq!(l_part_bits + f_part_bits, self.element_bits as usize);
            let l_mask = Self::mask_on_top_by_bits(l_part_bits as u8);
            let f_mask = Self::mask_by_bits(f_part_bits as u8);

            let l_part = (self.arr[u64idx_l] & l_mask) >> (64 - l_part_bits);
            let f_part = (self.arr[u64idx_f] & f_mask) << l_part_bits;

            let result = l_part | f_part;

            result
        }
    }
}

// #[derive(Debug)]
// pub struct Bitset {
//     arr: Vec<u64>,
//     length: usize,
// }


// impl Bitset {
//     pub fn new() -> Bitset {
//         return Bitset {
//             arr: Vec::new(),
//             length: 0,
//         }
//     }
//     pub fn len(&self) -> usize {
//         return self.length;
//     }
//
//     pub fn resize(&mut self, new_size: usize) {
//         let required_elements;
//         if new_size % 64 == 0 {
//             required_elements = new_size / 64;
//         } else {
//             required_elements = new_size / 64 + 1;
//         }
//         self.length = new_size;
//         self.arr.resize(required_elements, 0);
//     }
// }
//
// impl Index<usize> for Bitset {
//     type Output = (bool);
//
//     fn index(&self, index: usize) -> Self::Output {
//         assert!(index < self.arr.len() * 64);
//         assert!(index < self.length);
//
//         let u64_idx = index / 64;
//         let bit_idx = index % 64;
//         let mask = 1u64 << bit_idx;
//         return (self.arr[u64_idx] & mask) != 0;
//     }
// }
//
// impl From<&[u64]> for Bitset {
//     fn from(value: &[u64]) -> Bitset {
//         return Bitset {
//             arr: Vec::from(value),
//             length: value.len() * 64,
//         };
//     }
// }
//
// impl From<&[i64]> for Bitset {
//     fn from(value: &[i64]) -> Self {
//         let mut result = Bitset::new();
//         result.arr.reserve(value.len());
//         for val in value {
//             result.arr.push(u64::from_le_bytes(val.to_le_bytes()));
//         }
//         result.length = result.arr.len() * 64;
//         return result;
//     }
// }