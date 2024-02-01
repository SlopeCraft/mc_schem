use std::collections::HashMap;
use std::env;
use std::fs::{create_dir_all, File};
use fastnbt::Value;
use flate2::read::GzDecoder;
use rand::Rng;
use crate::schem;
use crate::schem::{DataVersion, LitematicaLoadOption, LitematicaSaveOption, MetaDataIR, Schematic, WorldEdit13LoadOption, WorldEdit13SaveOption};
use crate::old_block;
use crate::region::{BlockEntity, Region};
use crate::block::Block;

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

    let schem = Schematic::from_litematica_file(src_filename, &LitematicaLoadOption::default()).unwrap();

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
    let litematic = Schematic::from_litematica_file("./test_files/litematica/full-blocks-1.12.2.litematic", &LitematicaLoadOption::default()).unwrap();
    let lite_region = &litematic.regions[0];
    for dim in 0..3 {
        assert_eq!(num_id_array.shape()[dim], lite_region.shape()[dim] as usize);
    }

    let mut hash: HashMap<(u8, u8), String> = HashMap::new();
    hash.reserve(256 * 16);

    for x in 0..num_id_array.shape()[0] {
        for y in 0..num_id_array.shape()[1] {
            for z in 0..num_id_array.shape()[2] {
                let (id, damage) = num_id_array[[x, y, z]];
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

    // for (id, damage) in num_id_array {
    //     Block::from_old(id, damage, DataVersion::Java_1_12_2).unwrap();
    // }
}

#[test]
fn load_save_vanilla_structure() {
    use schem::{VanillaStructureLoadOption, VanillaStructureSaveOption};
    let schem =
        Schematic::from_vanilla_structure_file(
            "./test_files/vanilla_structure/test01.nbt",
            &VanillaStructureLoadOption::default()).unwrap();

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

        let schem = Schematic::from_litematica_file(&src_filename, &LitematicaLoadOption::default()).unwrap();

        schem.save_litematica_file(&dst_filename, &LitematicaSaveOption::default()).expect("Failed to save litematica file");

        Schematic::from_litematica_file(&dst_filename, &LitematicaLoadOption::default()).expect("Failed to load saved litematica file");


        //println!("Metadata: \n{:?}", schem.metadata_litematica());
    }

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

        let schem = Schematic::from_world_edit_13_file(&src_filename, &WorldEdit13LoadOption::default()).unwrap();

        schem.save_world_edit_13_file(&dst_filename, &WorldEdit13SaveOption::default()).expect("Failed to save .schem file");

        Schematic::from_world_edit_13_file(&dst_filename, &WorldEdit13LoadOption::default()).expect("Failed to load saved .schem file");

        //println!("Metadata: \n{:?}", schem.metadata_litematica());
    }
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

    let schem = Schematic::from_litematica_file("./test_files/litematica/correct_test.litematic", &LitematicaLoadOption::default()).unwrap();

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
fn correct_test_mc13_and_above() {
    let test_versions = ["1.20.2", "1.14.4", "1.18.2", "1.19.4", ];//,
    let mut err_counter = 0;
    for ver in test_versions {
        let litematica_file = format!("./test_files/litematica/full-blocks-{ver}.litematic");
        let schem_file = format!("./test_files/schem/full-blocks-{ver}.schem");
        let lite = Schematic::from_litematica_file(&litematica_file, &LitematicaLoadOption::default()).unwrap();
        let schem = Schematic::from_world_edit_13_file(&schem_file, &WorldEdit13LoadOption::default()).unwrap();
        let mut ok_counter = 0;
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
                    ok_counter += 1;
                }
            }
        }
    }
    assert_eq!(err_counter, 0);
}