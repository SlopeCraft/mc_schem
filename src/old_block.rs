use ndarray::s;
use strum::Display;
use crate::block::Block;
use crate::schem::DataVersion;

pub const OLD_BLOCK_ID: [&'static str; 256] = ["air", "stone", "grass", "dirt", "cobblestone", "planks", "sapling", "bedrock", "flowing_water", "water", "flowing_lava", "lava", "sand", "gravel", "gold_ore", "iron_ore", "coal_ore", "log", "leaves", "sponge", "glass", "lapis_ore", "lapis_block", "dispenser", "sandstone", "noteblock", "bed", "golden_rail", "detector_rail", "sticky_piston", "web", "tallgrass", "deadbush", "piston", "piston_head", "wool", "piston_extension", "yellow_flower", "red_flower", "brown_mushroom", "red_mushroom", "gold_block", "iron_block", "double_stone_slab", "stone_slab", "brick_block", "tnt", "bookshelf", "mossy_cobblestone", "obsidian", "torch", "fire", "mob_spawner", "oak_stairs", "chest", "redstone_wire", "diamond_ore", "diamond_block", "crafting_table", "wheat", "farmland", "furnace", "lit_furnace", "standing_sign", "wooden_door", "ladder", "rail", "stone_stairs", "wall_sign", "lever", "stone_pressure_plate", "iron_door", "wooden_pressure_plate", "redstone_ore", "lit_redstone_ore", "unlit_redstone_torch", "redstone_torch", "stone_button", "snow_layer", "ice", "snow", "cactus", "clay", "reeds", "jukebox", "fence", "pumpkin", "netherrack", "soul_sand", "glowstone", "portal", "lit_pumpkin", "cake", "unpowered_repeater", "powered_repeater", "stained_glass", "trapdoor", "monster_egg", "stonebrick", "brown_mushroom_block", "red_mushroom_block", "iron_bars", "glass_pane", "melon_block", "pumpkin_stem", "melon_stem", "vine", "fence_gate", "brick_stairs", "stone_brick_stairs", "mycelium", "waterlily", "nether_bricks", "nether_brick_fence", "nether_brick_stairs", "nether_wart", "enchanting_table", "brewing_stand", "cauldron", "end_portal", "end_portal_frame", "end_stone", "dragon_egg", "redstone_lamp", "lit_redstone_lamp", "double_wooden_slab", "wooden_slab", "cocoa", "sandstone_stairs", "emerald_ore", "ender_chest", "tripwire_hook", "tripwire", "emerald_block", "spruce_stairs", "birch_stairs", "jungle_stairs", "command_block", "beacon", "cobblestone_wall", "flower_pot", "carrots", "potatoes", "wooden_button", "skull", "anvil", "trapped_chest", "light_weighted_pressure_plate", "heavy_weighted_pressure_plate", "unpowered_comparator", "powered_comparator", "daylight_detector", "redstone_block", "quartz_ore", "hopper", "quartz_block", "quartz_stairs", "activator_rail", "dropper", "stained_hardened_clay", "stained_glass_pane", "leaves2", "log2", "acacia_stairs", "dark_oak_stairs", "slime", "barrier", "iron_trapdoor", "prismarine", "sea_lantern", "hay_block", "carpet", "hardened_clay", "coal_block", "packed_ice", "double_plant", "standing_banner", "wall_banner", "daylight_detector_inverted", "red_sandstone", "red_sandstone_stairs", "double_stone_slab2", "stone_slab2", "spruce_fence_gate", "birch_fence_gate", "jungle_fence_gate", "dark_oak_fence_gate", "acacia_fence_gate", "spruce_fence", "birch_fence", "jungle_fence", "dark_oak_fence", "acacia_fence", "spruce_door", "birch_door", "jungle_door", "acacia_door", "dark_oak_door", "end_rod", "chorus_plant", "chorus_flower", "purpur_block", "purpur_pillar", "purpur_stairs", "purpur_double_slab", "purpur_slab", "end_bricks", "beetroots", "grass_path", "end_gateway", "repeating_command_block", "chain_command_block", "frosted_ice", "magma", "nether_wart_block", "red_nether_bricks", "bone_block", "structure_void", "observer", "white_shulker_box", "orange_shulker_box", "magenta_shulker_box", "light_blue_shulker_box", "yellow_shulker_box", "lime_shulker_box", "pink_shulker_box", "gray_shulker_box", "silver_shulker_box", "cyan_shulker_box", "purple_shulker_box", "blue_shulker_box", "brown_shulker_box", "green_shulker_box", "red_shulker_box", "black_shulker_box", "white_glazed_terracotta", "orange_glazed_terracotta", "magenta_glazed_terracotta", "light_blue_glazed_terracotta", "yellow_glazed_terracotta", "lime_glazed_terracotta", "pink_glazed_terracotta", "gray_glazed_terracotta", "silver_glazed_terracotta", "cyan_glazed_terracotta", "purple_glazed_terracotta", "blue_glazed_terracotta", "brown_glazed_terracotta", "green_glazed_terracotta", "red_glazed_terracotta", "black_glazed_terracotta", "concrete", "concrete_powder", "", "", "structure_block"];

#[derive(Debug, Display)]
pub enum OldBlockParseError {
    ReservedBlockId { id: u8 },
    DamageNotDefinedForThisBlock { id: u8, damage: u8 },
    DamageMoreThan15 { damage: u8 },
    NotAnOldVersion { version: DataVersion },
    UnsupportedVersion { version: DataVersion },
}

pub fn is_number_id_valid(id: u8) -> Result<(), OldBlockParseError> {
    if id == 253 || id == 254 {
        return Err(OldBlockParseError::ReservedBlockId { id });
    }
    return Ok(());
}

const VALID_DAMAGE_LUT: [u16; 256] = [0b00000000000001, 0b00000000000001, 0b00000000000001, 0b00000000000001, 0b00000000000001, 0b00000000111111, 0b1111111111111111, 0b00000000000001, 0b1111111111111111, 0b1111111111111111, 0b1111111111111111, 0b1111111111111111, 0b00000000000011, 0b00000000000001, 0b00000000000001, 0b00000000000001, 0b00000000000001, 0b00000000111111, 0b00000000111111, 0b00000000000011, 0b00000000000001, 0b00000000000001, 0b00000000000001, 0b00000000111111, 0b00000000000111, 0b00000000000001, 0b1111111111111111, 0b1111111111111111, 0b1111111111111111, 0b11111100111111, 0b00000000000001, 0b00000000000011, 0b00000000000001, 0b11111100111111, 0b11111100111111, 0b1111111111111111, 0b11111100111111, 0b00000111111111, 0b00000111111111, 0b00000000000001, 0b00000000000001, 0b00000000000001, 0b00000000000001, 0b00000011111111, 0b1111111111111111, 0b00000000000001, 0b00000000000001, 0b00000000000001, 0b00000000000001, 0b00000000000001, 0b00000000111111, 0b1111111111111111, 0b00000000000001, 0b00000000001111, 0b00000000111100, 0b1111111111111111, 0b00000000000001, 0b00000000000001, 0b00000000000001, 0b00000011111111, 0b00000111111111, 0b00000000111100, 0b00000000111100, 0b1111111111111111, 0b1111111111111111, 0b00000000111100, 0b00001111111111, 0b00000000001111, 0b00000000111100, 0b1111111111111111, 0b00000000000011, 0b1111111111111111, 0b00000000000011, 0b00000000000001, 0b00000000000001, 0b00000000111111, 0b00000000111111, 0b1111111111111111, 0b00000011111111, 0b00000000000001, 0b1111111111111111, 0b1111111111111111, 0b00000000000001, 0b1111111111111111, 0b00000000000011, 0b00000000000001, 0b00000000001111, 0b00000000000001, 0b00000000000001, 0b00000000000001, 0b00000000000111, 0b00000000001111, 0b00000001111111, 0b1111111111111111, 0b1111111111111111, 0b1111111111111111, 0b00000011111111, 0b00000000111111, 0b00000000001111, 0b1111111111111111, 0b1111111111111111, 0b00000000000001, 0b00000000000001, 0b00000000000001, 0b00000011111111, 0b00000011111111, 0b1111111111111111, 0b1111111111111111, 0b00000000001111, 0b00000000001111, 0b00000000000001, 0b00000000000001, 0b00000000000001, 0b00000000000001, 0b00000000001111, 0b00000000001111, 0b00000000000001, 0b00000011111111, 0b1111111111111111, 0b00000000000001, 0b00000011111111, 0b00000000000001, 0b00000000000001, 0b00000000000001, 0b00000000000001, 0b00000000111111, 0b11111111111111, 0b1111111111111111, 0b00000000001111, 0b00000000000001, 0b00000000111100, 0b1111111111111111, 0b1111111111111111, 0b00000000000001, 0b00000000001111, 0b00000000001111, 0b00000000001111, 0b1111111111111111, 0b00000000000001, 0b00000000000011, 0b11111111111111, 0b00000011111111, 0b00000011111111, 0b1111111111111111, 0b00000000111110, 0b00111111111111, 0b00000000111100, 0b1111111111111111, 0b1111111111111111, 0b1111111111111111, 0b1111111111111111, 0b1111111111111111, 0b00000000000001, 0b00000000000001, 0b00000000111101, 0b1111111111111111, 0b00000000001111, 0b1111111111111111, 0b00000000111111, 0b00000000000001, 0b00000000000001, 0b00000000111111, 0b00000000111111, 0b00000000001111, 0b00000000001111, 0b00000000000001, 0b00000000000001, 0b00000011111111, 0b00000000000111, 0b00000000000001, 0b00000000000001, 0b1111111111111111, 0b1111111111111111, 0b00000000000001, 0b00000000000001, 0b11111100111111, 0b1111111111111111, 0b00000000111100, 0b1111111111111111, 0b00000000000111, 0b00000000001111, 0b00000000000001, 0b00000100000001, 0b1111111111111111, 0b1111111111111111, 0b1111111111111111, 0b1111111111111111, 0b1111111111111111, 0b00000000000001, 0b00000000000001, 0b00000000000001, 0b00000000000001, 0b00000000000001, 0b1111111111111111, 0b1111111111111111, 0b1111111111111111, 0b1111111111111111, 0b1111111111111111, 0b00000000111111, 0b00000000000001, 0b00000000111111, 0b00000000000001, 0b00000000000001, 0b00000000001111, 0b00000000001111, 0b00000000001111, 0b00000000000001, 0b00000000001111, 0b00000000000001, 0b00000000000001, 0b1111111111111111, 0b1111111111111111, 0b00000000000001, 0b00000000000001, 0b00000000000001, 0b00000000000001, 0b00000000000001, 0b00000000000001, 0b00000000111111, 0b00000000000001, 0b00000000000001, 0b00000000000001, 0b00000000000001, 0b00000000000001, 0b00000000000001, 0b00000000000001, 0b00000000000001, 0b00000000000001, 0b00000000000001, 0b00000000000001, 0b00000000000001, 0b00000000000001, 0b00000000000001, 0b00000000000001, 0b00000000000001, 0b00000000001111, 0b00000000001111, 0b00000000001111, 0b00000000001111, 0b00000000001111, 0b00000000001111, 0b00000000001111, 0b00000000001111, 0b00000000001111, 0b00000000001111, 0b00000000001111, 0b00000000001111, 0b00000000001111, 0b00000000001111, 0b00000000001111, 0b00000000001111, 0b1111111111111111, 0b1111111111111111, 0b00000000000001, 0b00000000000001, 0b00000000111111, ];

pub fn is_damage_valid(id: u8, damage: u8) -> Result<(), OldBlockParseError> {
    is_number_id_valid(id)?;

    if damage >= 16 {   //minecraft uses only first 4 bits in damage value
        return Err(OldBlockParseError::DamageMoreThan15 { damage });
    }

    let valid_damage_lut = VALID_DAMAGE_LUT[id as usize];
    let valid = valid_damage_lut & (1u16 << damage);

    if valid != 0 {
        return Ok(());
    }
    return Err(OldBlockParseError::DamageNotDefinedForThisBlock { id, damage });
}

pub fn num_valid_damage_values(id: u8) -> u8 {
    if let Err(_) = is_number_id_valid(id) {
        return 0;
    }

    let mut counter = 0;

    for damage in 0..16 {
        let mask = 1u16 << damage;
        if mask & VALID_DAMAGE_LUT[id as usize] != 0 {
            counter += 1;
        }
    }

    return counter;
}

pub fn get_valid_damage_values(id: u8, dest: &mut Vec<u8>) {
    dest.clear();
    dest.reserve(16);
    if let Err(_) = is_number_id_valid(id) {
        return;
    }


    for damage in 0..16 {
        let mask = 1u16 << damage;
        if mask & VALID_DAMAGE_LUT[id as usize] != 0 {
            dest.push(damage);
        }
    }
}

// empty string means invalid
pub fn index_to_wood_variant(idx: u8) -> &'static str {
    return match idx {
        0 => "oak",
        1 => "spruce",
        2 => "birch",
        3 => "jungle",
        4 => "acacia",
        5 => "dark_oak",
        _ => "",
    }
}

pub fn index_to_axis(idx: u8) -> &'static str {
    return match idx {
        0 => "y",
        1 => "x",
        2 => "z",
        _ => "",
    }
}

pub fn index_to_color_old(idx: u8) -> &'static str {
    let lut = ["white",
        "orange",
        "magenta",
        "light_blue",
        "yellow",
        "lime",
        "pink",
        "gray",
        "silver",
        "cyan",
        "purple",
        "blue",
        "brown",
        "green",
        "red",
        "black"];
    if idx >= 16 { return ""; }
    return lut[idx as usize];
}

pub fn index_to_torch_facing(idx: u8) -> &'static str {
    return match idx {
        1 => "east",
        2 => "west",
        3 => "south",
        4 => "north",
        5 => "up",
        _ => "",
    }
}

pub fn index_to_stairs_facing(idx: u8) -> &'static str {
    return index_to_torch_facing(idx);
}

pub fn index_to_stone_variant(idx: u8) -> &'static str {
    return match idx {
        0 => "stone",
        1 => "sandstone",
        2 => "wood_old",
        3 => "cobblestone",
        4 => "brick",
        5 => "stone_brick",
        6 => "nether_brick",
        7 => "quartz",
        _ => "",
    }
}

pub fn index_to_bed_facing(idx: u8) -> &'static str {
    return match idx {
        0 => "south",
        1 => "west",
        2 => "north",
        3 => "east",
        _ => "",
    }
}

pub fn index_to_pumpkin_facing(idx: u8) -> &'static str {
    return index_to_bed_facing(idx);
}

pub fn index_to_piston_facing(idx: u8) -> &'static str {
    return match idx {
        0 => "down",
        1 => "up",
        2 => "north",
        3 => "south",
        4 => "west",
        5 => "east",
        _ => "",
    }
}

pub fn index_to_wooden_door_facing(idx: u8) -> &'static str {
    return match idx {
        0 => "east",
        1 => "south",
        2 => "west",
        3 => "north",
        _ => "",
    }
}

pub fn index_to_rail_shape(idx: u8) -> &'static str {
    return match idx {
        0 => "north_south",
        1 => "east_south",
        2 => "ascending_east",
        3 => "ascending_west",
        4 => "ascending_north",
        5 => "ascending_south",
        6 => "south_east",
        7 => "south_west",
        8 => "north_west",
        9 => "north_east",
        _ => "",
    }
}

pub fn index_to_lever_facing(idx: u8) -> &'static str {
    return match idx {
        0 => "down_x",
        1 => "east",
        2 => "west",
        3 => "south",
        4 => "north",
        5 => "up_z",
        6 => "up_x",
        7 => "down_z",
        _ => "",
    }
}

pub fn index_to_repeater_facing(idx: u8) -> &'static str {
    return match idx {
        0 => "north",
        1 => "east",
        2 => "south",
        3 => "west",
        _ => "",
    }
}

pub fn index_to_trapdoor_facing(idx: u8) -> &'static str {
    return match idx {
        0 => "south",
        1 => "north",
        2 => "east",
        3 => "west",
        _ => "",
    }
}

#[allow(dead_code)]
impl Block {
    pub fn from_old(id: u8, damage: u8, version: DataVersion) -> Result<Block, OldBlockParseError> {
        if version > DataVersion::Java_1_12_2 {
            return Err(OldBlockParseError::NotAnOldVersion { version });
        }


        if let Err(e) = is_damage_valid(id, damage) {
            return Err(e);
        }

        let mut block = Block::from_id(OLD_BLOCK_ID[id as usize]);
        debug_assert!(block.is_ok());
        let mut block = block.unwrap();
        debug_assert!(block.attributes.is_empty());

        let allowed_damage_value_num = num_valid_damage_values(id);
        debug_assert!(allowed_damage_value_num > 0);
        // no properties
        if allowed_damage_value_num == 1 {
            return Ok(block);
        }

        if id == 5 {    // planks
            let variant = index_to_wood_variant(damage);
            if !variant.is_empty() {
                block.set_property("variant", variant);
            } else {
                return Err(OldBlockParseError::DamageNotDefinedForThisBlock { id, damage })
            }
            return Ok(block);
        }

        if id == 1 {  //stone
            match damage {
                0 => block.set_property("variant", "stone"),
                1 => block.set_property("variant", "granite"),
                2 => block.set_property("variant", "smooth_granite"),
                3 => block.set_property("variant", "diorite"),
                4 => block.set_property("variant", "smooth_diorite"),
                5 => block.set_property("variant", "andesite"),
                6 => block.set_property("variant", "smooth_andesite"),
                _ => return Err(OldBlockParseError::DamageNotDefinedForThisBlock { id, damage }),
            };
            return Ok(block);
        }

        if id == 3 {    //dirt
            block.set_property("snowy", "false");
            match damage {
                0 => block.set_property("variant", "dirt"),
                1 => block.set_property("variant", "coarse_dirt"),
                2 => block.set_property("variant", "podzol"),
                _ => return Err(OldBlockParseError::DamageNotDefinedForThisBlock { id, damage }),
            };
            return Ok(block);
        }

        if id == 6 {//sapling
            let variant = index_to_wood_variant(damage & 0b111);
            if !variant.is_empty() {
                block.set_property("variant", variant);
            } else {
                return return Err(OldBlockParseError::DamageNotDefinedForThisBlock { id, damage });
            }
            return Ok(block);
        }

        if id == 8 {//flowing_water
            block.id = "water".to_string();
            block.attributes.insert("level".to_string(), damage.to_string());
            return Ok(block);
        }
        if id == 9 {//water
            block.attributes.insert("level".to_string(), "0".to_string());
            return Ok(block);
        }

        if id == 10 {//flowing_lava
            block.id = "lava".to_string();
            block.attributes.insert("level".to_string(), damage.to_string());
            return Ok(block);
        }
        if id == 11 {//lava
            block.attributes.insert("level".to_string(), "0".to_string());
            return Ok(block);
        }

        if id == 12 {//sand
            match damage {
                0 => block.set_property("variant", "sand"),
                1 => block.set_property("variant", "red_sand"),
                _ => return Err(OldBlockParseError::DamageNotDefinedForThisBlock { id, damage }),
            };
            return Ok(block);
        }

        if id == 17 || id == 162 {// log and log2
            let variant;
            if id == 17 {
                let variant_idx = damage & 0b11;
                variant = index_to_wood_variant(variant_idx);
            } else {
                let variant_idx = damage & 0b11 + 4;
                variant = index_to_wood_variant(variant_idx);
            }
            if variant.is_empty() { return Err(OldBlockParseError::DamageNotDefinedForThisBlock { id, damage }); }
            let direction_idx = (damage & 0b1100) >> 2;

            if direction_idx == 3 {   // wood block.
                block.id = format!("{}_wood", variant);
                // Axis of wood block is not defined in 1.12-, in 1.13+ the default value is y
                block.set_property("axis", "y");
                return Ok(block);
            }
            let direction = index_to_axis(direction_idx);
            debug_assert!(!direction.is_empty());
            block.set_property("variant", variant);
            block.set_property("axis", direction);
            return Ok(block);
        }

        if id == 18 || id == 161 {//leaves and leaves2
            let variant_idx;
            if id == 18 {
                variant_idx = damage & 0b11;
            } else {
                variant_idx = (damage & 0b11) + 4;
            }
            let variant = index_to_wood_variant(variant_idx);
            if variant.is_empty() { return Err(OldBlockParseError::DamageNotDefinedForThisBlock { id, damage }); }

            let decayable;
            let check_decay;
            if id == 18 {
                decayable = (damage >= 4 && damage <= 7) || (damage >= 12 && damage <= 15);
                check_decay = damage >= 8;
            } else {
                decayable = (damage >= 4 && damage <= 5) || (damage >= 12 && damage <= 13);
                check_decay = damage >= 8;
            }
            block.set_property("variant", variant);
            block.set_property("check_decay", &check_decay);
            block.set_property("decayable", &decayable);
            return Ok(block);
        }

        if [35, 159, 95, 171].contains(&id) {//wool, hardened clay, carpet, stained glass
            let color = index_to_color_old(damage);
            debug_assert!(!color.is_empty());
            block.set_property("color", color);
            return Ok(block);
        }

        if [50, 75, 76].contains(&id) { // torch, redstone torch and unlit redstone torch
            let facing = index_to_torch_facing(damage);
            debug_assert!(!facing.is_empty());
            block.set_property("facing", facing);
            return Ok(block);
        }

        if [43, 181, 44, 182, ].contains(&id) {//stone slab
            let variant: &str;
            if [43, 44].contains(&id) {
                let variant_idx = damage & 0b111;
                variant = index_to_stone_variant(variant_idx);
            } else {
                variant = "red_sandstone";
            }
            debug_assert!(!variant.is_empty());
            let is_top: bool;
            let is_double: bool;
            let is_seamless: bool;
            match id {
                43 => {
                    is_top = false;
                    is_double = true;
                    is_seamless = damage >= 8;
                },
                44 => {
                    is_double = false;
                    is_top = damage >= 8;
                    is_seamless = false;
                },
                181 => {
                    is_top = false;
                    is_double = true;
                    is_seamless = damage >= 8;
                },
                182 => {
                    is_double = false;
                    is_top = damage >= 8;
                    is_seamless = false;
                },
                _ => { panic!("Unreachable"); }
            }
            block.set_property("variant", variant);
            if is_double {
                block.set_property("seamless", &is_seamless);
            } else {
                block.set_property("half", if is_top { "top" } else { "bottom" });
            }
            return Ok(block);
        }

        if [125, 126].contains(&id) {// wooded slab
            let variant_index = damage & 0b111;
            let variant = index_to_wood_variant(variant_index);
            debug_assert!(!variant.is_empty());
            let is_double;
            let is_top;
            let is_seamless: bool;
            if id == 125 {//double wood slab
                is_double = true;
                is_top = false;
                is_seamless = damage >= 8;
            } else {
                is_double = false;
                is_top = damage >= 8;
                is_seamless = false;
            }
            block.set_property("variant", variant);
            if is_double {
                block.set_property("seamless", &is_seamless);
            } else {
                block.set_property("half", if is_top { "top" } else { "bottom" });
            }
            return Ok(block);
        }

        if [204, 205].contains(&id) {// purpur_slab
            block.set_property("variant", "default");
            if id == 204 {
                let is_top = damage >= 8;
                block.set_property("half", if is_top { "top" } else { "bottom" });
            }
            return Ok(block);
        }

        if id == 51 {
            block.set_property("age", &damage);
            return Ok(block);
        }

        if [24, 179].contains(&id) {//sandstone, red_sandstone
            let type_: String;
            match id {
                0 => type_ = block.id.to_string(),
                1 => type_ = format!("chiseled_{}", block.id),
                2 => type_ = format!("smooth_{}", block.id),
                _ => return Err(OldBlockParseError::DamageNotDefinedForThisBlock { id, damage }),
            }
            block.attributes.insert("type".to_string(), type_);
            return Ok(block);
        }

        if id == 26 {//bed
            let facing = index_to_torch_facing(damage & 0b11);
            debug_assert!(!facing.is_empty());
            let occupied = (damage & 0x4) != 0;
            let part_head = (damage & 0x8) != 0;
            block.set_property("facing", facing);
            block.set_property("occupied", &occupied);
            block.set_property("part", if part_head { "head" } else { "foot" });
            return Ok(block);
        }

        if id == 31 {//grass (minecraft:tallgrass in 1.12
            let type_: &str;
            match damage {
                0 => type_ = "dead_bush",
                1 => type_ = "tall_grass",
                2 => type_ = "fern",
                _ => return Err(OldBlockParseError::DamageNotDefinedForThisBlock { id, damage }),
            }
            block.set_property("type", type_);
            return Ok(block);
        }

        if id == 37 {//yellow_flower
            block.set_property("type", "dandelion");
            return Ok(block);
        }
        if id == 38 {//red_flower
            let type_: &str;
            match damage {
                0 => type_ = "poppy",
                1 => type_ = "orchid",
                2 => type_ = "allium",
                3 => type_ = "houstonia",
                4 => type_ = "tulip_red",
                5 => type_ = "tulip_orange",
                6 => type_ = "tulip_white",
                7 => type_ = "tulip_pink",
                8 => type_ = "oxeye",
                _ => return Err(OldBlockParseError::DamageNotDefinedForThisBlock { id, damage }),
            }
            block.set_property("type", type_);
            return Ok(block);
        }

        if id == 175 {//double plant
            let lower = damage >= 8;
            block.set_property("half", if lower { "lower" } else { "upper" });
            if !lower {
                let variant: &str;
                match damage {
                    0 => variant = "sunflower",
                    1 => variant = "syringa",
                    2 => variant = "grass",
                    3 => variant = "fern",
                    4 => variant = "rose",
                    5 => variant = "paeonia",
                    _ => return Err(OldBlockParseError::DamageNotDefinedForThisBlock { id, damage }),
                }
                block.set_property("variant", variant);
            }
            return Ok(block);
        }

        if [29, 33].contains(&id) {//piston and sticky piston (main body)
            let facing = index_to_torch_facing(damage & 0b111);
            debug_assert!(!facing.is_empty());
            let extended = (damage & 0x8) != 0;
            block.set_property("facing", facing);
            block.set_property("extended", &extended);
            return Ok(block);
        }

        if [34, 36].contains(&id) {//piston_head and piston_extension (b36)
            let facing = index_to_torch_facing(damage & 0b111);
            debug_assert!(!facing.is_empty());
            let sticky = (damage & 0x8) != 0;
            block.set_property("facing", facing);
            block.set_property("sticky", if sticky { "sticky" } else { "normal" });
            return Ok(block);
        }

        if [53, 67, 108, 109, 114, 128, 134, 135, 136, 156, 163, 164, 180, 203].contains(&id) {//stairs
            let facing = index_to_stairs_facing(damage & 0b11);
            debug_assert!(!facing.is_empty());
            let is_top = (damage & 0x4) != 0;
            block.set_property("facing", facing);
            block.set_property("half", if is_top { "top" } else { "bottom" });
            return Ok(block);
        }

        if id == 55 {//redstone wire
            block.set_property("power", &damage);
            return Ok(block);
        }

        if [151, 178].contains(&id) { // day light detector
            block.set_property("power", &damage);
            return Ok(block);
        }

        if [59, 141, 142, 207].contains(&id) {//crops
            block.set_property("age", &damage);
            return Ok(block);
        }

        if id == 60 {
            block.set_property("moisture", &damage);
            return Ok(block);
        }
        if [176, 63].contains(&id) {//minecraft:standing_banner and standing_sign
            block.set_property("rotation", &damage);
            return Ok(block);
        }
        if [177, 68].contains(&id) {//minecraft:wall_banner and wall_sign
            if damage < 2 || damage > 5 {
                return Err(OldBlockParseError::DamageNotDefinedForThisBlock { id, damage });
            }
            let facing = index_to_piston_facing(id);
            debug_assert!(!facing.is_empty());
            block.set_property("facing", facing);
            return Ok(block);
        }

        if [64, 71, 193, 194, 195, 196, 197].contains(&id) {// doors
            let is_upper = (damage & 0x8) != 0;
            block.set_property("half", if is_upper { "upper" } else { "lower" });
            if is_upper {
                let is_hing_right = (damage & 0b1) != 0;
                let is_powered = (damage & 0b10) != 0;
                block.set_property("hing", if is_hing_right { "right" } else { "left" });
                block.set_property("powered", &is_powered);
            } else {
                let facing = index_to_wooden_door_facing(damage & 0b11);
                let is_open = (damage & 0x4) != 0;
                block.set_property("facing", facing);
                block.set_property("open", &is_open);
            }
            return Ok(block);
        }

        if id == 66 {//rail
            let shape = index_to_rail_shape(damage);
            debug_assert!(!shape.is_empty());
            block.set_property("shape", shape);
            return Ok(block);
        }

        if [27, 28, 157].contains(&id) {// golden, detector and activation rails
            let shape_idx = damage & 0b111;
            if shape_idx > 5 {
                return Err(OldBlockParseError::DamageNotDefinedForThisBlock { id, damage });
            }
            let shape = index_to_rail_shape(shape_idx);
            debug_assert!(!shape.is_empty());
            let powered = (damage & 0x8) != 0;
            block.set_property("shape", shape);
            block.set_property("powered", &powered);
            return Ok(block);
        }

        if [54, 146, 65, 61].contains(&id) {
            if damage < 2 || damage > 5 {
                return Err(OldBlockParseError::DamageNotDefinedForThisBlock { id, damage });
            }
            let facing = index_to_piston_facing(damage);
            debug_assert!(!facing.is_empty());
            block.set_property("facing", facing);
            return Ok(block);
        }

        if [23, 158].contains(&id) {//dispenser and dropper
            let facing = index_to_piston_facing(damage & 0b111);
            debug_assert!(!facing.is_empty());
            let triggered = (damage & 0x8) != 0;
            block.set_property("facing", facing);
            block.set_property("triggered", &triggered);
            return Ok(block);
        }

        if id == 154 {//hopper
            let facing = index_to_piston_facing(damage & 0b111);
            debug_assert!(!facing.is_empty());
            let enabled = (damage & 0x8) == 0;
            block.set_property("facing", facing);
            block.set_property("enabled", &enabled);
            return Ok(block);
        }

        if id == 69 {//lever
            let facing = index_to_lever_facing(damage & 0b111);
            debug_assert!(!facing.is_empty());
            let powered = (damage & 0x8) != 0;
            block.set_property("facing", facing);
            block.set_property("powered", &powered);
            return Ok(block);
        }

        if [70, 72].contains(&id) {//wooden and stone pressure plate
            debug_assert!(damage < 2);
            let powered = damage != 0;
            block.set_property("powered", &powered);
            return Ok(block);
        }

        if [147, 148].contains(&id) {//weighted pressure plate
            block.set_property("power", &damage);
            return Ok(block);
        }

        if [77, 143].contains(&id) {//buttons
            let facing = index_to_piston_facing(damage & 0b111);
            debug_assert!(!facing.is_empty());
            let powered = (damage & 0x8) != 0;
            block.set_property("facing", facing);
            block.set_property("powered", &powered);
            return Ok(block);
        }

        if id == 78 {//snow layers
            block.set_property("layers", &(damage + 1));
            return Ok(block);
        }

        if [81, 83].contains(&id) {//cactus, reeds
            block.set_property("age", &damage);
            return Ok(block);
        }

        if id == 84 {//jukebox
            block.set_property("has_record", &(damage != 0));
            return Ok(block);
        }

        if [86, 104].contains(&id) {//pumpkin and pumpkin light
            let facing = index_to_pumpkin_facing(damage);
            debug_assert!(!facing.is_empty());
            block.set_property("facing", facing);
            return Ok(block);
        }

        if id == 92 {//cake
            debug_assert!(damage <= 6);
            block.set_property("bites", &damage);
            return Ok(block);
        }

        if [93, 94].contains(&id) {//repeater
            let facing = index_to_repeater_facing(damage & 0b11);
            debug_assert!(!facing.is_empty());
            let delay = 1 + ((damage & 0b1100) >> 2);
            block.set_property("facing", facing);
            block.set_property("delay", &delay);
            return Ok(block);
        }

        if [149, 150].contains(&id) {//comparator
            let facing = index_to_repeater_facing(damage & 0b11);
            debug_assert!(!facing.is_empty());
            let subtract_mode = (damage & 0x4) != 0;
            let powered = (damage & 0x8) != 0;
            block.set_property("facing", facing);
            block.set_property("mode", if subtract_mode { "subtract" } else { "compare" });
            block.set_property("powered", &powered);
            return Ok(block);
        }

        if [96, 167].contains(&id) {//trapdoors
            let facing = index_to_trapdoor_facing(damage & 0b11);
            let open = (damage & 0x4) != 0;
            let top = (damage & 0x8) != 0;
            block.set_property("facing", facing);
            block.set_property("open", &open);
            block.set_property("half", if top { "top" } else { "bottom" });
            return Ok(block);
        }

        if id == 97 {//monster egg
            let variant = match damage {
                0 => "stone",
                1 => "cobblestone",
                2 => "stone_brick",
                3 => "mossy_stone_brick",
                4 => "cracked_stone_brick",
                5 => "chiseled_stone_brick",
                _ => { return Err(OldBlockParseError::DamageNotDefinedForThisBlock { id, damage }); },
            };
            block.set_property("variant", variant);
            return Ok(block);
        }


        //return Ok(block);
        !todo!();
    }
}