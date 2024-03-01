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
use std::fmt::Display;
use std::io::{Read};
use std::iter::Map;
use crate::error::Error;
use crate::Region;

pub mod mca;
mod files_reader;
mod chunk;


#[derive(Debug, Eq, Hash, PartialEq)]
pub struct XZCoordinate<T = i32> {
    pub x: T,
    pub z: T,
}

pub struct Chunk {
    pub time_stamp: u32,
    sub_chunks: Map<i32, Region>,
}

pub struct UnparsedChunkData {
    pub time_stamp: u32,
    region_data: Vec<u8>,//uncompressed
}

pub enum ChunkVariant {
    Parsed(Chunk),
    Unparsed(UnparsedChunkData),
}

pub struct Dimension {
    chunks: HashMap<XZCoordinate, ChunkVariant>,
}

pub struct FileInfo {
    pub name: String,
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

impl ChunkPos {
    pub fn from_global_pos(global_chunk_pos: &XZCoordinate) -> Self {
        // let local_coord = XZCoordinate {
        //     x: global_chunk_pos.x % 32,
        //     z: global_chunk_pos.z % 32,
        // };
        // debug_assert!((global_chunk_pos.x - local_coord.x) % 32 == 0);
        // debug_assert!((global_chunk_pos.z - local_coord.z) % 32 == 0);
        // return Self {
        //     file_coordinate: XZCoordinate {
        //         x: (global_chunk_pos.x - local_coord.x) / 32,
        //         z: (global_chunk_pos.z - local_coord.z) / 32,
        //     },
        //     coordinate_in_file: local_coord,
        // };

        return Self {
            global_x: global_chunk_pos.x,
            global_z: global_chunk_pos.z,
        };
    }

    pub fn to_global_pos(&self) -> XZCoordinate {
        return XZCoordinate {
            x: self.global_x,
            z: self.global_z,
        };
        // debug_assert!(self.coordinate_in_file.x >= 0 && self.coordinate_in_file.x < 32);
        // debug_assert!(self.coordinate_in_file.z >= 0 && self.coordinate_in_file.z < 32);
        //
        // return XZCoordinate {
        //     x: self.coordinate_in_file.x + self.file_coordinate.x * 32,
        //     z: self.coordinate_in_file.z + self.file_coordinate.z * 32,
        // };
    }

    pub fn from_local_pos(file_pos: &XZCoordinate, local_pos_in_file: &XZCoordinate<u32>) -> Self {
        assert!(local_pos_in_file.x < 32);
        assert!(local_pos_in_file.z < 32);

        return Self {
            global_x: file_pos.x * 32 + local_pos_in_file.x as i32,
            global_z: file_pos.z * 32 + local_pos_in_file.z as i32,
        };
    }

    pub fn local_coordinate(&self) -> XZCoordinate<u32> {
        return XZCoordinate {
            x: (self.global_x & 32) as u32,
            z: (self.global_x & 32) as u32,
        };
    }
    pub fn file_coordinate(&self) -> XZCoordinate {
        let local = self.local_coordinate();
        return XZCoordinate {
            x: (self.global_x - local.x as i32) / 32,
            z: (self.global_z - local.z as i32) / 32,
        };
    }

    pub fn filename(&self, suffix: &str) -> String {
        return format!("r.{}.{}.{}",
                       self.file_coordinate().x,
                       self.file_coordinate().z,
                       suffix);
    }

    pub fn filename_mca(&self) -> String {
        return self.filename("mca");
    }
    pub fn filename_mcr(&self) -> String {
        return self.filename("mcr");
    }
    pub fn filename_mcc(&self) -> String {
        return format!("c.{}.{}.mcc",
                       self.file_coordinate().x,
                       self.file_coordinate().z);
    }
}
