
use crate::block::Block;
use crate::schem;
use std::{env, fs};
use std::io::Read;


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
fn load_vanilla_structure() {
    println!("Current dir: {}", env::current_dir().unwrap().to_string_lossy());

    //let filename = "./test_files/vanilla_structure/test01.nbt";
    let filename = "./test_files/vanilla_structure/test01.nbt";

    let file_opt = fs::File::open(filename);
    let file;
    match file_opt {
        Ok(f) => file = f,
        Err(e) => panic!("Failed to open {} because {}", filename, e),
    }

    let mut src = flate2::read::GzDecoder::new(file);
    // let mut bytes = Vec::new();
    // let bytes = src.read_to_end(&mut bytes).unwrap();
    //
    // println!("Decompressed {} bytes", bytes);
    //
    // return;
    let parse_result = schem::Schematic::from_vanilla_structure(&mut src);

    if let Err(err) = parse_result {
        panic!("Failed to parse vanilla structure, detail: {:?}", err);
    }
}