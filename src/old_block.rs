use strum::Display;
use crate::block::Block;
use crate::schem::DataVersion;

pub const OLD_BLOCK_ID: [&'static str; 256] = ["minecraft:air", "minecraft:stone", "minecraft:grass", "minecraft:dirt", "minecraft:cobblestone", "minecraft:planks", "minecraft:sapling", "minecraft:bedrock", "minecraft:flowing_water", "minecraft:water", "minecraft:flowing_lava", "minecraft:lava", "minecraft:sand", "minecraft:gravel", "minecraft:gold_ore", "minecraft:iron_ore", "minecraft:coal_ore", "minecraft:log", "minecraft:leaves", "minecraft:sponge", "minecraft:glass", "minecraft:lapis_ore", "minecraft:lapis_block", "minecraft:dispenser", "minecraft:sandstone", "minecraft:noteblock", "minecraft:bed", "minecraft:golden_rail", "minecraft:detector_rail", "minecraft:sticky_piston", "minecraft:web", "minecraft:tallgrass", "minecraft:deadbush", "minecraft:piston", "minecraft:piston_head", "minecraft:wool", "minecraft:piston_extension", "minecraft:yellow_flower", "minecraft:red_flower", "minecraft:brown_mushroom", "minecraft:red_mushroom", "minecraft:gold_block", "minecraft:iron_block", "minecraft:double_stone_slab", "minecraft:stone_slab", "minecraft:brick_block", "minecraft:tnt", "minecraft:bookshelf", "minecraft:mossy_cobblestone", "minecraft:obsidian", "minecraft:torch", "minecraft:fire", "minecraft:mob_spawner", "minecraft:oak_stairs", "minecraft:chest", "minecraft:redstone_wire", "minecraft:diamond_ore", "minecraft:diamond_block", "minecraft:crafting_table", "minecraft:wheat", "minecraft:farmland", "minecraft:furnace", "minecraft:lit_furnace", "minecraft:standing_sign", "minecraft:wooden_door", "minecraft:ladder", "minecraft:rail", "minecraft:stone_stairs", "minecraft:wall_sign", "minecraft:lever", "minecraft:stone_pressure_plate", "minecraft:iron_door", "minecraft:wooden_pressure_plate", "minecraft:redstone_ore", "minecraft:lit_redstone_ore", "minecraft:unlit_redstone_torch", "minecraft:redstone_torch", "minecraft:stone_button", "minecraft:snow_layer", "minecraft:ice", "minecraft:snow", "minecraft:cactus", "minecraft:clay", "minecraft:reeds", "minecraft:jukebox", "minecraft:fence", "minecraft:pumpkin", "minecraft:netherrack", "minecraft:soul_sand", "minecraft:glowstone", "minecraft:portal", "minecraft:lit_pumpkin", "minecraft:cake", "minecraft:unpowered_repeater", "minecraft:powered_repeater", "minecraft:stained_glass", "minecraft:trapdoor", "minecraft:monster_egg", "minecraft:stonebrick", "minecraft:brown_mushroom_block", "minecraft:red_mushroom_block", "minecraft:iron_bars", "minecraft:glass_pane", "minecraft:melon_block", "minecraft:pumpkin_stem", "minecraft:melon_stem", "minecraft:vine", "minecraft:fence_gate", "minecraft:brick_stairs", "minecraft:stone_brick_stairs", "minecraft:mycelium", "minecraft:waterlily", "minecraft:nether_bricks", "minecraft:nether_brick_fence", "minecraft:nether_brick_stairs", "minecraft:nether_wart", "minecraft:enchanting_table", "minecraft:brewing_stand", "minecraft:cauldron", "minecraft:end_portal", "minecraft:end_portal_frame", "minecraft:end_stone", "minecraft:dragon_egg", "minecraft:redstone_lamp", "minecraft:lit_redstone_lamp", "minecraft:double_wooden_slab", "minecraft:wooden_slab", "minecraft:cocoa", "minecraft:sandstone_stairs", "minecraft:emerald_ore", "minecraft:ender_chest", "minecraft:tripwire_hook", "minecraft:tripwire", "minecraft:emerald_block", "minecraft:spruce_stairs", "minecraft:birch_stairs", "minecraft:jungle_stairs", "minecraft:command_block", "minecraft:beacon", "minecraft:cobblestone_wall", "minecraft:flower_pot", "minecraft:carrots", "minecraft:potatoes", "minecraft:wooden_button", "minecraft:skull", "minecraft:anvil", "minecraft:trapped_chest", "minecraft:light_weighted_pressure_plate", "minecraft:heavy_weighted_pressure_plate", "minecraft:unpowered_comparator", "minecraft:powered_comparator", "minecraft:daylight_detector", "minecraft:redstone_block", "minecraft:quartz_ore", "minecraft:hopper", "minecraft:quartz_block", "minecraft:quartz_stairs", "minecraft:activator_rail", "minecraft:dropper", "minecraft:stained_hardened_clay", "minecraft:stained_glass_pane", "minecraft:leaves2", "minecraft:log2", "minecraft:acacia_stairs", "minecraft:dark_oak_stairs", "minecraft:slime", "minecraft:barrier", "minecraft:iron_trapdoor", "minecraft:prismarine", "minecraft:sea_lantern", "minecraft:hay_block", "minecraft:carpet", "minecraft:hardened_clay", "minecraft:coal_block", "minecraft:packed_ice", "minecraft:double_plant", "minecraft:standing_banner", "minecraft:wall_banner", "minecraft:daylight_detector_inverted", "minecraft:red_sandstone", "minecraft:red_sandstone_stairs", "minecraft:double_stone_slab2", "minecraft:stone_slab2", "minecraft:spruce_fence_gate", "minecraft:birch_fence_gate", "minecraft:jungle_fence_gate", "minecraft:dark_oak_fence_gate", "minecraft:acacia_fence_gate", "minecraft:spruce_fence", "minecraft:birch_fence", "minecraft:jungle_fence", "minecraft:dark_oak_fence", "minecraft:acacia_fence", "minecraft:spruce_door", "minecraft:birch_door", "minecraft:jungle_door", "minecraft:acacia_door", "minecraft:dark_oak_door", "minecraft:end_rod", "minecraft:chorus_plant", "minecraft:chorus_flower", "minecraft:purpur_block", "minecraft:purpur_pillar", "minecraft:purpur_stairs", "minecraft:purpur_double_slab", "minecraft:purpur_slab", "minecraft:end_bricks", "minecraft:beetroots", "minecraft:grass_path", "minecraft:end_gateway", "minecraft:repeating_command_block", "minecraft:chain_command_block", "minecraft:frosted_ice", "minecraft:magma", "minecraft:nether_wart_block", "minecraft:red_nether_bricks", "minecraft:bone_block", "minecraft:structure_void", "minecraft:observer", "minecraft:white_shulker_box", "minecraft:orange_shulker_box", "minecraft:magenta_shulker_box", "minecraft:light_blue_shulker_box", "minecraft:yellow_shulker_box", "minecraft:lime_shulker_box", "minecraft:pink_shulker_box", "minecraft:gray_shulker_box", "minecraft:silver_shulker_box", "minecraft:cyan_shulker_box", "minecraft:purple_shulker_box", "minecraft:blue_shulker_box", "minecraft:brown_shulker_box", "minecraft:green_shulker_box", "minecraft:red_shulker_box", "minecraft:black_shulker_box", "minecraft:white_glazed_terracotta", "minecraft:orange_glazed_terracotta", "minecraft:magenta_glazed_terracotta", "minecraft:light_blue_glazed_terracotta", "minecraft:yellow_glazed_terracotta", "minecraft:lime_glazed_terracotta", "minecraft:pink_glazed_terracotta", "minecraft:gray_glazed_terracotta", "minecraft:silver_glazed_terracotta", "minecraft:cyan_glazed_terracotta", "minecraft:purple_glazed_terracotta", "minecraft:blue_glazed_terracotta", "minecraft:brown_glazed_terracotta", "minecraft:green_glazed_terracotta", "minecraft:red_glazed_terracotta", "minecraft:black_glazed_terracotta", "minecraft:concrete", "minecraft:concrete_powder", "", "", "minecraft:structure_block"];

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


    if id == 51 && damage == 0xFF { // eternal fire
        return Ok(());
    }

    if damage >= 16 {   //minecraft uses only first 4 bits in damage value, except eternal fire.
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
        _ => "",
    }
}

pub fn index_to_stone_variant(idx: u8) -> &'static str {
    return match idx {
        0 => "stone",
        1 => "sandstone",
        2 => "wooden",
        3 => "cobblestone",
        4 => "brick",
        5 => "stone_brick",
        6 => "nether_brick",
        7 => "quartz",
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

            if damage == 5 {//standing torch
                return Ok(block);
            }
            // wall torch
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
            match id {
                43 => {
                    is_top = false;
                    is_double = true;
                },
                44 => {
                    is_top = damage >= 8;
                    is_double = false;
                },
                181 => {
                    is_double = true;
                    is_top = false;
                },
                182 => {
                    is_double = false;
                    is_top = damage >= 8;
                },
                _ => { panic!("Unreachable"); }
            }
            block.set_property("variant", variant);
            if is_double {
                return Ok(block);
            }
            block.set_property("half", if is_top { "top" } else { "bottom" });
            return Ok(block);
        }

        if [125, 126].contains(&id) {// wooded slab
            let variant_index = damage & 0b111;
            let variant = index_to_wood_variant(variant_index);
            debug_assert!(!variant.is_empty());
            let is_double;
            let is_top;
            if id == 125 {//double wood slab
                is_double = true;
                is_top = false;
            } else {
                is_double = false;
                is_top = damage >= 8;
            }
            block.set_property("variant", variant);
            if is_double {
                return Ok(block);
            }
            block.set_property("half", if is_top { "top" } else { "bottom" });
            return Ok(block);
        }


        //return Ok(block);
        !todo!();
    }
}