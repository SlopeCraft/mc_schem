use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::time;
use fastnbt::Value;
use flate2::read::{GzDecoder, ZlibDecoder};
use regex::Regex;
use crate::error::Error;
use crate::world::{FilesInMemory, ChunkPos, ChunkVariant, FilesRead, XZCoordinate, Chunk, UnparsedChunkData, Dimension};


pub const SEGMENT_BYTES: usize = 4096;

impl Display for XZCoordinate {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {})", self.x, self.z)
    }
}

struct ChunkRawDataRef<'a> {
    data: &'a [u8],
    unix_timestamp: u32,
}

pub fn offset_in_mca_file(local_coord: &XZCoordinate<u32>) -> u32 {
    return 4 * ((local_coord.x & 31) + (local_coord.z & 31) * 32);
}

fn parse_mca_single_chunk<'a>(coord: &XZCoordinate<u32>, file_bytes: &'a [u8]) -> Result<Option<ChunkRawDataRef<'a>>, Error> {
    let header: [u8; 4];
    {
        let offset_by_byte = offset_in_mca_file(coord) as usize;
        debug_assert!((offset_by_byte + 3) < 4096);
        header = [file_bytes[offset_by_byte + 0],
            file_bytes[offset_by_byte + 1],
            file_bytes[offset_by_byte + 2],
            file_bytes[offset_by_byte + 3]];
    }
    if header == [0; 4] {
        return Ok(None);
    }

    let timestamp: u32;
    {
        let offset_by_byte = offset_in_mca_file(coord) as usize + SEGMENT_BYTES;
        debug_assert!((offset_by_byte + 3) < 8192);
        timestamp = u32::from_be_bytes([
            file_bytes[offset_by_byte + 0],
            file_bytes[offset_by_byte + 1],
            file_bytes[offset_by_byte + 2],
            file_bytes[offset_by_byte + 3],
        ]);
    }

    let offset_by_segment = u32::from_be_bytes([0, header[0], header[1], header[2]]);
    let num_segments = header[3] as u32;
    if offset_by_segment < 2 || (num_segments + offset_by_segment) as usize > (file_bytes.len() / SEGMENT_BYTES) {
        return Err(Error::InvalidSegmentRangeInMCA {
            chunk_local_x: coord.x as i32,
            chunk_local_z: coord.z as i32,
            offset_by_segment,
            num_segments,
            total_segments: file_bytes.len(),
        });
    }

    let data_beg_idx = offset_by_segment as usize * SEGMENT_BYTES;
    let data_end_idx = (offset_by_segment + num_segments) as usize * SEGMENT_BYTES;

    return Ok(Some(ChunkRawDataRef {
        data: &file_bytes[data_beg_idx..data_end_idx],
        unix_timestamp: timestamp,
    }));
}

fn parse_mca_from_bytes(bytes: &[u8]) -> Result<HashMap<XZCoordinate<u32>, ChunkRawDataRef>, Error> {
    if bytes.is_empty() {
        return Ok(HashMap::new());
    }
    if bytes.len() % 4096 != 0 {
        return Err(Error::IncompleteSegmentInMCA { bytes: bytes.len() });
    }

    let mut map = HashMap::with_capacity(1024);

    for z in 0..32 {
        for x in 0..32 {
            let coord = XZCoordinate { x, z };
            let raw_data = parse_mca_single_chunk(&coord, bytes)?;
            if let Some(raw_data) = raw_data {
                map.insert(coord, raw_data);
            }
        }
    }

    return Ok(map);
}


impl ChunkRawDataRef<'_> {
    pub fn to_nbt(&self) -> Result<HashMap<String, Value>, Error> {
        let data_bytes: usize;
        {
            let db = u32::from_be_bytes([self.data[0], self.data[1], self.data[2], self.data[3]]);
            data_bytes = db as usize;
        }
        let compress_type: u8 = self.data[4];

        let compressed_data = &self.data[5..(5 + data_bytes)];
        let parse_opt: Result<HashMap<String, Value>, fastnbt::error::Error>;
        match compress_type {
            1 | 128 => {//gzip
                let src = GzDecoder::new(compressed_data);
                parse_opt = fastnbt::from_reader(src);
            }
            2 | 129 => {//zlib
                let src = ZlibDecoder::new(compressed_data);
                parse_opt = fastnbt::from_reader(src);
            }
            3 | 130 => {// no compress
                parse_opt = fastnbt::from_reader(compressed_data);
            }
            _ => { return Err(Error::InvalidMCACompressType { compress_label: compress_type }); }
        }

        if compress_type > 127 {
            !todo!();
        }

        return match parse_opt {
            Ok(nbt) => Ok(nbt),
            Err(e) => Err(Error::NBTReadError(e)),
        };
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

pub fn parse_multiple_regions(region_dir: &dyn FilesRead, parse_directly: bool) -> Result<HashMap<ChunkPos, ChunkVariant>, Error> {
    let files = region_dir.files();
    let mut region_files = Vec::with_capacity(files.len());
    {
        for info in &files {
            if let Some(coord) = parse_mca_filename(&info.name) {
                region_files.push((info, coord));
            }
        }
    }
    let mut result = HashMap::new();
    let mut buffer: Vec<u8> = Vec::new();
    for (info, file_coord) in &region_files {
        let chunk;
        if let Some(file_content) = region_dir.read_file_nocopy(&info.name)? {
            chunk = parse_mca_from_bytes(file_content)?;
        } else {
            region_dir.read_file(&info.name, &mut buffer)?;
            chunk = parse_mca_from_bytes(&buffer)?;
        }

        for (local_coord, raw) in &chunk {
            let chunk_pos = ChunkPos::from_local_pos(file_coord, local_coord);
            let variant: ChunkVariant;
            if parse_directly {
                variant = ChunkVariant::Parsed(Chunk::from_nbt(raw.to_nbt()?,
                                                               &chunk_pos,
                                                               &info.full_name)?);
            } else {
                variant = ChunkVariant::Unparsed(UnparsedChunkData {
                    time_stamp: raw.unix_timestamp,
                    region_data: raw.data.to_vec(),
                    source_file: info.full_name.clone(),
                });
            }

            result.insert(chunk_pos, variant);
        }
    }
    return Ok(result);
}


impl ChunkVariant {
    pub fn parse(&mut self, chunk_pos: &ChunkPos) -> Result<(), Error> {
        if let ChunkVariant::Unparsed(raw) = self {
            let raw_ref = ChunkRawDataRef { data: &raw.region_data, unix_timestamp: raw.time_stamp };
            let nbt = raw_ref.to_nbt()?;

            let chunk = Chunk::from_nbt(nbt, &chunk_pos, &raw.source_file)?;
            *self = ChunkVariant::Parsed(chunk);
        }

        return Ok(());
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


#[test]
fn test_parse_mca_filename() {
    let names = ["r.0.1.mca", "r.-1.9.mca", "r.1.-1.mca", "r.-1.-1.mca", ];
    for name in names {
        let pos = parse_mca_filename(name).unwrap();
        let filename = format!("r.{}.{}.mca", pos.x, pos.z);
        assert_eq!(filename, name);
    }
}

#[test]
fn load_mca() {
    let afs = FilesInMemory::from_7z_file("test_files/world/00_1.20.2.7z", "").unwrap();

    let path = "region/r.0.0.mca";
    let r00mca = afs.read_file_as_bytes(path).unwrap();


    let mut chunks = parse_mca_from_bytes(&r00mca).unwrap();
    for (_coord, raw) in &mut chunks {
        raw.to_nbt().unwrap();
    }
}

