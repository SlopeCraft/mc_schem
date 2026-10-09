/*
 mc_schem is a rust library to generate, load, manipulate and save minecraft
 schematic files. Copyright (C) 2026 ToKiNoBug

 This program is free software: you can redistribute it and/or modify it under
 the terms of the GNU General Public License as published by the Free Software
 Foundation, either version 3 of the License, or (at your option) any later
 version.

 This program is distributed in the hope that it will be useful, but WITHOUT ANY
 WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A
 PARTICULAR PURPOSE.  See the GNU General Public License for more details.

 You should have received a copy of the GNU General Public License
 along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */

#[allow(unused_imports)]
use crate::world::{Dimension, FilesInMemory, FilesRead, World, WorldLoadOption};
use crate::Error;
use std::collections::BTreeMap;
#[allow(unused_imports)]
use web_time;

impl Default for WorldLoadOption {
    fn default() -> Self {
        Self {
            parse_directly: false,
        }
    }
}

impl World {
    pub fn from_files(files: &dyn FilesRead, option: &WorldLoadOption) -> Result<World, Error> {
        let mut world = World {
            dimensions: BTreeMap::new(),
        };
        for dim in [0, -1, 1] {
            let dimension = if dim == 0 {
                Dimension::from_files(files, option.parse_directly, -64..320, dim)?
            } else {
                let dir = format!("DIM{dim}");
                // let y_range = if dim == -1 { 0..256 } else { -64..320 };
                let y_range = 0..256;
                Dimension::from_files(
                    &files.sub_directory(&dir),
                    option.parse_directly,
                    y_range,
                    dim,
                )?
            };
            world.dimensions.insert(dim, dimension);
        }
        Ok(world)
    }

    pub fn overworld(&self) -> Option<&Dimension> {
        self.dimensions.get(&0)
    }
    pub fn nether(&self) -> Option<&Dimension> {
        self.dimensions.get(&-1)
    }
    pub fn the_end(&self) -> Option<&Dimension> {
        self.dimensions.get(&1)
    }

    pub fn parse_all_dimensions(&mut self) -> Result<(), Error> {
        for (id, dim) in &mut self.dimensions {
            dim.parse_all(*id)?;
        }
        Ok(())
    }
}

#[test]
fn load_all_worlds() {
    let files = [
        "test_files/world/00_1.20.2.7z",
        "test_files/world/01_large-world-1.20.2.7z",
        "test_files/world/02_mcc-block-entities.7z",
        "test_files/world/03_raids-1.20.2.7z",
    ];

    for file in files {
        println!("Parsing {file}...");
        let begin = web_time::SystemTime::now();
        let src = FilesInMemory::from_7z_file(file, "").expect("Read 7z file and decompress");
        let mut world = World::from_files(&src, &WorldLoadOption::default())
            .expect("Parse world from files in memory");
        world.parse_all_dimensions().expect("Parse all dimensions");
        let parsed = web_time::SystemTime::now();
        let cost = parsed.duration_since(begin).unwrap().as_millis();
        println!("Spend {cost} milliseconds");
    }
}
