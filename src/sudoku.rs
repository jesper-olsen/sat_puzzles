use std::fmt;
use varisat::Lit;

const N: usize = 9; // The dimension of the grid (9x9)
pub struct SudokuGrid([[u8; N]; N]); // 0 represents an empty cell.
const BOX_SIZE: usize = 3; // The dimension of a sub-box (3x3)

pub const PUZZLE_EASY: SudokuGrid = SudokuGrid([
    [0, 0, 3, 0, 2, 0, 6, 0, 0],
    [9, 0, 0, 3, 0, 5, 0, 0, 1],
    [0, 0, 1, 8, 0, 6, 4, 0, 0],
    [0, 0, 8, 1, 0, 2, 9, 0, 0],
    [7, 0, 0, 0, 0, 0, 0, 0, 8],
    [0, 0, 6, 7, 0, 8, 2, 0, 0],
    [0, 0, 2, 6, 0, 9, 5, 0, 0],
    [8, 0, 0, 2, 0, 3, 0, 0, 9],
    [0, 0, 5, 0, 1, 0, 3, 0, 0],
]);

pub const PUZZLE_HARDER: SudokuGrid = SudokuGrid([
    [4, 1, 7, 3, 6, 9, 8, 0, 5],
    [0, 3, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 7, 0, 0, 0, 0, 0],
    [0, 2, 0, 0, 0, 0, 0, 6, 0],
    [0, 0, 0, 0, 8, 0, 4, 0, 0],
    [0, 0, 0, 0, 1, 0, 0, 0, 0],
    [0, 0, 0, 6, 0, 3, 0, 7, 0],
    [5, 0, 0, 2, 0, 0, 0, 0, 0],
    [1, 0, 4, 0, 0, 0, 0, 0, 0],
]);
pub const PUZZLE_HARD: SudokuGrid = SudokuGrid([
    [8, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 3, 6, 0, 0, 0, 0, 0],
    [0, 7, 0, 0, 9, 0, 2, 0, 0],
    [0, 5, 0, 0, 0, 7, 0, 0, 0],
    [0, 0, 0, 0, 4, 5, 7, 0, 0],
    [0, 0, 0, 1, 0, 0, 0, 3, 0],
    [0, 0, 1, 0, 0, 0, 0, 6, 8],
    [0, 0, 8, 5, 0, 0, 0, 1, 0],
    [0, 9, 0, 0, 0, 0, 4, 0, 0],
]);

/// Helper to map a 0-indexed (row, col, digit) to a 1-indexed DIMACS variable number.
/// A variable is true if cell (r, c) contains digit d.
/// Digits are 1-9.
fn coords_to_var(r: usize, c: usize, d: usize) -> isize {
    // r: 0-8, c: 0-8, d: 1-9
    // We map d from 1-9 to a 0-8 index for calculation.
    (r * N * N + c * N + (d - 1) + 1) as isize
}

/// Helper to map a 1-indexed DIMACS variable number back to 0-indexed (row, col, digit).
fn var_to_coords(var: usize) -> (usize, usize, usize) {
    let zero_based_var = var - 1;
    let r = zero_based_var / (N * N);
    let c = (zero_based_var / N) % N;
    let d = (zero_based_var % N) + 1; // Convert back to 1-9 digit
    (r, c, d)
}

/// Generates the CNF clauses for a Sudoku puzzle.
pub fn generate_clauses(initial_grid: &SudokuGrid) -> Vec<Vec<isize>> {
    let mut clauses = Vec::new();

    // --- CONSTRAINT 1: Each cell contains at least one digit ---
    // For each cell (r, c), add the clause (x_r,c,1 OR x_r,c,2 OR ... OR x_r,c,9)
    for r in 0..N {
        for c in 0..N {
            clauses.push((1..=N).map(|d| coords_to_var(r, c, d)).collect());
        }
    }

    // --- CONSTRAINT 2: Each cell contains at most one digit ---
    // For each cell (r, c) and each pair of digits d1, d2: (-x_r,c,d1 OR -x_r,c,d2)
    for r in 0..N {
        for c in 0..N {
            for d1 in 1..=N {
                for d2 in (d1 + 1)..=N {
                    clauses.push(vec![-coords_to_var(r, c, d1), -coords_to_var(r, c, d2)]);
                }
            }
        }
    }

    // --- CONSTRAINT 3: Each digit appears at most once in each row ---
    // For each row r, digit d, and pair of columns c1, c2: (-x_r,c1,d OR -x_r,c2,d)
    for r in 0..N {
        for d in 1..=N {
            for c1 in 0..N {
                for c2 in (c1 + 1)..N {
                    clauses.push(vec![-coords_to_var(r, c1, d), -coords_to_var(r, c2, d)]);
                }
            }
        }
    }

    // --- CONSTRAINT 4: Each digit appears at most once in each column ---
    // For each column c, digit d, and pair of rows r1, r2: (-x_r1,c,d OR -x_r2,c,d)
    for c in 0..N {
        for d in 1..=N {
            for r1 in 0..N {
                for r2 in (r1 + 1)..N {
                    clauses.push(vec![-coords_to_var(r1, c, d), -coords_to_var(r2, c, d)]);
                }
            }
        }
    }

    // --- CONSTRAINT 5: Each digit appears at most once in each 3x3 box ---
    for d in 1..=N {
        for br in 0..BOX_SIZE {
            // Box row
            for bc in 0..BOX_SIZE {
                // Box col
                let mut cells_in_box = Vec::new();
                for r_offset in 0..BOX_SIZE {
                    for c_offset in 0..BOX_SIZE {
                        let r = br * BOX_SIZE + r_offset;
                        let c = bc * BOX_SIZE + c_offset;
                        cells_in_box.push((r, c));
                    }
                }

                for i in 0..cells_in_box.len() {
                    for j in (i + 1)..cells_in_box.len() {
                        let (r1, c1) = cells_in_box[i];
                        let (r2, c2) = cells_in_box[j];
                        clauses.push(vec![-coords_to_var(r1, c1, d), -coords_to_var(r2, c2, d)]);
                    }
                }
            }
        }
    }

    // Note: The "at least one" constraint for rows, columns, and boxes is implicitly satisfied
    // by the combination of "each cell has a number" and "at most one of each number per region".

    // --- CONSTRAINT 6: Add clauses for the pre-filled numbers (the puzzle seed) ---
    for r in 0..N {
        for c in 0..N {
            if initial_grid.0[r][c] != 0 {
                let d = initial_grid.0[r][c] as usize;
                // This is a unit clause, forcing the variable to be true.
                clauses.push(vec![coords_to_var(r, c, d)]);
            }
        }
    }

    clauses
}

pub fn decode_solution(model: &[Lit]) -> SudokuGrid {
    let mut current_solution = SudokuGrid([[0; N]; N]); // 0 represents an empty cell.
    for &lit in model.iter() {
        if lit.is_positive() {
            let (r, c, d) = var_to_coords(lit.var().to_dimacs() as usize);
            current_solution.0[r][c] = d as u8;
        }
    }
    current_solution
}

impl fmt::Display for SudokuGrid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "┌───────┬───────┬───────┐")?;

        for (r, row_data) in self.0.iter().enumerate() {
            write!(f, "│")?;
            for (c, &cell_value) in row_data.iter().enumerate() {
                if c > 0 && c % 3 == 0 {
                    write!(f, " │")?;
                }

                let ch = if cell_value == 0 {
                    '·'
                } else {
                    (b'0' + cell_value) as char
                };
                write!(f, " {ch}")?;
            }
            writeln!(f, " │")?;

            if r < 8 && (r + 1) % 3 == 0 {
                writeln!(f, "├───────┼───────┼───────┤")?;
            }
        }

        write!(f, "└───────┴───────┴───────┘")
    }
}
