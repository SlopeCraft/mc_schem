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

use std::collections::{BTreeMap, HashMap};
use std::io::Read;
use std::ops::Range;
use std::sync::Arc;
use fastnbt::Value;
use crate::biome::Biome;
use crate::block::Block;
use crate::{BlockEntity, Entity};
use crate::error::Error;
use crate::region::Light;

pub mod mca;
mod files_reader;
mod chunk;
mod dimension;
mod sub_chunk;


#[derive(Debug, Eq, Hash, PartialEq)]
pub struct XZCoordinate<T = i32> {
    pub x: T,
    pub z: T,
}

#[derive(Debug, Clone)]
pub struct SubChunk {
    /// All kinds of blocks
    pub palette: Vec<Block>,
    block_id_array: [u16; 4096],
    // pub region: Region,
    /// Skylight, yzx
    pub sky_block_light_array: [Light; 4096],

    /// Biomes, zx
    pub biome_array: [Biome; 64],
}

#[derive(Debug, Clone)]
pub struct Chunk {
    pub time_stamp: u32,
    /// Status of chunk
    pub status: ChunkStatus,
    /// Last update in game time
    pub last_update: i64,
    /// Sum of time(in game tick) that player stay in this chunk, used to compute difficulty
    pub inhabited_time: i64,
    /// If light compute is finished
    pub is_light_on: bool,
    sub_chunks: BTreeMap<i8, SubChunk>,
    pub region_source_file: String,
    pub entities: Vec<Entity>,
    pub block_entities: HashMap<[i32; 3], BlockEntity>,
}

pub enum RefOrObject<'a, T: Sized> {
    Ref(&'a T),
    Object(T),
}

#[derive(Debug, Clone)]
pub struct ArcSlice {
    data_owner: Arc<Vec<u8>>,
    range: Range<usize>,
}

pub struct NBTWithSource<'a> {
    pub nbt: HashMap<String, Value>,
    pub source: &'a str,
}

#[derive(Debug, Clone)]
pub struct MCARawData {
    pub time_stamp: u32,
    pub compress_method: u8,
    pub data: ArcSlice,
    //uncompressed
    pub source_file: String,
}

#[derive(Debug, Clone)]
pub struct UnparsedChunkData {
    pub region_data: MCARawData,
    pub entity_data: Option<MCARawData>,
}

#[derive(Debug, Clone)]
pub enum ChunkVariant {
    Parsed(Chunk),
    Unparsed(UnparsedChunkData),
}

#[derive(Debug, Clone)]
pub struct Dimension {
    pub chunks: HashMap<ChunkPos, ChunkVariant>,
}

#[derive(Debug, Clone)]
pub struct World {
    pub dimensions: BTreeMap<i32, Dimension>,
}

#[derive(Debug, Clone)]
pub struct FileInfo {
    pub name: String,
    pub full_name: String,
    pub size: u64,
}

#[derive(Clone)]
pub struct SubDirectory<'a> {
    root: &'a dyn FilesRead,
    dirname_with_slash: String,
}

pub trait FilesRead {
    fn sub_directory(&self, dir: &str) -> SubDirectory;

    fn path(&self) -> String;
    fn files(&self) -> Vec<FileInfo>;

    fn open_file(&self, filename: &str) -> Result<Box<dyn Read + '_>, Error>;

    fn read_file(&self, filename: &str, dest: &mut Vec<u8>) -> Result<(), Error> {
        let mut src = self.open_file(filename)?;
        dest.clear();
        return match src.read_to_end(dest) {
            Ok(_) => { Ok(()) }
            Err(e) => { Err(Error::IOReadError(e)) }
        };
    }

    fn read_file_as_bytes(&self, filename: &str) -> Result<Vec<u8>, Error> {
        let mut result = Vec::new();
        self.read_file(filename, &mut result)?;
        return Ok(result);
    }

    fn read_file_nocopy(&self, filename: &str) -> Result<Option<ArcSlice>, Error> {
        let _ = self.open_file(filename)?;
        return Ok(None);
    }
    fn read_file_as_arc_slice(&self, filename: &str) -> Result<ArcSlice, Error> {
        if let Some(arc_slice) = self.read_file_nocopy(filename)? {
            return Ok(arc_slice);
        }
        let vec = self.read_file_as_bytes(filename)?;
        return Ok(ArcSlice::from(Arc::new(vec)));
    }
}

#[derive(Debug, Clone)]
pub struct FolderOnDisk {
    path: String,
}

#[derive(Debug, Clone)]
pub struct FilesInMemory {
    files: HashMap<String, Arc<Vec<u8>>>,
    /// The source of 7z archive, including but not limited to filename
    pub source: String,
}

#[derive(Debug, Eq, Hash, PartialEq, Copy, Clone)]
pub struct ChunkPos {
    global_x: i32,
    global_z: i32,
    // pub file_coordinate: XZCoordinate,
    // pub coordinate_in_file: XZCoordinate,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum ChunkStatus {
    Empty,
    StructureStarts,
    StructureReferences,
    Biomes,
    Noise,
    Surface,
    Carvers,
    Features,
    InitializeLight,
    Light,
    Spawn,
    Full,
}

