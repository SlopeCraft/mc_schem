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

//! A rust library to generate, load, manipulate and save minecraft schematic files.
//! ## Supported formats:
//! - Litematica(`.litematica`)
//! - Vanilla structure(`.nbt`)
//! - WorldEdit schem (1.13+)(`.schem`)
//! - WorldEdit schem (1.12-)(`.schematic`)
//!
//! ## Contents
//! 1. `mc_schem` (rlib)
//!
//!    The main rust lib
//! 2. `mc_schem` (cdylib)
//!
//!    C ffi for mc_schem
//! 3. mc_schem C++ wrapper
//!
//!    A header-only c++ wrapper based on C ffi of mc_schem
//! 4. `schemtool` (executable)
//!
//!    An executable to do various manipulations on schematics
//!


use strum::Display;

/// Implement minecraft block and string id parsing
pub mod block;
/// Number id parsing
pub mod old_block;
/// Errors in loading, saving and manipulating
pub mod error;
/// Implement region, entity, block entity and pending ticks
pub mod region;
/// Implement metadata, schematics and loading/saving
pub mod schem;

pub mod world;

mod c_ffi;

/// `Block` is a type of block with namespace and properties(aka attributes) in MC.
pub type Block = block::Block;
/// Enumerate common blocks
pub type CommonBlock = block::CommonBlock;
/// An entity in MC, like zombie, minecart, etc.
pub type Entity = region::Entity;
/// Block entity(also known as tile entity) in MC, like chest, furnace, etc.
pub type BlockEntity = region::BlockEntity;
/// A tick waiting to be processed
pub type PendingTick = region::PendingTick;
/// Region is a 3d area in Minecraft, containing blocks and entities.
//pub trait WorldSlice = region::WorldSlice;
pub type Region = region::Region;
/// Schematic is part of a Minecraft world, like `.litematic` of litematica mod, `.schem` and
/// `.schematic` of world edit, `.nbt` of vanilla structure.
pub type Schematic = schem::Schematic;
/// A 3d slice of schematic
pub type SchemSlice<'a> = schem::schem_slice::SchemSlice<'a>;
/// Intermediate representation via different metadata formats
pub type MetaDataIR = schem::MetaDataIR;
/// Options to load litematica
pub type LitematicaLoadOption = schem::LitematicaLoadOption;
/// Options to save litematica
pub type LitematicaSaveOption = schem::LitematicaSaveOption;
/// Options to load vanilla structure
pub type VanillaStructureLoadOption = schem::VanillaStructureLoadOption;
/// Options to save vanilla structure
pub type VanillaStructureSaveOption = schem::VanillaStructureSaveOption;
/// Options to load litematica
pub type WorldEdit13LoadOption = schem::WorldEdit13LoadOption;
/// Options to save world edit 1.13+
pub type WorldEdit13SaveOption = schem::WorldEdit13SaveOption;
/// Options to load litematica
pub type WorldEdit12LoadOption = schem::WorldEdit12LoadOption;
/// Minecraft data versions.
pub type DataVersion = schem::DataVersion;
/// Errors when loading and saving schematic
pub type Error = error::Error;

/// Format of known schematics
#[repr(u8)]
#[derive(Debug, Display, Clone, PartialEq)]
pub enum SchemFormat {
    Litematica = 0,
    VanillaStructure = 1,
    WorldEdit13 = 2,
    WorldEdit12 = 3,
}

impl SchemFormat {
    /// Filename extension with `.`
    pub fn extension(&self) -> &'static str {
        return match self {
            SchemFormat::Litematica => ".litematic",
            SchemFormat::VanillaStructure => ".nbt",
            SchemFormat::WorldEdit13 => ".schem",
            SchemFormat::WorldEdit12 => ".schematic",
        }
    }

    /// Return all supported formats
    pub fn supported_formats() -> &'static [SchemFormat] {
        return Self::loadable_formats();
    }
    /// Return all loadable formats
    pub fn loadable_formats() -> &'static [SchemFormat] {
        use SchemFormat::*;
        return &[Litematica, VanillaStructure, WorldEdit13, WorldEdit12];
    }
    /// Return all savable formats
    pub fn savable_formats() -> &'static [SchemFormat] {
        use SchemFormat::*;
        return &[Litematica, VanillaStructure, WorldEdit13];
    }
    /// Return if the format can be loaded
    pub fn loadable(&self) -> bool {
        return Self::loadable_formats().contains(self);
    }
    /// Return if the format can be saved
    pub fn savable(&self) -> bool {
        return Self::savable_formats().contains(self);
    }
}
