use std::sync::mpsc::channel;
use std::time;
use crate::Error;
use crate::world::{Dimension, FilesInMemory, FilesRead, mca};
use rayon::prelude::*;
use crate::block::Block;
use crate::region::{BlockEntity, PendingTick, WorldSlice};

impl Dimension {
    pub fn from_files(files: &dyn FilesRead, parse_directly: bool) -> Result<Dimension, Error> {
        let chunks = mca::parse_multiple_regions(&files.sub_directory("region"),
                                                 Some(&files.sub_directory("entities")),
                                                 parse_directly)?;
        return Ok(Dimension {
            chunks
        });
    }

    pub fn check_all(&self) -> Result<(), Error> {
        let (tx, rx) = channel();

        self.chunks.par_iter().for_each(|(pos, variant)| {
            if let Err(e) = variant.check(pos) {
                tx.send(e).unwrap();
            }
        });

        if let Ok(e) = rx.try_recv() {
            return Err(e);
        }

        return Ok(());
    }

    pub fn parse_all(&mut self) -> Result<(), Error> {
        let (tx, rx) = channel();

        self.chunks.par_iter_mut().for_each(|(pos, variant)| {
            if let Err(e) = variant.parse_inplace(pos) {
                tx.send(e).unwrap();
            }
        });

        if let Ok(e) = rx.try_recv() {
            return Err(e);
        }

        return Ok(());
    }
}

impl WorldSlice for Dimension {
    fn offset(&self) -> [i32; 3] {
        return [0, 0, 0];
    }

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
        todo!()
    }

    fn block_info_at(&self, r_pos: [i32; 3]) -> Option<(u16, &Block, Option<&BlockEntity>, Option<&PendingTick>)> {
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