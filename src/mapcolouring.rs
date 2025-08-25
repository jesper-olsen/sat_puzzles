use std::collections::HashMap;
use std::fmt;
use varisat::Lit;

pub struct Colouring<'a>(HashMap<&'a str, &'a str>);

impl<'a> fmt::Display for Colouring<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // We need to sort the keys to get a deterministic print order
        let mut sorted_sol: Vec<_> = self.0.iter().collect();
        sorted_sol.sort_by_key(|&(state, _)| *state);
        for (state, colour) in sorted_sol {
            writeln!(f, "{state}: {colour}")?;
        }
        Ok(())
    }
}

/// Maps a SAT variable (as a 1-based integer) back to its corresponding
/// (state, color) pair.
fn var_to_state_color<'a>(
    var_dimacs: usize,
    states: &[&'a str],
    colors: &[&'a str],
) -> (&'a str, &'a str) {
    let num_colors = colors.len();
    // Convert from 1-based DIMACS format to 0-based index
    let index = var_dimacs - 1;

    let state_idx = index / num_colors;
    let color_idx = index % num_colors;

    (states[state_idx], colors[color_idx])
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
    states: &[&str],
    colors: &[&str],
    adjacencies: &HashMap<&str, Vec<&str>>,
) -> Vec<Vec<isize>> {
    let mut clauses = Vec::new();
    let num_states = states.len();
    let num_colors = colors.len();

    // Helper maps for easy lookup of indices
    let state_to_idx: HashMap<&str, usize> =
        states.iter().enumerate().map(|(i, &s)| (s, i)).collect();

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
    for (&s1_name, neighbors) in adjacencies {
        let s1_idx = state_to_idx[s1_name];
        for &s2_name in neighbors {
            // To avoid duplicate clauses (e.g., SA-WA and WA-SA), we only process
            // an edge if the name of the first state is lexicographically smaller.
            if s1_name < s2_name {
                let s2_idx = state_to_idx[s2_name];
                for c_idx in 0..num_colors {
                    clauses.push(vec![-var(s1_idx, c_idx), -var(s2_idx, c_idx)]);
                }
            }
        }
    }

    clauses
}

pub fn decode_solution<'a>(
    model: &[Lit],
    states: &'a [&'a str],
    colors: &'a [&'a str],
) -> Colouring<'a> {
    let mut current_solution = HashMap::new();

    for &lit in model.iter().filter(|l| l.is_positive()) {
        let (state, color) = var_to_state_color(lit.var().to_dimacs() as usize, states, colors);
        current_solution.insert(state, color);
    }

    Colouring(current_solution)
}
// usa_csp = MapColouringCSP(list('RGBY'),
//                          """WA: OR ID; OR: ID NV CA; CA: NV AZ; NV: ID UT AZ; ID: MT WY UT;
//                          UT: WY CO AZ; MT: ND SD WY; WY: SD NE CO; CO: NE KA OK NM; NM: OK TX AZ;
//                          ND: MN SD; SD: MN IA NE; NE: IA MO KA; KA: MO OK; OK: MO AR TX;
//                          TX: AR LA; MN: WI IA; IA: WI IL MO; MO: IL KY TN AR; AR: MS TN LA;
//                          LA: MS; WI: MI IL; IL: IN KY; IN: OH KY; MS: TN AL; AL: TN GA FL;
//                          MI: OH IN; OH: PA WV KY; KY: WV VA TN; TN: VA NC GA; GA: NC SC FL;
//                          PA: NY NJ DE MD WV; WV: MD VA; VA: MD DC NC; NC: SC; NY: VT MA CT NJ;
//                          NJ: DE; DE: MD; MD: DC; VT: NH MA; MA: NH RI CT; CT: RI; ME: NH;
//                          HI: ; AK: """)

// france_csp = MapColouringCSP(list('RGBY'),
//                             """AL: LO FC; AQ: MP LI PC; AU: LI CE BO RA LR MP; BO: CE IF CA FC RA
//                             AU; BR: NB PL; CA: IF PI LO FC BO; CE: PL NB NH IF BO AU LI PC; FC: BO
//                             CA LO AL RA; IF: NH PI CA BO CE; LI: PC CE AU MP AQ; LO: CA AL FC; LR:
//                             MP AU RA PA; MP: AQ LI AU LR; NB: NH CE PL BR; NH: PI IF CE NB; NO:
//                             PI; PA: LR RA; PC: PL CE LI AQ; PI: NH NO CA IF; PL: BR NB CE PC; RA:
//                             AU BO FC PA LR""")
