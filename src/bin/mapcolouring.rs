use anyhow::Result;
use clap::{Parser, Subcommand};
use sat_puzzles::mapcolouring::{Colouring, decode_solution, generate_clauses, load_map_from_file};
use std::path::PathBuf;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate a DIMACS CNF file for the map colouring problem
    Generate {
        /// Path to the map definition file
        map_file: PathBuf,
    },
    /// Solve the N-Queens problem and visualize the solution(s)
    Solve {
        /// Path to the map definition file
        map_file: PathBuf,
        /// Find all possible solutions instead of just one
        #[arg(short, long)]
        all: bool,
    },
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

fn main() -> Result<()> {
    let colours = ["R", "G", "B"];
    // let states_australia = ["WA", "NT", "SA", "Q", "NSW", "V", "T"];
    // let adjacencies_australia: HashMap<&str, Vec<&str>> = [
    //     ("SA", vec!["WA", "NT", "Q", "NSW", "V"]),
    //     ("NT", vec!["WA", "Q", "SA"]),
    //     ("NSW", vec!["Q", "SA", "V"]),
    //     ("WA", vec!["NT", "SA"]),
    //     ("Q", vec!["NT", "SA", "NSW"]),
    //     ("V", vec!["SA", "NSW"]),
    //     ("T", vec![]),
    // ]
    // .iter()
    // .cloned()
    // .collect();

    let cli = Cli::parse();

    match &cli.command {
        Commands::Generate { map_file } => {
            println!("Generating CNF for map: {map_file:?}");
            let (states, adjacencies) = load_map_from_file(map_file)?;

            // Our existing functions expect slices of &str, so we create them from our owned Strings.
            let states_ref: Vec<&str> = states.iter().map(AsRef::as_ref).collect();
            let adjacencies_ref: std::collections::HashMap<&str, Vec<&str>> = adjacencies
                .iter()
                .map(|(k, v)| (k.as_str(), v.iter().map(AsRef::as_ref).collect()))
                .collect();

            let clauses = generate_clauses(&states_ref, &colours, &adjacencies_ref);
            let output = "map.cnf";
            sat_puzzles::write_clauses(output, &clauses)?;
        }
        Commands::Solve { map_file, all } => {
            println!("Solving map colouring for: {map_file:?}");
            let (states, adjacencies) = load_map_from_file(map_file)?;

            // Our existing functions expect slices of &str, so we create them from our owned Strings.
            let states_ref: Vec<&str> = states.iter().map(AsRef::as_ref).collect();
            let adjacencies_ref: std::collections::HashMap<&str, Vec<&str>> = adjacencies
                .iter()
                .map(|(k, v)| (k.as_str(), v.iter().map(AsRef::as_ref).collect()))
                .collect();

            let clauses = generate_clauses(&states_ref, &colours, &adjacencies_ref);

            let raw_solutions_iterator = sat_puzzles::find_all_solutions(&clauses)?;

            let solutions: Vec<Colouring> = if *all {
                raw_solutions_iterator
                    .map(|model| decode_solution(&model, &states_ref, &colours))
                    .collect()
            } else {
                raw_solutions_iterator
                    .take(1)
                    .map(|model| decode_solution(&model, &states_ref, &colours))
                    .collect()
            };

            if solutions.is_empty() {
                println!("No solutions found for map of {map_file:?}");
            } else if *all {
                println!(
                    "Found {} unique solutions for map of {map_file:?}",
                    solutions.len()
                );
                for (i, sol) in solutions.iter().enumerate() {
                    println!("\n--- Solution {solution_num} ---", solution_num = i + 1);
                    println!("{sol}");
                }
            } else {
                println!("Found a solution for Sudoku {map_file:?}");
                println!("{}", solutions[0]);
            }
        }
    }

    Ok(())
}
