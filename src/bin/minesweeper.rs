use anyhow::Result;
use clap::Parser;
use minesweeper_rs::game::{CellState, Game};
use sat_puzzles::find_all_solutions;
use sat_puzzles::minesweeper_sat::{decode_solution, generate_clauses};
use std::collections::HashSet;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(
    author,
    version,
    about = "A hybrid SAT solver for Minesweeper probability analysis."
)]
struct Cli {
    /// Path to the map definition file ('*' for mines, '.' for empty).
    #[arg(short, long)]
    map_file: PathBuf,

    /// Reveal coordinates for the first click, e.g., `-r 0 0`.
    #[arg(short, long, num_args = 2, value_names = ["COL", "ROW"], default_value = "0 0")]
    reveal: Vec<usize>,

    /// Optional: Output filename for the generated local CNF clauses.
    #[arg(short, long)]
    cnf_file: Option<PathBuf>,
}

/// Calculates combinations "n choose k" using u128 to prevent overflow.
fn combinations(n: u128, k: u128) -> u128 {
    if k > n {
        return 0;
    }
    if k == 0 || k == n {
        return 1;
    }
    if k > n / 2 {
        return combinations(n, n - k);
    }
    (k + 1..=n).fold(1, |acc, val| acc * val / (val - k))
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    if cli.reveal.len() != 2 {
        anyhow::bail!("Reveal argument must have exactly two values: a row and a column.");
    }

    let (col, row) = (cli.reveal[0], cli.reveal[1]);

    let mut game = Game::from_file(&cli.map_file)?;
    if row >= game.height || col >= game.width {
        anyhow::bail!(
            "Reveal coordinates ({}, {}) are outside the grid ({}x{})",
            col,
            row,
            game.width,
            game.height,
        );
    }
    game.reveal(col, row);

    println!("Ground truth - revealed board:\n");
    game.display(true); // show all
    println!("Current game state:\n{game}");

    // STEP 1: Generate the clauses and the variable map.
    // `clauses` and `var_map` are now owned by the `main` function.
    let (global_constraint, local_constraints, sea_of_unknown) = game.get_constraints();
    let sea_set: HashSet<_> = sea_of_unknown.into_iter().collect();

    let unknown_indices: Vec<usize> = global_constraint
        .cells
        .into_iter()
        .filter(|index| !sea_set.contains(index))
        .collect();

    let (clauses, var_map) = generate_clauses(&unknown_indices, &local_constraints);

    if let Some(path) = &cli.cnf_file {
        sat_puzzles::write_clauses(path, &clauses)?;
    }

    // STEP 2: Create the SAT solution iterator.
    // We pass a reference `&clauses`, which is valid for the whole scope.
    let sat_iterator = find_all_solutions(&clauses)?;

    let n_cells = game.width * game.height;
    let mut total_weight = 0.0;
    let mut probs = vec![0.0; n_cells];
    let mut n_sat_solutions = 0;
    let mut remaining_mines_sum = 0.0;
    for model in sat_iterator {
        n_sat_solutions += 1;
        let solution = decode_solution(&model, game.width, game.height, &var_map);
        let local_mines = solution.mines.iter().filter(|&&b| b).count();
        let remaining_mines = global_constraint.count - local_mines as f64;
        remaining_mines_sum += remaining_mines;
        let weight = combinations(sea_set.len() as u128, remaining_mines as u128) as f64;
        total_weight += weight;

        let prob_contribution = weight; // Normalize later
        for (i, &is_mine) in solution.mines.iter().enumerate() {
            if is_mine {
                probs[i] += prob_contribution;
            }
        }
        let sea_prob = remaining_mines / sea_set.len() as f64;
        for &idx in &sea_set {
            probs[idx] += sea_prob * prob_contribution;
        }
    }
    probs.iter_mut().for_each(|p| *p /= total_weight);

    let remaining_mines_avg = if n_sat_solutions > 0 {
        remaining_mines_sum as u128 / n_sat_solutions
    } else {
        0
    };
    println!(
        "Found {n_sat_solutions} SAT solutions + avg {remaining_mines_avg:.2} mines in the sea of unknowns (size {sea_set_size}) for ~{total_weight:.3e} combinations",
        total_weight = total_weight as usize,
        sea_set_size = sea_set.len()
    );

    println!("\nProbability map:");
    for row in 0..game.height {
        for col in 0..game.width {
            let idx = row * game.width + col;
            if game.get_cell(col, row).state == CellState::Revealed {
                print!("  -  "); // Already revealed
            } else {
                print!("{:4.2} ", probs[idx]);
            }
        }
        println!();
    }
    Ok(())
}
