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

use std::collections::HashMap;
use std::env;
use std::fs::{create_dir_all, File};
// use std::io::Read;
use fastnbt::Value;
use flate2::{Compression, GzBuilder};
use flate2::read::{GzDecoder};
use ndarray::Array3;
use rand::Rng;
use mc_schem::block::CommonBlock;
use mc_schem::{Schematic, WorldEdit12LoadOption, LitematicaLoadOption, LitematicaSaveOption, Block, schem, old_block, DataVersion, WorldEdit13SaveOption, Region, BlockEntity, MetaDataIR, WorldEdit13LoadOption};

#[test]
fn block_id_parse() {

    let pass_ids = [
        "minecraft:air",
        "minecraft:stone",
        "minecraft:birch_log[axis=x]",
        "hay_block[axis=y]",
        "mod_name:stained_glass[color=white]",
        "minecraft:stone_slab[half=top,variant=brick]",
        "minecraft:red_mushroom_block[east=true,west=true,north=true,south=true,up=true,down=true]",
        "mushroom_stem[east=true,west=true,north=true,south=true,up=true,down=true]",
    ];

    for id in pass_ids {
        let blk_res = Block::from_id(id);

        match blk_res {
            Err(err) => {
                panic!("{} failed with {}", id, err);
            }
            Ok(blk) => {
                println!(
                    "namespace = {}, id = {}, attributes:",
                    blk.namespace, blk.id
                );
                for att in &blk.attributes {
                    println!("\t {} : {}", att.0, att.1);
                }
                let _ = blk.full_id();
            }
        }
    }

    let nopass_ids=[
            "minecraft::air",
            "minecraft:stone]",
            "minecraft:birch_log[",
            "jjj[axis=y]hay_block",
            "mod_name:[color=white]stained_glass",
            "stained_glass[mod_name:color=white]",
            "minecraft:stone_slab[half=top,,variant=brick]",
            "minecraft:red_mushroom_block[east=true,west=true,north=true,south=true,up=true,down=true,]",
            "mushroom_stem[east=true,west=true,north=true,south=true,up=true,down=true###]"
        ];
    for id in nopass_ids {
        let blk_res = Block::from_id(id);
        match blk_res {
            Err(err) => {
                println!("\"{}\" is invalid because {}", id, err);
            }
            Ok(blk) => {
                panic!(
                    "Invalid id \"{}\" is parsed successfully, result: \"{}\"",
                    id,
                    blk.full_id()
                );
            }
        }
    }
}



#[test]
fn block_bits_required() {
    for palette_size in 0..258 {
        let bits = schem::litematica::block_required_bits(palette_size);
        println!("{} blocks requires {} bit(s)", palette_size, bits);
    }
}

#[test]
fn ceil_up_to() {
    let tests = [
        ((0isize, 12isize), 0isize),
        ((13, 12), 24),
        ((120, 12), 120),
        ((121, 12), 132)
    ];
    for ((a, b), expected) in tests {
        let result = schem::common::ceil_up_to(a, b);
        if result != expected {
            panic!("{} ceil up to {} should b {}, but found {}", a, b, expected, result);
        } else {
            println!("{} ceil up to {} = {}", a, b, expected);
        }
    }
}

#[test]
fn litematica_local_bit_index_to_global_bit_index() {
    let lbi_list = [63, 0, -1, -64, -65, -128, -129, -192];
    let expected = [63, 0, 127, 64, 191, 128, 255, 192];
    for idx in 0..lbi_list.len() {
        let lbi = lbi_list[idx];
        let computed = schem::litematica::MultiBitSet::logic_bit_index_to_global_bit_index(lbi);
        if computed != expected[idx] {
            panic!("logical bit index {} should be mapped to {}, but found {}", lbi, expected[idx], computed);
        }
    }
}

#[test]
fn litematica_multi_bit_set_read() {
    let data = [14242959524133701664u64, 1244691354];
    let mbs = schem::litematica::MultiBitSet::from_data(&data, 18, 5).unwrap();

    assert_eq!(mbs.basic_mask(), 0b11111);

    for idx in 0..mbs.len() {
        println!("mbs[{}] = {}", idx, mbs.get(idx));
    }
}

#[test]
fn litematica_multi_bit_set_rw() {
    //use rand::Rng;
    let mut rng = rand::thread_rng();


    let bits = 1..65;
    let num_elements = 1 << 10;
    for element_bits in bits {
        let mut mbs = schem::litematica::MultiBitSet::new();
        mbs.reset(element_bits, num_elements);

        let value_mask = mbs.element_max_value();

        let mut values: Vec<u64> = Vec::with_capacity(num_elements);
        for _ in 0..num_elements {
            values.push(rng.gen::<u64>() & value_mask);
        }

        for (idx, val) in values.iter().enumerate() {
            mbs.set(idx, *val).unwrap();
        }

        for (idx, val) in values.iter().enumerate() {
            let found = mbs.get(idx);
            if found != *val {
                panic!("mbs[{}] should be {}, but found {}", idx, val, found);
            }
        }
    }
}

#[test]
fn litematica_3d_array_decode() {
    use crate::schem::{LitematicaLoadOption};
    println!("Current dir: {}", env::current_dir().unwrap().to_string_lossy());
    let src_filename = "./test_files/litematica/test01.litematic";

    let schem = Schematic::from_litematica_file(src_filename, &LitematicaLoadOption::default()).unwrap().0;

    for y in 0..19 {
        let bid = schem.first_block_index_at([0, y, 0]).unwrap();
        if bid != y as u16 {
            panic!("Block index at [0, {}, 0] should be {}, but found {}", y, y, bid);
        }
    }
}


// generate value for VALID_DAMAGE_LUT in old_blocks.rs
#[test]
fn process_mc12_damage_data() {
    let raw_data = [
        "0", "0-6", "0", "0-2", "0", "0-5", "0-5,8-13", "0", "0-15", "0-15", "0-15", "0-15", "0-1", "0", "0", "0", "0", "0-15", "0-15", "0-1", "0", "0", "0", "0-5,8-13", "0-2", "0", "0-3,8-15", "0-5,8-13", "0-5,8-13", "0-5,8-13", "0", "0-2", "0", "0-5,8-13", "0-5,8-13", "0-15", "0-5,8-13", "0", "0-8", "0", "0", "0", "0", "0-15", "0-15", "0", "0-1", "0", "0", "0", "1-5", "0-15", "0", "0-7", "2-5", "0-15", "0", "0", "0", "0-7", "0-7", "2-5", "2-5", "0-15", "0-11", "2-5", "0-9", "0-7", "2-5", "0-15", "0-1", "0-11", "0-1", "0", "0", "1-5", "1-5", "0-5,8-13", "0-7", "0", "0", "0-15", "0", "0-15", "0-1", "0", "0-3", "0", "0", "0", "1-2", "0-3", "0-6", "0-15", "0-15", "0-15", "0-15", "0-5", "0-3", "0-10,14-15", "0-10,14-15", "0", "0", "0", "0-7", "0-7", "0-15", "0-15", "0-7", "0-7", "0", "0", "0", "0", "0-7", "0-3", "0", "0-7", "0-3", "0", "0-7", "0", "0", "0", "0", "0-5", "0-5,8-13", "0-11", "0-7", "0", "2-5", "0-15", "0-1,4-5,8-9,12-13", "0", "0-7", "0-7", "0-7", "0-5,8-13", "0", "0-1", "0-15", "0-7", "0-7", "0-5,8-13", "0-5,8-13", "0-11", "2-5", "0-15", "0-15", "0-15", "0-15", "0-15", "0", "0", "0,2-5,8,10-13", "0-4", "0-7", "0-5,8-13", "0-5,8-13", "0-15", "0-15", "0-1,4-5,8-9,12-13", "0-1,4-5,8-9,12-13", "0-7", "0-7", "0", "0", "0-15", "0-2", "0", "0,4,8", "0-15", "0", "0", "0", "0-5,8-11", "0-15", "2-5", "0-15", "0-2", "0-7", "0,8", "0,8", "0-15", "0-15", "0-15", "0-15", "0-15", "0", "0", "0", "0", "0", "0-11", "0-11", "0-11", "0-11", "0-11", "0-5", "0", "0-5", "0", "0,4,8", "0-7", "0", "0,8", "0", "0-3", "0", "0", "0-5,8-13", "0-5,8-13", "0-3", "0", "0", "0", "0,4,8", "0", "0-5,8-13", "0-5", "0-5", "0-5", "0-5", "0-5", "0-5", "0-5", "0-5", "0-5", "0-5", "0-5", "0-5", "0-5", "0-5", "0-5", "0-5", "0-3", "0-3", "0-3", "0-3", "0-3", "0-3", "0-3", "0-3", "0-3", "0-3", "0-3", "0-3", "0-3", "0-3", "0-3", "0-3", "0-15", "0-15", "", "", "0-3"

    ];

    //let mut bit_represented = [0u16; 256];

    fn parse_raw_data(raw: &str) -> Option<u16> {
        if raw.is_empty() { return Some(0); }
        let mut result: u16 = 0;
        for part in raw.split(',') {
            if !part.contains('-') {
                let parsed;
                match str::parse::<u16>(part) {
                    Ok(p) => { parsed = p; }
                    Err(_) => return None,
                }
                assert!(parsed <= 15);
                result |= (1u16) << parsed;
            } else {
                let splitted = part.split('-');
                // check if valid
                if splitted.clone().count() != 2 {
                    return None;
                }
                let mut first_str: &str = "";
                let mut last_str: &str = "";
                for (idx, s) in splitted.enumerate() {
                    if idx == 0 {
                        first_str = s;
                    }
                    if idx == 1 {
                        last_str = s;
                    }
                }

                // get begin and end
                let first: u16;
                match str::parse::<u16>(first_str) {
                    Ok(n) => first = n,
                    Err(_) => return None,
                }
                let last: u16;
                match str::parse::<u16>(last_str) {
                    Ok(n) => last = n,
                    Err(_) => return None,
                }
                assert!(first <= last);
                assert!(last <= 15);
                for n in first..(last + 1) {
                    result |= (1u16) << n;
                }
            }
        }

        return Some(result);
    }

    print!("bit_represented = [");
    for idx in 0..256 {
        let bit_represented = parse_raw_data(raw_data[idx]).unwrap();
        print!("{:#016b}, ", bit_represented);
    }
    print!("];");
}


#[test]
fn test_old_block_number_id_damage() {
    let mut valid_damages = Vec::with_capacity(16);
    let skip_id = [253, 254];
    for id in 0..256 {
        let id = id as u8;
        if skip_id.contains(&id) {
            continue;
        }
        old_block::get_valid_damage_values(id, &mut valid_damages);
        assert_ne!(valid_damages.len(), 0);
        for damage in &valid_damages {
            let _ = Block::from_old(id, *damage, DataVersion::Java_1_12_2).unwrap();
        }
    }
}

#[test]
fn parse_full_blocks_mc12() {
    let num_id_array;
    {
        let decoder = GzDecoder::new(File::open("./test_files/schematic/full-blocks-1.12.2.schematic").unwrap());
        let nbt = fastnbt::from_reader(decoder).unwrap();
        num_id_array = Schematic::parse_number_id_from_we12(&nbt).unwrap();
    }
    let litematic = Schematic::from_litematica_file(
        "./test_files/litematica/full-blocks-1.12.2.litematic",
        &LitematicaLoadOption::default()).unwrap().0;
    let lite_region = &litematic.regions[0];
    for dim in 0..3 {
        assert_eq!(num_id_array.shape()[dim], lite_region.shape_yzx()[dim] as usize);
    }

    let mut hash: HashMap<(u8, u8), String> = HashMap::new();
    hash.reserve(256 * 16);

    for y in 0..num_id_array.shape()[0] {
        for z in 0..num_id_array.shape()[1] {
            for x in 0..num_id_array.shape()[2] {
                let (id, damage) = num_id_array[[y, z, x]];
                if hash.contains_key(&(id, damage)) {
                    continue;
                }

                let block = litematic.first_block_at([x as i32, y as i32, z as i32]).unwrap();

                hash.insert((id, damage), block.full_id());
            }
        }
    }
    println!(" id \t damage \t string id");
    for ((id, damage), val) in &hash {
        println!("{id}\t{damage}\t{val}");
    }

    let mut damage_list = [0u16; 256];
    for ((id, damage), _) in &hash {
        assert!(*damage < 16);
        damage_list[*id as usize] |= 1u16 << damage;
    }

    println!("\n\n\n id \t damage");
    for (id, damages) in damage_list.iter().enumerate() {
        let string = damage_list_u16_to_string(*damages);
        println!("{id}\t{string}");
    }

    fn damage_list_u16_to_string(damage_list: u16) -> String {
        if damage_list == 0 {
            return "".to_string();
        }

        let mut result = String::new();
        let mut temp = Vec::with_capacity(32);
        let mut prev: Option<u8> = None;
        for damage in 0..16 {
            if damage_list & (1u16 << damage) == 0 {
                continue;
            }
            if let Some(prev_damage) = prev {
                if damage - prev_damage > 1 {
                    temp.push(255);
                }
            }
            prev = Some(damage);
            temp.push(damage);
        }

        for slice in temp.split(|x| *x == 255) {
            assert!(slice.len() > 0);
            if slice.len() == 1 {
                result.push_str(&slice.first().unwrap().to_string());
            } else {
                result.push_str(&format!("{}-{}", slice.first().unwrap(), slice.last().unwrap()));
            }
            result.push(',');
        }
        if !result.is_empty() {
            result.pop();
        }

        return result;
    }

}

#[test]
fn make_mc12_numeric_lut() {
    let schem_file = "./test_files/schematic/full-blocks-1.12.2.schematic";

    let schem_option = WorldEdit12LoadOption::default();

    let (schem, _, num_id_array) = Schematic::from_world_edit_12_file(schem_file, &schem_option).unwrap();
    let lite = Schematic::from_litematica_file(
        "./test_files/litematica/full-blocks-1.12.2.litematic",
        &LitematicaLoadOption::default()).unwrap().0;

    for dim in 0..3 {
        let nia_shape_xyz = Region::pos_yzx_to_xyz(&[num_id_array.shape()[0], num_id_array.shape()[1], num_id_array.shape()[2]]);
        assert_eq!(nia_shape_xyz[dim], schem.shape()[dim] as usize);
        assert_eq!(schem.shape()[dim], lite.shape()[dim]);
    }
    let shape = lite.shape();
    let mut hash: HashMap<String, HashMap<String, HashMap<String, Value>>> = HashMap::new();
    hash.reserve(256);
    for y in 0..shape[1] as usize {
        for z in 0..shape[2] as usize {
            for x in 0..shape[0] as usize {
                let (id, damage) = num_id_array[[y, z, x]];
                let pos = [x as i32, y as i32, z as i32];
                let full_id = lite.first_block_at(pos).unwrap().full_id();
                let be_tags = match schem.first_block_entity_at(pos) {
                    Some(be) => be.tags.clone(),
                    None => HashMap::new(),
                };
                let hash = hash.entry(id.to_string()).or_insert(HashMap::new());
                let hash = hash.entry(damage.to_string()).or_insert(HashMap::new());
                hash.insert(full_id, Value::Compound(be_tags));
            }
        }
    }
    create_dir_all("./target/test/make_mc12_numeric_lut").unwrap();
    let file = File::create("./target/test/make_mc12_numeric_lut/out.nbt").unwrap();
    let encoder = GzBuilder::new().filename("out.nbt").write(file, Compression::best());
    fastnbt::to_writer(encoder, &hash).unwrap();
}

#[test]
fn load_save_vanilla_structure() {
    use schem::{VanillaStructureLoadOption, VanillaStructureSaveOption};
    let schem =
        Schematic::from_vanilla_structure_file(
            "./test_files/vanilla_structure/test01.nbt",
            &VanillaStructureLoadOption::default()).unwrap().0;

    create_dir_all("./target/test/load_save_vanilla_structure").unwrap();

    schem.save_vanilla_structure_file(
        "./target/test/load_save_vanilla_structure/out01.nbt",
        &VanillaStructureSaveOption::default()).expect("Failed to save vanilla structure file");

    Schematic::from_vanilla_structure_file(
        "./target/test/load_save_vanilla_structure/out01.nbt", &VanillaStructureLoadOption::default())
        .expect("Failed to load saved vanilla structure");
}

#[test]
fn load_save_litematica() {
    use schem::{LitematicaLoadOption};
    //println!("Current dir: {}", env::current_dir().unwrap().to_string_lossy());

    let src_dir = "./test_files/litematica";
    let out_dir = "./target/test/load_save_litematica";
    create_dir_all(out_dir).unwrap();

    for id in 1..4 {
        let src_filename = format!("{}/test{:02}.litematic", src_dir, id);
        let dst_filename = format!("{}/out{:02}.litematic", out_dir, id);

        let schem = Schematic::from_litematica_file(&src_filename, &LitematicaLoadOption::default()).unwrap().0;

        schem.save_litematica_file(&dst_filename, &LitematicaSaveOption::default()).expect("Failed to save litematica file");

        Schematic::from_litematica_file(&dst_filename, &LitematicaLoadOption::default()).expect("Failed to load saved litematica file");


        //println!("Metadata: \n{:?}", schem.metadata_litematica());
    }
}

#[test]
fn load_litematica_with_negative_size() {
    let src_dir = "./test_files/litematica";
    let src_filename = format!("{}/negative-size-Supercharged_contained_shulker_farm.litematic", src_dir);
    let _ = Schematic::from_litematica_file(&src_filename, &LitematicaLoadOption::default()).unwrap().0;
}

#[test]
fn load_save_world_edit13() {
    use schem::WorldEdit13LoadOption;

    let src_dir = "./test_files/schem";
    let out_dir = "./target/test/load_save_world_edit13";
    create_dir_all(out_dir).unwrap();

    for id in 1..3 {
        let src_filename = format!("{}/test{:02}.schem", src_dir, id);
        let dst_filename = format!("{}/out{:02}.schem", out_dir, id);

        let schem = Schematic::from_world_edit_13_file(&src_filename, &WorldEdit13LoadOption::default()).unwrap().0;

        schem.save_world_edit_13_file(&dst_filename, &WorldEdit13SaveOption::default()).expect("Failed to save .schem file");

        Schematic::from_world_edit_13_file(&dst_filename, &WorldEdit13LoadOption::default()).expect("Failed to load saved .schem file");

        //println!("Metadata: \n{:?}", schem.metadata_litematica());
    }
}

#[test]
fn load_save_world_edit12() {
    use schem::WorldEdit12LoadOption;
    //let src_dir = "./test_files/schematic";
    let out_dir = "./target/test/load_save_world_edit12";
    create_dir_all(out_dir).unwrap();
    let _ = Schematic::from_world_edit_12_file("./test_files/schematic/full-blocks-1.12.2.schematic", &WorldEdit12LoadOption::default()).unwrap();
}

#[test]
fn make_test_litematic() {
    let mut commands = Vec::with_capacity(16 * 16 * 16);
    for id in 0..256 {
        let x = (id / 16) * 2;
        let z = (id % 16) * 2;
        let str_id = old_block::OLD_BLOCK_ID[id as usize];
        for damage in 0..16 {
            let y = damage * 2;
            commands.push(format!("execute @p ~ ~ ~ setblock ~{x} ~{y} ~{z} {str_id} {damage} replace"));
        }
    }
    assert_eq!(commands.len(), 16 * 16 * 16);
    let schem_shape = [64, 1, 64];

    let mut schem = Schematic::new();
    {
        let mut region = Region::new();
        region.reshape(&schem_shape);
        region.fill_with(&Block::air());
        region.name = "main".to_string();

        let blk_first = Block::from_id("command_block[conditional=false,facing=east]").unwrap();
        let blk_x_positive = Block::from_id("chain_command_block[conditional=false,facing=east]").unwrap();
        let blk_x_negative = Block::from_id("chain_command_block[conditional=false,facing=west]").unwrap();
        let blk_z_positive = Block::from_id("chain_command_block[conditional=false,facing=south]").unwrap();

        let mut command_block_nbt = HashMap::new();
        command_block_nbt.insert("conditionMet".to_string(), Value::Byte(0));
        command_block_nbt.insert("auto".to_string(), Value::Byte(1));//always active
        command_block_nbt.insert("CustomName".to_string(), Value::String("@".to_string()));
        command_block_nbt.insert("id".to_string(), Value::String("minecraft:command_block".to_string()));
        command_block_nbt.insert("SuccessCount".to_string(), Value::Int(0));
        command_block_nbt.insert("TrackOutput".to_string(), Value::Byte(1));
        command_block_nbt.insert("UpdateLastExecution".to_string(), Value::Byte(1));

        let mut counter = 0;
        for z in 0..64 {
            for x_offset in 0..64 {
                let is_first_block = (x_offset == 0) && (z == 0);
                let x = if z % 2 == 0 { x_offset } else { 63 - x_offset };
                let cur_blk: &Block =
                    if is_first_block {
                        &blk_first
                    } else if x_offset == 63 {
                        &blk_z_positive
                    } else if z % 2 == 0 {
                        &blk_x_positive
                    } else {
                        &blk_x_negative
                    };
                region.set_block([x, 0, z], cur_blk).expect("Failed to set block");
                command_block_nbt.insert("Command".to_string(), Value::String(commands[counter].to_string()));

                let mut be = BlockEntity::new();
                *(command_block_nbt.get_mut("auto").unwrap()) = Value::Byte(if is_first_block { 0 } else { 1 });
                be.tags = command_block_nbt.clone();
                region.set_block_entity_at([x, 0, z], be);
                counter += 1;
            }
        }
        schem.regions.push(region);
    }
    {
        let mut md = MetaDataIR::default();
        md.mc_data_version = DataVersion::Java_1_12_2 as i32;
        schem.metadata = md;
    }
    create_dir_all("./target/test/make_test_litematic").unwrap();
    schem.save_litematica_file("./target/test/make_test_litematic/out.litematic", &LitematicaSaveOption::default()).unwrap();
}

#[test]
fn correct_test_litematica() {
    let pos_block = [
        ([0, 0, 0], "white_concrete"),
        ([10, 0, 0], "light_gray_concrete"),
        ([0, 10, 0], "gray_concrete"),
        ([0, 0, 10], "black_concrete"),
        ([10, 10, 0], "brown_concrete"),
        ([10, 0, 10], "red_concrete"),
        ([0, 10, 10], "orange_concrete"),
        ([10, 10, 10], "yellow_concrete"),
    ];

    let schem = Schematic::from_litematica_file(
        "./test_files/litematica/correct_test.litematic",
        &LitematicaLoadOption::default()).unwrap().0;

    for x in 0..schem.shape()[0] {
        for y in 0..schem.shape()[1] {
            for z in 0..schem.shape()[2] {
                let blk = schem.first_block_at([x, y, z]).unwrap();
                if blk.is_air() {
                    continue;
                }
                println!("[{x}, {y}, {z}] => {}", blk.id);
            }
        }
    }

    for (pos, id) in pos_block {
        let parsed_id = schem.first_block_at(pos).unwrap();
        assert_eq!(parsed_id.id, id);
    }
}

#[test]
#[allow(unused_assignments)]
fn correct_test_mc13_plus() {
    let test_versions = ["1.14.4", "1.18.2", "1.19.4", "1.20.2", ];//,
    let mut err_counter = 0;
    for ver in test_versions {
        let litematica_file = format!("./test_files/litematica/full-blocks-{ver}.litematic");


        let schem_file = format!("./test_files/schem/full-blocks-{ver}.schem");
        let schem = Schematic::from_world_edit_13_file(&schem_file, &WorldEdit13LoadOption::default()).unwrap().0;

        let lite = Schematic::from_litematica_file(&litematica_file, &LitematicaLoadOption::default()).unwrap().0;
        assert_eq!(lite.shape(), schem.shape());
        for x in 0..lite.shape()[0] {
            for y in 0..lite.shape()[1] {
                for z in 0..lite.shape()[2] {
                    let pos = [x, y, z];
                    let blk_l = lite.first_block_at(pos).unwrap();
                    let blk_s = schem.first_block_at(pos).unwrap();
                    if blk_l != blk_s {
                        err_counter += 1;
                        let id_l: u16 = lite.first_block_index_at(pos).unwrap();
                        let id_s: u16 = schem.first_block_index_at(pos).unwrap();
                        panic!("In {ver}, block at [{x}, {y}, {z}] is different: \n litematica => {}, id= {id_l}\n schem => {}, id = {id_s}", blk_l, blk_s);
                    }
                }
            }
        }
    }
    println!("err_counter = {err_counter}");
    assert_eq!(err_counter, 0);
}


#[test]
#[allow(unused_assignments)]
fn correct_test_mc12() {
    let test_versions = ["1.12.2"];//,
    let mut err_counter = 0;

    let soft_check_ids = ["grass", "dirt", "bed", "piston_head",
        "fire", "oak_stairs", "wooden_door", "stone_stairs", "iron_door",
        "unpowered_repeater", "powered_repeater", "fence_gate", "double_plant",
        "spruce_fence_gate", "birch_fence_gate", "jungle_fence_gate",
        "dark_oak_fence_gate", "acacia_fence_gate", "spruce_door", "birch_door",
        "jungle_door", "acacia_door", "dark_oak_door"];

    for ver in test_versions {
        let litematica_file = format!("./test_files/litematica/full-blocks-{ver}.litematic");

        let schem;
        let schem_file = format!("./test_files/schematic/full-blocks-{ver}.schematic");
        schem = Schematic::from_world_edit_12_file(&schem_file, &WorldEdit12LoadOption::default()).unwrap().0;

        let lite = Schematic::from_litematica_file(&litematica_file, &LitematicaLoadOption::default()).unwrap().0;
        let mut ok_counter = 0;
        assert_eq!(lite.shape(), schem.shape());
        for x in 0..lite.shape()[0] {
            for y in 0..lite.shape()[1] {
                for z in 0..lite.shape()[2] {
                    let pos = [x, y, z];
                    let blk_l = lite.first_block_at(pos).unwrap();
                    let blk_s = schem.first_block_at(pos).unwrap();

                    if blk_l != blk_s {
                        if blk_l.is_inherited_from(blk_l) && soft_check_ids.contains(&blk_l.id.as_str()) {
                            continue;
                        }
                        err_counter += 1;
                        let id_l: u16 = lite.first_block_index_at(pos).unwrap();
                        let id_s: u16 = schem.first_block_index_at(pos).unwrap();
                        panic!("In {ver}, block at [{x}, {y}, {z}] is different: \n litematica => {}, id= {id_l}\n schem => {}, id = {id_s}", blk_l, blk_s);
                    }
                    ok_counter += 1;
                }
            }
        }
        println!("ok_counter = {ok_counter}");
    }
    println!("err_counter = {}", err_counter);
    assert_eq!(err_counter, 0);
}


#[test]
fn test_merge_regions() {
    let mut schem = Schematic::from_litematica_file("./test_files/litematica/multi-region01.litematic",
                                                    &LitematicaLoadOption::default()).unwrap().0;
    schem.merge_regions(&CommonBlock::Air.to_block());

    create_dir_all("./target/test/test_merge_regions").unwrap();
    let out_file = "./target/test/test_merge_regions/multi-region01-out.litematic";
    schem.save_litematica_file(out_file, &LitematicaSaveOption::default()).unwrap()
}

#[test]
fn test_3d_array_order() {
    let mut arr: ndarray::Array3<u16> = Array3::zeros([2, 3, 4]);

    {
        let mut i = 0;
        for y in 0..arr.shape()[0] {
            for z in 0..arr.shape()[1] {
                for x in 0..arr.shape()[2] {
                    arr[[y, z, x]] = i;
                    i += 1;
                }
            }
        }
    }

    for i in arr {
        print!("{}, ", i);
    }
}

// #[test]
// fn check_mca() {
//     let filename = "F:\\Users\\Joseph\\Documents\\Games\\Minecraft\\PCL2\\.minecraft\\versions\\1.20.2-Fabric 0.15.6\\saves\\New World\\region\\r.0.0.mca";
//
//     let file_len = std::fs::metadata(filename).unwrap().len();
//     let mut src = File::open(filename).unwrap();
//
//     let mut segments: Vec<[u8; 4096]> = vec![];
//     segments.reserve((file_len / 4096) as usize);
//
//     loop {
//         let mut seg = [0u8; 4096];
//         let len = src.read(&mut seg).unwrap();
//         if len == 0 {
//             break;
//         }
//         if len != 4096 {
//             panic!("Incomplete segment, expected 4096 bytes, but only {len} bytes");
//         }
//         segments.push(seg);
//     }
//
//     fn parse_segment(segments: &[[u8; 4096]]) -> Result<HashMap<String, Value>, fastnbt::error::Error> {
//         let mut bytes: &[u8];
//         unsafe {
//             bytes = &*slice_from_raw_parts(segments.as_ptr() as *const u8, segments.len() * 4096);
//         }
//
//         let data_bytes;
//         {
//             let mut len_be: [u8; 4] = [0; 4];
//             let temp = bytes.read(&mut len_be).unwrap();
//             debug_assert!(temp == 4);
//             data_bytes = u32::from_be_bytes(len_be);
//         }
//         let compress_byte: u8;
//         {
//             let mut b: [u8; 1] = [0];
//             let temp = bytes.read(&mut b).unwrap();
//             debug_assert!(temp == 1);
//             compress_byte = b[0];
//             assert!(compress_byte >= 1);
//             assert!(compress_byte <= 3);
//         }
//         {
//             assert_eq!(compress_byte, 2);//zlib
//             let decoder = ZlibDecoder::new(bytes);
//             fastnbt::from_reader(decoder)
//         }
//     }
//
//     let nbt = parse_segment(&segments[2..3]).unwrap();
//     for (key, _) in &nbt {
//         println!("\t{key}");
//     }
// }