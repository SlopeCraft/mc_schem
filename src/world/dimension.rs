use std::collections::HashMap;
use std::ops::Range;
use std::sync::mpsc::{channel, Receiver};
use std::time;
use crate::Error;
use crate::world::{Chunk, ChunkPos, ChunkVariant, Dimension, FilesInMemory, FilesRead, mca, RefOrObject, XZCoordinate};
use rayon::prelude::*;
use crate::block::Block;
use crate::region::{BlockEntity, HasOffset, PendingTick, WorldSlice};

impl<T> RefOrObject<'_, T> {
    pub fn to_ref(&self) -> &T {
        return match self {
            RefOrObject::Ref(r) => r,
            RefOrObject::Object(o) => &o
        };
    }
}

fn check_chunk_infos(recv: Receiver<(&ChunkPos, Range<i32>)>, num_chunks: usize)
                     -> Result<(), Error> {
    if let Err(_) = recv.try_recv() {
        return Ok(());
    }
    let mut y_range_hist = HashMap::new();
    for (pos, y_range) in recv.try_iter() {
        if !y_range_hist.contains_key(&y_range) {
            y_range_hist.insert(y_range.clone(), Vec::with_capacity(num_chunks));
        }
        let bin = y_range_hist.get_mut(&y_range).unwrap();
        bin.push(*pos);
    }

    if y_range_hist.len() <= 1 {
        return Ok(());
    }
    let mut majority_y_range = 0..0;
    let mut num = 0;

    for (range, bin) in &y_range_hist {
        if bin.len() > num {
            debug_assert!(bin.len() > 0);
            num = bin.len();
            majority_y_range = range.clone();
        }
    }
    if num <= 0 {
        return Ok(());
    }
    let mut exception_value = i32::MAX..i32::MAX;
    let mut exception_chunk = ChunkPos::from_global_pos(&XZCoordinate { x: i32::MAX, z: i32::MAX });
    for (range, bin) in &y_range_hist {
        if range != &majority_y_range {
            exception_value = range.clone();
            exception_chunk = bin[0];
        }
    }
    return Err(Error::DifferentYRangeInOneDimension {
        majority_y_range,
        exception_value,
        exception_chunk_x: exception_chunk.global_x,
        exception_chunk_z: exception_chunk.global_z,
    });
}

impl Dimension {
    pub fn from_files(files: &dyn FilesRead, parse_directly: bool) -> Result<Dimension, Error> {
        let chunks = mca::parse_multiple_regions(&files.sub_directory("region"),
                                                 Some(&files.sub_directory("entities")),
                                                 parse_directly)?;
        return Ok(Dimension {
            chunks
        });
    }

    pub fn block_pos_to_chunk_pos(block_pos: [i32; 3]) -> (ChunkPos, i8) {
        let cpos = ChunkPos::from_global_pos(&XZCoordinate { x: block_pos[0] / 16, z: block_pos[2] / 16 });
        let y = block_pos[1] / 16;
        return (cpos, y as i8);
    }

    pub fn get_chunk(&self, chunk_pos: &ChunkPos) -> Option<&Chunk> {
        return match self.chunks.get(chunk_pos)? {
            ChunkVariant::Parsed(chunk) => Some(chunk),
            ChunkVariant::Unparsed(_) => None
        };
    }

    pub fn get_chunk_mut(&mut self, chunk_pos: &ChunkPos) -> Option<&mut Chunk> {
        return match self.chunks.get_mut(chunk_pos)? {
            ChunkVariant::Parsed(chunk) => Some(chunk),
            ChunkVariant::Unparsed(_) => None
        };
    }

    pub fn check_all(&self) -> Result<(), Error> {
        let (tx, rx) = channel();

        // Collect chunk infos of all chunks
        let (chunk_info_tx, chunk_info_rx) = channel();

        self.chunks.par_iter().for_each(|(pos, variant)| {
            match variant.check(pos) {
                Err(e) => { tx.send(e).unwrap(); },
                Ok(chunk) => {
                    chunk_info_tx.send((pos, chunk.to_ref().y_range())).unwrap()
                }
            }
        });

        if let Ok(e) = rx.try_recv() {
            return Err(e);
        }

        check_chunk_infos(chunk_info_rx, self.chunks.len())?;

        return Ok(());
    }

    pub fn parse_all(&mut self) -> Result<(), Error> {
        let (tx, rx) = channel();
        let (chunk_info_tx, chunk_info_rx) = channel();
        let num_chunks = self.chunks.len();

        self.chunks.par_iter_mut().for_each(|(pos, variant)| {
            match variant.parse_inplace(pos) {
                Err(e) => tx.send(e).unwrap(),
                Ok(chunk) => {
                    chunk_info_tx.send((pos, chunk.y_range())).unwrap();
                }
            }
        });

        if let Ok(e) = rx.try_recv() {
            return Err(e);
        }

        check_chunk_infos(chunk_info_rx, num_chunks)?;

        return Ok(());
    }
}

impl HasOffset for Dimension {
    fn offset(&self) -> [i32; 3] {
        return [0, 0, 0];
    }
}

impl WorldSlice for Dimension {

    fn shape(&self) -> [i32; 3] {
        let mut xmin = i32::MAX;
        let mut xmax = i32::MIN;
        let mut zmin = i32::MAX;
        let mut zmax = i32::MIN;
        for (pos, _) in &self.chunks {
            let x = pos.to_global_pos().x;
            let z = pos.to_global_pos().z;
            xmin = xmin.min(x);
            xmax = xmax.max(x);
            zmin = zmin.min(z);
            zmax = zmax.max(z);
        }
        let height = 384;
        return [(xmax - xmin + 1) * 16, height, (zmax - zmin + 1) * 16];
    }

    fn total_blocks(&self, include_air: bool) -> u64 {
        let mut num_blocks = 0;
        for (_, chunk) in &self.chunks {
            if let ChunkVariant::Parsed(chunk) = chunk {
                num_blocks += chunk.total_blocks(include_air);
            }
        }
        return num_blocks;
    }

    fn block_index_at(&self, r_pos: [i32; 3]) -> Option<u16> {
        todo!()
    }

    fn block_at(&self, r_pos: [i32; 3]) -> Option<&Block> {
        todo!()
    }

    fn block_entity_at(&self, r_pos: [i32; 3]) -> Option<&BlockEntity> {
        todo!()
    }

    fn pending_tick_at(&self, r_pos: [i32; 3]) -> Option<&PendingTick> {
        todo!()
    }
}

#[test]
fn test_load_dimension() {
    let begin = time::SystemTime::now();
    let files = FilesInMemory::from_7z_file("test_files/world/00_1.20.2.7z", "").unwrap();
    let decompressed = time::SystemTime::now();

    let mut dim = Dimension::from_files(&files, false).unwrap();
    dim.parse_all().unwrap();

    let parsed = time::SystemTime::now();

    println!("{} chunks parsed in {} ms.", dim.chunks.len(), parsed.duration_since(begin).unwrap().as_millis());
    println!("Decompression takes {} ms, parsing takes {} ms",
             decompressed.duration_since(begin).unwrap().as_millis(),
             parsed.duration_since(decompressed).unwrap().as_millis());
}

#[test]
fn test_large_overworld() {
    let begin = time::SystemTime::now();
    let files = FilesInMemory::from_7z_file("test_files/world/01_large-world-1.20.2.7z", "").unwrap();
    let decompressed = time::SystemTime::now();

    let mut dim = Dimension::from_files(&files, false).unwrap();

    dim.parse_all().unwrap();
    //dim.check_all().unwrap();

    let parsed = time::SystemTime::now();

    println!("{} chunks parsed in {} ms.", dim.chunks.len(), parsed.duration_since(begin).unwrap().as_millis());
    println!("Decompression takes {} ms, parsing takes {} ms",
             decompressed.duration_since(begin).unwrap().as_millis(),
             parsed.duration_since(decompressed).unwrap().as_millis());
}

#[test]
fn test_load_dimension_mcc_block_entities() {
    let begin = time::SystemTime::now();
    let files = FilesInMemory::from_7z_file("test_files/world/02_mcc-block-entities.7z", "").unwrap();
    let decompressed = time::SystemTime::now();

    let mut dim = Dimension::from_files(&files, false).unwrap();
    dim.parse_all().unwrap();

    let parsed = time::SystemTime::now();

    println!("{} chunks parsed in {} ms.", dim.chunks.len(), parsed.duration_since(begin).unwrap().as_millis());
    println!("Decompression takes {} ms, parsing takes {} ms",
             decompressed.duration_since(begin).unwrap().as_millis(),
             parsed.duration_since(decompressed).unwrap().as_millis());
}