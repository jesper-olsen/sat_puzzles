pub mod nqueens;
pub mod sudoku;

use anyhow::Result;
use std::collections::HashSet;
use std::fs::File;
use std::io::{BufWriter, Write};
use varisat::{ExtendFormula, Lit, Solver};

fn num_vars(clauses: &[Vec<isize>]) -> usize {
    let mut set = HashSet::new();
    for clause in clauses {
        for &lit in clause {
            set.insert(lit.abs());
        }
    }
    set.len()
}

pub fn write_clauses(output: &str, clauses: &[Vec<isize>]) -> Result<()> {
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
        "Successfully wrote problem to '{output}' ({num_vars} variables, {} clauses)",
        clauses.len()
    );
    Ok(())
}

// pub fn find_all_solutions(clauses: &[Vec<isize>]) -> Result<Vec<Vec<Lit>>> {
//     let mut solver = Solver::new();
//     for clause in clauses {
//         solver.add_clause(
//             &clause
//                 .iter()
//                 .map(|&lit| Lit::from_dimacs(lit))
//                 .collect::<Vec<_>>(),
//         );
//     }

//     let mut all_solutions = Vec::new();
//     while solver.solve()? {
//         let model = solver
//             .model()
//             .expect("Solver returned true but no model found.");

//         // block the exact same solution from being found again
//         // !(l1 AND l2 ... and lN) = (!l1 OR !l2 OR ... OR !lN)
//         let blocking_clause: Vec<Lit> = model.iter().map(|&lit| !lit).collect();
//         solver.add_clause(&blocking_clause);

//         all_solutions.push(model);
//     }

//     Ok(all_solutions)
// }

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
