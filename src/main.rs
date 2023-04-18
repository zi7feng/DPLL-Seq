mod lib;

use std::rc::Rc;
use std::time::Instant;
use lib::*;
use rayon::prelude::*;

fn main() {
    let path = "700.cnf";
    let formula = read_cnf_file(path);
    let mut assignment = initial_assignment(&formula);

    // Start the timer
    let start_time = Instant::now();

    let simplified_formula = pure_literal_elimination(&formula, &mut assignment);
    let root = Rc::new(Node::new(
        simplified_formula,
        None,
        0,
        assignment.clone(),
    ));
    let mut tasklist: Vec<Rc<Node>> = Vec::new();
    tasklist.push(root);
    let mut flag = false;
    while !tasklist.is_empty() {
        let node = get_task(&mut tasklist).unwrap();
        let c = build_search_tree(node.clone(), &mut tasklist);
        if c {
            flag = true;
            break;
        }
    }
    if flag == false {
        println!("UNSATISFIED");
    }

    // Stop the timer
    let end_time = Instant::now();

    let elapsed_time = end_time.duration_since(start_time).as_secs_f64() * 1000.0;
    println!("Elapsed time: {:.3} milliseconds", elapsed_time);

}