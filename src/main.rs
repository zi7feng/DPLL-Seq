mod lib;

use std::rc::Rc;
use lib::*;

fn main() {
    let path = "124test.cnf";
    let formula = read_cnf_file(path);
    let mut assignment = initial_assignment(&formula);
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
}