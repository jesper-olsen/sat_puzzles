/// A library for solving logic puzzles using SAT techniques.
///
/// This crate provides tools to encode puzzles into Conjunctive Normal Form (CNF)
/// and find solutions using a SAT solver.
pub mod mapcolouring_sat;
pub mod minesweeper_sat;
pub mod nqueens_sat;
pub mod sudoku_sat;

use anyhow::Result;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;
use varisat::{ExtendFormula, Lit, Solver};

// number of variables - which is largest index
fn num_vars(clauses: &[Vec<isize>]) -> usize {
    clauses
        .iter()
        .flat_map(|clause| clause.iter())
        .map(|&lit| lit.unsigned_abs())
        .max()
        .unwrap_or(0) // Handle case with no clauses
}

pub fn write_clauses<P: AsRef<Path>>(output: P, clauses: &[Vec<isize>]) -> Result<()> {
    let num_vars = num_vars(clauses); //n * n;

    let file = File::create(&output)?;
    let mut writer = BufWriter::new(file);
    //let stdout = std::io::stdout();
    // lock stdout so we can borrow it safely
    //let mut writer = BufWriter::new(stdout.lock());

    writeln!(writer, "p cnf {num_vars} {}", clauses.len())?;
    for clause in clauses {
        for literal in clause {
            write!(writer, "{literal} ")?;
        }
        writeln!(writer, "0")?;
    }
    writer.flush()?;

    println!(
        "Successfully wrote problem to '{}' ({num_vars} variables, {} clauses)",
        output.as_ref().display(),
        clauses.len()
    );
    Ok(())
}

// holds the state needed to keep finding the next solution.
pub struct SolutionIterator<'a> {
    solver: Solver<'a>,
}

impl<'a> Iterator for SolutionIterator<'a> {
    type Item = Vec<Lit>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.solver.solve().unwrap_or(false) {
            let model = self.solver.model().expect("No model found");
            // block the exact same solution from being found again
            // !(l1 AND l2 ... and lN) = (!l1 OR !l2 OR ... OR !lN)
            let blocking_clause: Vec<Lit> = model.iter().map(|&lit| !lit).collect();
            self.solver.add_clause(&blocking_clause);
            Some(model)
        } else {
            None
        }
    }
}

/// Finds all solutions and returns them as a memory-efficient iterator.
pub fn find_all_solutions(clauses: &[Vec<isize>]) -> Result<SolutionIterator> {
    let mut solver = Solver::new();
    for clause in clauses {
        solver.add_clause(
            &clause
                .iter()
                .map(|&lit| Lit::from_dimacs(lit))
                .collect::<Vec<_>>(),
        );
    }
    Ok(SolutionIterator { solver })
}
