use std::collections::{HashMap, HashSet};
use std::fmt;
use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;
use varisat::Lit;

/// Loads map data from a file and creates a symmetric adjacency map.
///
/// The file format should be: `STATE_CODE:NEIGHBOR1 NEIGHBOR2 ...`
///
/// # Returns
/// A tuple containing:
/// 1. A sorted Vec of all unique state names (owned Strings).
/// 2. A HashMap mapping each state to its neighbors.
pub fn load_map_from_file(path: &Path) -> io::Result<(Vec<String>, HashMap<String, Vec<String>>)> {
    let file = File::open(path)?;
    let reader = io::BufReader::new(file);

    // Step 1: Read all states and defined adjacencies from the file
    let mut all_states = HashSet::new();
    let mut parsed_adjacencies = HashMap::new();

    for line in reader.lines() {
        let line = line?;
        let trimmed = line.trim();

        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        let parts: Vec<&str> = trimmed.split(':').collect();
        if parts.len() != 2 {
            eprintln!("Warning: Skipping malformed line: {trimmed}");
            continue;
        }

        let state = parts[0].trim().to_string();
        all_states.insert(state.clone());

        let neighbors_str = parts[1].trim();
        let neighbors: Vec<String> = if neighbors_str.is_empty() {
            vec![]
        } else {
            neighbors_str
                .split_whitespace()
                .map(|s| s.to_string())
                .collect()
        };

        for neighbor in &neighbors {
            all_states.insert(neighbor.clone());
        }

        parsed_adjacencies.insert(state, neighbors);
    }

    // Step 2: Build the final, symmetric adjacency map
    let mut final_adjacencies = HashMap::new();
    for state in &all_states {
        // Use a HashSet to automatically handle duplicate neighbors
        let mut neighbors_set = HashSet::new();

        // Add neighbors defined in the file for this state
        if let Some(neighbors) = parsed_adjacencies.get(state) {
            for neighbor in neighbors {
                neighbors_set.insert(neighbor.clone());
            }
        }

        // Add this state to the neighbor list of other states
        // (This enforces symmetry)
        for (other_state, other_neighbors) in &parsed_adjacencies {
            if other_neighbors.contains(state) {
                neighbors_set.insert(other_state.clone());
            }
        }

        let mut final_neighbors: Vec<String> = neighbors_set.into_iter().collect();
        final_neighbors.sort(); // For deterministic output
        final_adjacencies.insert(state.clone(), final_neighbors);
    }

    let mut sorted_states: Vec<String> = all_states.into_iter().collect();
    sorted_states.sort();

    Ok((sorted_states, final_adjacencies))
}

pub struct Colouring(pub HashMap<String, String>);

impl fmt::Display for Colouring {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut sorted_sol: Vec<_> = self.0.iter().collect();
        sorted_sol.sort_by_key(|(state, _)| *state); // state is now &String
        for (state, colour) in sorted_sol {
            writeln!(f, "{state}: {colour}")?;
        }
        Ok(())
    }
}

/// Generates CNF clauses for the Map Colouring problem.
///
/// # Arguments
/// * `states` - A list of state names.
/// * `colors` - A list of available colors.
/// * `adjacencies` - A map from a state to a list of its neighbors.
///
/// # Returns
/// A vector of clauses, where each clause is a vector of integers.
/// A positive integer `k` represents a variable, and a negative integer `-k` its negation.
pub fn generate_clauses(
    states: &[String],
    colors: &[String],
    adjacencies: &HashMap<String, Vec<String>>,
) -> Vec<Vec<isize>> {
    let mut clauses = Vec::new();
    let num_states = states.len();
    let num_colors = colors.len();

    // Helper maps for easy lookup of indices
    let state_to_idx: HashMap<&str, usize> = states
        .iter()
        .enumerate()
        .map(|(i, s)| (s.as_str(), i)) // s is a &String, s.as_str() is a &str
        .collect();

    // Helper function to map a (state, color) pair to a unique integer variable.
    // Variables are 1-based, which is standard for SAT solvers (DIMACS format).
    let var = |state_idx: usize, color_idx: usize| -> isize {
        (state_idx * num_colors + color_idx + 1) as isize
    };

    // --- CONSTRAINT 1: Each state must have at least one color ---
    // For each state `s`, the clause is (V_s,c1 OR V_s,c2 OR ...).
    for s_idx in 0..num_states {
        let clause: Vec<isize> = (0..num_colors).map(|c_idx| var(s_idx, c_idx)).collect();
        clauses.push(clause);
    }

    // --- CONSTRAINT 2: Each state has at most one color ---
    // For each state `s` and each pair of colors `c1, c2`: (-V_s,c1 OR -V_s,c2).
    for s_idx in 0..num_states {
        for c1_idx in 0..num_colors {
            for c2_idx in (c1_idx + 1)..num_colors {
                clauses.push(vec![-var(s_idx, c1_idx), -var(s_idx, c2_idx)]);
            }
        }
    }

    // --- CONSTRAINT 3: Adjacent states cannot have the same color ---
    // For each pair of adjacent states `s1, s2` and each color `c`: (-V_s1,c OR -V_s2,c).
    for (s1_name, neighbors) in adjacencies {
        let s1_idx = state_to_idx[s1_name.as_str()];
        for s2_name in neighbors {
            if s1_name < s2_name {
                // Comparing a &String and &String works
                let s2_idx = state_to_idx[s2_name.as_str()];
                for c_idx in 0..num_colors {
                    clauses.push(vec![-var(s1_idx, c_idx), -var(s2_idx, c_idx)]);
                }
            }
        }
    }

    clauses
}

/// Decodes a SAT model into an owned Colouring solution.
pub fn decode_solution(model: &[Lit], states: &[String], colors: &[String]) -> Colouring {
    fn var_to_state_color<'a>(
        var_dimacs: usize,
        states: &'a [String],
        colors: &'a [String],
    ) -> (&'a str, &'a str) {
        let num_colors = colors.len();
        let index = var_dimacs - 1;
        let state_idx = index / num_colors;
        let color_idx = index % num_colors;
        (states[state_idx].as_str(), colors[color_idx].as_str())
    }

    let solution_map = model
        .iter()
        .filter(|lit| lit.is_positive())
        .map(|lit| {
            let (state, color) = var_to_state_color(lit.var().to_dimacs() as usize, states, colors);
            // Convert to owned Strings for the final result
            (state.to_string(), color.to_string())
        })
        .collect::<HashMap<_, _>>();

    Colouring(solution_map)
}
