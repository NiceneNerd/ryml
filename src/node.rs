use crate::inner::NodeData;

use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Seed<'k> {
    None,
    Index(usize),
    Key(&'k str),
}

impl From<usize> for Seed<'_> {
    fn from(index: usize) -> Self {
        Seed::Index(index)
    }
}

impl<'k> From<&'k str> for Seed<'k> {
    fn from(key: &'k str) -> Self {
        Seed::Key(key)
    }
}

/// A reference to a node in the tree.
#[derive(Debug, Clone)]
pub struct NodeRef<'t, 'k, T: AsRef<Tree<'t>>> {
    tree: T,
    index: usize,
    seed: Seed<'k>,
    _compiler_hack: PhantomData<&'t ()>,
}

impl<'t, T: AsRef<Tree<'t>>> PartialEq for NodeRef<'t, '_, T> {
    fn eq(&self, other: &Self) -> bool {
        self.tree.as_ref() == other.tree.as_ref() && self.index == other.index
    }
}

impl<'t, T: AsRef<Tree<'t>>> NodeRef<'t, '_, T> {
    pub(crate) fn new_exists<'na>(tree: T, index: usize) -> NodeRef<'t, 'na, T> {
        NodeRef {
            tree,
            index,
            seed: Seed::None,
            _compiler_hack: PhantomData,
        }
    }

    pub(crate) fn new_with_key(tree: T, key: &'_ str) -> NodeRef<'t, '_, T> {
        NodeRef {
            tree,
            index: NONE,
            seed: Seed::Key(key),
            _compiler_hack: PhantomData,
        }
    }

    pub(crate) fn new_with_index<'na>(tree: T, index: usize) -> NodeRef<'t, 'na, T> {
        NodeRef {
            tree,
            index: NONE,
            seed: Seed::Index(index),
            _compiler_hack: PhantomData,
        }
    }

    /// Get the tree the node belongs to.
    #[inline(always)]
    pub fn tree(&self) -> &Tree<'_> {
        self.tree.as_ref()
    }

    /// Get the node data, if it exists and is still valid.
    ///
    /// **Note**: If the tree has been
    /// [`reordered`](Tree#method.reorder), this may not return the
    /// same node as it orginally pointed to. However, it is guaranteed to
    /// still point to a valid node on the tree.
    #[inline(always)]
    pub fn data(&'t self) -> Option<&NodeData<'t>> {
        let ptr = self.tree.as_ref().inner.get(self.index).ok()?;
        unsafe { ptr.as_ref() }
    }

    /// Get the node data, assuming it still exists and is valid.
    ///
    /// # Safety
    /// Calling this method if the node no longer exists is undefined
    /// behaviour.
    #[inline(always)]
    pub unsafe fn data_unchecked(&'t self) -> &NodeData<'t> {
        self.tree
            .as_ref()
            .inner
            .get(self.index)
            .unwrap_unchecked()
            .as_ref()
            .unwrap_unchecked()
    }

    /// Check if the node reference points to a valid node.
    #[inline(always)]
    pub fn is_valid(&self) -> bool {
        self.index != NONE && self.index < self.tree.as_ref().len()
    }

    /// Check if the node reference holds a seed for a non-existent node.
    #[inline(always)]
    pub fn is_seed(&self) -> bool {
        self.seed != Seed::None
    }

    /// Get the node type, if it exists.
    #[inline(always)]
    pub fn node_type(&self) -> Result<NodeType> {
        self.tree.as_ref().node_type(self.index)
    }

    /// Get the node type name, if it exists.
    #[inline(always)]
    pub fn node_type_as_str(&self) -> Result<&str> {
        self.tree.as_ref().node_type_as_str(self.index)
    }

    /// Get the node key, if it exists.
    #[inline(always)]
    pub fn key(&self) -> Result<&str> {
        self.tree.as_ref().key(self.index)
    }

    /// Get the tag on the node key, if it exists.
    #[inline(always)]
    pub fn key_tag(&self) -> Result<&str> {
        self.tree.as_ref().key_tag(self.index)
    }

    /// Get the reference on the node key, if it exists.
    #[inline(always)]
    pub fn key_ref(&self) -> Result<&str> {
        self.tree.as_ref().key_ref(self.index)
    }

    /// Get the anchor on the node key, if it exists.
    #[inline(always)]
    pub fn key_anchor(&self) -> Result<&str> {
        self.tree.as_ref().key_anchor(self.index)
    }

    /// Get the scalar data of the node key, if it exists.
    #[inline(always)]
    pub fn key_scalar(&self) -> Result<&NodeScalar> {
        self.tree.as_ref().key_scalar(self.index)
    }

    /// Get the node value, if it exists.
    #[inline(always)]
    pub fn val(&self) -> Result<&str> {
        self.tree.as_ref().val(self.index)
    }

    /// Get the tag on the node value, if it exists.
    #[inline(always)]
    pub fn val_tag(&self) -> Result<&str> {
        self.tree.as_ref().val_tag(self.index)
    }

    /// Get the reference on the node value, if it exists.
    #[inline(always)]
    pub fn val_ref(&self) -> Result<&str> {
        self.tree.as_ref().val_ref(self.index)
    }

    /// Get the anchor on the node value, if it exists.
    #[inline(always)]
    pub fn val_anchor(&self) -> Result<&str> {
        self.tree.as_ref().val_anchor(self.index)
    }

    /// Get the scalar data of the node value, if it exists.
    #[inline(always)]
    pub fn val_scalar(&self) -> Result<&NodeScalar> {
        self.tree.as_ref().val_scalar(self.index)
    }

    /// Check if the node is a stream
    #[inline(always)]
    pub fn is_stream(&self) -> Result<bool> {
        self.tree.as_ref().is_stream(self.index)
    }

    /// Check if the node is a doc
    #[inline(always)]
    pub fn is_doc(&self) -> Result<bool> {
        self.tree.as_ref().is_doc(self.index)
    }

    /// Check if the node is a container
    #[inline(always)]
    pub fn is_container(&self) -> Result<bool> {
        self.tree.as_ref().is_container(self.index)
    }

    /// Check if the node is a map
    #[inline(always)]
    pub fn is_map(&self) -> Result<bool> {
        self.tree.as_ref().is_map(self.index)
    }

    /// Check if the node is a seq
    #[inline(always)]
    pub fn is_seq(&self) -> Result<bool> {
        self.tree.as_ref().is_seq(self.index)
    }

    /// Check if the node has a value
    #[inline(always)]
    pub fn has_val(&self) -> Result<bool> {
        self.tree.as_ref().has_val(self.index)
    }

    /// Check if the node has a key
    #[inline(always)]
    pub fn has_key(&self) -> Result<bool> {
        self.tree.as_ref().has_key(self.index)
    }

    /// Check if the node is a value
    #[inline(always)]
    pub fn is_val(&self) -> Result<bool> {
        self.tree.as_ref().is_val(self.index)
    }

    /// Check if the node is a keyval
    #[inline(always)]
    pub fn is_keyval(&self) -> Result<bool> {
        self.tree.as_ref().is_keyval(self.index)
    }

    /// Check if the node has a key tag
    #[inline(always)]
    pub fn has_key_tag(&self) -> Result<bool> {
        self.tree.as_ref().has_key_tag(self.index)
    }

    /// Check if the node has a value tag
    #[inline(always)]
    pub fn has_val_tag(&self) -> Result<bool> {
        self.tree.as_ref().has_val_tag(self.index)
    }

    /// Check if the node has a key anchor
    #[inline(always)]
    pub fn has_key_anchor(&self) -> Result<bool> {
        self.tree.as_ref().has_key_anchor(self.index)
    }

    /// Check if the node has a value anchor
    #[inline(always)]
    pub fn has_val_anchor(&self) -> Result<bool> {
        self.tree.as_ref().has_val_anchor(self.index)
    }

    /// Check if the node has a anchor
    #[inline(always)]
    pub fn has_anchor(&self, anchor: &str) -> Result<bool> {
        self.tree.as_ref().has_anchor(self.index, anchor)
    }

    /// Check if the node is a anchor
    #[inline(always)]
    pub fn is_anchor(&self) -> Result<bool> {
        self.tree.as_ref().is_anchor(self.index)
    }

    /// Check if the node is a key ref
    #[inline(always)]
    pub fn is_key_ref(&self) -> Result<bool> {
        self.tree.as_ref().is_key_ref(self.index)
    }

    /// Check if the node is a value ref
    #[inline(always)]
    pub fn is_val_ref(&self) -> Result<bool> {
        self.tree.as_ref().is_val_ref(self.index)
    }

    /// Check if the node is a ref
    #[inline(always)]
    pub fn is_ref(&self) -> Result<bool> {
        self.tree.as_ref().is_ref(self.index)
    }

    /// Check if the node is a anchor or ref
    #[inline(always)]
    pub fn is_anchor_or_ref(&self) -> Result<bool> {
        self.tree.as_ref().is_anchor_or_ref(self.index)
    }

    /// Check if the node is a key quoted
    #[inline(always)]
    pub fn is_key_quoted(&self) -> Result<bool> {
        self.tree.as_ref().is_key_quoted(self.index)
    }

    /// Check if the node is a value quoted
    #[inline(always)]
    pub fn is_val_quoted(&self) -> Result<bool> {
        self.tree.as_ref().is_val_quoted(self.index)
    }

    /// Check if the node is a quoted
    #[inline(always)]
    pub fn is_quoted(&self) -> Result<bool> {
        self.tree.as_ref().is_quoted(self.index)
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

    /// Get a mutable reference to the node data, if it exists and is still
    /// valid.
    pub fn data_mut(&'t mut self) -> Option<&mut NodeData<'t>> {
        let ptr = inner::ffi::Tree::get_mut(self.tree.as_mut().inner.pin_mut(), self.index).ok()?;
        unsafe { ptr.as_mut() }
    }

    /// Get a mutable reference to the node data, assuming it still exists and
    /// is valid.
    ///
    /// # Safety
    /// Calling this method if the node no longer exists is undefined behaviour
    /// and should be used with the utmost caution.
    #[inline(always)]
    pub unsafe fn data_unchecked_mut(&'t mut self) -> &mut NodeData<'t> {
        inner::ffi::Tree::get_mut(self.tree.as_mut().inner.pin_mut(), self.index)
            .unwrap_unchecked()
            .as_mut()
            .unwrap_unchecked()
    }
}
