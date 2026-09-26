use serde::{Deserialize, Serialize};

impl Node {
    pub fn index(&self, id: u32) -> usize {
        let mut i: usize = 0;
        let mut ir = self.keys.iter();

        while let Some(x) = ir.next()
            && x.id < id
        {
            i += 1;
        }
        i
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Node {
    pub keys: Vec<Element>,
    pub children: Vec<Node>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Element {
    pub id: u32,
    pub name: String,
}

impl Node {
    pub fn new() -> Self {
        Node {
            keys: Vec::new(),
            children: Vec::new(),
        }
    }
}

impl PartialEq for Element {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}
