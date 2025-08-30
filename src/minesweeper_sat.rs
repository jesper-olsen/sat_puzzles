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
///
/// This implementation is simple but not efficient because of potential combinatorial explosion for some constraints
pub fn generate_clauses_combinatorial(
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

/// Generates CNF clauses using the more efficient Sequential Counter encoding.
///
/// This implements the "exactly-k" constraint based on the paper by Carsten Sinz.
/// It introduces auxiliary variables to represent a running sum, which results
/// in a polynomial number of clauses instead of an exponential one.
///
/// # Arguments
/// * `clauses` - A mutable reference to the list of clauses to add to.
/// * `next_var` - A mutable reference to the next available SAT variable ID.
/// * `sat_vars` - The list of input variables for this single constraint.
/// * `k` - The cardinality value (the `count` for the constraint).
/// See also: "A comparison of encodings for cardinality constraints in a SAT solver", Ed Wynn, 2018
fn add_sequential_encoding_clauses(
    clauses: &mut Vec<Vec<isize>>,
    next_var: &mut isize,
    sat_vars: &[isize],
    k: usize,
) {
    let n = sat_vars.len();
    if k > n {
        // This constraint is unsatisfiable, e.g., "exactly 5 bombs in 3 cells".
        // Add a single empty clause to make the formula UNSAT.
        clauses.push(vec![]);
        return;
    }
    if n == 0 {
        // If there are no variables, the constraint is only satisfiable if k=0.
        if k > 0 {
            clauses.push(vec![]);
        }
        return;
    }

    // --- Special case for k=0 ---
    if k == 0 {
        // All variables must be false.
        for &var in sat_vars {
            clauses.push(vec![-var]);
        }
        return;
    }

    // s[i][j] is an auxiliary variable meaning "at least j+1 of the first i+1
    // variables (x_0...x_i) are true".
    // We need j to go from 0 to k, so the width is k+1. This is the fix.
    let width = k + 1;
    let mut s = vec![0; n * width];
    for i in 0..n {
        for j in 0..width {
            s[i * width + j] = *next_var;
            *next_var += 1;
        }
    }

    // --- Base cases for the first variable x_0 (i=0) ---
    // s_0,0 <=> x_0  (The sum of the first 1 var is >= 1 iff x_0 is true)
    clauses.push(vec![-sat_vars[0], s[0 * width + 0]]);
    clauses.push(vec![sat_vars[0], -s[0 * width + 0]]);

    // For j > 0, s_0,j is false (The sum of the first 1 var cannot be >= j+1 if j>0)
    for j in 1..width {
        clauses.push(vec![-s[0 * width + j]]);
    }

    // --- Inductive step for i from 1 to n-1 ---
    for i in 1..n {
        // Case j=0: s_i,0 <=> s_{i-1},0 OR x_i
        // (Sum of x_0..x_i is >= 1) <=> (Sum of x_0..x_{i-1} was >= 1) OR (x_i is 1)
        clauses.push(vec![-s[(i - 1) * width + 0], s[i * width + 0]]);
        clauses.push(vec![-sat_vars[i], s[i * width + 0]]);
        clauses.push(vec![s[(i - 1) * width + 0], sat_vars[i], -s[i * width + 0]]);

        // Case j>0: s_i,j <=> s_{i-1},j OR (x_i AND s_{i-1},{j-1})
        for j in 1..width {
            // s_{i-1},j => s_i,j
            clauses.push(vec![-s[(i - 1) * width + j], s[i * width + j]]);
            // (x_i AND s_{i-1},j-1) => s_i,j
            clauses.push(vec![
                -sat_vars[i],
                -s[(i - 1) * width + (j - 1)],
                s[i * width + j],
            ]);
            // not s_i,j => (not s_{i-1},j) AND (not x_i OR not s_{i-1},j-1)
            clauses.push(vec![s[(i - 1) * width + j], sat_vars[i], -s[i * width + j]]);
            clauses.push(vec![
                s[(i - 1) * width + j],
                s[(i - 1) * width + (j - 1)],
                -s[i * width + j],
            ]);
        }
    }

    // --- Enforce the final count k for all n variables ---

    // At Least k: The sum of all n variables must be at least k.
    // This means s_{n-1},{k-1} must be true.
    clauses.push(vec![s[(n - 1) * width + (k - 1)]]);

    // At Most k: The sum of all n variables must be at most k.
    // This means it cannot be "at least k+1".
    // So, s_{n-1},k must be false.
    // This is the line that panicked before, but is now valid.
    if k < n {
        clauses.push(vec![-s[(n - 1) * width + k]]);
    }
}

/// Main function to generate all clauses for the Minesweeper board.
/// Ref: Carsten Sinz, "Towards an Optimal CNF Encoding of Boolean Cardinality Constraints", 2005
pub fn generate_clauses(
    unknown_indices: &[usize],
    local_constraints: &[Constraint],
) -> (Vec<Vec<isize>>, HashMap<usize, isize>) {
    let mut clauses = Vec::new();

    let var_map: HashMap<usize, isize> = unknown_indices
        .iter()
        .enumerate()
        .map(|(i, &cell_idx)| (cell_idx, (i + 1) as isize))
        .collect();

    // We need to keep track of the next available variable for our helpers.
    // It must start after all the variables used for cells.
    let mut next_var = (unknown_indices.len() + 1) as isize;

    for constraint in local_constraints {
        let sat_vars: Vec<isize> = constraint
            .cells
            .iter()
            .filter_map(|idx| var_map.get(idx).copied())
            .collect();

        if sat_vars.is_empty() {
            continue;
        }

        add_sequential_encoding_clauses(
            &mut clauses,
            &mut next_var,
            &sat_vars,
            constraint.count as usize,
        );
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
