use std::collections::{BTreeMap, HashMap};
use fastnbt::Value;
use serde::Deserialize;
use crate::{Error, unwrap_tag};
use crate::error::{unwrap_opt_i8, unwrap_opt_string};
use crate::schem::id_of_nbt_tag;
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
            unwrap_tag!(t,Compound,HashMap::new(),format!("{tag_path}/tag")).clone()
        } else {
            HashMap::new()
        };

        return Ok(Item {
            count,
            id,
            tags,
        });

        // let nbt = Value::Compound(nbt);
        // let deserializer = NBTWithPath {
        //     nbt: &nbt,
        //     path: tag_path.to_string(),
        // };
        //
        // return Item::deserialize(deserializer);
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
            let nbt = unwrap_tag!(nbt,Compound,HashMap::new(),tag_path);
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
        return Ok(Self(result));
    }
}