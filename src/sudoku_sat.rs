use std::error::Error;
use std::fmt;
use std::fs;
use std::io;
use std::path::Path;
use varisat::Lit;

const N: usize = 9; // The dimension of the grid (9x9)
pub struct SudokuGrid([[u8; N]; N]); // 0 represents an empty cell.
const BOX_SIZE: usize = 3; // The dimension of a sub-box (3x3)

// Error type for parsing
#[derive(Debug)]
pub enum SudokuParseError {
    IoError(io::Error),
    InvalidFormat(String),
}

impl From<io::Error> for SudokuParseError {
    fn from(error: io::Error) -> Self {
        SudokuParseError::IoError(error)
    }
}

impl Error for SudokuParseError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            SudokuParseError::IoError(e) => Some(e),
            SudokuParseError::InvalidFormat(_) => None,
        }
    }
}

impl fmt::Display for SudokuParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SudokuParseError::IoError(e) => write!(f, "IO error: {e}"),
            SudokuParseError::InvalidFormat(msg) => write!(f, "Invalid format: {msg}"),
        }
    }
}

impl SudokuGrid {
    /// Parse a Sudoku puzzle from text
    /// Accepts formats with or without spaces, using 0 or . for empty cells
    pub fn from_text(text: &str) -> Result<Self, SudokuParseError> {
        let mut grid = [[0u8; N]; N];

        // Remove all whitespace and collect digits/dots
        let chars: Vec<char> = text
            .chars()
            .filter(|&c| c.is_ascii_digit() || c == '.')
            .collect();

        let expected_count = N * N;
        if chars.len() != expected_count {
            return Err(SudokuParseError::InvalidFormat(format!(
                "Expected {expected_count} cells, found {}",
                chars.len()
            )));
        }

        for (idx, &ch) in chars.iter().enumerate() {
            let row = idx / N;
            let col = idx % N;

            let num = match ch {
                '.' | '0' => 0,
                '1'..='9' => ch.to_digit(10).unwrap() as u8,
                _ => {
                    return Err(SudokuParseError::InvalidFormat(format!(
                        "Invalid character '{ch}' at position {}",
                        idx + 1
                    )));
                }
            };

            grid[row][col] = num;
        }

        Ok(SudokuGrid(grid))
    }

    /// Read a Sudoku puzzle from a file
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, SudokuParseError> {
        let content = fs::read_to_string(path)?;
        Self::from_text(&content)
    }
}

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

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_from_text_with_dots() {
        let input = "
            ..3.2.6..
            9..3.5..1
            ..18.64..
            ..81.29..
            7.......8
            ..67.82..
            ..26.95..
            8..2.3..9
            ..5.1.3..";
        let result = SudokuGrid::from_text(input);
        assert!(result.is_ok());
    }
}
