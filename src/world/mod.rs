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

use std::collections::HashMap;
use std::io::{Read};
use crate::error::Error;
use crate::Region;

pub mod mca;
mod files_reader;
mod chunk;
mod dimension;


#[derive(Debug, Eq, Hash, PartialEq)]
pub struct XZCoordinate<T = i32> {
    pub x: T,
    pub z: T,
}

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
    sub_chunks: [Region; 24],
    pub source_file: String,

}

pub struct UnparsedChunkData {
    pub time_stamp: u32,
    region_data: Vec<u8>,//uncompressed
    source_file: String,
}

pub enum ChunkVariant {
    Parsed(Chunk),
    Unparsed(UnparsedChunkData),
}

pub struct Dimension {
    pub chunks: HashMap<ChunkPos, ChunkVariant>,
}

pub struct FileInfo {
    pub name: String,
    pub full_name: String,
    pub size: u64,
}

pub struct SubDirectory<'a> {
    root: &'a dyn FilesRead,
    dirname_with_slash: String,
}

pub trait FilesRead {
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

    fn read_file_nocopy(&self, filename: &str) -> Result<Option<&[u8]>, Error> {
        let _ = self.open_file(filename)?;
        return Ok(None);
    }

    fn sub_directory(&self, dir: &str) -> SubDirectory
        where Self: Sized {
        let mut dir = dir.replace('\\', "/");
        if !dir.ends_with('/') {
            dir.push('/');
        }
        return SubDirectory {
            root: self,
            dirname_with_slash: dir,
        };
    }
}

pub struct FolderOnDisk {
    path: String,
}

pub struct FilesInMemory {
    files: HashMap<String, Vec<u8>>,
    /// The source of 7z archive, including but not limited to filename
    pub source: String,
}

#[derive(Debug, Eq, Hash, PartialEq)]
pub struct ChunkPos {
    global_x: i32,
    global_z: i32,
    // pub file_coordinate: XZCoordinate,
    // pub coordinate_in_file: XZCoordinate,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
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

