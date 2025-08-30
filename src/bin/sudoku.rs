use anyhow::Result;
use clap::{Parser, Subcommand};
use sat_puzzles::sudoku_sat::{SudokuGrid, decode_solution, generate_clauses};
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
    /// Generate a DIMACS CNF file for the Sudoku problem
    Generate {
        /// Path to the puzzle file
        #[arg(value_name = "FILE")]
        puzzle_file: PathBuf,
        /// Output file name (default: sudoku.cnf)
        #[arg(short, long, default_value = "sudoku.cnf")]
        output: PathBuf,
    },
    /// Solve the N-Queens problem and visualize the solution(s)
    Solve {
        /// Path to the puzzle file
        #[arg(value_name = "FILE")]
        puzzle_file: PathBuf,

        /// Find all possible solutions instead of just one
        #[arg(short, long)]
        all: bool,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match &cli.command {
        Commands::Generate {
            puzzle_file,
            output,
        } => {
            println!(
                "Generating CNF for Sudoku from {}...",
                puzzle_file.display()
            );
            let grid = SudokuGrid::from_file(puzzle_file)?;
            let clauses = generate_clauses(&grid);
            sat_puzzles::write_clauses(output, &clauses)?;
            println!("CNF written to {}", output.display());
        }
        Commands::Solve { puzzle_file, all } => {
            println!("Solving sudoku from {puzzle_file:?}");
            let grid = SudokuGrid::from_file(puzzle_file)?;
            println!("{grid}");
            let clauses = generate_clauses(&grid);

            let raw_solutions_iterator = sat_puzzles::find_all_solutions(&clauses)?;

            let solutions: Vec<SudokuGrid> = if *all {
                raw_solutions_iterator
                    .map(|model| decode_solution(&model))
                    .collect()
            } else {
                raw_solutions_iterator
                    .take(1)
                    .map(|model| decode_solution(&model))
                    .collect()
            };

            if solutions.is_empty() {
                println!("No solutions found for {puzzle_file:?}");
            } else if *all {
                println!(
                    "Found {} unique solution(s) for Sudoku {puzzle_file:?}",
                    solutions.len()
                );
                for (i, sol) in solutions.iter().enumerate() {
                    println!("\n--- Solution {solution_num} ---", solution_num = i + 1);
                    println!("{sol}");
                }
            } else {
                println!("Found a solution for Sudoku {puzzle_file:?}");
                println!("{}", solutions[0]);
            }
        }
    }

    Ok(())
}
