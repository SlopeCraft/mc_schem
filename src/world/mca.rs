use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::sync::Arc;
use fastnbt::Value;
use flate2::read::{GzDecoder, ZlibDecoder};
use regex::Regex;
use world::{XZCoordinate, ChunkPos};
use crate::error::Error;
use crate::world;
use crate::world::{ArcSlice, Chunk, ChunkVariant, Dimension, FileInfo, MCARawData, NBTWithSource, UnparsedChunkData};
use world::FilesRead;

pub const SEGMENT_BYTES: usize = 4096;

impl Display for XZCoordinate {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {})", self.x, self.z)
    }
}

pub fn parse_mca_filename(filename: &str) -> Option<XZCoordinate> {
    let reg = Regex::new(r"^r.(-*\d+).(-*\d+).mca$").unwrap();
    let cap = reg.captures(filename)?;
    debug_assert!(cap.iter().len() == 3);

    let mut iter = cap.iter();
    iter.next();
    let x = i32::from_str_radix(iter.next().unwrap().unwrap().as_str(), 10).ok()?;
    let z = i32::from_str_radix(iter.next().unwrap().unwrap().as_str(), 10).ok()?;

    return Some(XZCoordinate { x, z });
}

pub fn parse_mcc_filename(filename: &str) -> Option<ChunkPos> {
    let reg = Regex::new(r"^c.(-*\d+).(-*\d+).mcc$").unwrap();
    let cap = reg.captures(filename)?;
    debug_assert!(cap.iter().len() == 3);

    let mut iter = cap.iter();
    iter.next();
    let x = i32::from_str_radix(iter.next().unwrap().unwrap().as_str(), 10).ok()?;
    let z = i32::from_str_radix(iter.next().unwrap().unwrap().as_str(), 10).ok()?;
    return Some(ChunkPos::from_global_pos(&XZCoordinate { x, z }));
}


#[test]
fn test_parse_mca_filename() {
    let names = ["r.0.1.mca", "r.-1.9.mca", "r.1.-1.mca", "r.-1.-1.mca", ];
    for name in names {
        let pos = parse_mca_filename(name).unwrap();
        let filename = format!("r.{}.{}.mca", pos.x, pos.z);
        assert_eq!(filename, name);
    }
}


impl ChunkPos {
    pub fn from_global_pos(global_chunk_pos: &XZCoordinate) -> Self {
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
            x: (self.global_x & 31) as u32,
            z: (self.global_z & 31) as u32,
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
                       self.global_x,
                       self.global_z);
    }

    pub fn block_pos_lower_bound(&self) -> [i32; 2] {
        return [self.global_x * 16, self.global_z * 16];
    }
    pub fn block_pos_upper_bound(&self) -> [i32; 2] {
        return [self.global_x * 16 + 16, self.global_z * 16 + 16];
    }
}

impl ChunkVariant {
    pub fn check(&self, chunk_pos: &ChunkPos) -> Result<(), Error> {
        if let ChunkVariant::Unparsed(raw) = self {
            raw.parse(chunk_pos)?;
        }
        return Ok(());
    }
    pub fn parse_inplace(&mut self, chunk_pos: &ChunkPos) -> Result<(), Error> {
        if let ChunkVariant::Unparsed(raw) = self {
            *self = ChunkVariant::Parsed(raw.parse(chunk_pos)?);
        }

        return Ok(());
    }
}

impl MCARawData {
    pub fn to_nbt(&self) -> Result<NBTWithSource, Error> {
        let parse_opt: Result<HashMap<String, Value>, fastnbt::error::Error>;

        if self.data[0..2] == [0x78, 0x9c] {//zlib
            // Some mcc files are stored as zlib, but the compress method is 130.
            // This is to fix minecraft's error(at least in 1.20.2
            let src = ZlibDecoder::new(self.data.as_slice());
            parse_opt = fastnbt::from_reader(src);
        } else {
            match self.compress_method {
                1 | 128 => {//gzip
                    let src = GzDecoder::new(self.data.as_slice());
                    parse_opt = fastnbt::from_reader(src);
                }
                2 | 129 => {//zlib
                    let src = ZlibDecoder::new(self.data.as_slice());
                    parse_opt = fastnbt::from_reader(src);
                }
                3 | 130 => {// no compress
                    parse_opt = fastnbt::from_reader(self.data.as_slice());
                }
                _ => { return Err(Error::InvalidMCACompressType { compress_label: self.compress_method }); }
            }
        }

        return match parse_opt {
            Ok(nbt) => Ok(NBTWithSource {
                nbt,
                source: &self.source_file,
            }),
            Err(e) => Err(Error::NBTReadError(e)),
        };
    }
}

impl UnparsedChunkData {
    pub fn to_nbt(&self) -> Result<(NBTWithSource, Option<NBTWithSource>), Error> {
        let region_data = self.region_data.to_nbt()?;
        let entity_data = if let Some(raw) = &self.entity_data {
            Some(raw.to_nbt()?)
        } else {
            None
        };
        return Ok((region_data, entity_data));
    }

    pub fn parse(&self, chunk_pos: &ChunkPos) -> Result<Chunk, Error> {
        let (region_nbt, entity_nbt) = self.to_nbt()?;

        let chunk = Chunk::from_nbt(region_nbt,
                                    entity_nbt,
                                    &chunk_pos, )?;
        return Ok(chunk);
    }
}


fn get_compress_label(mca_data: &[u8]) -> (u8, usize) {
    let data_bytes: usize;
    {
        let db = u32::from_be_bytes([mca_data[0], mca_data[1], mca_data[2], mca_data[3]]);
        data_bytes = db as usize;
    }
    let compress_type: u8 = mca_data[4];
    return (compress_type, data_bytes);
}

pub fn parse_multiple_mca_files(dir: &dyn FilesRead) -> Result<HashMap<ChunkPos, MCARawData>, Error> {
    let files = dir.files();
    let mut mca_files = Vec::with_capacity(files.len());
    for info in files {
        if let Some(coord) = parse_mca_filename(&info.name) {
            mca_files.push((info, coord));
        }
    }

    let mut result = HashMap::new();

    for (info, file_coord) in &mca_files {
        let mut chunk_region_data = parse_mca_file(info, file_coord, dir)?;
        for (chunk_pos, raw) in chunk_region_data {
            result.insert(chunk_pos, raw);
        }
    }

    return Ok(result);
}

// pub fn parse_multiple_regions(region_dir: &dyn FilesRead, parse_directly: bool) -> Result<HashMap<ChunkPos, ChunkVariant>, Error> {
//     let region_data = parse_multiple_mca_files(region_dir)?;
//     let mut result = HashMap::with_capacity(region_data.len());
//     for (chunk_pos, raw) in region_data {
//         let mut variant = ChunkVariant::Unparsed(UnparsedChunkData {
//             region_data: raw,
//             entity_data: None,
//         });
//         if parse_directly {
//             variant.parse_inplace(&chunk_pos)?;
//         }
//         result.insert(chunk_pos, variant);
//     }
//
//     return Ok(result);
// }

pub fn parse_multiple_regions(region_dir: &dyn FilesRead,
                              entity_dir: Option<&dyn FilesRead>,
                              parse_directly: bool)
                              -> Result<HashMap<ChunkPos, ChunkVariant>, Error> {
    let region_data = parse_multiple_mca_files(region_dir)?;
    let mut entity_data = if let Some(entity_dir) = entity_dir {
        parse_multiple_mca_files(entity_dir)?
    } else {
        HashMap::new()
    };
    let mut result = HashMap::with_capacity(region_data.len());

    for (chunk_pos, r_data) in region_data {
        let e_data = entity_data.remove(&chunk_pos);
        let unparsed = UnparsedChunkData {
            region_data: r_data,
            entity_data: e_data,
        };
        result.insert(chunk_pos, ChunkVariant::Unparsed(unparsed));
    }

    if parse_directly {
        let mut temp = Dimension { chunks: result };
        temp.parse_all()?;
        return Ok(temp.chunks);
    }

    return Ok(result);
}

fn parse_mca_file(file_info: &FileInfo, file_coord: &XZCoordinate,
                  region_dir: &dyn FilesRead) -> Result<HashMap<ChunkPos, MCARawData>, Error> {
    let mca_bytes: ArcSlice = if let Some(slice) = region_dir.read_file_nocopy(&file_info.name)? {
        slice
    } else {
        let bytes = region_dir.read_file_as_bytes(&file_info.name)?;
        ArcSlice::from(Arc::new(bytes))
    };

    let mut result = HashMap::new();
    if mca_bytes.is_empty() {
        return Ok(result);
    }
    if mca_bytes.len() % 4096 != 0 {
        return Err(Error::IncompleteSegmentInMCA { bytes: mca_bytes.len() });
    }

    for z in 0..32 {
        for x in 0..32 {
            let local_pos = XZCoordinate { x, z };
            let pos = ChunkPos::from_local_pos(file_coord, &local_pos);
            let unparsed = parse_mca_single_chunk(&pos, mca_bytes.clone(), region_dir)?;
            if let Some(raw) = unparsed {
                result.insert(pos, raw);
            }
        }
    }
    return Ok(result);
}

pub fn offset_in_mca_file(local_coord: &XZCoordinate<u32>) -> u32 {
    return 4 * ((local_coord.x & 31) + (local_coord.z & 31) * 32);
}

fn parse_mca_single_chunk(chunk_pos: &ChunkPos, mca_bytes: ArcSlice, region_dir: &dyn FilesRead) -> Result<Option<MCARawData>, Error> {
    let header: [u8; 4];
    let local_coord = chunk_pos.local_coordinate();
    {
        let offset_by_byte = offset_in_mca_file(&local_coord) as usize;
        debug_assert!((offset_by_byte + 3) < 4096);
        header = [mca_bytes[offset_by_byte + 0],
            mca_bytes[offset_by_byte + 1],
            mca_bytes[offset_by_byte + 2],
            mca_bytes[offset_by_byte + 3]];
    }
    if header == [0; 4] {
        // no such chunk
        return Ok(None);
    }

    let time_stamp: u32;
    {
        let offset_by_byte = offset_in_mca_file(&local_coord) as usize + SEGMENT_BYTES;
        debug_assert!((offset_by_byte + 3) < 8192);
        time_stamp = u32::from_be_bytes([
            mca_bytes[offset_by_byte + 0],
            mca_bytes[offset_by_byte + 1],
            mca_bytes[offset_by_byte + 2],
            mca_bytes[offset_by_byte + 3],
        ]);
    }


    let offset_by_segment = u32::from_be_bytes([0, header[0], header[1], header[2]]);
    let num_segments = header[3] as u32;
    if offset_by_segment < 2 || (num_segments + offset_by_segment) as usize > (mca_bytes.len() / SEGMENT_BYTES) {
        return Err(Error::InvalidSegmentRangeInMCA {
            chunk_local_x: local_coord.x as i32,
            chunk_local_z: local_coord.z as i32,
            offset_by_segment,
            num_segments,
            total_segments: mca_bytes.len() / SEGMENT_BYTES,
        });
    }

    let data_beg_idx = offset_by_segment as usize * SEGMENT_BYTES;
    let data_end_idx = (offset_by_segment + num_segments) as usize * SEGMENT_BYTES;
    //let range = ;
    let (compress_label, compressed_len) = get_compress_label(&mca_bytes[data_beg_idx..data_end_idx]);

    if ![1, 128, 2, 129, 3, 130].contains(&compress_label) {
        return Err(Error::InvalidMCACompressType { compress_label });
    }

    if compress_label > 127 {
        let mcc_filename = chunk_pos.filename_mcc();
        let mcc_bytes = match region_dir.read_file_as_arc_slice(&mcc_filename) {
            Err(e) => return Err(Error::MissingMCCFile {
                filename: mcc_filename,
                detail: Box::new(e),
            }),
            Ok(bytes) => bytes,
        };

        return Ok(Some(MCARawData {
            time_stamp,
            compress_method: compress_label,
            data: mcc_bytes,
            source_file: format!("{}/{}", region_dir.path(), mcc_filename),
        }));
    }

    return Ok(Some(MCARawData {
        time_stamp,
        compress_method: compress_label,
        data: mca_bytes.slice((data_beg_idx + 5)..(data_beg_idx + 5 + compressed_len)),
        source_file: format!("{}/{}", region_dir.path(), chunk_pos.filename_mca()),
    }));
}