use anyhow::Result;
use clap::{Parser, Subcommand};
use sat_puzzles::nqueens_sat::{Queens, decode_solution, generate_clauses};

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

fn main() -> Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Generate { n } => {
            println!("Generating CNF for {n}-Queens problem...");
            let clauses = generate_clauses(*n);
            let output = format!("{n}-queens.cnf");
            sat_puzzles::write_clauses(&output, &clauses)?;
        }
        Commands::Solve { n, all } => {
            println!("Solving for {n}-Queens...");
            let clauses = generate_clauses(*n);

            let raw_solutions_iterator = sat_puzzles::find_all_solutions(&clauses)?;
            let solutions: Vec<Queens> = if *all {
                raw_solutions_iterator
                    .map(|model| decode_solution(&model, *n))
                    .collect()
            } else {
                raw_solutions_iterator
                    .take(1)
                    .map(|model| decode_solution(&model, *n))
                    .collect()
            };

            if solutions.is_empty() {
                println!("No solutions found for N={n}");
            } else if *all {
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
