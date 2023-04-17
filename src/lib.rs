use std::collections::HashMap;
use std::fs::File;
use std::hash::Hash;
use std::io::{BufRead, BufReader};
use std::rc::Rc;

pub fn read_cnf_file(path: &str) -> Vec<Vec<i32>> {
    let file = File::open(path).expect("Failed to open file");
    let reader = BufReader::new(file);
    let mut clauses = Vec::new();
    for line in reader.lines() {
        let line = line.expect("Failed to read line");
        let tokens: Vec<&str> = line.trim().split_whitespace().collect();
        if tokens.is_empty() || tokens[0] == "c" {
            // Skip comments and empty lines
            continue;
        } else if tokens[0] == "p" {
            // Parse problem line
            assert_eq!(tokens.len(), 4, "Invalid problem line");
            assert_eq!(tokens[1], "cnf", "Invalid problem line");
            continue;
        } else {
            // Parse clauses
            let mut clause = Vec::new();
            for token in &tokens {
                if *token == "0" {
                    // End of clause
                    clauses.push(clause);
                    clause = Vec::new();
                } else {
                    // Parse literal
                    let lit: i32 = token.parse().expect("Failed to parse literal");
                    clause.push(lit);
                }
            }
        }
    }

    clauses
}

// Create an initial assignment for the literals in the CNF formula
pub fn initial_assignment(formula: &Vec<Vec<i32>>) -> HashMap<i32, Option<bool>> {
    let mut assignment = HashMap::new();
    for clause in formula.iter() {
        for &lit in clause.iter() {
            let key = lit.abs();
            if !assignment.contains_key(&key) {
                // Assign a random truth value to the literal
                assignment.insert(key, None);
            }
        }
    }
    assignment
}

// Remove pure literals from the CNF formula
pub fn pure_literal_elimination(formula: &Vec<Vec<i32>>, assignment: &mut HashMap<i32, Option<bool>>) -> Vec<Vec<i32>>{
    let mut pure_literals = HashMap::new();
    let mut removed_literals = HashMap::new();
    let mut new_formula = Vec::new();

    //Find all pure literals in the formula
    for clause in formula.iter() {
        for &lit in clause.iter() {
            let key = lit.abs();
            if !removed_literals.contains_key(&key) && !removed_literals.contains_key(&-key) {
                // First occurrence of the literal in the formula
                if !pure_literals.contains_key(&key) && !pure_literals.contains_key(&-key) {
                    pure_literals.insert(lit, lit > 0);
                } else if pure_literals.contains_key(&-lit) {
                    // The literal has opposite polarity to a pure literal
                    removed_literals.insert(key, false);
                    removed_literals.insert(-key, false);
                    pure_literals.remove(&-lit);
                }
            }
        }
    }

    // Assign values to pure literals
    for (key, val) in pure_literals.iter() {
        let abs_key = &key.abs();
        assignment.insert(*abs_key, Some(*val));
    }

    // Simplify the formula with new assignment
    new_formula = simplify_formula(formula, assignment);

    new_formula
}

// Struct of the node in a tree
#[derive(Clone)]
pub struct Node {
    pub formula: Vec<Vec<i32>>,
    pub value: Option<bool>,
    pub variable: i32,
    pub assignment: HashMap<i32, Option<bool>>,
}

impl Node {
    // Create a new node
    pub fn new(
        formula: Vec<Vec<i32>>,
        value: Option<bool>,
        variable: i32,
        assignment: HashMap<i32, Option<bool>>
    ) -> Self {
        Node {
            formula,
            value,
            variable,
            assignment,
        }
    }
}

fn simplify_formula(formula: &Vec<Vec<i32>>, assignment: &HashMap<i32, Option<bool>>) -> Vec<Vec<i32>> {
    let mut new_formula = Vec::new();
    for clause in formula.iter() {
        let mut satisfied = false;
        for &lit in clause.iter() {
            if lit > 0 {
                if assignment.get(&lit) == Some(&Some(true)) {
                    // The literal is already satisfied
                    satisfied = true;
                    break;
                }
            } else {
                if assignment.get(&lit.abs()) == Some(&Some(false)) {
                    // The literal is already satisfied
                    satisfied = true;
                    break;
                }
            }
        }
        if !satisfied {
            // Add the clause to the new formula if it is not already satisfied
            new_formula.push(clause.clone());
        }
    }
    new_formula
}

// Check whether the node could continue
fn false_check(node: &Rc<Node>) -> i32 {
    let formula = node.formula.clone();
    let mut true_num = 0;
    for clause in formula.iter() {
        let mut false_num = 0;
        let mut lit_num = 0;
        let mut true_flag = false;
        for &lit in clause.iter() {
            lit_num += 1;
            if lit > 0 {
                if lit == node.variable {
                    if node.value == Some(true) {
                        true_flag = true;
                        break;
                    } else {
                        false_num += 1;
                    }
                } else if node.assignment.get(&lit) == Some(&None) {
                    continue;
                } else if node.assignment.get(&lit) == Some(&Some(true)) {
                    true_flag = true;
                    break;
                } else if node.assignment.get(&lit) == Some(&Some(false)) {
                    false_num += 1;
                }
            } else {
                // println!("lit = {}, v = {}", lit, node.variable);
                if lit == -node.variable {
                    if node.value == Some(false) {
                        true_flag = true;
                        break;
                    } else {
                        false_num += 1;
                    }
                } else if node.assignment.get(&lit.abs()) == Some(&None) {
                    continue;
                } else if node.assignment.get(&lit.abs()) == Some(&Some(false)) {
                    true_flag = true;
                    break;
                } else if node.assignment.get(&lit.abs()) == Some(&Some(true)) {
                    false_num += 1;
                }
            }

        }
        // One clause is false
        if false_num == lit_num {
            return 0;
        }

        if true_flag {
            true_num += 1;
        }

    }

    // All clauses are true
    if true_num == formula.len() {
        return 2;
    }

    // Pass
    1
}

// Add a task to the task list
fn add_task(node: Rc<Node>, tasklist: &mut Vec<Rc<Node>>) {
    tasklist.push(node);
}

// Get a task from the task list
pub fn get_task(tasklist: &mut Vec<Rc<Node>>) -> Option<Rc<Node>> {
    tasklist.pop()
}

// Get all keys from the assignment and put them into a vector in order
fn get_assignment_keys(assignment: &HashMap<i32, Option<bool>>) -> Vec<i32> {
    let mut keys = assignment.iter().filter(|(_, val)| val.is_none())
        .map(|(&key, _)| key)
        .collect::<Vec<_>>();
    keys.sort_unstable();
    keys
}

// build a tree from the root
pub fn build_search_tree(node: Rc<Node>, tasklist: &mut Vec<Rc<Node>>) -> bool {
    if node.variable == 0 {
        let unassigned_var = get_assignment_keys(&node.assignment);
        let node_t = Rc::new(Node {
            formula: node.formula.clone(),
            value: Some(true),
            variable: unassigned_var[0],
            assignment: node.assignment.clone(),
        });
        let node_f = Rc::new(Node {
            formula: node.formula.clone(),
            value: Some(false),
            variable: unassigned_var[0],
            assignment: node.assignment.clone(),
        });
        add_task(node_f, tasklist);
        return build_search_tree(node_t, tasklist);
    } else if false_check(&node) == 0 {
        return false;
    } else if false_check(&node) == 2 {
        let mut solution = node.assignment.clone();
        solution.insert(node.variable, node.value);
        for (_, val) in solution.iter_mut() {
            if val.is_none() {
                *val = Some(true);
            }
        }
        for (key, value) in solution {
            println!("{}: {:?}", key, value);
        }
        // find a solution
        return true;
    } else {
        // let new_formula = simplify_formula(&node.formula, &node.assignment);
        // println!("formula of Node {}:{} is: {:?}",node.variable, node.value.unwrap(),new_formula.clone());

        let mut new_assignment = node.assignment.clone();
        new_assignment.insert(node.variable, node.value);
        let new_formula = simplify_formula(&node.formula, &new_assignment);
        let unassigned_var = get_assignment_keys(&new_assignment);
        let node_t = Rc::new(Node {
            formula: new_formula.clone(),
            value: Some(true),
            variable: unassigned_var[0],
            assignment: new_assignment.clone(),
        });
        let node_f = Rc::new(Node {
            formula: new_formula.clone(),
            value: Some(false),
            variable: unassigned_var[0],
            assignment: new_assignment.clone(),
        });
        // println!("node_f {}:{} ass: {:?}", node_f.variable, node_f.value.unwrap(), node_f.assignment);
        add_task(node_f, tasklist);
        return build_search_tree(node_t, tasklist);
    }
}



#[cfg(test)]
mod tests {
    use std::mem::transmute;
    use super::*;
    use maplit::hashmap;
    //
    // #[test]
    // fn test_read_cnf_file() {
    //     let path = "500250.cnf";
    //     let expected = vec![
    //         vec![50, 136, 36],
    //         vec![-250, -113, 17],
    //         vec![236, -241, -219],
    //         vec![-25, -205, 168]
    //     ];
    //     let result = read_cnf_file(path);
    //     println!("{:?}", result);
    //     assert_eq!(result, expected);
    // }

    #[test]
    fn test_initial_assignment() {
        let formula = vec![
            vec![1, 2, 3],
            vec![-1, -2],
            vec![-3, 4],
        ];
        let expected = hashmap! {
            1 => None,
            2 => None,
            3 => None,
            4 => None,
        };
        let result = initial_assignment(&formula);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_pure_literal_elimination_and_simplify() {
        let formula = vec![
            vec![1, 2],
            vec![2, 3],
            vec![-4, -2, 3],
            vec![-1, -2, 3],
            vec![-4, 2, 3],
        ];
        let mut assignment = hashmap! {
            1 => None,
            2 => None,
            3 => None,
            4 => None,
        };
        let new_formula = pure_literal_elimination(&formula, &mut assignment);
        assert_eq!(assignment.get(&1), Some(&None));
        assert_eq!(assignment.get(&2), Some(&None));
        assert_eq!(assignment.get(&3), Some(&Some(true)));
        assert_eq!(assignment.get(&4), Some(&Some(false)));
        assert_eq!(
            new_formula,
            vec![vec![1, 2]]
        );
    }

    #[test]
    fn test_false_check() {
        let formula = vec![
            vec![1, 2],
            vec![2, 3],
            vec![-4, -2, 3],
            vec![-1, -2, 3],
            vec![-4, 2, 3],
        ];
        let mut assignment = hashmap! {
            1 => None,
            2 => Some(true),
            3 => Some(false),
            4 => Some(false),
        };
        let node = Rc::new(Node::new(formula, Some(false), 1, assignment));
        assert_eq!(false_check(&node), 2);
    }

    #[test]
    fn test_add_and_get_task() {
        let mut tasklist: Vec<Rc<Node>> = vec![];

        let node1 = Rc::new(Node {
            formula: vec![vec![1, -2], vec![-1, 3], vec![-3, -4]],
            value: None,
            variable: 1,
            assignment: HashMap::new(),
        });

        let node2 = Rc::new(Node {
            formula: vec![vec![-2, 3], vec![-1, 3], vec![1, 2]],
            value: None,
            variable: 2,
            assignment: HashMap::new(),
        });

        let node3 = Rc::new(Node {
            formula: vec![vec![-1, 3], vec![-1, 3], vec![1, 2]],
            value: None,
            variable: 2,
            assignment: HashMap::new(),
        });


        add_task(node1.clone(), &mut tasklist);
        add_task(node2.clone(), &mut tasklist);
        add_task(node3.clone(), &mut tasklist);

        let popped_node = get_task(&mut tasklist).unwrap();

        let popped_node = get_task(&mut tasklist).unwrap();
        assert_eq!(popped_node.variable, 2);

        let popped_node = get_task(&mut tasklist).unwrap();
        assert_eq!(popped_node.variable, 1);



        assert!(tasklist.is_empty());
    }

    #[test]
    fn test_get_assignment_keys() {
        let mut assignment = hashmap! {
            4 => Some(false),
            2 => None,
            1 => None,
            3 => None,
        };
        let keys = get_assignment_keys(&assignment);
        assert_eq!(keys, vec![1, 2, 3]);
    }




}
