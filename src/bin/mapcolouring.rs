use anyhow::Result;
use sat_puzzles::mapcolouring::{Colouring, decode_solution, generate_clauses};
use std::collections::HashMap;

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
    let states = ["WA", "NT", "SA", "Q", "NSW", "V", "T"];
    let adjacencies: HashMap<&str, Vec<&str>> = [
        ("SA", vec!["WA", "NT", "Q", "NSW", "V"]),
        ("NT", vec!["WA", "Q", "SA"]),
        ("NSW", vec!["Q", "SA", "V"]),
        ("WA", vec!["NT", "SA"]),
        ("Q", vec!["NT", "SA", "NSW"]),
        ("V", vec!["SA", "NSW"]),
        ("T", vec![]),
    ]
    .iter()
    .cloned()
    .collect();

    let clauses = generate_clauses(&states, &colours, &adjacencies);
    let raw_solutions_iterator = sat_puzzles::find_all_solutions(&clauses)?;
    let all = true;
    let solutions: Vec<Colouring> = if all {
        raw_solutions_iterator
            .map(|model| decode_solution(&model, &states, &colours))
            .collect()
    } else {
        raw_solutions_iterator
            .take(1)
            .map(|model| decode_solution(&model, &states, &colours))
            .collect()
    };

    println!("Found {} unique colourings.", solutions.len());
    // Print the first 5 solutions for brevity
    for (i, sol) in solutions.iter().take(5).enumerate() {
        println!("--- Solution {} ---", i + 1);
        println!("{sol}");
    }
    Ok(())
}
