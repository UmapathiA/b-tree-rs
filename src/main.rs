pub const MAX_NODE_SIZE: usize = 4;
pub const MAX_DATA_INPUT: u32 = 100;
pub const NO_OF_SEARCHES: u32 = MAX_DATA_INPUT * 75 / 100;
pub mod components;
pub mod tests;
pub mod utils;

fn main() {
    SimpleLogger::new().init().unwrap();
    let root = Node::new();
    let (mut root, generated_ids) = generate_tree(root);
    let mut generated_ids_clone = generated_ids.clone();
    let _ = generated_ids_clone.split_off(80);
    generated_ids_clone.sort();
    generated_ids_clone.dedup();
    // generated_ids_clone.shuffle(&mut rand::rng());

    in_order_traversal(&root);
    info!("Deleting ids {:?}", generated_ids_clone);
    let id_to_be_deleted = delete_random_nodes(&mut root, &mut generated_ids_clone);
    info!("After deleting id : {:?}", id_to_be_deleted);
    in_order_traversal(&root);

    let json_string = serde_json::to_string_pretty(&root).unwrap();

    fs::write("out.json", json_string).unwrap();

    benchmark_utils(&root, generated_ids);
}

use std::fs;

use crate::{
    components::Node,
    utils::{benchmark_utils, delete_random_nodes, generate_tree},
};

pub fn find(root: &Node, id: u32) {
    if let Some(n) = root.keys.iter().find(|k| k.id == id) {
        // info!("Results: ");
        info!("{}: {}", id, n.name)
    } else {
        if !root.children.is_empty() {
            if let Some(n) = root.children.get(root.index(id)) {
                find(n, id);
            }
        }
    }
}
use components::Element;
use log::{debug, info};
use simple_logger::SimpleLogger;
pub fn insert(mut root: Node, mut v: Element) -> (Node, bool) {
    let i = root.index(v.id);
    let mut split_again = false;
    if root.children.is_empty() {
        //When child overflows, split. We reached the tail of the
        //recursion
        if root.keys.contains(&v) {
            let _ = root
                .keys
                .remove(root.keys.iter().position(|x| x.id == v.id).unwrap());
            v.name.push_str("(Updated)");
        }
        root.keys.insert(i, v);
        if root.keys.len() > MAX_NODE_SIZE {
            root = split_node(root);
            split_again = true; //Let the caller know that there is a split
        }
    } else {
        let next_node = root.children.remove(i); //Remove the node next in line and
        //take full ownership
        let (mut next_node, split) = insert(next_node, v);
        if split {
            //In case of split, we expect a node with only one key. Deconstruct the node, extract
            //the key and insert in parent
            root.keys.insert(i, next_node.keys.pop().unwrap());
            //Get the grand-children node and insert them in parent relative to the index
            //calculated
            let e_1 = next_node.children.pop().unwrap();
            let e_2 = next_node.children.pop().unwrap();
            root.children.insert(i, e_2);
            root.children.insert(i + 1, e_1);
            if root.keys.len() > MAX_NODE_SIZE {
                //In case of overflow, split again and let the
                //caller know
                root = split_node(root);
                split_again = true;
            }
        } else {
            root.children.insert(i, next_node); //In case there is no split, insert
            //the previously removed node in the same place. Since there is no new nodes, no need
            //to further process the tree.
        }
    }
    (root, split_again)
}

fn split_node(mut root: Node) -> Node {
    let mut left_node = Node::new();
    let mut right_node = Node::new();

    let right_keys = root.keys.split_off(MAX_NODE_SIZE / 2 + 1);
    let parent = root.keys.pop().unwrap();
    let left_keys = root.keys;

    root.keys = vec![parent];
    left_node.keys = left_keys;
    right_node.keys = right_keys;

    if !root.children.is_empty() {
        right_node.children = root.children.split_off(MAX_NODE_SIZE / 2 + 1);
        left_node.children = root.children;
    }

    root.children = vec![left_node, right_node];

    root
}

fn in_order_traversal(root: &Node) {
    if root.children.is_empty() {
        root.keys
            .iter()
            .for_each(|x| info!("{} - {}", x.id, x.name));
    } else {
        root.keys.iter().enumerate().for_each(|(i, v)| {
            let child_node = root.children.get(i).unwrap();
            in_order_traversal(child_node);
            info!("{} - {}", v.id, v.name);
        });
        let child_node = root.children.get(root.children.len() - 1).unwrap();
        in_order_traversal(&child_node);
    }
}

pub fn delete_node(root: &mut Node, id: u32) -> bool {
    if root.keys.iter().find(|v| v.id == id).is_some() {
        //Found the key that we are searching
        let index = root
            .keys
            .iter()
            .enumerate()
            .find(|(_, v)| v.id == id)
            .unwrap()
            .0;

        //delete child node
        let _ = root.keys.remove(index); //If this is the child node, this is where it ends.
        if !root.children.is_empty() {
            //If it is not a child node, we need to replace the element
            //with the in order successor
            let mut left_sub_tree = root.children.remove(index);
            if !left_sub_tree.keys.is_empty() {
                //Somehow we endup with empty key and empty
                //childnren nodes. For now we are just not inserting such nodes back. This may
                //never occur if we properly build node rebalancing cases
                debug!("Popped children to find next successor {:?}", left_sub_tree);
                let successor = get_successor(&mut left_sub_tree);
                root.children.insert(index, left_sub_tree);
                root.keys.insert(index, successor);
            }
        }
        true
    } else {
        //NOTE: Handles rebalanicng only at leaf nodes
        let mut merged_with_left = false;
        let index = if let Some(idx) = root.keys.iter().enumerate().find(|(_, v)| v.id > id) {
            idx.0
        } else {
            root.keys.len()
        };
        let mut node_with_deleted_key = root.children.remove(index);
        let deleted = delete_node(&mut node_with_deleted_key, id);

        if deleted && node_with_deleted_key.keys.len() < MAX_NODE_SIZE / 2 {
            let (dir, operation) = find_dir_and_operation(&root, index);
            if dir == 'l' {
                info!("Rebalancing with left node");
                info!("{}", yaml_serde::to_string(&root).unwrap());
                let mut left_child = root.children.remove(index - 1);
                node_with_deleted_key
                    .keys
                    .insert(0, root.keys.remove(index - 1));
                if operation == 's' {
                    root.keys.insert(index - 1, left_child.keys.pop().unwrap());
                    root.children.insert(index - 1, left_child);
                } else {
                    for (i, k) in left_child.keys.into_iter().enumerate() {
                        node_with_deleted_key.keys.insert(i, k);
                    }

                    merged_with_left = true;
                }
            } else {
                if root.children.len() > index {
                    let mut right_child = root.children.remove(index); //Since we already removed
                    node_with_deleted_key.keys.push(root.keys.remove(index));
                    if operation == 's' {
                        //a node to delete key, its right sibling will take that index
                        root.keys.insert(index, right_child.keys.remove(0));
                        root.children.insert(index, right_child);
                    } else {
                        //Merge with right child
                        node_with_deleted_key.keys.append(&mut right_child.keys);
                    }
                }
            }
        }
        if merged_with_left {
            root.children.insert(index - 1, node_with_deleted_key);
        } else {
            root.children.insert(index, node_with_deleted_key);
        }
        deleted
    }
}
fn find_dir_and_operation(root: &Node, index: usize) -> (char, char) {
    if index > 0
        && let Some(ln) = root.children.get(index - 1)
    {
        if ln.keys.len() > MAX_NODE_SIZE / 2 {
            ('l', 's')
        } else if let Some(rn) = root.children.get(index) {
            if rn.keys.len() > MAX_NODE_SIZE / 2 {
                ('r', 's')
            } else {
                ('l', 'm')
            }
        } else {
            ('l', 'm')
        }
    } else {
        let rn = root.children.get(index).unwrap();

        if rn.keys.len() > MAX_NODE_SIZE / 2 {
            ('r', 's')
        } else {
            ('r', 'm')
        }
    }
}

fn get_successor(root: &mut Node) -> Element {
    debug!("Node Keys {:?}", root.keys);
    if root.children.is_empty() {
        root.keys.pop().unwrap()
    } else {
        let mut next_node = root.children.pop().unwrap();
        if next_node.keys.len() < MAX_NODE_SIZE / 2 {
            info!("Rebalancing");
            let mut next_last_node = root.children.pop().unwrap();
            debug!("Next last node {:?}", next_last_node.keys);
            let borrowed_child = next_last_node.keys.pop().unwrap();
            next_node.keys.push(root.keys.pop().unwrap());
            root.keys.push(borrowed_child);
            root.children.push(next_last_node);
        }

        info!("Key len {}", next_node.keys.len());
        let successor_node = get_successor(&mut next_node);
        if !next_node.keys.is_empty() {
            root.children.push(next_node);
        }
        successor_node
    }
}
