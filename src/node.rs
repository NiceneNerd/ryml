use super::*;
use crate::inner::NodeData;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SeedInner<'k> {
    None,
    Index(usize),
    Key(&'k str),
}

/// A seed value used for lazy assignment of new nodes by a [`NodeRef`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
// The real value is wrapped to prevent public construction.
pub struct Seed<'k>(SeedInner<'k>);

impl From<usize> for Seed<'_> {
    fn from(index: usize) -> Self {
        Self(SeedInner::Index(index))
    }
}

impl<'k> From<&'k str> for Seed<'k> {
    fn from(key: &'k str) -> Self {
        Self(SeedInner::Key(key))
    }
}

macro_rules! tree_ref_mut {
    ($tree:expr) => {{
        let tree_ref = $tree as *mut Tree<'_>;
        unsafe { tree_ref.as_mut().unwrap() }
    }};
}

/// An iterator over the children of a [`NodeRef`].
pub struct NodeIterator<'a, 't, 'k, T: 't + AsRef<Tree<'a>>> {
    tree: T,
    node_index: usize,
    index: usize,
    len: usize,
    _hack: PhantomData<(&'a (), &'k (), &'t ())>,
}

impl<'a, 't, 'k> Iterator for NodeIterator<'a, 't, 'k, &'t Tree<'a>> {
    type Item = NodeRef<'a, 't, 'k, &'t Tree<'a>>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.len {
            None
        } else {
            let index = self.tree.child_at(self.node_index, self.index).ok()?;
            let node = NodeRef::new_exists(self.tree, index);
            self.index += 1;
            Some(node)
        }
    }
}

impl<'a, 't, 'k> Iterator for NodeIterator<'a, 't, 'k, &'t mut Tree<'a>> {
    type Item = NodeRef<'a, 't, 'k, &'t mut Tree<'a>>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.len {
            None
        } else {
            let index = self.tree.child_at(self.node_index, self.index).ok()?;
            let node = NodeRef::new_exists(tree_ref_mut!(self.tree), index);
            self.index += 1;
            Some(node)
        }
    }
}

impl<'a, 't, 'k> ExactSizeIterator for NodeIterator<'a, 't, 'k, &'t Tree<'a>> {
    fn len(&self) -> usize {
        self.len
    }
}

impl<'a, 't, 'k> ExactSizeIterator for NodeIterator<'a, 't, 'k, &'t mut Tree<'a>> {
    fn len(&self) -> usize {
        self.len
    }
}

/// A reference to a node in the tree.
#[derive(Debug, Clone)]
pub struct NodeRef<'a, 't, 'k, T>
where
    T: AsRef<Tree<'a>> + 't,
    'a: 't,
{
    tree: T,
    index: usize,
    seed: Seed<'k>,
    _hack: PhantomData<(&'t (), &'a ())>,
}

impl<'a, 't, T: AsRef<Tree<'a>> + 't> PartialEq for NodeRef<'a, 't, '_, T> {
    fn eq(&self, other: &Self) -> bool {
        self.tree.as_ref() == other.tree.as_ref() && self.index == other.index
    }
}

macro_rules! tree_ref {
    ($tree:expr) => {{
        let tree_ref = $tree.as_ref() as *const Tree<'_>;
        unsafe { tree_ref.as_ref().unwrap() }
    }};
}

impl<'a, 't, T> NodeRef<'a, 't, '_, T>
where
    T: AsRef<Tree<'a>> + 't,
    'a: 't,
{
    pub(crate) fn new_exists<'na>(tree: T, index: usize) -> NodeRef<'a, 't, 'na, T> {
        NodeRef {
            tree,
            index,
            seed: Seed(SeedInner::None),
            _hack: PhantomData,
        }
    }

    /// Get the tree the node belongs to.
    #[inline(always)]
    pub fn tree<'r>(&'r self) -> &'t Tree<'a> {
        tree_ref!(self.tree)
    }

    /// Get the node data, if it exists and is still valid.
    ///
    /// **Note**: If the tree has been
    /// [`reordered`](Tree#method.reorder), this may not return the
    /// same node as it orginally pointed to. However, it is guaranteed to
    /// still point to a valid node on the tree.
    #[inline(always)]
    pub fn data<'r>(&'r self) -> Option<&NodeData<'t>> {
        let tree_ref = tree_ref!(self.tree);
        let ptr = tree_ref.inner.get(self.index).ok()?;
        unsafe { ptr.as_ref() }
    }

    /// Get the node data, assuming it still exists and is valid.
    ///
    /// # Safety
    /// Calling this method if the node no longer exists is undefined
    /// behaviour.
    #[inline(always)]
    pub unsafe fn data_unchecked<'r>(&'r self) -> &'t NodeData<'t> {
        #[allow(unused_unsafe)]
        let tree_ref = tree_ref!(self.tree);
        tree_ref
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
        self.seed != Seed(SeedInner::None)
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

    /// Check if the parent is a sequence
    #[inline(always)]
    pub fn parent_is_seq(&self) -> Result<bool> {
        self.tree.as_ref().parent_is_seq(self.index)
    }

    /// Check if the parent is a map
    #[inline(always)]
    pub fn parent_is_map(&self) -> Result<bool> {
        self.tree.as_ref().parent_is_map(self.index)
    }

    /// Returns true if the name and value are empty and the node has no
    /// children.
    #[inline(always)]
    pub fn is_empty(&self) -> Result<bool> {
        self.tree.as_ref().is_node_empty(self.index)
    }

    /// Returns true if the node is the root of the tree.
    #[inline(always)]
    pub fn is_root(&self) -> Result<bool> {
        self.tree.as_ref().is_root(self.index)
    }

    /// Returns true if the node has a parent.
    #[inline(always)]
    pub fn has_parent(&self) -> Result<bool> {
        self.tree.as_ref().has_parent(self.index)
    }

    /// Returns true if the node is a map and has a child with the given key.
    #[inline(always)]
    pub fn has_child(&self, key: &str) -> Result<bool> {
        match self.tree.as_ref().find_child(self.index, key) {
            Ok(_) => Ok(true),
            Err(e) => match e {
                Error::NodeNotFound => Ok(false),
                _ => Err(e),
            },
        }
    }

    /// Returns true if the node is a sequence and has a child at the given
    /// position.
    #[inline(always)]
    pub fn has_child_at(&self, pos: usize) -> Result<bool> {
        match self.tree.as_ref().child_pos(self.index, pos) {
            Ok(_) => Ok(true),
            Err(e) => match e {
                Error::NodeNotFound => Ok(false),
                _ => Err(e),
            },
        }
    }

    /// Returns true if the node has children.
    #[inline(always)]
    pub fn has_children(&self) -> Result<bool> {
        self.tree.as_ref().has_children(self.index)
    }

    /// Returns true if the node has a sibling with the given key.
    #[inline(always)]
    pub fn has_sibling(&self, key: &str) -> Result<bool> {
        match self.tree.as_ref().find_sibling(self.index, key) {
            Ok(_) => Ok(true),
            Err(e) => match e {
                Error::NodeNotFound => Ok(false),
                _ => Err(e),
            },
        }
    }

    /// Returns true if the node has a sibling at the given position.
    #[inline(always)]
    pub fn has_sibling_at(&self, index: usize) -> Result<bool> {
        match self.tree.as_ref().has_parent(self.index) {
            Ok(true) => match self.tree.as_ref().child_pos(self.index, index) {
                Ok(_) => Ok(true),
                Err(e) => match e {
                    Error::NodeNotFound => Ok(false),
                    _ => Err(e),
                },
            },
            Ok(false) => Ok(false),
            Err(e) => Err(e),
        }
    }

    /// Returns true if the node has siblings.
    ///
    /// **Note**: This corresponds to `has_other_siblings()` in the C++ API, as
    /// the plain `has_siblings()` function always returns true by counting the
    /// node itself, which seems rather pointless and is not the obvious meaning
    /// of a method by the name.
    #[inline(always)]
    pub fn has_siblings(&self) -> Result<bool> {
        self.tree.as_ref().has_siblings(self.index)
    }

    /// Returns a [`NodeRef`] to the parent node, if it exists.
    #[inline(always)]
    pub fn parent<'r>(&'r self) -> Result<NodeRef<'a, 't, '_, &'t Tree<'a>>> {
        let parent = self.tree.as_ref().parent(self.index)?;
        Ok(NodeRef {
            tree: tree_ref!(self.tree),
            index: parent,
            seed: Seed(SeedInner::None),
            _hack: PhantomData,
        })
    }

    /// Returns a [`NodeRef`] to the previous sibling, if it exists.
    #[inline(always)]
    pub fn prev_sibling<'r>(&'r self) -> Result<NodeRef<'a, 't, '_, &'t Tree<'a>>> {
        let sibling = self.tree.as_ref().prev_sibling(self.index)?;
        Ok(NodeRef {
            tree: tree_ref!(self.tree),
            index: sibling,
            seed: Seed(SeedInner::None),
            _hack: PhantomData,
        })
    }

    /// Returns a [`NodeRef`] to the next sibling, if it exists.
    #[inline(always)]
    pub fn next_sibling<'r>(&'r self) -> Result<NodeRef<'a, 't, '_, &'t Tree<'a>>> {
        let sibling = self.tree.as_ref().next_sibling(self.index)?;
        Ok(NodeRef {
            tree: tree_ref!(self.tree),
            index: sibling,
            seed: Seed(SeedInner::None),
            _hack: PhantomData,
        })
    }

    /// Get the number of children of the node.
    #[inline(always)]
    pub fn num_children(&self) -> Result<usize> {
        self.tree.as_ref().num_children(self.index)
    }

    /// Get the position of the child node in this node's children.
    #[inline(always)]
    pub fn child_pos(&self, child: &NodeRef<'a, 't, '_, T>) -> Result<usize> {
        self.tree.as_ref().child_pos(self.index, child.index)
    }

    /// Get a [`NodeRef`] to the first child of this node, if it exists.
    #[inline(always)]
    pub fn first_child<'r>(&'r self) -> Result<NodeRef<'a, 't, '_, &'t Tree<'a>>> {
        let child = self.tree.as_ref().first_child(self.index)?;
        Ok(NodeRef {
            tree: tree_ref!(self.tree),
            index: child,
            seed: Seed(SeedInner::None),
            _hack: PhantomData,
        })
    }

    /// Get a [`NodeRef`] to the last child of this node, if it exists.
    #[inline(always)]
    pub fn last_child<'r>(&'r self) -> Result<NodeRef<'a, 't, '_, &'t Tree<'a>>> {
        let child = self.tree.as_ref().last_child(self.index)?;
        Ok(NodeRef {
            tree: tree_ref!(self.tree),
            index: child,
            seed: Seed(SeedInner::None),
            _hack: PhantomData,
        })
    }

    /// Get a [`NodeRef`] to the child of this node at the given position, if
    /// it exists.
    #[inline(always)]
    pub fn child_at<'r>(&'r self, pos: usize) -> Result<NodeRef<'a, 't, '_, &'t Tree<'a>>> {
        let child = self.tree.as_ref().child_at(self.index, pos)?;
        Ok(NodeRef {
            tree: tree_ref!(self.tree),
            index: child,
            seed: Seed(SeedInner::None),
            _hack: PhantomData,
        })
    }

    /// Get a [`NodeRef`] to the child of this node with the given key, if it
    /// exists.
    #[inline(always)]
    pub fn find_child<'r>(&'r self, key: &str) -> Result<NodeRef<'a, 't, '_, &'t Tree<'a>>> {
        let child = self.tree.as_ref().find_child(self.index, key)?;
        Ok(NodeRef {
            tree: tree_ref!(self.tree),
            index: child,
            seed: Seed(SeedInner::None),
            _hack: PhantomData,
        })
    }

    /// Get a [`NodeRef`] to the first sibling of this node, if it exists.
    #[inline(always)]
    pub fn first_sibling<'r>(&'r self) -> Result<NodeRef<'a, 't, '_, &'t Tree<'a>>> {
        let sibling = self.tree.as_ref().first_sibling(self.index)?;
        Ok(NodeRef {
            tree: tree_ref!(self.tree),
            index: sibling,
            seed: Seed(SeedInner::None),
            _hack: PhantomData,
        })
    }

    /// Get a [`NodeRef`] to the last sibling of this node, if it exists.
    #[inline(always)]
    pub fn last_sibling<'r>(&'r self) -> Result<NodeRef<'a, 't, '_, &'t Tree<'a>>> {
        let sibling = self.tree.as_ref().last_sibling(self.index)?;
        Ok(NodeRef {
            tree: tree_ref!(self.tree),
            index: sibling,
            seed: Seed(SeedInner::None),
            _hack: PhantomData,
        })
    }

    /// Get a [`NodeRef`] to the sibling of this node at the given position, if
    /// it exists.
    #[inline(always)]
    pub fn sibling_at<'r>(&'r self, pos: usize) -> Result<NodeRef<'a, 't, '_, &'t Tree<'a>>> {
        let sibling = self.tree.as_ref().sibling_at(self.index, pos)?;
        Ok(NodeRef {
            tree: tree_ref!(self.tree),
            index: sibling,
            seed: Seed(SeedInner::None),
            _hack: PhantomData,
        })
    }

    /// Get a [`NodeRef`] to the sibling of this node with the given key, if it
    /// exists.
    #[inline(always)]
    pub fn find_sibling<'r>(&'r self, key: &str) -> Result<NodeRef<'a, 't, '_, &'t Tree<'a>>> {
        let sibling = self.tree.as_ref().find_sibling(self.index, key)?;
        Ok(NodeRef {
            tree: tree_ref!(self.tree),
            index: sibling,
            seed: Seed(SeedInner::None),
            _hack: PhantomData,
        })
    }

    /// Get a [`NodeRef`] to a child of this node by its given key (if this node
    /// is a map) or given position (if this node is a sequence).
    ///
    /// Unlike [`get_mut`](#method.get_mut), this method will return a
    /// `NodeNotFound` error if the child node does not exist. It will also
    /// return this error if the current node does not exist.
    pub fn get<'r, 'k2, S: Into<Seed<'k2>>>(
        &'r self,
        lookup: S,
    ) -> Result<NodeRef<'a, 't, 'k2, &'t Tree<'a>>> {
        if self.seed != Seed(SeedInner::None) {
            return Err(Error::NodeNotFound);
        }
        let seed = lookup.into();
        match seed.0 {
            SeedInner::Index(child_pos) => Ok(NodeRef {
                tree: tree_ref!(self.tree),
                index: self.tree.as_ref().child_at(self.index, child_pos)?,
                seed: Seed(SeedInner::None),
                _hack: PhantomData,
            }),
            SeedInner::Key(child_key) => Ok(NodeRef {
                tree: tree_ref!(self.tree),
                index: self.tree.as_ref().find_child(self.index, child_key)?,
                seed: Seed(SeedInner::None),
                _hack: PhantomData,
            }),
            // This is unreachable because the public API does not expose any methods to pass a
            // `Seed` set to `SeedInner::None`.
            _ => unsafe { core::hint::unreachable_unchecked() },
        }
    }

    /// Iterate over the children of this node, if it exists and is valid.
    #[inline(always)]
    pub fn iter(&self) -> Result<NodeIterator<'a, 't, '_, &'t Tree<'a>>> {
        if self.seed.0 != SeedInner::None {
            return Err(Error::NodeNotFound);
        }
        Ok(NodeIterator {
            tree: tree_ref!(self.tree),
            node_index: self.index,
            index: 0,
            len: self.num_children()?,
            _hack: PhantomData,
        })
    }
}

/// Lazy assignment for a node reference based on its seed. If the node already
/// exists, we simply use the existing node ID. If the node doesn't exist, we
/// use the child index or key by which it was queried to construct it, and then
/// make use of the inserted node ID.
macro_rules! maybe_construct {
    ($self:expr) => {
        match $self.seed.0 {
            SeedInner::None => $self.index,
            SeedInner::Index(idx) => {
                let after = $self.tree.as_ref().child_at($self.index, idx - 1)?;
                let index = $self.tree.insert_child($self.index, after)?;
                $self.index = index;
                $self.seed = Seed(SeedInner::None);
                index
            }
            SeedInner::Key(key) => {
                let index = $self.tree.append_child($self.index)?;
                $self.tree.set_key(index, key)?;
                $self.index = index;
                $self.seed = Seed(SeedInner::None);
                index
            }
        }
    };
}

impl<'a, 't> NodeRef<'a, 't, '_, &'t mut Tree<'a>> {
    pub(crate) fn new_exists_mut<'na>(
        tree: &'t mut Tree<'a>,
        index: usize,
    ) -> NodeRef<'a, 't, 'na, &'t mut Tree<'a>> {
        NodeRef {
            tree,
            index,
            seed: Seed(SeedInner::None),
            _hack: PhantomData,
        }
    }

    /// Get a mutable reference to the tree the node belongs to.
    #[inline(always)]
    pub fn tree_mut<'r>(&'r mut self) -> &'t mut Tree<'a> {
        tree_ref_mut!(self.tree)
    }

    /// Get a mutable reference to the node data, if it exists and is still
    /// valid.
    pub fn data_mut<'r>(&'r mut self) -> Option<&'t mut NodeData<'t>> {
        let tree_ref = tree_ref_mut!(self.tree);
        let ptr = inner::ffi::Tree::get_mut(tree_ref.inner.pin_mut(), self.index).ok()?;
        unsafe { ptr.as_mut() }
    }

    /// Get a mutable reference to the node data, assuming it still exists and
    /// is valid.
    ///
    /// # Safety
    /// Calling this method if the node no longer exists is undefined behaviour
    /// and should be used with the utmost caution.
    #[inline(always)]
    pub unsafe fn data_unchecked_mut<'r>(&'r mut self) -> &'t mut NodeData<'t> {
        #[allow(unused_unsafe)]
        let tree_ref = tree_ref_mut!(self.tree);
        inner::ffi::Tree::get_mut(tree_ref.inner.pin_mut(), self.index)
            .unwrap_unchecked()
            .as_mut()
            .unwrap_unchecked()
    }

    /// Returns a mutable [`NodeRef`] to the parent node, if it exists.
    #[inline(always)]
    pub fn parent_mut<'r>(&'r mut self) -> Result<NodeRef<'a, 't, '_, &'t mut Tree<'a>>> {
        let parent = self.tree.as_ref().parent(self.index)?;
        Ok(NodeRef {
            tree: tree_ref_mut!(self.tree),
            index: parent,
            seed: Seed(SeedInner::None),
            _hack: PhantomData,
        })
    }

    /// Returns a mutable [`NodeRef`] to the previous sibling, if it exists.
    #[inline(always)]
    pub fn prev_sibling_mut<'r>(&'r mut self) -> Result<NodeRef<'a, 't, '_, &'t mut Tree<'a>>> {
        let sibling = self.tree.as_ref().prev_sibling(self.index)?;
        Ok(NodeRef {
            tree: tree_ref_mut!(self.tree),
            index: sibling,
            seed: Seed(SeedInner::None),
            _hack: PhantomData,
        })
    }

    /// Returns a mutable [`NodeRef`] to the next sibling, if it exists.
    #[inline(always)]
    pub fn next_sibling_mut<'r>(&'r mut self) -> Result<NodeRef<'a, 't, '_, &'t mut Tree<'a>>> {
        let sibling = self.tree.as_ref().next_sibling(self.index)?;
        Ok(NodeRef {
            tree: tree_ref_mut!(self.tree),
            index: sibling,
            seed: Seed(SeedInner::None),
            _hack: PhantomData,
        })
    }

    /// Get a mutable [`NodeRef`] to the first child of this node, if it exists.
    #[inline(always)]
    pub fn first_child_mut<'r>(&'r mut self) -> Result<NodeRef<'a, 't, '_, &'t mut Tree<'a>>> {
        let child = self.tree.as_ref().first_child(self.index)?;
        Ok(NodeRef {
            tree: tree_ref_mut!(self.tree),
            index: child,
            seed: Seed(SeedInner::None),
            _hack: PhantomData,
        })
    }

    /// Get a mutable [`NodeRef`] to the last child of this node, if it exists.
    #[inline(always)]
    pub fn last_child_mut<'r>(&'r mut self) -> Result<NodeRef<'a, 't, '_, &'t mut Tree<'a>>> {
        let child = self.tree.as_ref().last_child(self.index)?;
        Ok(NodeRef {
            tree: tree_ref_mut!(self.tree),
            index: child,
            seed: Seed(SeedInner::None),
            _hack: PhantomData,
        })
    }

    /// Get a mutable [`NodeRef`] to the child of this node at the given
    /// position, if it exists.
    #[inline(always)]
    pub fn child_at_mut<'r>(
        &'r mut self,
        pos: usize,
    ) -> Result<NodeRef<'a, 't, '_, &'t mut Tree<'a>>> {
        let child = self.tree.as_ref().child_at(self.index, pos)?;
        Ok(NodeRef {
            tree: tree_ref_mut!(self.tree),
            index: child,
            seed: Seed(SeedInner::None),
            _hack: PhantomData,
        })
    }

    /// Get a mutable [`NodeRef`] to the child of this node with the given key,
    /// if it exists.
    #[inline(always)]
    pub fn find_child_mut<'r>(
        &'r mut self,
        key: &str,
    ) -> Result<NodeRef<'a, 't, '_, &'t mut Tree<'a>>> {
        let child = self.tree.as_ref().find_child(self.index, key)?;
        Ok(NodeRef {
            tree: tree_ref_mut!(self.tree),
            index: child,
            seed: Seed(SeedInner::None),
            _hack: PhantomData,
        })
    }

    /// Get a mutable [`NodeRef`] to the first sibling of this node, if it
    /// exists.
    #[inline(always)]
    pub fn first_sibling_mut<'r>(&'r mut self) -> Result<NodeRef<'a, 't, '_, &'t mut Tree<'a>>> {
        let sibling = self.tree.as_ref().first_sibling(self.index)?;
        Ok(NodeRef {
            tree: tree_ref_mut!(self.tree),
            index: sibling,
            seed: Seed(SeedInner::None),
            _hack: PhantomData,
        })
    }

    /// Get a mutable [`NodeRef`] to the last sibling of this node, if it
    /// exists.
    #[inline(always)]
    pub fn last_sibling_mut<'r>(&'r mut self) -> Result<NodeRef<'a, 't, '_, &'t mut Tree<'a>>> {
        let sibling = self.tree.as_ref().last_sibling(self.index)?;
        Ok(NodeRef {
            tree: tree_ref_mut!(self.tree),
            index: sibling,
            seed: Seed(SeedInner::None),
            _hack: PhantomData,
        })
    }

    /// Get a mutable [`NodeRef`] to the sibling of this node at the given
    /// position, if it exists.
    #[inline(always)]
    pub fn sibling_at_mut<'r>(
        &'r mut self,
        pos: usize,
    ) -> Result<NodeRef<'a, 't, '_, &'t mut Tree<'a>>> {
        let sibling = self.tree.as_ref().sibling_at(self.index, pos)?;
        Ok(NodeRef {
            tree: tree_ref_mut!(self.tree),
            index: sibling,
            seed: Seed(SeedInner::None),
            _hack: PhantomData,
        })
    }

    /// Get a mutable [`NodeRef`] to the sibling of this node with the given
    /// key, if it exists.
    #[inline(always)]
    pub fn find_sibling_mut<'r>(
        &'r mut self,
        key: &str,
    ) -> Result<NodeRef<'a, 't, '_, &'t mut Tree<'a>>> {
        let sibling = self.tree.as_ref().find_sibling(self.index, key)?;
        Ok(NodeRef {
            tree: tree_ref_mut!(self.tree),
            index: sibling,
            seed: Seed(SeedInner::None),
            _hack: PhantomData,
        })
    }

    /// Change the type of the node, resetting its contents if necessary and
    /// returning whether the change was possible.
    #[inline(always)]
    pub fn change_type(&mut self, node_type: NodeType) -> Result<bool> {
        let index = maybe_construct!(self);
        self.tree.change_type(index, node_type)
    }

    /// Set flags on the node.
    #[inline(always)]
    pub fn set_type_flags(&mut self, more_flags: NodeType) -> Result<()> {
        let index = maybe_construct!(self);
        self.tree.set_flags(index, more_flags)
    }

    /// Sets the node's key.
    #[inline(always)]
    pub fn set_key(&mut self, key: &str) -> Result<()> {
        let index = maybe_construct!(self);
        self.tree.set_key(index, key)
    }

    /// Sets the node's value.
    #[inline(always)]
    pub fn set_val(&mut self, value: &str) -> Result<()> {
        let index = maybe_construct!(self);
        self.tree.set_val(index, value)
    }

    /// Set the tag on the node key.
    #[inline(always)]
    pub fn set_key_tag(&mut self, v: &str) -> Result<()> {
        let index = maybe_construct!(self);
        self.tree.set_key_tag(index, v)
    }

    /// Set the tag on the node val.
    #[inline(always)]
    pub fn set_val_tag(&mut self, v: &str) -> Result<()> {
        let index = maybe_construct!(self);
        self.tree.set_val_tag(index, v)
    }

    /// Set the anchor on the node key.
    #[inline(always)]
    pub fn set_key_anchor(&mut self, v: &str) -> Result<()> {
        let index = maybe_construct!(self);
        self.tree.set_key_anchor(index, v)
    }

    /// Set the anchor on the node val.
    #[inline(always)]
    pub fn set_val_anchor(&mut self, v: &str) -> Result<()> {
        let index = maybe_construct!(self);
        self.tree.set_val_anchor(index, v)
    }

    /// Set the ref on the node key.
    #[inline(always)]
    pub fn set_key_ref(&mut self, v: &str) -> Result<()> {
        let index = maybe_construct!(self);
        self.tree.set_key_ref(index, v)
    }

    /// Set the ref on the node val.
    #[inline(always)]
    pub fn set_val_ref(&mut self, v: &str) -> Result<()> {
        let index = maybe_construct!(self);
        self.tree.set_val_ref(index, v)
    }

    /// Empties the node and removes any children.
    #[inline(always)]
    pub fn clear(&mut self) -> Result<()> {
        if let Seed(SeedInner::None) = self.seed {
            self.tree.clear_node(self.index)
        } else {
            Ok(())
        }
    }

    /// Clears the node key, if it exists.
    #[inline(always)]
    pub fn clear_key(&mut self) -> Result<()> {
        if let Seed(SeedInner::None) = self.seed {
            self.tree.clear_key(self.index)
        } else {
            Ok(())
        }
    }

    /// Clears the node value, if it exists.
    #[inline(always)]
    pub fn clear_val(&mut self) -> Result<()> {
        if let Seed(SeedInner::None) = self.seed {
            self.tree.clear_val(self.index)
        } else {
            Ok(())
        }
    }

    /// Clear the node's children, if it exists and has any.
    #[inline(always)]
    pub fn clear_children(&mut self) -> Result<()> {
        if let Seed(SeedInner::None) = self.seed {
            self.tree.remove_children(self.index)
        } else {
            Ok(())
        }
    }

    /// Insert a new node as a child of this node, returning a [`NodeRef`] to
    /// the new node.
    #[inline(always)]
    pub fn insert_child<R: AsRef<Tree<'a>>>(
        &mut self,
        after: NodeRef<'a, 't, '_, R>,
    ) -> Result<NodeRef<'a, 't, '_, &'t mut Tree<'a>>> {
        let index = maybe_construct!(self);
        let child_index = self.tree.insert_child(index, after.index)?;
        Ok(NodeRef {
            tree: tree_ref_mut!(self.tree),
            index: child_index,
            seed: Seed(SeedInner::None),
            _hack: PhantomData,
        })
    }

    /// Insert a new node as the first child of this node, returning a
    /// [`NodeRef`] to the new node.
    #[inline(always)]
    pub fn prepend_child(&mut self) -> Result<NodeRef<'a, 't, '_, &'t mut Tree<'a>>> {
        let index = maybe_construct!(self);
        let child_index = self.tree.prepend_child(index)?;
        Ok(NodeRef {
            tree: tree_ref_mut!(self.tree),
            index: child_index,
            seed: Seed(SeedInner::None),
            _hack: PhantomData,
        })
    }

    /// Insert a new node as the last child of this node, returning a
    /// [`NodeRef`] to the new node.
    #[inline(always)]
    pub fn append_child(&mut self) -> Result<NodeRef<'a, 't, '_, &'t mut Tree<'a>>> {
        let index = maybe_construct!(self);
        let child_index = self.tree.append_child(index)?;
        Ok(NodeRef {
            tree: tree_ref_mut!(self.tree),
            index: child_index,
            seed: Seed(SeedInner::None),
            _hack: PhantomData,
        })
    }

    /// Insert a new node as a sibling of this node, returning a [`NodeRef`]
    /// to the new node.
    #[inline(always)]
    pub fn insert_sibling<R: AsRef<Tree<'a>>>(
        &mut self,
        after: NodeRef<'a, 't, '_, R>,
    ) -> Result<NodeRef<'a, 't, '_, &'t mut Tree<'a>>> {
        let index = maybe_construct!(self);
        let sibling_index = self.tree.insert_sibling(index, after.index)?;
        Ok(NodeRef {
            tree: tree_ref_mut!(self.tree),
            index: sibling_index,
            seed: Seed(SeedInner::None),
            _hack: PhantomData,
        })
    }

    /// Insert a new node as the first sibling of this node, returning a
    /// [`NodeRef`] to the new node.
    #[inline(always)]
    pub fn prepend_sibling(&mut self) -> Result<NodeRef<'a, 't, '_, &'t mut Tree<'a>>> {
        let index = maybe_construct!(self);
        let sibling_index = self.tree.prepend_sibling(index)?;
        Ok(NodeRef {
            tree: tree_ref_mut!(self.tree),
            index: sibling_index,
            seed: Seed(SeedInner::None),
            _hack: PhantomData,
        })
    }

    /// Insert a new node as the last sibling of this node, returning a
    /// [`NodeRef`] to the new node.
    #[inline(always)]
    pub fn append_sibling(&mut self) -> Result<NodeRef<'a, 't, '_, &'t mut Tree<'a>>> {
        let index = maybe_construct!(self);
        let sibling_index = self.tree.append_sibling(index)?;
        Ok(NodeRef {
            tree: tree_ref_mut!(self.tree),
            index: sibling_index,
            seed: Seed(SeedInner::None),
            _hack: PhantomData,
        })
    }

    /// Remove the given child from this node.
    #[inline(always)]
    pub fn remove_child(&mut self, child: NodeRef<'a, 't, '_, &'t mut Tree<'a>>) -> Result<()> {
        if self.seed.0 == SeedInner::None {
            self.tree.remove(child.index)
        } else {
            Ok(())
        }
    }

    /// Remove the child at the given index from this node.
    #[inline(always)]
    pub fn remove_child_at(&mut self, pos: usize) -> Result<()> {
        if self.seed.0 == SeedInner::None && pos < self.num_children()? {
            let child_index = self.tree.child_at(self.index, pos)?;
            self.tree.remove(child_index)
        } else {
            Ok(())
        }
    }

    /// Remove the child with the given key from this node.
    #[inline(always)]
    pub fn remove_child_with_key(&mut self, key: &str) -> Result<()> {
        if self.seed.0 == SeedInner::None {
            match self.tree.find_child(self.index, key) {
                Ok(child_index) => self.tree.remove(child_index),
                Err(e) => match e {
                    Error::NodeNotFound => Ok(()),
                    e => Err(e),
                },
            }
        } else {
            Ok(())
        }
    }

    /// Change the node's position within its parent.
    #[inline(always)]
    pub fn move_<R: AsRef<Tree<'a>>>(&mut self, after: NodeRef<'a, 't, '_, R>) -> Result<()> {
        if self.seed.0 == SeedInner::None {
            self.tree.move_node(self.index, after.index)
        } else {
            Ok(())
        }
    }

    /// Move the node to a different parent, which may be in a new tree.
    #[inline(always)]
    pub fn move_to_parent<R: AsRef<Tree<'a>>>(
        &mut self,
        parent: NodeRef<'a, 't, '_, &'t mut Tree<'a>>,
        after: NodeRef<'a, 't, '_, R>,
    ) -> Result<()> {
        if self.seed.0 == SeedInner::None && parent.seed.0 == SeedInner::None {
            if self.tree == parent.tree {
                self.tree
                    .move_node_to_new_parent(self.index, parent.index, after.index)
            } else {
                self.index = parent.tree.move_node_from_tree(
                    self.tree,
                    self.index,
                    parent.index,
                    after.index,
                )?;
                self.tree = tree_ref_mut!(parent.tree);
                Ok(())
            }
        } else {
            Ok(())
        }
    }

    /// Duplicate the node under a new parent, returning a [`NodeRef`] to the
    /// new node.
    #[inline(always)]
    pub fn duplicate<R: AsRef<Tree<'a>>>(
        &mut self,
        parent: NodeRef<'a, 't, '_, &'t mut Tree<'a>>,
        after: NodeRef<'a, 't, '_, R>,
    ) -> Result<NodeRef<'a, 't, '_, &'t mut Tree<'a>>> {
        if self.seed.0 != SeedInner::None || parent.seed.0 != SeedInner::None {
            return Err(Error::NodeNotFound);
        }
        if self.tree == parent.tree {
            let index = self.tree.duplicate(self.index, parent.index, after.index)?;
            Ok(NodeRef {
                tree: tree_ref_mut!(self.tree),
                index,
                seed: Seed(SeedInner::None),
                _hack: PhantomData,
            })
        } else {
            let index = parent.tree.duplicate_from_tree(
                self.tree,
                self.index,
                parent.index,
                after.index,
            )?;
            Ok(NodeRef {
                tree: tree_ref_mut!(parent.tree),
                index,
                seed: Seed(SeedInner::None),
                _hack: PhantomData,
            })
        }
    }

    /// Duplicate the node's children under a new parent.
    #[inline(always)]
    pub fn duplicate_children<R: AsRef<Tree<'a>>>(
        &mut self,
        parent: NodeRef<'a, 't, '_, &'t mut Tree<'a>>,
        after: NodeRef<'a, 't, '_, R>,
    ) -> Result<()> {
        if self.seed.0 != SeedInner::None || parent.seed.0 != SeedInner::None {
            return Err(Error::NodeNotFound);
        }
        if self.tree == parent.tree {
            self.tree
                .duplicate_children(self.index, parent.index, after.index)?;
        } else {
            parent.tree.duplicate_children_from_tree(
                self.tree,
                self.index,
                parent.index,
                after.index,
            )?;
        }
        Ok(())
    }

    /// Get a mutable [`NodeRef`] to a child of this node by its given key (if
    /// this node is a map) or given position (if this node is a sequence).
    ///
    /// Unlike [`get`](#method.get), this method will succeed if the node does
    /// not exist and will retain the seed for lazy assignment. It will still
    /// return a `NodeNotFound` error if the current node does not exist.
    pub fn get_mut<'r, 'k2, S: Into<Seed<'k2>>>(
        &'r mut self,
        lookup: S,
    ) -> Result<NodeRef<'a, 't, 'k2, &'t mut Tree<'a>>> {
        if self.seed != Seed(SeedInner::None) {
            return Err(Error::NodeNotFound);
        }
        let seed = lookup.into();
        let tree_ref = self.tree as *mut Tree;
        match seed.0 {
            SeedInner::Index(child_pos) => match self.tree.as_ref().child_at(self.index, child_pos)
            {
                Ok(index) => Ok(NodeRef {
                    tree: unsafe { tree_ref.as_mut().unwrap() },
                    index,
                    seed: Seed(SeedInner::None),
                    _hack: PhantomData,
                }),
                Err(Error::NodeNotFound) => Ok(NodeRef {
                    tree: unsafe { tree_ref.as_mut().unwrap() },
                    index: self.index,
                    seed,
                    _hack: PhantomData,
                }),
                Err(e) => Err(e),
            },
            SeedInner::Key(child_key) => match self.tree.as_ref().find_child(self.index, child_key)
            {
                Ok(index) => Ok(NodeRef {
                    tree: unsafe { tree_ref.as_mut().unwrap() },
                    index,
                    seed: Seed(SeedInner::None),
                    _hack: PhantomData,
                }),
                Err(Error::NodeNotFound) => Ok(NodeRef {
                    tree: unsafe { tree_ref.as_mut().unwrap() },
                    index: self.index,
                    seed,
                    _hack: PhantomData,
                }),
                Err(e) => Err(e),
            },
            _ => unreachable!(),
        }
    }

    /// Iterate mutably over the children of this node, if it exists and is
    /// valid.
    #[inline(always)]
    pub fn iter_mut(&mut self) -> Result<NodeIterator<'a, 't, '_, &'t mut Tree<'a>>> {
        if self.seed.0 != SeedInner::None {
            return Err(Error::NodeNotFound);
        }
        Ok(NodeIterator {
            tree: tree_ref_mut!(self.tree),
            node_index: self.index,
            index: 0,
            len: self.num_children()?,
            _hack: PhantomData,
        })
    }
}
