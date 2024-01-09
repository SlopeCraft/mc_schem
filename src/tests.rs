
use crate::block::Block;
use crate::schem;
use std::{env, fs};
use std::any::Any;
use std::collections::HashMap;
use std::fs::create_dir_all;
use std::io::Read;
use fastnbt;
use fastnbt::Value;


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
fn load_save_vanilla_structure() {
    use crate::schem::VanillaStructureLoadOption;
    println!("Current dir: {}", env::current_dir().unwrap().to_string_lossy());

    let src_file;
    //let filename = "./test_files/vanilla_structure/test01.nbt";
    {
    let filename = "./test_files/vanilla_structure/test01.nbt";

    let file_opt = fs::File::open(filename);
    match file_opt {
        Ok(f) => src_file = f,
        Err(e) => panic!("Failed to open {} because {}", filename, e),
    }
    }


    let mut src = flate2::read::GzDecoder::new(src_file);

    // let nbt: Result<HashMap<String, Value>, fastnbt::error::Error> = fastnbt::from_reader(&mut src);
    // let nbt = nbt.unwrap();
    // let dv = nbt.get("DataVersion").unwrap();
    // if let fastnbt::Value::Int(i) = dv {
    //     println!("DataVersion = {} i32", i);
    // } else {
    //     panic!("Type of DataVersion mismatch, it is {:?}", dv);
    // }

    // let mut bytes = Vec::new();
    // let bytes = src.read_to_end(&mut bytes).unwrap();
    //
    // println!("Decompressed {} bytes", bytes);
    //
    // return;
    let parse_result = schem::Schematic::from_vanilla_structure(&mut src,
                                                                &VanillaStructureLoadOption::default());

    if let Err(err) = parse_result {
        panic!("Failed to parse vanilla structure, detail: {:?}", err);
    }

    let dst_file;
    {
        create_dir_all("./target/test/load_save_vanilla_structure").unwrap();

        let dst_filename = "./target/test/load_save_vanilla_structure/out.nbt";
        let dst_file_opt = fs::File::create(dst_filename);
        match dst_file_opt {
            Ok(f) => dst_file = f,
            Err(e) => panic!("Failed to create {} because {}", dst_filename, e),
        }
    }
    let mut dst = flate2::write::GzEncoder::new(dst_file, flate2::Compression::best());

    let write_error = parse_result.unwrap().save_vanilla_structure(&mut dst, &schem::VanillaStructureSaveOption::default());

    if let Err(err) = write_error {
        panic!("Failed to write vanilla structure, detail: {}", err);
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
        let result = schem::litematica::ceil_up_to(a, b);
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
fn litematica_multi_bit_set() {
    let data = [14242959524133701664u64, 1244691354];
    let mbs = schem::litematica::MultiBitSet::from_data(&data, 18, 5).unwrap();

    assert_eq!(mbs.basic_mask(), 0b11111);

    for idx in 0..mbs.len() {
        println!("mbs[{}] = {}", idx, mbs.get(idx));
    }
}