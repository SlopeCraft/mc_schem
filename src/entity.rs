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

//! A thin wrapper around NBT of minecraft entity.
//!
//! Entity is a huge content with too many parent and child classes. This crate don't try to
//! parse every kind of entity, but just a subset of compulsory tags. `id` and `position` are
//! compulsory for this crate, and all other unknown tags are stored in `tags`.

use crate::region::Entity;
use crate::schem::common::parse_position_list;
use crate::schem::{common, id_of_nbt_tag};
use crate::{unwrap_opt_tag, Error};
use fastnbt::Value;
use std::collections::HashMap;

impl Default for Entity {
    fn default() -> Self {
        Entity {
            tags: HashMap::new(),
            position: [0.0, 0.0, 0.0],
            block_pos: [0, 0, 0],
            id: "".to_string(),
        }
    }
}

impl Entity {
    pub fn new() -> Entity {
        Entity::default()
    }

    /// Add adder to position and block_pos
    pub fn pos_shift(&mut self, adder: [i32; 3]) {
        for dim in 0..3 {
            self.block_pos[dim] += adder[dim];
            self.position[dim] += adder[dim] as f64;
        }
    }
    pub fn from_nbt(nbt: HashMap<String, Value>, tag_path: &str) -> Result<Entity, Error> {
        let mut nbt = nbt;
        let mut entity = Entity::new();
        // parse Pos and blockPos
        {
            let tag_pos_path = format!("{}/Pos", tag_path);
            let pos_nbt_list = unwrap_opt_tag!(nbt.remove("Pos"), List, vec![], tag_pos_path);
            let pos_d = parse_position_list(&pos_nbt_list, &tag_pos_path)?;

            entity.position = pos_d;
            entity.block_pos = [pos_d[0] as i32, pos_d[1] as i32, pos_d[2] as i32];
        }
        // parse id

        entity.id = unwrap_opt_tag!(
            nbt.remove("id"),
            String,
            "".to_string(),
            format!("{}/id", tag_path)
        );

        entity.tags = nbt;

        Ok(entity)
    }

    pub fn to_nbt(&self) -> HashMap<String, Value> {
        let mut ret = self.tags.clone();
        // Pos
        ret.insert(
            "Pos".to_string(),
            Value::List(common::size_to_list(&self.position)),
        );
        // id
        ret.insert("id".to_string(), Value::String(self.id.clone()));
        ret
    }
}
