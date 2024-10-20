mod tree;
pub mod page; 
use core::fmt::Debug;

#[derive(Debug)]
struct BTree<K: Ord + Clone + Debug, V: Clone + Debug> {
    root: BTreeNode<K, V>,
    order: usize,
}

#[derive(Debug)]
pub enum BTreeNode<K: Ord + Clone + Debug, V: Clone + Debug> {
    Leaf(LeafNode<K, V>),
    Internal(InternalNode<K, V>),
}

#[derive(Debug)]
struct LeafNode<K: Ord + Clone + Debug, V: Clone + Debug> {
    keys: Vec<K>,
    values: Vec<V>,
}

#[derive(Debug)]
struct InternalNode<K: Ord + Clone + Debug, V: Clone + Debug> {
    keys: Vec<K>,
    children: Vec<Box<BTreeNode<K, V>>>,
}