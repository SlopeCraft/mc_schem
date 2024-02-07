use clap::{command, Parser, Subcommand};
use mc_schem::schem;
use mc_schem::schem::Schematic;

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
        Commands::Convert { input, output, benchmark } => {
            let begin_time = std::time::SystemTime::now();
            let schem = match Schematic::from_file(&input) {
                Ok(s) => s,
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
                let load_time = parsed_time.duration_since(begin_time).expect("Failed to compute time cost.");
                let save_time = finish_time.duration_since(parsed_time).expect("Failed to compute time cost.");
                let total_time = finish_time.duration_since(begin_time).expect("Failed to compute time cost.");
                print!("Loading cost {} seconds, saving cost {} seconds, {} seconds in total.",
                       load_time.as_secs_f64(),
                       save_time.as_secs_f64(),
                       total_time.as_secs_f64());
            }
        }
        Commands::See { file, mut all, size, metadata } => {
            let schematic = match Schematic::from_file(&file) {
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
                println!("Size: {}, volume: {}", schem::common::format_size(&schematic.shape()), schematic.volume());
            }
        }
        Commands::Print { supported_formats, loadable_formats, savable_formats } => {
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