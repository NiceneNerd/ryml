use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Seed<'k> {
    None,
    Index(usize),
    Key(&'k str),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NodeRef<'t, 'k> {
    tree: &'t Tree<'t>,
    index: usize,
    seed: Seed<'k>,
}
