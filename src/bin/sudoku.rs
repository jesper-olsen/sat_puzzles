use anyhow::Result;
use clap::{Parser, Subcommand, ValueEnum};
use sat_puzzles::sudoku::{PUZZLE_EASY, PUZZLE_HARD, PUZZLE_HARDER};
use sat_puzzles::sudoku::{SudokuGrid, decode_solution, generate_clauses};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(ValueEnum, Copy, Clone, Debug)]
enum Puzzle {
    Easy1,
    Hard1,
    Harder1,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate a DIMACS CNF file for the Sudoku problem
    Generate {
        /// The puzzle to solve
        puzzle: Puzzle,
    },
    /// Solve the N-Queens problem and visualize the solution(s)
    Solve {
        /// The puzzle to solve
        puzzle: Puzzle,
        /// Find all possible solutions instead of just one
        #[arg(short, long)]
        all: bool,
    },
}

fn puzzle2sudoku(puzzle: Puzzle) -> SudokuGrid {
    match puzzle {
        Puzzle::Easy1 => PUZZLE_EASY,
        Puzzle::Harder1 => PUZZLE_HARDER,
        Puzzle::Hard1 => PUZZLE_HARD,
    }
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match &cli.command {
        Commands::Generate { puzzle } => {
            println!("Generating CNF for Sudoku {puzzle:?}...");
            let grid = puzzle2sudoku(*puzzle);
            let clauses = generate_clauses(&grid);
            let output = format!("sudoku.cnf");
            sat_puzzles::write_clauses(&output, &clauses)?;
        }
        Commands::Solve { puzzle, all } => {
            let grid = puzzle2sudoku(*puzzle);
            println!("Solving sudoku for {puzzle:?}");
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
                println!("No solutions found for {puzzle:?}");
            } else if *all {
                println!(
                    "Found {} unique solutions for Sudoku {puzzle:?}",
                    solutions.len()
                );
                for (i, sol) in solutions.iter().enumerate() {
                    println!("\n--- Solution {solution_num} ---", solution_num = i + 1);
                    println!("{sol}");
                }
            } else {
                println!("Found a solution for Sudoku {puzzle:?}");
                println!("{}", solutions[0]);
            }
        }
    }

    Ok(())
}
