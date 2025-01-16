use std::collections::VecDeque;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct Node {
    pub address: Option<String>,
    pub stake: f64,
    pub children: Vec<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TreeUpdate {}

#[derive(Debug)]
pub(crate) struct EpidemicTree {
    pub nodes: Vec<Node>,
}

impl EpidemicTree {
    fn new(addresses: Vec<String>) -> Self {
        let total_nodes = addresses.len();
        let max_height = 4;

        // Calculate the number of children per node based on total nodes
        let children_count = match total_nodes {
            0..=40 => 3,
            _ => {
                // General case: calculate children needed to fit nodes in max_height
                let mut count = 4 as usize; // Start with 4 children
                while count.pow(max_height as u32 - 1) < total_nodes {
                    count += 1;
                }
                count
            }
        };

        let mut tree = EpidemicTree {
            nodes: vec![Node {
                address: None,
                stake: 0.0,
                children: Vec::new(),
            }],
        };

        let mut queue = VecDeque::new();
        queue.push_back(0); // Root node index

        let mut addresses_chunks = VecDeque::from(
            addresses
                .chunks(children_count)
                .map(|chunk| chunk.to_vec())
                .collect::<Vec<Vec<String>>>()
        );

        while let Some(parent_index) = queue.pop_front() {
            if let Some(chunk) = addresses_chunks.pop_front() {
                for address in chunk {
                    let new_node = Node {
                        address: Some(address),
                        stake: 0.0,
                        children: Vec::new(),
                    };
                    let new_index = tree.nodes.len();
                    tree.nodes.push(new_node);
                    tree.nodes[parent_index].children.push(new_index);
                    queue.push_back(new_index);
                }
            } else {
                break;
            }
        }

        tree
    }

    fn print_tree(&self) {
        fn print_node(tree: &EpidemicTree, node_index: usize, indent: usize) {
            let node = &tree.nodes[node_index];
            let address = node.address.as_ref().map_or("Empty", String::as_str);
            println!("{}Address: {}, Stake: {}", " ".repeat(indent), address, node.stake);
            for &child_index in &node.children {
                print_node(tree, child_index, indent + 2);
            }
        }
        print_node(self, 0, 0);
    }

    pub fn get_root(&self) -> &Node {
        return self.nodes.get(0).unwrap();
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tree_creation_small() {
        let addresses = vec!["0x1".to_string(), "0x2".to_string(), "0x3".to_string()];
        let tree = EpidemicTree::new(addresses);

        assert_eq!(tree.nodes[0].address, None);
        assert_eq!(tree.nodes[0].children.len(), 3);
        assert!(tree.nodes[0].children.iter().all(|&child_index| tree.nodes[child_index].children.is_empty()));
    }

    #[test]
    fn test_tree_creation_medium() {
        let addresses = (1..=40).map(|i| format!("0x{}", i)).collect();
        let tree = EpidemicTree::new(addresses);

        assert_eq!(tree.nodes[0].address, None);
        assert_eq!(tree.nodes[0].children.len(), 3);
        assert!(tree.nodes[0].children.iter().all(|&child_index| tree.nodes[child_index].children.len() == 3));
    }

    #[test]
    fn test_tree_creation_large() {
        let addresses = (1..=200).map(|i| format!("0x{}", i)).collect();
        let tree = EpidemicTree::new(addresses);

        assert_eq!(tree.nodes[0].address, None);
        assert!(tree.nodes[0].children.len() > 3);
    }

    #[test]
    fn test_max_height() {
        let addresses = (1..=10000).map(|i| format!("0x{}", i)).collect();
        let tree = EpidemicTree::new(addresses);

        fn get_max_height(tree: &EpidemicTree, node_index: usize) -> usize {
            let node = &tree.nodes[node_index];
            if node.children.is_empty() {
                1
            } else {
                1 + node.children.iter().map(|&child_index| get_max_height(tree, child_index)).max().unwrap_or(0)
            }
        }

        assert_eq!(get_max_height(&tree, 0), 4);
    }
}