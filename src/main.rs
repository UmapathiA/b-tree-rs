pub const MAX_NODE_SIZE: usize = 24;
pub const MAX_DATA_INPUT: u32 = 100;
pub const NO_OF_SEARCHES: u32 = MAX_DATA_INPUT * 75 / 100;
pub mod components;
pub mod utils;

fn main() {
    let root = Node::new();
    let (mut root, generated_ids) = generate_tree(root);
    let mut generated_ids_clone = generated_ids.clone();
    let _ = generated_ids_clone.split_off(20);
    generated_ids_clone.sort();
    generated_ids_clone.dedup();
    // generated_ids_clone.shuffle(&mut rand::rng());

    in_order_traversal(&root);
    println!("Deleting ids {:?}", generated_ids_clone);
    let id_to_be_deleted = delete_random_nodes(&mut root, &mut generated_ids_clone);
    println!("After deleting id : {:?}", id_to_be_deleted);
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
        // println!("Results: ");
        println!("{}: {}", id, n.name)
    } else {
        if !root.children.is_empty() {
            if let Some(n) = root.children.get(root.index(id)) {
                find(n, id);
            }
        }
    }
}
use components::Element;
pub fn insert(mut root: Node, mut v: Element) -> (Node, bool) {
    let i = root.index(v.id);
    let mut split_again = false;
    if root.children.is_empty() {
        //When child overflows, split. We reached the tail of the
        //recxursion
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
            .for_each(|x| println!("{} - {}", x.id, x.name));
    } else {
        root.keys.iter().enumerate().for_each(|(i, v)| {
            let child_node = root.children.get(i).unwrap();
            in_order_traversal(child_node);
            println!("{} - {}", v.id, v.name);
        });
        let child_node = root.children.get(root.children.len() - 1).unwrap();
        in_order_traversal(&child_node);
    }
}

pub fn delete_node(root: &mut Node, id: u32) {
    if root.keys.iter().find(|v| v.id == id).is_some() {
        let index = root
            .keys
            .iter()
            .enumerate()
            .find(|(_, v)| v.id == id)
            .unwrap()
            .0;

        let _ = root.keys.remove(index);
        //delete child node
        if !root.children.is_empty() {
            let mut left_sub_tree = root.children.remove(index);
            let successor = get_successor(&mut left_sub_tree);
            root.children.insert(index, left_sub_tree);
            root.keys.insert(index, successor);
        }
    } else {
        let index = if let Some(idx) = root.keys.iter().enumerate().find(|(_, v)| v.id > id) {
            idx.0
        } else {
            root.keys.len()
        };
        let mut n = root.children.remove(index);
        delete_node(&mut n, id);
        root.children.insert(index, n);
    }
}

fn get_successor(root: &mut Node) -> Element {
    if root.children.is_empty() {
        root.keys.pop().unwrap()
    } else {
        let mut next_node = root.children.pop().unwrap();
        if next_node.keys.len() < MAX_NODE_SIZE / 2 {
            println!("Rebalancing");
            let mut next_last_node = root.children.pop().unwrap();
            let borrowed_child = next_last_node.keys.pop().unwrap();
            next_node.keys.push(root.keys.pop().unwrap());
            root.keys.push(borrowed_child);
            root.children.push(next_last_node);
        }

        println!("Key len {}", next_node.keys.len());
        let successor_node = get_successor(&mut next_node);
        if !next_node.keys.is_empty() {
            root.children.push(next_node);
        }
        successor_node
    }
}
