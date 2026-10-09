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

use crate::error::{unwrap_opt_i8, unwrap_opt_string};
use crate::schem::id_of_nbt_tag;
use crate::{unwrap_tag, Error};
use fastnbt::Value;
use serde::Deserialize;
use std::collections::{BTreeMap, HashMap};
//use crate::error::NBTWithPath;

#[derive(Debug, Clone, Default, Deserialize)]
pub struct Item {
    #[serde(rename = "Count")]
    pub count: i8,
    pub id: String,
    #[serde(rename = "tag")]
    pub tags: HashMap<String, Value>,
}

impl Item {
    pub fn from_nbt(nbt: &HashMap<String, Value>, tag_path: &str) -> Result<Item, Error> {
        let count = unwrap_opt_i8(&nbt, "Count", tag_path)?;
        let id = unwrap_opt_string(&nbt, "id", tag_path)?.clone();
        let tags = if let Some(t) = nbt.get("tag") {
            unwrap_tag!(t, Compound, HashMap::new(), format!("{tag_path}/tag")).clone()
        } else {
            HashMap::new()
        };

        Ok(Item { count, id, tags })
    }
}

#[derive(Debug, Clone, Default)]
pub struct Inventory(pub BTreeMap<i8, Item>);

#[allow(dead_code)]
impl Inventory {
    pub fn from_nbt(nbt: &[Value], tag_path: &str) -> Result<Inventory, Error> {
        let mut result = BTreeMap::new();
        let mut parsed: HashMap<i8, String> = HashMap::with_capacity(nbt.len());
        for (idx, nbt) in nbt.iter().enumerate() {
            let tag_path = format!("{tag_path}/[{idx}]");
            let nbt = unwrap_tag!(nbt, Compound, HashMap::new(), tag_path);
            let item = Item::from_nbt(nbt, &tag_path)?;
            let slot = unwrap_opt_i8(nbt, "Slot", &tag_path)?;
            if result.contains_key(&slot) {
                return Err(Error::MultipleItemsInOneSlot {
                    slot,
                    former: (result.remove(&slot).unwrap(), parsed.remove(&slot).unwrap()),
                    latter: (item, tag_path),
                });
            }
            result.insert(slot, item);
            parsed.insert(slot, tag_path);
        }
        Ok(Self(result))
    }
}
