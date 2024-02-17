use strum::Display;

pub mod block;
pub mod schem;
pub mod error;
pub mod region;
pub mod old_block;

mod c_ffi;
#[cfg(test)]
mod tests;

pub type Block = block::Block;
pub type Entity = region::Entity;
pub type BlockEntity = region::BlockEntity;
pub type PendingTick = region::PendingTick;
pub type Region = region::Region;
pub type Schematic = schem::Schematic;
pub type MetaDataIR = schem::MetaDataIR;
pub type LitematicaLoadOption = schem::LitematicaLoadOption;
pub type LitematicaSaveOption = schem::LitematicaSaveOption;
pub type VanillaStructureLoadOption = schem::VanillaStructureLoadOption;
pub type VanillaStructureSaveOption = schem::VanillaStructureSaveOption;

#[repr(u8)]
#[derive(Debug, Display, Clone, PartialEq)]
pub enum SchemFormat {
    Litematica = 0,
    VanillaStructure = 1,
    WorldEdit13 = 2,
    WorldEdit12 = 3,
}

impl SchemFormat {
    pub fn extension(&self) -> &'static str {
        return match self {
            SchemFormat::Litematica => ".litematic",
            SchemFormat::VanillaStructure => ".nbt",
            SchemFormat::WorldEdit13 => ".schem",
            SchemFormat::WorldEdit12 => ".schematic",
        }
    }

    pub fn supported_formats() -> &'static [SchemFormat] {
        return Self::loadable_formats();
    }
    pub fn loadable_formats() -> &'static [SchemFormat] {
        use SchemFormat::*;
        return &[Litematica, VanillaStructure, WorldEdit13, WorldEdit12];
    }
    pub fn savable_formats() -> &'static [SchemFormat] {
        use SchemFormat::*;
        return &[Litematica, VanillaStructure, WorldEdit13];
    }

    pub fn loadable(&self) -> bool {
        return Self::loadable_formats().contains(self);
    }

    pub fn savable(&self) -> bool {
        return Self::savable_formats().contains(self);
    }
}
