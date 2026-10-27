use fastnbt::Value;
use mc_schem::schem::{
    LitematicaSaveOption, MetaDataIR, VanillaStructureSaveOption, WorldEdit13SaveOption,
};
use mc_schem::{Block, CommonBlock, DataVersion, Entity, Region, Schematic};
use std::collections::HashMap;

fn main() {
    let mut region = Region::with_shape([1, 3, 1]);
    // Set up palette
    region.palette = vec![
        CommonBlock::Air.to_block(),
        Block::from_id("minecraft:cauldron").unwrap(),
        Block::from_id("minecraft:glass").unwrap(),
    ];

    // construct the cage
    region.set_block_id([0, 0, 0], 1).unwrap();
    region.set_block_id([0, 1, 0], 2).unwrap();
    // This also works, but slow
    region
        .set_block(
            [0, 2, 0],
            &Block::from_id("minecraft:white_carpet").unwrap(),
        )
        .unwrap();

    // Create a surprising entity
    {
        let creeper;
        let mut tags: HashMap<String, Value> = HashMap::new();
        tags.insert(
            "id".to_string(),
            Value::String("minecraft:creeper".to_string()),
        );
        tags.insert(
            "Pos".to_string(),
            Value::List(vec![
                Value::Double(0.5),
                Value::Double(0.25),
                Value::Double(0.5),
            ]),
        );
        // Compulsory tag. Otherwise, litematica will crash once pasting
        tags.insert(
            "Rotation".to_string(),
            Value::List(vec![Value::Float(0.), Value::Float(0.)]),
        );
        tags.insert("ignited".to_string(), Value::Byte(1));
        tags.insert("powered".to_string(), Value::Byte(1));
        creeper = Entity::from_nbt(tags, "").unwrap();
        region.entities.push(creeper);
    }

    // Create schematic and set up metadata
    let mut schem = Schematic::new();
    schem.metadata = MetaDataIR::from_data_version(DataVersion::Java_26_3).unwrap();
    // Add region to schematic
    schem.regions.push(region);

    // Save schematic
    schem
        .save_litematica_file("creeper.litematic", &LitematicaSaveOption::default())
        .unwrap();

    schem
        .save_world_edit_13_file("creeper.schem", &WorldEdit13SaveOption::default())
        .unwrap();
    schem
        .save_vanilla_structure_file("creeper.nbt", &VanillaStructureSaveOption::default())
        .unwrap();
}
