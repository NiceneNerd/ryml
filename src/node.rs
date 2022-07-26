use crate::inner::NodeData;

use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Seed<'k> {
    None,
    Index(usize),
    Key(&'k str),
}

/// A reference to a node in the tree.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NodeRef<'t, 'k, T: AsRef<Tree<'t>>> {
    pub(crate) tree: T,
    pub(crate) index: usize,
    pub(crate) seed: Seed<'k>,
    pub(crate) _compiler_hack: PhantomData<&'t ()>,
}

impl<'t, T: AsRef<Tree<'t>>> NodeRef<'t, '_, T> {
    /// Get the tree the node belongs to.
    #[inline(always)]
    pub fn tree(&self) -> &Tree<'_> {
        self.tree.as_ref()
    }

    /// Get the node data, if it exists and is still valid.
    #[inline(always)]
    pub fn get(&'t self) -> Option<&NodeData<'t>> {
        let ptr = self.tree.as_ref().inner.get(self.index).ok()?;
        unsafe { ptr.as_ref() }
    }
}

impl<'t, T> NodeRef<'t, '_, T>
where
    T: AsRef<Tree<'t>> + AsMut<Tree<'t>>,
{
    /// Get a mutable reference to the tree the node belongs to.
    #[inline(always)]
    pub fn tree_mut(&'t mut self) -> &mut Tree<'t> {
        self.tree.as_mut()
    }

    /// Get a mutable reference to the node data.
    pub fn get_mut(&'t mut self) -> Option<&mut NodeData<'t>> {
        let ptr = inner::ffi::Tree::get_mut(self.tree.as_mut().inner.pin_mut(), self.index).ok()?;
        unsafe { ptr.as_mut() }
    }
}
