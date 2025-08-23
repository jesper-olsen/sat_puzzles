use anyhow::Result;
use clap::{Parser, Subcommand};
use nqueens_sat::{Solution, find_all_solutions, generate_clauses};
use std::fs::File;
use std::io::{BufWriter, Write};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate a DIMACS CNF file for the N-Queens problem
    Generate {
        /// The number of queens (the size of the board, N x N)
        n: usize,
        // // The output .cnf file path
        // #[arg(short, long, value_name = "FILE")]
        // output: PathBuf,
    },
    /// Solve the N-Queens problem and visualize the solution(s)
    Solve {
        /// The number of queens (the size of the board, N x N)
        n: usize,
        /// Find all possible solutions instead of just one
        #[arg(short, long)]
        all: bool,
    },
}

/// Prints a single N-Queens solution to the console as a board.
fn print_solution(solution: &Solution, n: usize, solution_num: usize) {
    println!("\n--- Solution {solution_num} ---");
    let mut board = vec![vec!['.'; n]; n];
    for &(r, c) in solution {
        board[r][c] = 'Q';
    }

    for row in board {
        for cell in row {
            print!("{cell} ");
        }
        println!();
    }
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        //Commands::Generate { n, output } => {
        Commands::Generate { n } => {
            println!("Generating CNF for {n}-Queens problem...");
            let clauses = generate_clauses(*n);
            let num_vars = n * n;

            let output = format!("{n}-queens.cnf");
            let file = File::create(&output)?;
            let mut writer = BufWriter::new(file);
            //let stdout = std::io::stdout();
            // lock stdout so we can borrow it safely
            //let mut writer = BufWriter::new(stdout.lock());

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
        Commands::Solve { n, all } => {
            println!("Solving for {n}-Queens...");
            let solutions = find_all_solutions(*n)?;

            if solutions.is_empty() {
                println!("No solutions found for N={n}");
            } else if *all {
                println!("Found {} unique solutions for N={}", solutions.len(), n);
                for (i, sol) in solutions.iter().enumerate() {
                    print_solution(sol, *n, i + 1);
                }
            } else {
                println!("Found a solution for N={n}");
                print_solution(&solutions[0], *n, 1);
            }
        }
    }

    Ok(())
}
