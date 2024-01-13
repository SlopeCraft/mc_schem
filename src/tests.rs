use std::env;
use std::fs::create_dir_all;
use rand::Rng;
use crate::block::Block;
use crate::schem;
use crate::schem::{LitematicaSaveOption, Schematic};


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
        &VanillaStructureSaveOption::default()).unwrap();
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

        schem.save_litematica_file(&dst_filename, &LitematicaSaveOption::default()).unwrap();

        //println!("Metadata: \n{:?}", schem.metadata_litematica());
    }

}