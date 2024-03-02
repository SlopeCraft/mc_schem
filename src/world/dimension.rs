use std::sync::atomic::AtomicBool;
use std::sync::mpsc::channel;
use std::time;
use crate::Error;
use crate::world::{Dimension, FilesInMemory, FilesRead, mca};
use rayon::prelude::*;

impl Dimension {
    pub fn from_files(files: &dyn FilesRead, parse_directly: bool) -> Result<Dimension, Error> {
        let chunks = mca::parse_multiple_regions(files, parse_directly)?;
        return Ok(Dimension {
            chunks
        });
    }

    pub fn parse_all(&mut self) -> Result<(), Error> {
        let (mut tx, mut rx) = channel();

        self.chunks.par_iter_mut().for_each(|(pos, variant)| {
            if let Err(e) = variant.parse(pos) {
                tx.send(e).unwrap();
            }
        });

        if let Ok(e) = rx.try_recv() {
            return Err(e);
        }

        return Ok(());
    }
}


#[test]
fn test_load_dimension() {
    let begin = time::SystemTime::now();
    let files = FilesInMemory::from_7z_file("test_files/world/00_1.20.2.7z", "").unwrap();
    let decompressed = time::SystemTime::now();

    let mut dim = Dimension::from_files(&files.sub_directory("region"), false).unwrap();
    dim.parse_all().unwrap();

    let parsed = time::SystemTime::now();

    println!("{} chunks parsed in {} ms.", dim.chunks.len(), parsed.duration_since(begin).unwrap().as_millis());
    println!("Decompression takes {} ms, parsing takes {} ms",
             decompressed.duration_since(begin).unwrap().as_millis(),
             parsed.duration_since(decompressed).unwrap().as_millis());
    //stdin().read_line(&mut String::new()).unwrap();
}