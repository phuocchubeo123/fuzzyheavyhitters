use clap::Parser;
use mosaic::data_toolkit::read_csv_and_convert;
use serde_json;
use std::error::Error;
use std::fs::File;
use std::io::BufWriter;
use std::path::Path;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    input: String,
    #[arg(short, long)]
    output: String,
    #[arg(short, long)]
    output_type: u8,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    let input_file = &args.input;
    let output_file = &args.output;

    println!("Starting RideAustin client points JSON generation...");

    // Check if input file exists
    if !Path::new(input_file).exists() {
        eprintln!("Error: Input file '{}' does not exist.", input_file);
        eprintln!("Please make sure you have the ride data CSV file at this location.");
        return Ok(());
    }

    // Read and convert CSV data
    println!("Reading and converting CSV data from {}...", input_file);
    let client_points = read_csv_and_convert(input_file, args.output_type)?;
    println!(
        "Converted {} ride points to client format",
        client_points.len()
    );

    // Write to JSON file in the same format as data/synthetic/client_points.json
    println!("Writing client points JSON to {}...", output_file);
    let file = File::create(output_file)?;
    let writer = BufWriter::new(file);
    serde_json::to_writer_pretty(writer, &client_points)?;

    println!("Successfully generated client points JSON!");
    println!("Total points: {}", client_points.len());
    println!("Output file: {}", output_file);

    // Print some sample statistics
    if !client_points.is_empty() {
        println!("\nSample points:");
        for (i, point) in client_points.iter().take(5).enumerate() {
            println!(
                "Point {}: [start_lat_grid: {}, start_lon_grid: {}]",
                i + 1,
                point[0],
                point[1]
            );
            // println!("  Point {}: [start_lat_grid: {}, start_lon_grid: {}, end_lat_grid: {}, end_lon_grid: {}]", i+1, point[0], point[1], point[2], point[3]);
        }
    }

    Ok(())
}
