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

use strum::{Display, EnumString};
use std::collections::{BTreeMap, HashMap};
use std::fmt::{Display, Formatter};
use std::hash::{Hash, Hasher};
use fastnbt::Value;

#[derive(Debug, Clone, Eq)]
pub struct Block {
    pub namespace: String,
    pub id: String,
    pub attributes: BTreeMap<String, String>,

}

#[repr(u8)]
#[derive(Debug,EnumString,Display,PartialEq,Copy,Clone)]
pub enum BlockIdParseError {
    TooManyColons = 0,
    TooManyLeftBrackets = 1,
    TooManyRightBrackets = 2,
    MissingBlockId = 3,
    BracketsNotInPairs = 4,
    BracketInWrongPosition = 5,
    ColonsInWrongPosition = 6,
    MissingEqualInAttributes = 7,
    TooManyEqualsInAttributes = 8,
    MissingAttributeName = 9,
    MissingAttributeValue = 10,
    ExtraStringAfterRightBracket = 11,
    InvalidCharacter = 12,
}

fn check_blockid_characters(blkid:&str) ->Result<(),BlockIdParseError> {
    for ch in blkid.chars() {
        if ch>='a' && ch <='z' {
            continue;
        }
        if ch >= '0' && ch <= '9' {
            continue;
        }

        let other_valid_chars=[',','=','[',']',':','_'];
        if other_valid_chars.contains(&ch) {
            continue;
        }
        //panic!("Invalid char {}", ch);
        return Err(BlockIdParseError::InvalidCharacter);
    }
    return Ok(());
}
fn check_for_bracket(full_id: &str) -> Result<Option<(usize, usize)>, BlockIdParseError> {
    if full_id.find('[') != full_id.rfind('[') {
        return Err(BlockIdParseError::TooManyLeftBrackets);
    }
    if full_id.find(']') != full_id.rfind(']') {
        return Err(BlockIdParseError::TooManyRightBrackets);
    }

    let left_loc = full_id.find('[');
    let right_loc = full_id.find(']');

    if left_loc.is_some() != right_loc.is_some() {
        return Err(BlockIdParseError::BracketsNotInPairs);
    }

    return if left_loc.is_some() {
        let left_loc = left_loc.unwrap();
        let right_loc = right_loc.unwrap();

        if left_loc >= right_loc {
            return Err(BlockIdParseError::BracketInWrongPosition);
        }

        Ok(Some((left_loc, right_loc)))
    } else {
        Ok(None)
    };
}

fn check_attributes_segment(att_seg: &str) -> Result<(), BlockIdParseError> {
    if att_seg.is_empty() {
        return Ok(());
    }

    for seg in att_seg.split(',') {
        let eq_loc = seg.find('=');
        match eq_loc {
            None => return Err(BlockIdParseError::MissingEqualInAttributes),
            Some(eq_loc) => {
                if eq_loc != seg.rfind('=').unwrap() {
                    return Err(BlockIdParseError::TooManyEqualsInAttributes);
                }

                if eq_loc <= 0 {
                    return Err(BlockIdParseError::MissingAttributeName);
                }
                if eq_loc + 1 >= seg.len() {
                    return Err(BlockIdParseError::MissingAttributeValue);
                }

                continue;
            }
        }
    }
    return Ok(());
}

pub fn parse_block_id(full_id: &str) -> Result<(&str, &str, &str), BlockIdParseError> {
    match check_blockid_characters(full_id) {
        Err(err) => return Err(err),
        _ => {},
    }

    let mut namespace = "";
    let colon_loc_opt = full_id.find(':');
    match colon_loc_opt {
        Some(colon_loc) => if colon_loc != full_id.rfind(':').unwrap()
        { return Err(BlockIdParseError::TooManyColons); } else {
            namespace = &full_id[0..colon_loc];
        }
        None => {}
    }

    let id;
    let id_begin_idx = match colon_loc_opt {
        Some(col_loc) => col_loc + 1,
        None => 0,
    };


    let bracket_locs_opt = check_for_bracket(full_id);
    let bracket_locs: (usize, usize);
    match bracket_locs_opt {
        Err(e) => return Err(e),
        Ok(locs_opt) => {
            match locs_opt {
                None => {
                    id = &full_id[id_begin_idx..full_id.len()];
                    if id.is_empty() {return Err(BlockIdParseError::MissingBlockId);}
                    return Ok((namespace, id, ""));
                }
                Some(locs) => {
                    if locs.0 <= id_begin_idx {
                        return Err(BlockIdParseError::ColonsInWrongPosition);
                    }
                    bracket_locs = locs;
                    id=&full_id[id_begin_idx..bracket_locs.0];
                    if id.is_empty() {return Err(BlockIdParseError::MissingBlockId);}
                }
            }
        }
    }

    if bracket_locs.1+1 <full_id.len() {
        return Err(BlockIdParseError::ExtraStringAfterRightBracket);
    }

    let attributes = &full_id[(bracket_locs.0 + 1)..bracket_locs.1];

    let check_res = check_attributes_segment(attributes);
    match check_res {
        Err(e) => return Err(e),
        _ => {}
    }

    return Ok((namespace, id, attributes));
}

pub fn parse_attributes_segment(att_seg: &str) -> Result<Vec<(&str, &str)>, BlockIdParseError> {
    let mut result: Vec<(&str, &str)> = Vec::new();

    if att_seg.is_empty() {
        return Ok(result);
    }

    for seg in att_seg.split(',') {
        let eq_loc = seg.find('=');
        match eq_loc {
            None => return Err(BlockIdParseError::MissingEqualInAttributes),
            Some(eq_loc) => {
                if eq_loc != seg.rfind('=').unwrap() {
                    return Err(BlockIdParseError::TooManyEqualsInAttributes);
                }

                if eq_loc <= 0 {
                    return Err(BlockIdParseError::MissingAttributeName);
                }
                if eq_loc + 1 >= seg.len() {
                    return Err(BlockIdParseError::MissingAttributeValue);
                }

                result.push((&seg[0..eq_loc], &seg[(eq_loc + 1)..seg.len()]));
            }
        }
    }
    return Ok(result);
}

impl PartialEq<Self> for Block {
    fn eq(&self, other: &Self) -> bool {
        if self.namespace != other.namespace {
            return false;
        }
        if self.id != other.id {
            return false;
        }

        if self.attributes.len() != other.attributes.len() {
            return false;
        }

        for att in &self.attributes {
            let find_res = other.attributes.get(att.0);
            match find_res {
                None => return false,
                Some(vaule) => {
                    if vaule == att.1 { continue; } else { return false; }
                }
            }
        }

        return true;
    }
}

#[allow(dead_code)]
impl Block {
    pub fn new() -> Block {
        return Block {
            namespace: String::from("minecraft"),
            id: String::from("air"),
            attributes: BTreeMap::new(),
        };
    }

    pub fn from_id(blkid: &str) -> Result<Block, BlockIdParseError> {
        let parse_res = parse_block_id(blkid);
        let segmented: (&str, &str, &str);
        match parse_res {
            Err(e) => return Err(e),
            Ok(segs) => segmented = segs,
        }

        let attri_res = parse_attributes_segment(segmented.2);
        let attri_list: Vec<(&str, &str)>;
        match attri_res {
            Err(e) => return Err(e),
            Ok(attri_l) => attri_list = attri_l,
        }

        let mut blk: Block = Block::new();
        blk.namespace = segmented.0.to_string();
        blk.id = segmented.1.to_string();

        for attri in attri_list {
            blk.attributes.insert(String::from(attri.0), String::from(attri.1));
        }

        return Ok(blk);
    }

    pub fn attribute_str(&self) -> String {
        let mut result: String = String::new();
        for (k, v) in &self.attributes {
            result.push_str(k.as_str());
            result.push('=');
            result.push_str(v.as_str());
            result.push(',');
        }

        result.pop();
        return result;
    }

    pub fn fmt_attributes(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for (idx, (k, v)) in self.attributes.iter().enumerate() {
            write!(f, "{}={}", k, v)?;
            if idx < self.attributes.len() - 1 {
                write!(f, ",")?;
            }
        }

        return Ok(());
    }


    pub fn full_id(&self) -> String {
        // return if self.attributes.is_empty() {
        //     format!("{}:{}", self.namespace.as_str(), self.id.as_str())
        // } else {
        //     let attrib_str = self.attribute_str();
        //     format!("{}:{}[{}]", self.namespace.as_str(), self.id.as_str(), attrib_str)
        // };
        return self.to_string();
    }

    pub fn is_structure_void(&self) -> bool {
        if self.namespace != "minecraft" {
            return false;
        }
        if self.id != "structure_void" {
            return false;
        }
        if !self.attributes.is_empty() {
            return false;
        }

        return true;
    }
    pub fn is_air(&self) -> bool {
        if self.namespace != "minecraft" {
            return false;
        }
        if self.id != "air" {
            return false;
        }
        if !self.attributes.is_empty() {
            return false;
        }
        return true;
    }

    pub fn air() -> Block {
        return Block {
            namespace: String::from("minecraft"),
            id: String::from("air"),
            attributes: BTreeMap::new(),
        }
    }

    pub fn empty_block() -> Block {
        return Block {
            namespace: "".to_string(),
            id: "".to_string(),
            attributes: BTreeMap::new(),
        }
    }

    pub fn structure_void() -> Block {
        return Block {
            namespace: String::from("minecraft"),
            id: String::from("structure_void"),
            attributes: BTreeMap::new(),
        }
    }

    pub fn to_nbt(&self) -> HashMap<String, Value> {
        let mut nbt: HashMap<String, Value> = HashMap::new();
        nbt.insert(String::from("Name"),
                   Value::String(format!("{}:{}", self.namespace, self.id)));
        if !self.attributes.is_empty() {
            let mut props: HashMap<String, Value> = HashMap::new();
            for (key, val) in &self.attributes {
                props.insert(key.clone(), Value::String(val.clone()));
            }
            nbt.insert(String::from("Properties"), Value::Compound(props));
        }

        return nbt;
    }


    pub fn set_property<V: ?Sized>(&mut self, key: &str, value: &V)
        where for<'a> &'a V: Display {
        self.attributes.insert(key.to_string(), value.to_string());
    }

    pub fn is_inherited_from(&self, blk_less_attr: &Block) -> bool {
        if self.attributes.len() < blk_less_attr.attributes.len() {
            return false;
        }
        if self.id != blk_less_attr.id {
            return false;
        }

        for (key, val) in &blk_less_attr.attributes {
            if let Some(val_self) = self.attributes.get(key) {
                if val_self != val {
                    return false;
                }
                continue;
            } else {
                return false;
            }
        }
        return true;
    }
}

impl Hash for Block {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.namespace.hash(state);
        self.id.hash(state);
        for (key, val) in &self.attributes {
            key.hash(state);
            val.hash(state);
        }
    }
}

impl Display for Block {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if !self.namespace.is_empty() {
            write!(f, "{}:", self.namespace)?;
        }

        write!(f, "{}", self.id)?;

        if !self.attributes.is_empty() {
            write!(f, "[")?;
            self.fmt_attributes(f)?;
            write!(f, "]")?;
        }

        return Ok(());
        //return write!(f, "{}", &self.full_id());
    }
}

// impl<T> Borrow<T> for Block
//     where T: ?Sized {
//     fn borrow(&self) -> &T
//     {
//         return self;
//     }
// }

#[repr(u16)]
#[derive(Debug, Display, Clone, Copy)]
#[allow(dead_code)]
pub enum CommonBlock {
    Air = 0,
    StructureVoid = 1,
}


impl CommonBlock {
    pub fn to_block(&self) -> Block {
        return match self {
            CommonBlock::Air => Block::air(),
            CommonBlock::StructureVoid => Block::structure_void(),
        }
    }
}