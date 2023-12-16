
use crate::block::Block;
use std::process::abort;

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
