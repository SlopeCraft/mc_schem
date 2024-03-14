use std::collections::HashMap;
use std::str::FromStr;
use fastnbt::Value;
use strum::{Display, EnumString};
use crate::error::{Error, unwrap_opt_f32, unwrap_opt_i8, unwrap_opt_i32, unwrap_opt_i64};
use crate::{unwrap_opt_tag, unwrap_tag};
use crate::schem::id_of_nbt_tag;

#[allow(non_camel_case_types)]
#[derive(Debug, Copy, Clone, Display, EnumString)]
#[repr(u8)]
pub enum RaidStatus {
    ongoing = 0,
    stopped = 1,
    victory = 2,
    loss = 3,
}

#[derive(Debug, Clone)]
pub struct Raid {
    pub status: RaidStatus,
    pub started: bool,
    pub ticks_active: i64,
    /// UUIDs
    pub heroes_of_the_village: Vec<[i32; 4]>,
    pub groups_spawned: i32,
    pub post_raid_ticks: i32,
    pub total_health: f32,
    pub active: bool,
    pub bad_omen_level: i32,
    pub center: [i32; 3],
    pub id: i32,
    pub pre_raid_ticks: i32,
}

#[derive(Debug, Clone)]
pub struct RaidList {
    pub raids: Vec<Raid>,
    pub next_available_id: i32,
    pub tick: i32,
}

impl Raid {
    pub fn from_nbt(nbt: &HashMap<String, Value>, tag_path: &str) -> Result<Raid, Error> {
        let status;
        {
            let status_path = format!("{tag_path}/Status");
            let status_str = unwrap_opt_tag!(nbt.get("Status"),String,"".to_string(),status_path);
            if let Ok(s) = RaidStatus::from_str(&status_str) {
                status = s;
            } else {
                return Err(Error::InvalidValue {
                    tag_path: status_path,
                    error: format!("\"{status_str}\" is not a valid raid status."),
                });
            }
        }

        let started = unwrap_opt_i8(nbt, "Started", tag_path)?;
        let started = started != 0;

        let ticks_active = unwrap_opt_i64(nbt, "TicksActive", tag_path)?;

        let heroes_of_the_village;
        {
            let tag_path = format!("{tag_path}/HeroesOfTheVillage");
            let list = unwrap_opt_tag!(nbt.get("HeroesOfTheVillage"),List,vec![],tag_path);
            let mut hov = Vec::with_capacity(list.len());
            for (idx, tag) in list.iter().enumerate() {
                let tag_path = format!("{tag_path}/[{idx}]");
                let tag = unwrap_tag!(tag,IntArray,fastnbt::IntArray::new(vec![]),tag_path);
                if tag.len() != 4 {
                    return Err(Error::InvalidValue {
                        tag_path,
                        error: format!("UUID should contains 4 ints, but found {}", tag.len()),
                    });
                }
                let mut uuid = [0; 4];
                for idx in 0..4 {
                    uuid[idx] = tag[idx];
                }
                hov.push(uuid);
            }
            heroes_of_the_village = hov;
        }

        let groups_spawned = unwrap_opt_i32(nbt, "GroupsSpawned", tag_path)?;
        let post_raid_ticks = unwrap_opt_i32(nbt, "PostRaidTicks", tag_path)?;
        let total_health = unwrap_opt_f32(nbt, "TotalHealth", tag_path)?;
        let active = unwrap_opt_i8(nbt, "Active", tag_path)? != 0;
        let bad_omen_level = unwrap_opt_i32(nbt, "BadOmenLevel", tag_path)?;
        let center: [i32; 3] = [
            unwrap_opt_i32(nbt, "CX", tag_path)?,
            unwrap_opt_i32(nbt, "CY", tag_path)?,
            unwrap_opt_i32(nbt, "CZ", tag_path)?,
        ];
        //let num_groups = unwrap_opt_i32(nbt, "NumGroups", tag_path)?;
        let id = unwrap_opt_i32(nbt, "Id", tag_path)?;
        let pre_raid_ticks = unwrap_opt_i32(nbt, "PreRaidTicks", tag_path)?;


        let raid = Raid {
            status,
            started,
            ticks_active,
            heroes_of_the_village,
            groups_spawned,
            post_raid_ticks,
            total_health,
            active,
            bad_omen_level,
            center,
            id,
            pre_raid_ticks,
        };
        return Ok(raid);
    }
}

impl Default for RaidList {
    fn default() -> Self {
        return Self {
            raids: vec![],
            next_available_id: 0,
            tick: 0,
        };
    }
}