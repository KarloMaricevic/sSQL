use core::fmt::Debug;
use crate::bptree::InternalNode;

use super::{BTree, BTreeNode, LeafNode};

impl<K: Ord + Clone + Debug, V: Clone + Debug> BTree<K, V> {
    fn new(order: usize) -> Self {
        BTree {
            root: BTreeNode::Leaf(LeafNode::new(order)),
            order,
        }
    }

    fn insert(&mut self, key: K, value: V) -> Result<(), String> {
        let mut current = &mut self.root as *mut BTreeNode<K, V>;
        let mut stack = vec![];
        println!("NEW INSERT {:?}, {:?}", key, value);
        unsafe {
            loop {
                match &mut *current {
                    BTreeNode::Internal(ref mut internal) => {
                        let child_pos = match internal.keys.binary_search(&key) {
                            Ok(_) => return Err(format!("Value with given key already exists")),
                            Err(pos) => pos,
                        };
                        stack.push((current, child_pos));
                        println!(
                            "Pushed node: {:?}, child index is {:?}",
                            internal, child_pos
                        );
                        current = &mut *(internal.children[child_pos]) as *mut BTreeNode<K, V>;
                    }
                    BTreeNode::Leaf(leaf) => {
                        let pos_to_insert_value = match leaf.keys.binary_search(&key) {
                            Ok(pos) => pos + 1,
                            Err(pos) => pos,
                        };
                        println!(
                            "Pushed node: {:?}, value index is {:?}",
                            leaf, pos_to_insert_value
                        );
                        stack.push((current, pos_to_insert_value));
                        break;
                    }
                }
            }
        }

        let mut pushed_up: Option<(K, BTreeNode<K, V>)> = None;
        unsafe {
            while let Some((entry, child_index)) = stack.pop() {
                match &mut *entry {
                    BTreeNode::Leaf(leaf) => {
                        println!("Stack is {:?}", stack);
                        if leaf.keys.len() + 1 >= self.order {
                            println!("SPLITTING LEAF {:?}", leaf);
                            let split_off = if leaf.keys.len() / 2 < child_index {
                                (leaf.keys.len() + 1) / 2
                            } else {
                                leaf.keys.len() / 2
                            };
                            let mut new_leaf_keys = leaf.keys.split_off(split_off);
                            let mut new_leaf_values = leaf.values.split_off(split_off);
                            if child_index > leaf.keys.len() / 2 {
                                println!("Inserting into right node");
                                println!("Position to place gotten from stack: {:?}", child_index);
                                println!(
                                    "Position to place new key: {:?}",
                                    child_index - leaf.keys.len()
                                );
                                new_leaf_keys.insert(child_index - leaf.keys.len(), key.clone());
                                new_leaf_values
                                    .insert(child_index - leaf.values.len(), value.clone());
                            } else {
                                println!("Inserting into left node");
                                println!("Position to place new key {:?}", child_index);
                                leaf.keys.insert(child_index, key.clone());
                                leaf.values.insert(child_index, value.clone());
                            }
                            println!("Left leaf is {:?}", leaf);
                            pushed_up = Some((
                                new_leaf_keys[0].clone(),
                                BTreeNode::Leaf(LeafNode {
                                    keys: new_leaf_keys,
                                    values: new_leaf_values,
                                }),
                            ));
                            println!("Right leaf is {:?}", pushed_up);
                        } else {
                            leaf.keys.insert(child_index, key.clone());
                            leaf.values.insert(child_index, value.clone());
                        }
                    }
                    BTreeNode::Internal(internal) => {
                        println!("Stack is {:?}", stack);
                        if let Some((new_key, new_child)) = pushed_up.take() {
                            if internal.keys.len() + 1 >= self.order {
                                println!(
                                    "Tree is {:?}, pushed key is {:?}, new child is {:?}",
                                    self, new_key, new_child
                                );
                                println!("SPLITTIG INTERNAL NODE: {:?}", internal);
                                let split_off = if internal.keys.len() % 2 == 1 {
                                    (internal.keys.len() + 1) / 2
                                } else {
                                    internal.keys.len() / 2
                                };
                                println!("After splitting internal node:");
                                let mut new_internal_keys = internal.keys.split_off(split_off);
                                let mut new_internal_children =
                                    internal.children.split_off(split_off + 1);
                                if child_index >= split_off {
                                    println!("Inserting into right node");
                                    println!(
                                        "Inserting into place {:?}",
                                        child_index - internal.keys.len()
                                    );
                                    println!(
                                        "Children in right node before inserting are {:?}",
                                        new_internal_children,
                                    );
                                    println!("Child to be inserted is {:?}", new_child);
                                    new_internal_keys
                                        .insert(child_index - internal.keys.len(), new_key.clone());
                                    new_internal_children.insert(
                                        child_index - internal.keys.len(),
                                        Box::new(new_child),
                                    );
                                    // println!("After splitting internal node:");
                                    //          println!("Left node is {:?}", internal);
                                    println!(
                                        "Right node keys are {:?}, Roght children are {:?}",
                                        new_internal_keys, new_internal_children
                                    );
                                } else {
                                    println!("Inserting into left node");
                                    internal
                                        .children
                                        .insert(child_index + 1, Box::new(new_child));
                                }
                                let middle_key = new_internal_keys.remove(0);
                                pushed_up = Some((
                                    middle_key.clone(),
                                    BTreeNode::Internal(InternalNode {
                                        keys: new_internal_keys,
                                        children: new_internal_children,
                                    }),
                                ));
                                println!("Right iner node is {:?}", pushed_up);
                                println!("Tree is now {:?}", self);
                            } else {
                                internal.keys.insert(child_index, new_key.clone());
                                internal
                                    .children
                                    .insert(child_index + 1, Box::new(new_child));
                                pushed_up = None;
                            }
                        } else {
                            break;
                        }
                    }
                }
            }
        }
        println!("Last push is {:?}", pushed_up);
        if let Some((new_key, new_child)) = pushed_up {
            let old_root = std::mem::replace(
                &mut self.root,
                BTreeNode::Leaf(LeafNode {
                    keys: vec![],
                    values: vec![],
                }),
            );
            self.root = BTreeNode::Internal(InternalNode {
                keys: vec![new_key],
                children: vec![Box::new(old_root), Box::new(new_child)],
            });
        }
        println!("Tree is now: {:?}", self);
        Ok(())
    }

    pub fn delete(&mut self, key: K) -> Result<(), String> {
        let mut current = &mut self.root as *mut BTreeNode<K, V>;
        let mut stack: Vec<(*mut BTreeNode<K, V>, usize)> = vec![];
        unsafe {
            loop {
                match &mut *current {
                    BTreeNode::Internal(ref mut internal) => {
                        let child_pos = internal
                            .keys
                            .binary_search(&key)
                            .unwrap_or_else(|position| position - 1);
                        stack.push((current, child_pos));
                        println!(
                            "Pushed node: {:?}, child index is {:?}",
                            internal, child_pos
                        );
                        current = &mut *(internal.children[child_pos]) as *mut BTreeNode<K, V>;
                    }
                    BTreeNode::Leaf(leaf) => {
                        println!("Searching keys {:?} for key {:?}", leaf.keys, key);
                        let value_pos = leaf.keys.binary_search(&key).map_err(|_| "Unknown key")?;
                        println!(
                            "Want to delete value for leaf {:?}, position {:?}",
                            leaf, value_pos
                        );
                        stack.push((current, value_pos));
                        break;
                    }
                }
            }
        }

        println!("Calculating underflow!");
        let underflow = self.order / 2;
        unsafe {
            while let Some((entry, child_index)) = stack.pop() {
                println!("Enrty is {:?}", *entry);
                match &mut *entry {
                    BTreeNode::Leaf(ref mut leaf) => {
                        leaf.keys.remove(child_index);
                        leaf.values.remove(child_index);
                        if stack.last().is_none() {
                            continue;
                        } else if leaf.keys.len() >= underflow {
                            println!("There isnt any underflow!");
                            if child_index == 0 {
                                if let Some((parent_pointer, leaf_pos)) = stack.last() {
                                    if let BTreeNode::Internal(parent) = &mut **parent_pointer {
                                        parent.keys[*leaf_pos] = leaf.keys.first().unwrap().clone();
                                    }
                                }
                            }
                            continue;
                        } else {
                            if let Some((parent_pointer, leaf_pos)) = stack.last() {
                                if let BTreeNode::Internal(parent) = &mut **parent_pointer {
                                    if *leaf_pos > 0 {
                                        println!("Trying to borrow from left");
                                        println!("Child pos is {:?}", leaf_pos);
                                        if let BTreeNode::Leaf(left_sibling) =
                                            &mut *parent.children[leaf_pos - 1]
                                        {
                                            if left_sibling.values.len() > underflow {
                                                println!("Borrowing from left");
                                                let borrow_key = left_sibling.keys.pop().unwrap();
                                                let borrow_value =
                                                    left_sibling.values.pop().unwrap();
                                                parent.keys[*leaf_pos] = borrow_key.clone();
                                                leaf.keys.insert(0, borrow_key);
                                                leaf.values.insert(0, borrow_value);
                                                println!("Ive gotten here!");
                                                continue;
                                            }
                                        }
                                    }
                                    if *leaf_pos != parent.keys.len() {
                                        println!("Trying to borrow from the right!");
                                        if let BTreeNode::Leaf(right_sibling) =
                                            &mut *parent.children[leaf_pos + 1]
                                        {
                                            if right_sibling.values.len() > underflow {
                                                println!("Borrowing from the right");
                                                let borrow_key = right_sibling.keys.remove(0);
                                                let borrow_value = right_sibling.values.remove(0);
                                                println!("After removing key/value for borrow, sibling is {:?}",right_sibling );
                                                parent.keys[*leaf_pos] =
                                                    leaf.keys.first().unwrap().clone();
                                                println!(
                                                    "Keys of the right node are {:?}",
                                                    right_sibling.keys
                                                );
                                                parent.keys[*leaf_pos + 1] =
                                                    right_sibling.keys.first().unwrap().clone();
                                                leaf.keys.push(borrow_key);
                                                leaf.values.push(borrow_value);
                                                continue;
                                            }
                                        }
                                    }
                                    if *leaf_pos > 0 {
                                        println!("Trying to merge with left node");
                                        if let BTreeNode::Leaf(left_sibling) =
                                            &mut *parent.children[leaf_pos - 1]
                                        {
                                            println!("Merging with right node");
                                            left_sibling.keys.append(&mut leaf.keys);
                                            left_sibling.values.append(&mut leaf.values);
                                            parent.children.remove(*leaf_pos);
                                            parent.keys.remove(*leaf_pos);
                                            continue;
                                        }
                                    }
                                    if *leaf_pos != parent.keys.len() {
                                        println!("Trying to merge with right node");
                                        if let BTreeNode::Leaf(right_sibling) =
                                            &mut *parent.children[leaf_pos + 1]
                                        {
                                            println!("Merging with right node");
                                            right_sibling.keys.splice(0..0, leaf.keys.clone());
                                            right_sibling.values.splice(0..0, leaf.values.clone());
                                            parent.keys[*leaf_pos + 1] =
                                                right_sibling.keys.first().unwrap().clone();
                                            parent.keys.remove(*leaf_pos);
                                            parent.children.remove(*leaf_pos);
                                            continue;
                                        }
                                    }
                                }
                            }
                        }
                    }
                    BTreeNode::Internal(internal) => {}
                }
            }
        }
        Ok(())
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        let mut current = &self.root;

        loop {
            match current {
                BTreeNode::Internal(internal_node) => {
                    let child_pos = match internal_node.keys.binary_search(key) {
                        Ok(pos) => pos,
                        Err(pos) => pos + 1,
                    };
                    current = &internal_node.children[child_pos];
                }
                BTreeNode::Leaf(leaf_node) => {
                    return match leaf_node.keys.binary_search(key) {
                        Ok(pos) => Some(&leaf_node.values[pos]),
                        Err(_) => None,
                    };
                }
            }
        }
    }
}

#[cfg(test)]
impl<K: Ord + Clone + Debug, V: Clone + Debug> BTree<K, V> {
    fn create_from(order: usize, internal: InternalNode<K, V>) -> Self {
        BTree {
            root: BTreeNode::Internal(internal),
            order,
        }
    }
}

impl<K: Ord + Clone + Debug, V: Clone + Debug> LeafNode<K, V> {
    fn new(order: usize) -> Self {
        LeafNode {
            keys: Vec::with_capacity(order),
            values: Vec::with_capacity(order),
        }
    }
}
#[cfg(test)]
mod tests {

    use super::*;

    impl<K: PartialEq + Debug + Clone + Ord, V: PartialEq + Debug + Clone> PartialEq
        for LeafNode<K, V>
    {
        fn eq(&self, other: &Self) -> bool {
            self.keys == other.keys && self.values == other.values
        }
    }

    impl<K: PartialEq + Debug + Clone + Ord, V: PartialEq + Debug + Clone> PartialEq
        for InternalNode<K, V>
    {
        fn eq(&self, other: &Self) -> bool {
            self.keys == other.keys && self.children == other.children
        }
    }

    impl<K: PartialEq + Debug + Clone + Ord, V: PartialEq + Debug + Clone> PartialEq
        for BTreeNode<K, V>
    {
        fn eq(&self, other: &Self) -> bool {
            match (self, other) {
                (BTreeNode::Internal(a), BTreeNode::Internal(b)) => a == b,
                (BTreeNode::Leaf(a), BTreeNode::Leaf(b)) => a == b,
                _ => false,
            }
        }
    }

    impl<K: PartialEq + Debug + Clone + Ord, V: PartialEq + Debug + Clone> PartialEq for BTree<K, V> {
        fn eq(&self, other: &Self) -> bool {
            self.order == other.order && self.root == other.root
        }
    }

    #[test]
    fn when_inserting_values_to_tree_it_should_correcty_rebalance_itself() {
        let mut actual = BTree::new(3);
        let expected = BTree {
            order: 3,
            root: BTreeNode::Internal(InternalNode {
                keys: vec![30, 50],
                children: vec![
                    Box::from(BTreeNode::Internal(InternalNode {
                        keys: vec![15, 20],
                        children: vec![
                            Box::from(BTreeNode::Leaf(LeafNode {
                                keys: vec![10],
                                values: vec!['a'],
                            })),
                            Box::from(BTreeNode::Leaf(LeafNode {
                                keys: vec![15, 18],
                                values: vec!['h', 'i'],
                            })),
                            Box::from(BTreeNode::Leaf(LeafNode {
                                keys: vec![20],
                                values: vec!['b'],
                            })),
                        ],
                    })),
                    Box::from(BTreeNode::Internal(InternalNode {
                        keys: vec![40],
                        children: vec![
                            Box::from(BTreeNode::Leaf(LeafNode {
                                keys: vec![30],
                                values: vec!['c'],
                            })),
                            Box::from(BTreeNode::Leaf(LeafNode {
                                keys: vec![40],
                                values: vec!['d'],
                            })),
                        ],
                    })),
                    Box::from(BTreeNode::Internal(InternalNode {
                        keys: vec![60],
                        children: vec![
                            Box::from(BTreeNode::Leaf(LeafNode {
                                keys: vec![50],
                                values: vec!['e'],
                            })),
                            Box::from(BTreeNode::Leaf(LeafNode {
                                keys: vec![60, 70],
                                values: vec!['f', 'g'],
                            })),
                        ],
                    })),
                ],
            }),
        };

        actual.insert(10, 'a').unwrap();
        actual.insert(20, 'b').unwrap();
        actual.insert(30, 'c').unwrap();
        actual.insert(40, 'd').unwrap();
        actual.insert(50, 'e').unwrap();
        actual.insert(60, 'f').unwrap();
        actual.insert(70, 'g').unwrap();
        actual.insert(15, 'h').unwrap();
        actual.insert(18, 'i').unwrap();

        assert_eq!(actual, expected);
    }

    #[test]
    fn when_deleting_value_and_leaf_dosent_underfows_it_should_only_delete_value_and_update_key_if_needed(
    ) {
        let mut tree = BTree::create_from(
            4,
            InternalNode {
                keys: vec![1, 50, 70],
                children: vec![
                    Box::from(BTreeNode::Leaf(LeafNode {
                        keys: vec![1, 10],
                        values: vec!['a', 'b'],
                    })),
                    Box::from(BTreeNode::Leaf(LeafNode {
                        keys: vec![50, 60],
                        values: vec!['c', 'd'],
                    })),
                    Box::from(BTreeNode::Leaf(LeafNode {
                        keys: vec![70, 80, 90],
                        values: vec!['e', 'f', 'g'],
                    })),
                ],
            },
        );
        let expected = BTree {
            order: 4,
            root: BTreeNode::Internal(InternalNode {
                keys: vec![1, 50, 80],
                children: vec![
                    Box::from(BTreeNode::Leaf(LeafNode {
                        keys: vec![1, 10],
                        values: vec!['a', 'b'],
                    })),
                    Box::from(BTreeNode::Leaf(LeafNode {
                        keys: vec![50, 60],
                        values: vec!['c', 'd'],
                    })),
                    Box::from(BTreeNode::Leaf(LeafNode {
                        keys: vec![80, 90],
                        values: vec!['f', 'g'],
                    })),
                ],
            }),
        };

        tree.delete(70).unwrap();

        assert_eq!(tree, expected);
    }

    #[test]
    fn when_deleting_value_and_leaf_underfows_it_should_borrow_from_leaf_on_left_when_possible() {
        let mut tree = BTree::create_from(
            4,
            InternalNode {
                keys: vec![1, 50],
                children: vec![
                    Box::from(BTreeNode::Leaf(LeafNode {
                        keys: vec![1, 10, 20],
                        values: vec!['a', 'b', 'c'],
                    })),
                    Box::from(BTreeNode::Leaf(LeafNode {
                        keys: vec![50, 60],
                        values: vec!['d', 'e'],
                    })),
                ],
            },
        );
        let expected = BTree {
            order: 4,
            root: BTreeNode::Internal(InternalNode {
                keys: vec![1, 20],
                children: vec![
                    Box::from(BTreeNode::Leaf(LeafNode {
                        keys: vec![1, 10],
                        values: vec!['a', 'b'],
                    })),
                    Box::from(BTreeNode::Leaf(LeafNode {
                        keys: vec![20, 60],
                        values: vec!['c', 'e'],
                    })),
                ],
            }),
        };

        tree.delete(50).unwrap();

        assert_eq!(tree, expected);
    }

    #[test]
    fn when_deleting_value_and_leaf_underfows_it_should_borrow_from_leaf_on_right_when_possible() {
        let mut tree = BTree::create_from(
            4,
            InternalNode {
                keys: vec![1, 50, 70],
                children: vec![
                    Box::from(BTreeNode::Leaf(LeafNode {
                        keys: vec![1, 10],
                        values: vec!['a', 'b'],
                    })),
                    Box::from(BTreeNode::Leaf(LeafNode {
                        keys: vec![50, 60],
                        values: vec!['c', 'd'],
                    })),
                    Box::from(BTreeNode::Leaf(LeafNode {
                        keys: vec![70, 80, 90],
                        values: vec!['e', 'f', 'g'],
                    })),
                ],
            },
        );
        let expected = BTree {
            order: 4,
            root: BTreeNode::Internal(InternalNode {
                keys: vec![1, 60, 80],
                children: vec![
                    Box::from(BTreeNode::Leaf(LeafNode {
                        keys: vec![1, 10],
                        values: vec!['a', 'b'],
                    })),
                    Box::from(BTreeNode::Leaf(LeafNode {
                        keys: vec![60, 70],
                        values: vec!['d', 'e'],
                    })),
                    Box::from(BTreeNode::Leaf(LeafNode {
                        keys: vec![80, 90],
                        values: vec!['f', 'g'],
                    })),
                ],
            }),
        };

        tree.delete(50).unwrap();

        assert_eq!(tree, expected);
    }

    #[test]
    fn when_deleting_value_and_leaf_cannot_borrow_it_should_merge_with_left_leaf_when_possible() {
        let mut tree = BTree::create_from(
            4,
            InternalNode {
                keys: vec![1, 50, 70],
                children: vec![
                    Box::from(BTreeNode::Leaf(LeafNode {
                        keys: vec![1, 10],
                        values: vec!['a', 'b'],
                    })),
                    Box::from(BTreeNode::Leaf(LeafNode {
                        keys: vec![50, 60],
                        values: vec!['c', 'd'],
                    })),
                    Box::from(BTreeNode::Leaf(LeafNode {
                        keys: vec![70, 90],
                        values: vec!['e', 'g'],
                    })),
                ],
            },
        );
        let expected = BTree {
            order: 4,
            root: BTreeNode::Internal(InternalNode {
                keys: vec![1, 70],
                children: vec![
                    Box::from(BTreeNode::Leaf(LeafNode {
                        keys: vec![1, 10, 60],
                        values: vec!['a', 'b', 'd'],
                    })),
                    Box::from(BTreeNode::Leaf(LeafNode {
                        keys: vec![70, 90],
                        values: vec!['e', 'g'],
                    })),
                ],
            }),
        };

        tree.delete(50).unwrap();

        assert_eq!(tree, expected);
    }

    #[test]
    fn when_deleting_value_and_leaf_cannot_borrow_it_should_merge_with_right_leaf() {
        let mut tree = BTree::create_from(
            4,
            InternalNode {
                keys: vec![1, 50, 70],
                children: vec![
                    Box::from(BTreeNode::Leaf(LeafNode {
                        keys: vec![1, 10],
                        values: vec!['a', 'b'],
                    })),
                    Box::from(BTreeNode::Leaf(LeafNode {
                        keys: vec![50, 60],
                        values: vec!['c', 'd'],
                    })),
                    Box::from(BTreeNode::Leaf(LeafNode {
                        keys: vec![70, 80],
                        values: vec!['e', 'f'],
                    })),
                ],
            },
        );
        let expected = BTree {
            order: 4,
            root: BTreeNode::Internal(InternalNode {
                keys: vec![1, 70],
                children: vec![
                    Box::from(BTreeNode::Leaf(LeafNode {
                        keys: vec![1, 50, 60],
                        values: vec!['a', 'c', 'd'],
                    })),
                    Box::from(BTreeNode::Leaf(LeafNode {
                        keys: vec![70, 80],
                        values: vec!['e', 'f'],
                    })),
                ],
            }),
        };

        tree.delete(10).unwrap();

        assert_eq!(tree, expected);
    }
}
