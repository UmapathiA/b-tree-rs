mod unit_tests;

const MAX_NODE_SIZE: usize = 4;

fn main() {
    let mut root = Node::new();

    for _ in 1..1500000 {
        let r = random();
        (root, _) = insert(root, r);
    }

    let json_string = serde_json::to_string_pretty(&root).unwrap();

    fs::write("out.json", json_string).unwrap();
}

fn add(a: u32, b: u32) -> u32 {
    a + b
}

use std::fs;

use rand::random;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct Node {
    keys: Vec<u32>,
    children: Vec<Node>,
}

impl Node {
    fn new() -> Self {
        Node {
            keys: Vec::new(),
            children: Vec::new(),
        }
    }
}

fn insert(mut root: Node, v: u32) -> (Node, bool) {
    let i = root.index(v);
    let mut split_again = false;
    if root.children.is_empty() {
        //When child overflows, split. We reached the tail of the
        //recxursion
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

    let right_keys = root.keys.split_off(3);
    let parent = root.keys.pop().unwrap();
    let left_keys = root.keys;

    root.keys = vec![parent];
    left_node.keys = left_keys;
    right_node.keys = right_keys;

    if !root.children.is_empty() {
        right_node.children = root.children.split_off(3);
        left_node.children = root.children;
    }

    root.children = vec![left_node, right_node];

    root
}

impl Node {
    fn index(&self, id: u32) -> usize {
        let mut i: usize = 0;
        let mut ir = self.keys.iter();

        while let Some(x) = ir.next()
            && x < &id
        {
            i += 1;
        }
        i
    }
}
