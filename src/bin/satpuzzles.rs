use anyhow::Result;
use clap::{Parser, Subcommand};
use sat_puzzles::{n_queens, sudoku};
use std::collections::HashSet;
use std::fs::File;
use std::io::{BufWriter, Write};

#[derive(Parser)]
#[command(author, version, about = "SAT-based puzzle solver collection")]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    puzzle: PuzzleType,
}

#[derive(Subcommand)]
enum PuzzleType {
    /// Solve N-Queens problems
    #[command(name = "nqueens")]
    NQueens {
        #[command(subcommand)]
        command: NQueensCommand,
    },
    /// Solve Sudoku puzzles
    #[command(name = "sudoku")]
    Sudoku {
        #[command(subcommand)]
        command: SudokuCommand,
    },
    /// Solve map coloring problems
    #[command(name = "mapcolor")]
    MapColour {
        #[command(subcommand)]
        command: MapColourCommand,
    },
}

#[derive(Subcommand)]
enum NQueensCommand {
    /// Generate a DIMACS CNF file
    Generate {
        /// The number of queens
        n: usize,
    },
    /// Solve and visualize solutions
    Solve {
        /// The number of queens
        n: usize,
        /// Find all solutions
        #[arg(short, long)]
        all: bool,
    },
}

#[derive(Subcommand)]
enum SudokuCommand {
    /// List available puzzles
    List,
    /// Generate a DIMACS CNF file
    Generate {
        /// Puzzle name (easy, harder, hard)
        #[arg(default_value = "easy")]
        puzzle: String,
    },
    /// Solve a specific puzzle
    Solve {
        /// Puzzle name (easy, harder, hard)
        #[arg(default_value = "easy")]
        puzzle: String,
    },
}

#[derive(Subcommand)]
enum MapColourCommand {
    /// List available maps
    List,
    /// Generate CNF for a map
    Generate {
        /// Map name
        map: String,
        /// Number of colours
        #[arg(short, long, default_value = "4")]
        colours: usize,
    },
    /// Solve map coloring
    Solve {
        /// Map name
        map: String,
        /// Number of colours
        #[arg(short, long, default_value = "4")]
        colours: usize,
    },
}

fn num_vars(clauses: &[Vec<isize>]) -> usize {
    let mut set = HashSet::new();
    for clause in clauses {
        for &lit in clause {
            set.insert(lit.abs());
        }
    }
    set.len()
}

fn handle_nqueens(command: NQueensCommand) -> Result<()> {
    match command {
        NQueensCommand::Generate { n } => {
            println!("Generating CNF for {n}-Queens problem...");
            let clauses: Vec<Vec<isize>> = n_queens::generate_clauses(n);
            let num_vars = n * n;

            let output = format!("{n}-queens.cnf");
            let file = File::create(&output)?;
            let mut writer = BufWriter::new(file);

            writeln!(writer, "p cnf {num_vars} {}", clauses.len())?;
            for clause in &clauses {
                for literal in clause {
                    write!(writer, "{literal} ")?;
                }
                writeln!(writer, "0")?;
            }
            writer.flush()?;

            println!(
                "Successfully wrote problem to '{output}' ({num_vars} variables, {} clauses)",
                clauses.len()
            );
        }
        NQueensCommand::Solve { n, all } => {
            println!("Solving for {n}-Queens...");
            let solutions = n_queens::find_all_solutions(n)?;

            if solutions.is_empty() {
                println!("No solutions found for N={n}");
            } else if all {
                println!("Found {} unique solutions for N={}", solutions.len(), n);
                for (i, sol) in solutions.iter().enumerate() {
                    println!("\n--- Solution {solution_num} ---", solution_num = i + 1);
                    println!("{sol}");
                }
            } else {
                println!("Found a solution for N={n}");
                println!("{}", solutions[0]);
            }
        }
    }

    Ok(())
}

fn get_sudoku_grid(puzzle: &str) -> Option<sudoku::SudokuGrid> {
    match puzzle {
        "easy" => Some(sudoku::PUZZLE_EASY),
        "harder" => Some(sudoku::PUZZLE_HARDER),
        "hard" => Some(sudoku::PUZZLE_HARD),
        _ => None,
    }
}

fn handle_sudoku(command: SudokuCommand) -> Result<()> {
    match command {
        SudokuCommand::List => println!("easy, harder, hard"),
        SudokuCommand::Generate { puzzle } => {
            let Some(grid) = get_sudoku_grid(&puzzle) else {
                println!("Can't find {puzzle} - try easy, harder or hard");
                return Ok(());
            };
            println!("Generating CNF for {puzzle} Sudoku problem...");
            let clauses = sudoku::generate_clauses(&grid);
            let output = "sudoku.cnf";
            let file = File::create(output)?;
            let mut writer = BufWriter::new(file);
            let num_vars = num_vars(&clauses);

            writeln!(writer, "p cnf {num_vars} {}", clauses.len())?;
            for clause in &clauses {
                for literal in clause {
                    write!(writer, "{literal} ")?;
                }
                writeln!(writer, "0")?;
            }
            writer.flush()?;

            println!(
                "Successfully wrote problem to '{output}' ({num_vars} variables, {} clauses)",
                clauses.len()
            );
        }
        SudokuCommand::Solve { puzzle } => {
            let Some(grid) = get_sudoku_grid(&puzzle) else {
                println!("Can't find {puzzle} - try easy, harder or hard");
                return Ok(());
            };

            println!("Attempting to solve puzzle...");
            println!("{grid}");

            match sudoku::solve_sudoku(&grid) {
                Ok(Some(solution)) => {
                    println!("Solution found:");
                    println!("{solution}");
                }
                Ok(None) => println!("This puzzle has no solution."),
                Err(e) => println!("An error occurred: {e}"),
            }

            println!("\nChecking how many solutions this puzzle has...");
            match sudoku::find_all_solutions(&grid) {
                Ok(solutions) => {
                    println!("Found {} solution(s).", solutions.len());
                    // A well-formed puzzle should have exactly 1.
                }
                Err(e) => println!("An error occurred: {e}"),
            }
        }
    }

    Ok(())
}

fn handle_map_colour(command: MapColourCommand) -> Result<()> {
    match command {
        MapColourCommand::List => println!("australia, usa"),
        MapColourCommand::Generate { map, colours } => {
            println!("Generate CNF for {map} with {colours} colours");
        }
        MapColourCommand::Solve { map, colours } => {
            println!("Solving {map} with {colours} colours");
        }
    }
    Ok(())
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.puzzle {
        PuzzleType::NQueens { command } => handle_nqueens(command),
        PuzzleType::Sudoku { command } => handle_sudoku(command),
        PuzzleType::MapColour { command } => handle_map_colour(command),
    }
}
