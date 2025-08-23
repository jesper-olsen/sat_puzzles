use anyhow::Result;
use varisat::{ExtendFormula, Lit, Solver};

// The N-Queens problem asks for all possible ways to place N chess queens on an N x N chessboard
// such that no two queens attack each other. In chess, a queen can attack any piece located on
// the same row, column, or diagonal. Therefore, the conditions for a valid placement are:
// 1) No two queens share the same row.
// 2) No two queens share the same column.
// 3) No two queens share the same positive diagonal (where row - column is constant).
// 4) No two queens share the same negative diagonal (where row + column is constant).

// A solution is represented as a vector of (row, col) coordinates for each queen.
pub type Solution = Vec<(usize, usize)>;

/// Helper to map a 0-indexed (row, col) to a 1-indexed DIMACS variable number.
// Changed to return isize to match Lit::from_dimacs requirement.
fn coords_to_var(r: usize, c: usize, n: usize) -> isize {
    (r * n + c + 1) as isize
}

/// Helper to map a 1-indexed DIMACS variable number back to 0-indexed (row, col).
// Changed to accept isize.
fn var_to_coords(var: usize, n: usize) -> (usize, usize) {
    let zero_based_var = var - 1;
    let r = zero_based_var / n;
    let c = zero_based_var % n;
    (r, c)
}

/// Generates the CNF clauses for the N-Queens problem.
pub fn generate_clauses(n: usize) -> Vec<Vec<isize>> {
    let mut clauses = Vec::new();

    // --- CONSTRAINT 1: At least one queen in each row ---
    // For each row r, the clause is (x_r,0 OR x_r,1 OR ... OR x_r,n-1)
    for r in 0..n {
        clauses.push((0..n).map(|c| coords_to_var(r, c, n)).collect());
    }

    // --- CONSTRAINT 2: At most one queen per row ---
    // For each row r, for each pair of columns c1, c2: (-x_r,c1 OR -x_r,c2)
    for r in 0..n {
        for c1 in 0..n {
            for c2 in (c1 + 1)..n {
                clauses.push(vec![-coords_to_var(r, c1, n), -coords_to_var(r, c2, n)]);
            }
        }
    }

    // --- CONSTRAINT 3: At most one queen per column ---
    // For each column c, for each pair of rows r1, r2: (-x_r1,c OR -x_r2,c)
    for c in 0..n {
        for r1 in 0..n {
            for r2 in (r1 + 1)..n {
                clauses.push(vec![-coords_to_var(r1, c, n), -coords_to_var(r2, c, n)]);
            }
        }
    }

    // --- CONSTRAINT 4: At most one queen per diagonal ---
    // For each pair of distinct cells (r1, c1) and (r2, c2) on the same diagonal,
    // add a clause (-x_r1,c1 OR -x_r2,c2).
    for r1 in 0..n {
        for c1 in 0..n {
            // Check squares down and to the right to avoid duplicates.
            // Main diagonals (r1-c1 = r2-c2)
            for i in 1..(n - r1).min(n - c1) {
                let r2 = r1 + i;
                let c2 = c1 + i;
                clauses.push(vec![-coords_to_var(r1, c1, n), -coords_to_var(r2, c2, n)]);
            }
            // Anti-diagonals (r1 + c1 = r2 + c2)
            for i in 1..=(n - r1 - 1).min(c1) {
                let r2 = r1 + i;
                let c2 = c1 - i;
                clauses.push(vec![-coords_to_var(r1, c1, n), -coords_to_var(r2, c2, n)]);
            }
        }
    }

    clauses
}

/// Finds all unique solutions for the N-Queens problem.
pub fn find_all_solutions(n: usize) -> Result<Vec<Solution>> {
    let mut solver = Solver::new();
    for clause in generate_clauses(n) {
        solver.add_clause(
            &clause
                .iter()
                .map(|&lit| Lit::from_dimacs(lit))
                .collect::<Vec<_>>(),
        );
    }

    let mut all_solutions = Vec::new();
    while solver.solve()? {
        let Some(model) = solver.model() else {
            break; // Should not happen if we got this far
        };
        let mut current_solution = Vec::new();
        let mut blocking_clause = Vec::new();

        for &lit in model.iter() {
            if lit.is_positive() {
                current_solution.push(var_to_coords(lit.var().to_dimacs() as usize, n));
            }
            // block the exact same solution from being found again
            // !(l1 AND l2 ... and lN) = (!l1 OR !l2 OR ... OR !lN)
            blocking_clause.push(!lit);
        }
        current_solution.sort(); // for deterministic output
        all_solutions.push(current_solution);
        solver.add_clause(&blocking_clause);
    }

    Ok(all_solutions)
}
