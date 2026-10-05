use simple_logger::SimpleLogger;

use crate::{
    components::{Element, Node},
    insert,
};

use std::sync::Once;
static START: Once = Once::new();

fn init() {
    START.call_once(|| {
        SimpleLogger::default()
            .with_colors(true)
            .with_level(log::LevelFilter::Error)
            .init()
            .unwrap()
    });
}
#[cfg(test)]
mod test {

    use log::{debug, info};
    use simple_logger::SimpleLogger;

    use crate::{
        components::{Element, Node},
        delete_node, get_successor, insert,
        tests::{build_tree, init},
    };

    #[test]
    fn tree_with_only_root() {
        init();
        let mut root: Node = Node::new();
        root = insert(root, Element::new(21)).0;
        root = insert(root, Element::new(19)).0;
        root = insert(root, Element::new(18)).0;
        root = insert(root, Element::new(20)).0;

        assert_eq!(root.keys.get(0).unwrap().id, 18);
        assert_eq!(root.keys.get(1).unwrap().id, 19);
        assert_eq!(root.keys.get(2).unwrap().id, 20);
        assert_eq!(root.keys.get(3).unwrap().id, 21);
    }

    #[test]
    fn verify_node_split() {
        //Set MAX_NODE_SIZE to 4
        init();
        let mut root: Node = Node::new();
        root = insert(root, Element::new(21)).0;
        root = insert(root, Element::new(19)).0;
        root = insert(root, Element::new(22)).0;
        root = insert(root, Element::new(18)).0;
        root = insert(root, Element::new(20)).0;

        let left = root.children.remove(0);
        let right = root.children.remove(0);

        assert_eq!(left.keys.get(0).unwrap().id, 18);
        assert_eq!(left.keys.get(1).unwrap().id, 19);
        assert_eq!(root.keys.get(0).unwrap().id, 20);
        assert_eq!(right.keys.get(0).unwrap().id, 21);
        assert_eq!(right.keys.get(1).unwrap().id, 22);
    }

    #[test]
    fn verify_node_propagation_on_split() {
        //Set MAX_NODE_SIZE to 4
        init();
        let mut root: Node = Node::new();
        root = insert(root, Element::new(10)).0;
        root = insert(root, Element::new(20)).0;
        root = insert(root, Element::new(30)).0;
        root = insert(root, Element::new(40)).0;
        root = insert(root, Element::new(50)).0; //Root split with 2 child nodes. Child nodes created
        //with 2 keys each
        root = insert(root, Element::new(21)).0; // First child gets 3rd key
        root = insert(root, Element::new(22)).0; // First child gets 4th key
        root = insert(root, Element::new(23)).0; // First child gets 5th key. Split and move median
        // which is 21 to root

        debug!("{}", serde_json::to_string_pretty(&root).unwrap());

        //Root has only 2 elements - One element from first root split and 2nd element from child split
        //As nodes are always sorted, 21 will be inserted before 30, and index of child nodes are
        //relative to the node insertion point
        let left = root.children.remove(0); // New left child added relative to the position of new key
        // in root node which is 1
        let right = root.children.remove(0); // After left key removal, right key take that same position

        assert_eq!(root.keys.get(0).unwrap().id, 21); //Added from child node 21|30
        assert_eq!(left.keys.get(0).unwrap().id, 10); // Newly split child node 
        assert_eq!(left.keys.get(1).unwrap().id, 20);
        assert_eq!(right.keys.get(0).unwrap().id, 22);
        assert_eq!(right.keys.get(1).unwrap().id, 23);
    }

    #[test]
    fn delete_node_from_leaf_without_rebalancing() {
        init();
        let mut root = build_tree();
        delete_node(&mut root, 11);

        let child = root.children.remove(0);

        info!("{}", serde_json::to_string_pretty(&child).unwrap());

        assert!(
            !child.keys.contains(&Element::new(11)),
            "Tree still contains element with id 11"
        );

        assert!(
            child.keys.contains(&Element::new(10)),
            "Element with id 10 is missing from the child node"
        );
        assert!(
            child.keys.contains(&Element::new(20)),
            "Element with id 20 is missing from the child node"
        );
    }

    #[test]
    fn delete_node_and_rebalance_from_left_node() {
        init();
        let mut root = build_tree();
        //                     |21|30|
        //
        //     (|10|11|20| | |)  (|22|23| | |) (|40|50| | |)
        root = insert(root, Element::new(12)).0;
        // root = insert(root, Element::new(13)).0;

        //                     |21|30|
        //
        //     (|10|11|12|20|)  (|22|23| | |) (|40|50| | |)
        delete_node(&mut root, 23);
        //                     |20|30|
        //
        //     (|10|11|12|  |)  (|21|22| | |) (|40|50| | |)
        info!("{}", serde_json::to_string_pretty(&root).unwrap());
        assert_eq!(root.keys.get(0).unwrap().id, 20);
        assert_eq!(root.children.get(1).unwrap().keys.get(0).unwrap().id, 21);
    }

    #[test]
    fn delete_node_and_rebalance_from_right_node() {
        init();
        let mut root = build_tree();
        //                     |21|30|
        //
        //     (|10|11|20| | |)  (|22|23| | |) (|40|50| | |)
        root = insert(root, Element::new(24)).0;
        // root = insert(root, Element::new(13)).0;

        //                     |21|30|
        //
        //     (|10|11|20|)  (|22|23|24| |) (|40|50| | |)
        delete_node(&mut root, 20);
        //                     |21|30|
        //
        //     (|10|11|  | |)  (|22|23|24| |) (|40|50| | |)

        delete_node(&mut root, 11);

        //                     |22|30|
        //
        //     (|10|21|  | |)  (|23|24| |) (|40|50| | |)

        info!("{}", yaml_serde::to_string(&root).unwrap());
        assert_eq!(root.keys.get(0).unwrap().id, 22);

        assert_eq!(root.children.get(0).unwrap().keys.get(1).unwrap().id, 21);
        assert_eq!(root.children.get(1).unwrap().keys.get(0).unwrap().id, 23);
    }
    #[test]
    fn delete_node_and_rebalance_from_far_right_node() {
        init();
        let mut root = build_tree();
        //                     |21|30|
        //
        //     (|10|11|20| | |)  (|22|23| | |) (|40|50| | |)
        root = insert(root, Element::new(60)).0;
        // root = insert(root, Element::new(13)).0;

        //                     |21|30|
        //
        //     (|10|11|20|)  (|22|23| | |) (|40|50|60| |)
        delete_node(&mut root, 20);
        //                     |21|30|
        //
        //     (|10|11|  | |)  (|22|23|| |) (|40|50|60| |)

        delete_node(&mut root, 23);

        //                     |21|40|
        //
        //     (|10|11|  | |)  (|22|30| |) (|50|60| | |)

        info!("{}", yaml_serde::to_string(&root).unwrap());
        assert_eq!(root.keys.get(1).unwrap().id, 40);

        assert_eq!(root.children.get(1).unwrap().keys.get(1).unwrap().id, 30);
        assert_eq!(root.children.get(2).unwrap().keys.get(0).unwrap().id, 50);
    }

    #[test]
    fn delete_node_and_and_merge_with_right_node() {
        init();
        let mut root = build_tree();
        //                     |21|30|
        //
        //     (|10|11|20| | |)  (|22|23| | |) (|40|50| | |)
        root = insert(root, Element::new(60)).0;
        // root = insert(root, Element::new(13)).0;

        //                     |21|30|
        //
        //     (|10|11|20|)  (|22|23| | |) (|40|50|60| |)
        delete_node(&mut root, 20);
        //                     |21|30|
        //
        //     (|10|11|  | |)  (|22|23|| |) (|40|50|60| |)

        delete_node(&mut root, 11);

        //                     |30|
        //
        //     (|10|21|22|23|)  (|40|50|60| |)

        info!("{}", yaml_serde::to_string(&root).unwrap());
        assert_eq!(root.keys.get(0).unwrap().id, 30);

        assert_eq!(root.children.get(0).unwrap().keys.get(1).unwrap().id, 21);
        assert_eq!(root.children.get(0).unwrap().keys.get(2).unwrap().id, 22);
        assert_eq!(root.children.get(0).unwrap().keys.get(3).unwrap().id, 23);
        assert_eq!(root.children.get(1).unwrap().keys.get(0).unwrap().id, 40);
        assert_eq!(root.children.get(1).unwrap().keys.get(1).unwrap().id, 50);
        assert_eq!(root.children.get(1).unwrap().keys.get(2).unwrap().id, 60);
    }

    #[test]
    fn delete_node_and_and_merge_with_left_node() {
        init();
        let mut root = build_tree();
        //                     |21|30|
        //
        //     (|10|11|20| | |)  (|22|23| | |) (|40|50| | |)
        root = insert(root, Element::new(60)).0;
        // root = insert(root, Element::new(13)).0;

        //                     |21|30|
        //
        //     (|10|11|20|)  (|22|23| | |) (|40|50|60| |)
        delete_node(&mut root, 20);
        //                     |21|30|
        //
        //     (|10|11|  | |)  (|22|23|| |) (|40|50|60| |)

        delete_node(&mut root, 23);

        info!(
            "After deleting 23 {}",
            yaml_serde::to_string(&root).unwrap()
        );

        //                     |30|
        //
        //     (|10|11|21|22|) (|40|50|60| |)

        info!("{}", yaml_serde::to_string(&root).unwrap());
        assert_eq!(root.keys.get(0).unwrap().id, 30);

        assert_eq!(root.children.get(0).unwrap().keys.get(0).unwrap().id, 10);
        assert_eq!(root.children.get(0).unwrap().keys.get(1).unwrap().id, 11);
        assert_eq!(root.children.get(0).unwrap().keys.get(2).unwrap().id, 21);
        assert_eq!(root.children.get(0).unwrap().keys.get(3).unwrap().id, 22);
        assert_eq!(root.children.get(1).unwrap().keys.get(0).unwrap().id, 40);
        assert_eq!(root.children.get(1).unwrap().keys.get(1).unwrap().id, 50);
        assert_eq!(root.children.get(1).unwrap().keys.get(2).unwrap().id, 60);
    }
}

pub fn build_tree() -> Node {
    //Set MAX_NODE_SIZE to 4
    init();
    let mut root: Node = Node::new();
    root = insert(root, Element::new(10)).0;
    root = insert(root, Element::new(20)).0;
    root = insert(root, Element::new(30)).0;
    root = insert(root, Element::new(40)).0;
    root = insert(root, Element::new(50)).0; //Root split with 2 child nodes. Child nodes created
    //with 2 keys each
    root = insert(root, Element::new(21)).0; // First child gets 3rd key
    root = insert(root, Element::new(22)).0; // First child gets 4th key
    root = insert(root, Element::new(23)).0; // First child gets 5th key. Split and move median
    // which is 21 to root
    //                     |21|30|
    //
    //     (|10|20| | |)  (|22|23| | |) (|40|50| | |)
    //
    //

    root = insert(root, Element::new(11)).0;

    //                     |21|30|
    //
    //     (|10|11|20| | |)  (|22|23| | |) (|40|50| | |)
    root
}
