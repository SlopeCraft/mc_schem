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
    Red,
    Brown,
    Stem,
}

impl MushroomType {
    pub fn from_str(id: &str) -> Option<MushroomType> {
        let valid_ids = [
            (MUSHROOM_ID_BROWN, MushroomType::Brown),
            (MUSHROOM_ID_RED, MushroomType::Red),
            (MUSHROOM_ID_STEM, MushroomType::Stem),
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
    #[strum(serialize = "up")]
    Up = 0,
    #[strum(serialize = "down")]
    Down = 1,
    #[strum(serialize = "north")]
    North = 2,
    #[strum(serialize = "south")]
    South = 3,
    #[strum(serialize = "east")]
    East = 4,
    #[strum(serialize = "west")]
    West = 5,
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
    #[allow(dead_code)]
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
        MushroomType::Brown => MUSHROOM_ID_BROWN,
        MushroomType::Red => MUSHROOM_ID_RED,
        MushroomType::Stem => MUSHROOM_ID_STEM,
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

#[derive(Debug, Copy, Clone)]
pub struct MushroomStatistic {
    pub mushroom_total_num: u64,
    pub mushroom_corrected_num: u64,
    /// More blocks appended to palette
    pub palette_growth: u16,
}

impl Default for MushroomStatistic {
    fn default() -> Self {
        MushroomStatistic {
            mushroom_total_num: 0,
            mushroom_corrected_num: 0,
            palette_growth: 0,
        }
    }
}

impl Region {
    /// Update block-state of mushroom blocks (full block, not small mushroom). In real game,
    /// contacting mushroom blocks have stoma texture on contacting face. This should be done by
    /// updating block state. This function updates block index and palette for every mushroom block
    /// (if need change), just like Minecraft does.
    pub fn update_mushroom_state(&mut self) -> MushroomStatistic {
        let mut new_palette = self.palette.clone();
        // Make a mapping between mushroom state and element index
        let mut mushroom_map = MushroomMap::new(&mut new_palette);

        let shape_yzx = self.shape_yzx();
        let shape_yzx = [
            shape_yzx[0] as usize,
            shape_yzx[1] as usize,
            shape_yzx[2] as usize,
        ];

        let mut stat = MushroomStatistic::default();

        if let Array3DVariant::Dense(dense_arr) = &mut self.array_yzx {
            for y in 0..shape_yzx[0] {
                for z in 0..shape_yzx[1] {
                    for x in 0..shape_yzx[2] {
                        // current mushroom block type
                        let kind;
                        // current mushroom block state
                        let mut state;
                        let old_ele_idx = dense_arr[[y, z, x]];
                        if let Some(b) = mushroom_map.at(old_ele_idx) {
                            kind = b.0;
                            state = b.1;
                            stat.mushroom_total_num += 1;
                            println!("Found mushroom block {}", self.palette[old_ele_idx as usize].full_id());
                        } else {
                            // current block is not mushroom
                            continue;
                        }

                        // Has mushroom block on West, set West side to stoma (false)
                        if (x > 0) && mushroom_map.is_mushroom(dense_arr[[y, z, x - 1]]) {
                            state.set_outside(Direction::West, false);
                        }
                        if (x + 1 < shape_yzx[2])
                            && mushroom_map.is_mushroom(dense_arr[[y, z, x + 1]])
                        {
                            state.set_outside(Direction::East, false);
                        }
                        // Has mushroom block on West, set West side to stoma (false)
                        if (y > 0) && mushroom_map.is_mushroom(dense_arr[[y - 1, z, x]]) {
                            state.set_outside(Direction::Down, false);
                        }
                        if (y + 1 < shape_yzx[0])
                            && mushroom_map.is_mushroom(dense_arr[[y + 1, z, x]])
                        {
                            state.set_outside(Direction::Up, false);
                        }
                        // Has mushroom block on West, set West side to stoma (false)
                        if (z > 0) && mushroom_map.is_mushroom(dense_arr[[y, z - 1, x]]) {
                            state.set_outside(Direction::North, false);
                        }
                        if (z + 1 < shape_yzx[1])
                            && mushroom_map.is_mushroom(dense_arr[[y, z + 1, x]])
                        {
                            state.set_outside(Direction::South, false);
                        }

                        let new_ele_idx = mushroom_map.get_or_emplace(kind, state);
                        if new_ele_idx != old_ele_idx {
                            stat.mushroom_corrected_num += 1;
                        }
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
            let mut mushroom_collector = |_idx1: usize, pos: &[usize; 3], ele_idx: u16| {
                if let Some(mush_info) = mushroom_map.at(ele_idx) {
                    mushroom_table.insert(*pos, *mush_info);
                }
            };
            sparse_arr.visit_non_zero(&mut mushroom_collector);
            stat.mushroom_total_num = mushroom_table.len() as u64;

            //Second loop: visit all mushroom blocks, compute correct state, write into region. mushroom_table is not updated because borrowing rule.
            for (pos, (kind, original_mush_state)) in mushroom_table.iter() {
                let [y, z, x] = *pos;
                let mut state = *original_mush_state;

                // Has mushroom block on West, set West side to stoma (false)
                if (x > 0) && mushroom_table.contains_key(&[y, z, x - 1]) {
                    state.set_outside(Direction::West, false);
                }
                if (x + 1 < shape_yzx[2]) && mushroom_table.contains_key(&[y, z, x + 1]) {
                    state.set_outside(Direction::East, false);
                }
                // Has mushroom block on West, set West side to stoma (false)
                if (y > 0) && mushroom_table.contains_key(&[y - 1, z, x]) {
                    state.set_outside(Direction::Down, false);
                }
                if (y + 1 < shape_yzx[0]) && mushroom_table.contains_key(&[y + 1, z, x]) {
                    state.set_outside(Direction::Up, false);
                }
                // Has mushroom block on West, set West side to stoma (false)
                if (z > 0) && mushroom_table.contains_key(&[y, z - 1, x]) {
                    state.set_outside(Direction::North, false);
                }
                if (z + 1 < shape_yzx[1]) && mushroom_table.contains_key(&[y, z + 1, x]) {
                    state.set_outside(Direction::South, false);
                }
                if state != *original_mush_state {
                    stat.mushroom_corrected_num += 1;
                }
                // Write correct mushroom state into palette
                let new_idx = mushroom_map.get_or_emplace(*kind, state);
                // Write correct index into sparse array
                sparse_arr.set_3d(pos, new_idx);
            }
        }
        drop(mushroom_map); // mushroom map is finished; new palette is also finished. Update palette
        debug_assert!(new_palette.len() >= self.palette.len());
        stat.palette_growth = (new_palette.len() - self.palette.len()) as u16;
        self.palette = new_palette;

        debug_assert!(stat.mushroom_corrected_num <= stat.mushroom_total_num);

        stat
    }
}
