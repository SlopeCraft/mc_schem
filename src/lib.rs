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

use strum::Display;

pub mod block;
pub mod schem;
pub mod error;
pub mod region;
pub mod old_block;

mod c_ffi;

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
pub type WorldEdit13LoadOption = schem::WorldEdit13LoadOption;
pub type WorldEdit13SaveOption = schem::WorldEdit13SaveOption;
pub type WorldEdit12LoadOption = schem::WorldEdit12LoadOption;
pub type DataVersion = schem::DataVersion;

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
