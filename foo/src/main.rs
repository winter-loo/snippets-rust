// (1, 2), (2, 3), (3, 1), (2, 4), (2, 4)
// 1,2,3,4

// 1 -> 2, 3
// 2 -> 3, 1, 4
// 3 -> 2, 1
// 4 -> 2


//  1 <--> 2 <--> 4
// /|\    /|\
//  |      |
//  |     \|/
//  +----->3
use std::collections::HashMap;
pub fn chain(input: &[(u8, u8)]) -> Option<Vec<(u8, u8)>> {
    let mut graph: HashMap<u8, HashMap<u8, u32>> = HashMap::new();

    for (n1, n2) in input {
        graph.entry(*n1).and_modify(|m| {
            m.entry(*n2).and_modify(|i| *i += 1).or_insert(1);
        }).or_insert(HashMap::from([(*n2, 1u32)]));
        if n1 != n2 {
            graph.entry(*n2).and_modify(|m| {
                m.entry(*n1).and_modify(|i| *i += 1).or_insert(1);
            }).or_insert(HashMap::from([(*n1, 1u32)]));
        }
    }

    println!("graph: {graph:#?}");

    let nodes: Vec<_> = graph.keys().copied().collect();

    for node in nodes {
        let mut path: Vec<(u8, u8)> = vec![];
        walk_graph(node, node, &mut graph, &mut path);
        if !path.is_empty() {
            return Some(path);
        }
    }

    None
}

fn walk_graph(initial: u8, node: u8, graph: &mut HashMap<u8, HashMap<u8, u32>>, path: &mut Vec<(u8, u8)>) {
    println!("initial: {initial}, node: {node}, graph: {:?}, path: {:?}", graph, path);
    if path.len() == graph.len() && initial == node {
        return;
    }
    // use `Option::cloned()` to avoid the immutable/mutable issue
    let edges = graph.get(&node).cloned().unwrap_or_default();
    for (adjacent_node, count) in edges {
        if count == 0 {
            continue;
        }
        *graph.get_mut(&node).unwrap().get_mut(&adjacent_node).unwrap() -= 1;
        *graph.get_mut(&adjacent_node).unwrap().get_mut(&node).unwrap() -= 1;

        path.push((node, adjacent_node));
        walk_graph(initial, adjacent_node, graph, path);
        path.pop();

        *graph.get_mut(&node).unwrap().get_mut(&adjacent_node).unwrap() += 1;
        *graph.get_mut(&adjacent_node).unwrap().get_mut(&node).unwrap() += 1;
    }
}
