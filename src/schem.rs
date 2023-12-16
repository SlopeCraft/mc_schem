use std::collections::HashMap;
use ndarray::Array3;
use crate::block::Block;
use nbt;

pub struct BlockEntity {
    pub tags: nbt::Blob,
}

pub struct Entity {
    pub tags: nbt::Blob,
}
pub struct Region {
    pub name: String,
    array: Array3<u16>,//XYZ
    pub palette: Vec<Block>,
    block_entities: HashMap<[i64;3],BlockEntity>,
    pub entities: Vec<Entity>,

    pub offset: [i64;3],
}

impl Region {
    fn shape(&self)->[i64;3] {
        let shape=self.array.shape();
        if shape.len()!=3 {
            panic!("Invalid array dimensions: shoule be 3 but now it is {}",shape.len());
        }
        return [shape[0] as i64,shape[1] as i64,shape[2] as i64];
    }
    fn contains_coord(&self,coord:[i64;3]) -> bool {
        for dim in 0..3 {
            if coord[dim]>=0 && coord[dim] <=self.shape()[dim] {
                continue;
            }
            return false;

        }
        return true;
    }
    pub fn block_at(&self,coord:[i64;3]) -> Option<&Block> {
        if !self.contains_coord(coord) {
            return None;
        }

        let x=coord[0] as usize;
        let y=coord[1] as usize;
        let z=coord[2] as usize;

        let pid=self.array[[x,y,z]] as usize;
        return Some(&self.palette[pid]);
    }
}

pub struct LitematicaMetaData {
    pub version:i32,

    pub time_created:i64,
    pub time_modified:i64,
    pub author:String,
    pub name:String,
    pub description:String,
    pub total_volume:i64,
}

pub struct WE12MetaData {

}
pub struct WE13MetaData {
    pub version: i32,
    pub we_offset: [i32; 3],
    pub offset: [i32; 3],
}

pub struct VanillaStructureMetaData {

}

pub enum MetaData {
    Litematica(LitematicaMetaData),
    WE12(WE12MetaData),
    WE13(WE13MetaData),
    VanillaStructure(VanillaStructureMetaData),
}
pub struct Schematic {
    pub data_version:i32,

    pub metadata:MetaData,

    pub regions: Vec<Region>,
    pub enclosing_size: [i64;3],
}