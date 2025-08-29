use itertools::Itertools;
use minesweeper_rs::Constraint;
use std::collections::HashMap;
use std::fmt;
use varisat::Lit;

/// Represents a single valid solution for the minefield layout.
/// It contains a boolean vector indicating the position of all mines.
#[derive(Debug)] // Added Debug for easier printing
pub struct MinefieldSolution {
    /// A flat vector representing the board. `true` means a mine is present.
    pub mines: Vec<bool>,
    pub width: usize,
    pub height: usize,
}

impl fmt::Display for MinefieldSolution {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // No change here
        for y in 0..self.height {
            for x in 0..self.width {
                let idx = y * self.width + x;
                let symbol = if self.mines[idx] { '*' } else { '#' };
                write!(f, "{symbol} ")?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

/// Generates CNF clauses from the game's constraints.
///
/// A SAT variable is created for each unknown (covered or flagged) cell.
/// Each constraint from the game is translated into clauses that enforce
/// that "exactly k" of the relevant variables can be true.
///
/// Returns the list of clauses and a map from the original cell index
/// to the new SAT variable number. The caller owns this data.
pub fn generate_clauses(
    unknown_indices: &[usize],
    local_constraints: &[Constraint],
) -> (Vec<Vec<isize>>, HashMap<usize, isize>) {
    let mut clauses = Vec::new();

    // Map each unknown cell's index to a unique, 1-based SAT variable.
    let var_map: HashMap<usize, isize> = unknown_indices
        .iter()
        .enumerate()
        .map(|(i, &cell_idx)| (cell_idx, (i + 1) as isize))
        .collect();

    for constraint in local_constraints {
        // Get the SAT variables corresponding to the cells in this constraint.
        let sat_vars: Vec<isize> = constraint
            .cells
            .iter()
            .filter_map(|idx| var_map.get(idx).copied())
            .collect();

        let n = sat_vars.len();
        let k = constraint.count as usize;

        // --- Cardinality Constraint: "Exactly k of these n variables are true" ---
        // 1. "At most k": For any subset of k+1 variables, at least one must be false.
        if n > k {
            for combo in sat_vars.iter().combinations(k + 1) {
                clauses.push(combo.into_iter().map(|&var| -var).collect());
            }
        }

        // 2. "At least k": For any subset of n-k+1 variables, at least one must be true.
        if n > 0 && n >= k {
            for combo in sat_vars.iter().combinations(n - k + 1) {
                clauses.push(combo.into_iter().map(|&var| var).collect());
            }
        }
    }

    (clauses, var_map)
}

/// Decodes a SAT solver model back into a MinefieldSolution.
pub fn decode_solution(
    model: &[Lit],
    width: usize,
    height: usize,
    var_map: &HashMap<usize, isize>,
) -> MinefieldSolution {
    // Create a reverse map from SAT variable -> cell index for efficient lookup.
    let rev_var_map: HashMap<isize, usize> = var_map.iter().map(|(&k, &v)| (v, k)).collect();

    let mut mines = vec![false; width * height];
    for &lit in model.iter() {
        if lit.is_positive() {
            let sat_var = lit.var().to_dimacs();
            if let Some(&cell_idx) = rev_var_map.get(&sat_var) {
                mines[cell_idx] = true;
            }
        }
    }

    MinefieldSolution {
        mines,
        width,
        height,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::find_all_solutions;
    use minesweeper_rs::game::Game;
    use std::collections::HashSet;

    #[test]
    fn test_simple_solver() {
        let board_layout = "
        .*.
        ...
        ...
        ";
        let mut game = Game::from_text(board_layout).unwrap();
        // The real constraints come from revealed numbers and total mines.
        // Let's reveal the '1' to create a constraint on its neighbors.
        game.reveal(0, 0);
        game.num_mines = 1;

        let (global_constraint, local_constraints, sea_of_unknown) = game.get_constraints();
        let sea_set: HashSet<_> = sea_of_unknown.into_iter().collect();

        let unknown_indices: Vec<usize> = global_constraint
            .cells
            .into_iter()
            .filter(|index| !sea_set.contains(index))
            .collect();

        let (clauses, var_map) = generate_clauses(&unknown_indices, &local_constraints);

        let raw_solutions_iterator = find_all_solutions(&clauses).unwrap();

        // Map the raw solutions to our domain-specific solution type.
        let solutions: Vec<MinefieldSolution> = raw_solutions_iterator
            .map(|model| decode_solution(&model, game.width, game.height, &var_map))
            .collect();

        // The constraints are:
        // 1. Exactly 1 of {cell(1,0), cell(0,1), cell(1,1)} is a mine.
        // 2. Exactly 1 of all 8 unknown cells is a mine.
        // The solver should correctly infer the mine must be one of the 3 adjacent cells.
        assert_eq!(solutions.len(), 3);

        // Check that one of the solutions is correct (mine at (1,0))
        let has_expected_solution = solutions.iter().any(|s| {
            s.mines[1] == true && // Mine at (1, 0)
            s.mines[3] == false && // No mine at (0, 1)
            s.mines[4] == false // No mine at (1, 1)
        });
        assert!(has_expected_solution);
    }
}
