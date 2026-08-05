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

use chrono::DateTime;
use clap::{command, Parser, Subcommand};
use mc_schem::schem;
use mc_schem::schem::{RawMetaData, Schematic};

/// Read, write, convert minecraft schematic files via different versions
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = "")]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Get information about a schematic
    See {
        /// Schematic file to load
        #[arg()]
        file: String,
        /// Print all information
        #[arg(long, default_value_t = false)]
        all: bool,
        /// Print size, volume and block count
        #[arg(long, default_value_t = false)]
        size: bool,
        /// print metadata
        #[arg(long, default_value_t = false)]
        metadata: bool,
    },
    ///Print information about this executable
    Print {
        /// Print supported formats(loadable OR savable)
        #[arg(long, default_value_t = false)]
        supported_formats: bool,
        /// Print loadable formats
        #[arg(long, default_value_t = false)]
        loadable_formats: bool,
        /// Print savable formats
        #[arg(long, default_value_t = false)]
        savable_formats: bool,
    },
    /// Convert schematic via different formats
    Convert {
        /// Input file
        #[arg()]
        input: String,

        /// Output file
        #[arg(short, long, default_value_t = String::from("out.litematic"))]
        output: String,

        /// Record time used
        #[arg(long, default_value_t = false)]
        benchmark: bool,
    },
}

fn main() {
    let args = Args::parse();

    match args.command {
        Commands::Convert {
            input,
            output,
            benchmark,
        } => {
            let begin_time = std::time::SystemTime::now();
            let schem = match Schematic::from_file(&input) {
                Ok(s) => s.0,
                Err(e) => {
                    eprintln!("Failed to load {}: {e}", input);
                    std::process::exit(1);
                }
            };

            let parsed_time = std::time::SystemTime::now();

            match schem.save_to_file(&output) {
                Ok(_) => {}
                Err(e) => {
                    eprintln!("Failed to save {}: {e}", output);
                    std::process::exit(2);
                }
            }

            let finish_time = std::time::SystemTime::now();
            if benchmark {
                let load_time = parsed_time
                    .duration_since(begin_time)
                    .expect("Failed to compute time cost.");
                let save_time = finish_time
                    .duration_since(parsed_time)
                    .expect("Failed to compute time cost.");
                let total_time = finish_time
                    .duration_since(begin_time)
                    .expect("Failed to compute time cost.");
                print!(
                    "Loading cost {} seconds, saving cost {} seconds, {} seconds in total.",
                    load_time.as_secs_f64(),
                    save_time.as_secs_f64(),
                    total_time.as_secs_f64()
                );
            }
        }
        Commands::See {
            file,
            mut all,
            size,
            metadata,
        } => {
            let (schematic, raw) = match Schematic::from_file(&file) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("Failed to load {}: {e}", file);
                    std::process::exit(1);
                }
            };
            if !all {
                all = !(size || metadata);
            }

            if size || all {
                println!(
                    "Size: {}, volume: {}",
                    schem::common::format_size(&schematic.shape()),
                    schematic.volume()
                );
            }

            if metadata || all {
                let dim_letters = ['X', 'Y', 'Z'];
                println!("Metadata: ");
                type DT = DateTime<chrono::Utc>;
                match raw {
                    RawMetaData::Litematica(raw) => {
                        println!("\tDataVersion: {}", raw.data_version);
                        println!("\tVersion: {}", raw.version);
                        if let Some(sv) = &raw.sub_version {
                            println!("\tSubVersion: {sv}");
                        }
                        //(schem::common::i64_ms_timestamp_to_date(raw.time_created))
                        println!(
                            "\tTimeCreated: {} (aka {})",
                            raw.time_created,
                            DT::from(schem::common::i64_ms_timestamp_to_system_time(
                                raw.time_created
                            ))
                        );
                        println!(
                            "\tTimeModified: {} (aka {})",
                            raw.time_modified,
                            DT::from(schem::common::i64_ms_timestamp_to_system_time(
                                raw.time_modified
                            ))
                        );
                        println!("\tAuthor: {}", raw.author);
                        println!("\tName: {}", raw.name);
                        println!("\tDescription: {}", raw.description);
                        println!("\tRegionCount: {}", raw.region_count);
                        println!("\tTotalBlocks: {}", raw.total_blocks);
                        println!("\tTotalVolume: {}", raw.total_volume);
                        println!(
                            "\tEnclosingSize: {}",
                            schem::common::format_size(&raw.enclosing_size)
                        );
                    }
                    RawMetaData::VanillaStructure(raw) => {
                        println!("\tDataVersion: {}", raw.data_version);
                    }
                    RawMetaData::WE13(raw) => {
                        println!("\tDataVersion: {}", raw.data_version);
                        println!("\tVersion: {}", raw.version);
                        println!("\tWidth: {}", raw.width);
                        println!("\tHeight: {}", raw.height);
                        println!("\tLength: {}", raw.length);
                        for dim in 0..3 {
                            println!("\tWEOffset{}: {}", dim_letters[dim], raw.we_offset[dim]);
                        }
                        println!("\tOffset: {}", schem::common::format_size(&raw.offset));
                        if let Some(date) = raw.date {
                            println!(
                                "\tDate: {} (aka {})",
                                date,
                                DT::from(schem::common::i64_ms_timestamp_to_system_time(date))
                            );
                        }
                        if let Some(extra) = raw.v3_extra {
                            println!("\tMetadata/WorldEdit/Version: {}", extra.world_edit_version);
                            println!("\tEditingPlatform: {}", extra.editing_platform);
                            println!("\tOrigin: {}", schem::common::format_size(&extra.origin));
                        }
                    }
                    RawMetaData::WE12(raw) => {
                        println!("\tMaterials: {}", raw.materials);
                        println!("\tWidth: {}", raw.width);
                        println!("\tHeight: {}", raw.height);
                        println!("\tLength: {}", raw.length);
                        for dim in 0..3 {
                            println!("\tWEOffset{}: {}", dim_letters[dim], raw.we_offset[dim]);
                        }
                        for dim in 0..3 {
                            println!("\tWEOrigin{}: {}", dim_letters[dim], raw.we_origin[dim]);
                        }
                    }
                }
            }
        }
        Commands::Print {
            supported_formats,
            loadable_formats,
            savable_formats,
        } => {
            if supported_formats {
                println!("Supported formats:");
                for f in mc_schem::SchemFormat::supported_formats() {
                    println!("\t{f}({})", f.extension());
                }
            }
            if loadable_formats {
                println!("Loadable formats:");
                for f in mc_schem::SchemFormat::loadable_formats() {
                    println!("\t{f}({})", f.extension());
                }
            }
            if savable_formats {
                println!("Savable formats:");
                for f in mc_schem::SchemFormat::savable_formats() {
                    println!("\t{f}({})", f.extension());
                }
            }
        }
    }
}
