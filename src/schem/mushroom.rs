/*
mc_schem is a rust library to generate, load, manipulate and save minecraft
schematic files. Copyright (C) 2026 ToKiNoBug

This program is free software: you can redistribute it and/or modify it under
the terms of the GNU General Public License as published by the Free Software
Foundation, either version 3 of the License, or (at your option) any later
version.

This program is distributed in the hope that it will be useful, but WITHOUT ANY
WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A
PARTICULAR PURPOSE.  See the GNU General Public License for more details.

You should have received a copy of the GNU General Public License
along with this program.  If not, see <https://www.gnu.org/licenses/>.
*/
use crate::region::{Array3DVariant, Region};
use crate::Block;
use std::collections::{BTreeMap, HashMap};
use std::str::FromStr;
use strum::IntoEnumIterator;
use strum::{EnumIter, EnumString};

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
enum MushroomType {
    red,
    brown,
    stem,
}

impl MushroomType {
    pub fn from_str(id: &str) -> Option<MushroomType> {
        let valid_ids = [
            (MUSHROOM_ID_BROWN, MushroomType::brown),
            (MUSHROOM_ID_RED, MushroomType::red),
            (MUSHROOM_ID_STEM, MushroomType::stem),
        ];

        for (type_str, type_enum) in valid_ids {
            if id == type_str {
                return Some(type_enum);
            }
        }
        None
    }
}

#[derive(
    Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, EnumString, EnumIter, strum::Display,
)]
enum Direction {
    up = 0,
    down = 1,
    north = 2,
    south = 3,
    east = 4,
    west = 5,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
struct MushroomState([bool; 6]); // is_outside. True for outside, false for stoma

impl MushroomState {
    /// Minecraft tells that by default all sides are outside, so true
    pub fn new() -> MushroomState {
        MushroomState([true; 6])
    }

    pub fn is_outside(&self, dir: Direction) -> bool {
        let idx = dir as usize;
        debug_assert!(idx < 6);
        self.0[idx]
    }
    pub fn set_outside(&mut self, dir: Direction, value: bool) {
        let idx = dir as usize;
        debug_assert!(idx < 6);
        self.0[idx] = value;
    }

    pub fn as_u8(&self) -> u8 {
        let mut ret: u8 = 0;
        for val in self.0 {
            ret |= val as u8;
            ret = ret << 1;
        }
        debug_assert!(ret < 64);
        ret
    }

    pub fn as_attributes(&self) -> BTreeMap<String, String> {
        let mut ret = BTreeMap::new();

        for direction in Direction::iter() {
            let value = match self.is_outside(direction) {
                true => "true",
                false => "false",
            };
            ret.insert(direction.to_string(), value.to_string());
        }
        ret
    }

    pub fn from_attributes(attribute: &BTreeMap<String, String>) -> Option<MushroomState> {
        let mut ret = MushroomState::new();
        for (key, value) in attribute.iter() {
            let dir: Direction;
            match Direction::from_str(key) {
                Ok(d) => {
                    dir = d;
                }
                Err(_) => {
                    return None;
                }
            };
            let is_outside;
            if value == "true" {
                is_outside = true;
            } else if value == "false" {
                is_outside = false;
            } else {
                return None;
            }
            ret.set_outside(dir, is_outside);
        }

        Some(ret)
    }
}

const MUSHROOM_ID_BROWN: &str = "brown_mushroom_block";
const MUSHROOM_ID_RED: &str = "red_mushroom_block";
const MUSHROOM_ID_STEM: &str = "mushroom_stem";
fn make_block_from_mushroom_info(kind: MushroomType, state: MushroomState) -> Block {
    let mut blk = Block::air();
    blk.id = match kind {
        MushroomType::brown => MUSHROOM_ID_BROWN,
        MushroomType::red => MUSHROOM_ID_RED,
        MushroomType::stem => MUSHROOM_ID_STEM,
    }
        .to_string();
    blk.attributes = state.as_attributes();

    blk
}

fn parse_mushroom_info_from_block(blk: &Block) -> Option<(MushroomType, MushroomState)> {
    // Non-vanilla non-trivial namespace
    if blk.namespace != "minecraft" && !blk.namespace.is_empty() {
        return None;
    }
    let mush_type = MushroomType::from_str(&blk.id)?;
    let mush_state = MushroomState::from_attributes(&blk.attributes)?;

    Some((mush_type, mush_state))
}

#[derive(Debug)]
struct MushroomMap<'palette> {
    block_to_element_index: BTreeMap<(MushroomType, MushroomState), u16>,
    element_index_to_block: BTreeMap<u16, (MushroomType, MushroomState)>,
    palette: &'palette mut Vec<Block>,
}
impl<'palette> MushroomMap<'palette> {
    pub fn new(palette: &'palette mut Vec<Block>) -> MushroomMap<'palette> {
        let mut ret = MushroomMap {
            block_to_element_index: BTreeMap::new(),
            element_index_to_block: BTreeMap::new(),
            palette,
        };
        // Build region index with given region palette
        for (idx, blk) in ret.palette.iter().enumerate() {
            if let Some(mush) = parse_mushroom_info_from_block(blk) {
                let idx = idx as u16;
                ret.element_index_to_block.insert(idx, mush);
                let previous_value = ret.block_to_element_index.insert(mush, idx);
                // Region palette shouldn't duplicate. If duplicates, crash in debug and just insert in release
                debug_assert!(previous_value.is_none());
            }
        }
        ret
    }

    pub fn is_mushroom(&self, element_idx: u16) -> bool {
        self.element_index_to_block.contains_key(&element_idx)
    }

    /// like operator[]. Returns index of this block. If it doesn't exist, insert.
    pub fn get_or_emplace(&mut self, kind: MushroomType, state: MushroomState) -> u16 {
        if let Some(idx) = self.block_to_element_index.get(&(kind, state)) {
            return *idx;
        }
        let new_idx = self.palette.len();
        debug_assert!(new_idx <= (u16::MAX as usize)); // almost impossible to fail
        let new_idx = new_idx as u16;
        self.palette
            .push(make_block_from_mushroom_info(kind, state));
        self.block_to_element_index.insert((kind, state), new_idx);
        self.element_index_to_block.insert(new_idx, (kind, state));
        new_idx
    }

    pub fn at(&self, element_idx: u16) -> Option<&(MushroomType, MushroomState)> {
        self.element_index_to_block.get(&element_idx)
    }
}

impl Region {
    pub fn process_mushroom_state(&mut self) {
        let mut new_palette = self.palette.clone();
        // Make a mapping between mushroom state and element index
        let mut mushroom_map = MushroomMap::new(&mut new_palette);

        let shape_yzx = self.shape_yzx();
        let shape_yzx = [
            shape_yzx[0] as usize,
            shape_yzx[1] as usize,
            shape_yzx[2] as usize,
        ];

        if let Array3DVariant::Dense(dense_arr) = &mut self.array_yzx {
            for y in 0..shape_yzx[0] {
                for z in 0..shape_yzx[1] {
                    for x in 0..shape_yzx[2] {
                        // current mushroom block type
                        let kind;
                        // current mushroom block state
                        let mut state;
                        if let Some(b) = mushroom_map.at(dense_arr[[y, z, x]]) {
                            kind = b.0;
                            state = b.1;
                        } else {
                            // current block is not mushroom
                            continue;
                        }

                        // Has mushroom block on west, set west side to stoma (false)
                        if (x > 0) && mushroom_map.is_mushroom(dense_arr[[y, z, x - 1]]) {
                            state.set_outside(Direction::west, false);
                        }
                        if (x + 1 < shape_yzx[2])
                            && mushroom_map.is_mushroom(dense_arr[[y, z, x + 1]])
                        {
                            state.set_outside(Direction::east, false);
                        }
                        // Has mushroom block on west, set west side to stoma (false)
                        if (y > 0) && mushroom_map.is_mushroom(dense_arr[[y - 1, z, x]]) {
                            state.set_outside(Direction::down, false);
                        }
                        if (y + 1 < shape_yzx[0])
                            && mushroom_map.is_mushroom(dense_arr[[y + 1, z, x]])
                        {
                            state.set_outside(Direction::up, false);
                        }
                        // Has mushroom block on west, set west side to stoma (false)
                        if (z > 0) && mushroom_map.is_mushroom(dense_arr[[y, z - 1, x]]) {
                            state.set_outside(Direction::north, false);
                        }
                        if (z + 1 < shape_yzx[1])
                            && mushroom_map.is_mushroom(dense_arr[[y, z + 1, x]])
                        {
                            state.set_outside(Direction::south, false);
                        }

                        let new_ele_idx = mushroom_map.get_or_emplace(kind, state);
                        dense_arr[[y, z, x]] = new_ele_idx;
                    }
                }
            }
        }

        if let Array3DVariant::Sparse(sparse_arr) = &mut self.array_yzx {
            // Records original mushroom info.
            let mut mushroom_table: HashMap<[usize; 3], (MushroomType, MushroomState)> =
                HashMap::new();
            mushroom_table.reserve(sparse_arr.num_non_zero() / 2);

            // First loop: visit and collect all mushroom info.
            let mut mushroom_collector = |idx1: usize, pos: &[usize; 3], ele_idx: u16| {
                if let Some(mush_info) = mushroom_map.at(ele_idx) {
                    mushroom_table.insert(*pos, *mush_info);
                }
            };
            sparse_arr.visit_non_zero(&mut mushroom_collector);

            //Second loop: visit all mushroom blocks, compute correct state, write into region. mushroom_table is not updated because borrowing rule.
            for (pos, (kind, original_mush_state)) in mushroom_table.iter() {
                let [y, z, x] = *pos;
                let mut state = *original_mush_state;

                // Has mushroom block on west, set west side to stoma (false)
                if (x > 0) && mushroom_table.contains_key(&[y, z, x - 1]) {
                    state.set_outside(Direction::west, false);
                }
                if (x + 1 < shape_yzx[2]) && mushroom_table.contains_key(&[y, z, x + 1]) {
                    state.set_outside(Direction::east, false);
                }
                // Has mushroom block on west, set west side to stoma (false)
                if (y > 0) && mushroom_table.contains_key(&[y - 1, z, x]) {
                    state.set_outside(Direction::down, false);
                }
                if (y + 1 < shape_yzx[0]) && mushroom_table.contains_key(&[y + 1, z, x]) {
                    state.set_outside(Direction::up, false);
                }
                // Has mushroom block on west, set west side to stoma (false)
                if (z > 0) && mushroom_table.contains_key(&[y, z - 1, x]) {
                    state.set_outside(Direction::north, false);
                }
                if (z + 1 < shape_yzx[1]) && mushroom_table.contains_key(&[y, z + 1, x]) {
                    state.set_outside(Direction::south, false);
                }
                // Write correct mushroom state into palette
                let new_idx = mushroom_map.get_or_emplace(*kind, state);
                // Write correct index into sparse array
                sparse_arr.set_3d(pos, new_idx);
            }
        }

        drop(mushroom_map); // mushroom map is finished; new palette is also finished. Update palette
        self.palette = new_palette;
    }
}
