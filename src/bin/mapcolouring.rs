use anyhow::Result;
use clap::{Parser, Subcommand};
use sat_puzzles::mapcolouring_sat::{
    Colouring, decode_solution, generate_clauses, load_map_from_file,
};
use std::path::PathBuf;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate a DIMACS CNF file for the map colouring problem
    Generate {
        /// Path to the map definition file
        map_file: PathBuf,

        /// A list of colours to use (e.g., R G B Y)
        #[arg(short, long, value_delimiter = ' ', num_args = 1.., default_values = ["R","G","B","Y"])]
        colours: Vec<String>,
    },
    /// Solve the N-Queens problem and visualize the solution(s)
    Solve {
        /// Path to the map definition file
        map_file: PathBuf,

        /// A list of colours to use (e.g., R G B Y)
        #[arg(short, long, value_delimiter = ' ', num_args = 1.., default_values = ["R","G","B","Y"])]
        colours: Vec<String>,

        /// Find all possible solutions instead of just one
        #[arg(short, long)]
        all: bool,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Generate { map_file, colours } => {
            println!("Generating CNF for map: {map_file:?}");
            let (states, adjacencies) = load_map_from_file(map_file)?;
            let clauses = generate_clauses(&states, colours, &adjacencies);
            let output = "map.cnf";
            sat_puzzles::write_clauses(output, &clauses)?;
        }
        Commands::Solve {
            map_file,
            colours,
            all,
        } => {
            println!("Solving map colouring for: {map_file:?}");
            let (states, adjacencies) = load_map_from_file(map_file)?;
            let clauses = generate_clauses(&states, colours, &adjacencies);

            let raw_solutions_iterator = sat_puzzles::find_all_solutions(&clauses)?;
            let solutions: Vec<Colouring> = if *all {
                raw_solutions_iterator
                    .map(|model| decode_solution(&model, &states, colours))
                    .collect()
            } else {
                raw_solutions_iterator
                    .take(1)
                    .map(|model| decode_solution(&model, &states, colours))
                    .collect()
            };

            if solutions.is_empty() {
                println!("No solutions found for map of {map_file:?}");
            } else if *all {
                println!(
                    "Found {} unique solutions for map of {map_file:?}",
                    solutions.len()
                );
                for (i, sol) in solutions.iter().enumerate() {
                    println!("\n--- Solution {solution_num} ---", solution_num = i + 1);
                    println!("{sol}");
                }
            } else {
                println!("Found a solution for {map_file:?}");
                println!("{}", solutions[0]);
            }
        }
    }

    Ok(())
}
