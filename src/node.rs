use crate::inner::NodeData;

use super::*;

/// A seed value used for lazy assignment of new nodes by a [`NodeRef`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Seed<'k> {
    /// No seed needed when the node exists.
    None,
    /// Position in the parent node for a new node.
    Index(usize),
    /// Key in the parent map for a new node.
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
            seed: Seed::None,
            _hack: PhantomData,
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
    pub fn parent<'r1>(&'r1 self) -> Result<NodeRef<'a, 't, '_, &'t Tree<'a>>>
    where
        't: 'r1,
    {
        let parent = self.tree.as_ref().parent(self.index)?;
        let tree_ref = self.tree.as_ref() as *const Tree<'a>;
        Ok(NodeRef {
            tree: unsafe { tree_ref.as_ref().unwrap() },
            index: parent,
            seed: Seed::None,
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
            seed: Seed::None,
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
            seed: Seed::None,
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
            seed: Seed::None,
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
            seed: Seed::None,
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
            seed: Seed::None,
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
            seed: Seed::None,
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
            seed: Seed::None,
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
            seed: Seed::None,
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
            seed: Seed::None,
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
            seed: Seed::None,
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
        if self.seed != Seed::None {
            return Err(Error::NodeNotFound);
        }
        let seed = lookup.into();
        match seed {
            Seed::Index(child_pos) => Ok(NodeRef {
                tree: tree_ref!(self.tree),
                index: self.tree.as_ref().child_at(self.index, child_pos)?,
                seed: Seed::None,
                _hack: PhantomData,
            }),
            Seed::Key(child_key) => Ok(NodeRef {
                tree: tree_ref!(self.tree),
                index: self.tree.as_ref().find_child(self.index, child_key)?,
                seed: Seed::None,
                _hack: PhantomData,
            }),
            _ => unreachable!(),
        }
    }
}

/// Lazy assignment for a node reference based on its seed. If the node already
/// exists, we simply use the existing node ID. If the node doesn't exist, we
/// use the child index or key by which it was queried to construct it, and then
/// make use of the inserted node ID.
macro_rules! maybe_construct {
    ($self:expr) => {
        match $self.seed {
            Seed::None => $self.index,
            Seed::Index(idx) => {
                let index = $self.tree.as_mut().insert_child($self.index, idx)?;
                $self.index = index;
                $self.seed = Seed::None;
                index
            }
            Seed::Key(key) => {
                let index = $self.tree.as_mut().append_child($self.index)?;
                $self.tree.as_mut().set_key(index, key)?;
                $self.index = index;
                $self.seed = Seed::None;
                index
            }
        }
    };
}

macro_rules! tree_ref_mut {
    ($tree:expr) => {{
        let tree_ref = $tree.as_mut() as *mut Tree<'_>;
        unsafe { tree_ref.as_mut().unwrap() }
    }};
}

impl<'a, 't, T> NodeRef<'a, 't, '_, T>
where
    T: AsRef<Tree<'a>> + AsMut<Tree<'a>> + 't,
{
    pub(crate) fn new_exists_mut<'na>(tree: T, index: usize) -> NodeRef<'a, 't, 'na, T> {
        NodeRef {
            tree,
            index,
            seed: Seed::None,
            _hack: PhantomData,
        }
    }

    /// Get a mutable reference to the tree the node belongs to.
    #[inline(always)]
    pub fn tree_mut(&'t mut self) -> &mut Tree<'a> {
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

    /// Returns a mutable [`NodeRef`] to the parent node, if it exists.
    #[inline(always)]
    pub fn parent_mut<'r>(&'r mut self) -> Result<NodeRef<'a, 't, '_, &'t mut Tree<'a>>> {
        let parent = self.tree.as_ref().parent(self.index)?;
        Ok(NodeRef {
            tree: tree_ref_mut!(self.tree),
            index: parent,
            seed: Seed::None,
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
            seed: Seed::None,
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
            seed: Seed::None,
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
            seed: Seed::None,
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
            seed: Seed::None,
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
            seed: Seed::None,
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
            seed: Seed::None,
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
            seed: Seed::None,
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
            seed: Seed::None,
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
            seed: Seed::None,
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
            seed: Seed::None,
            _hack: PhantomData,
        })
    }

    /// Change the type of the node, resetting its contents if necessary and
    /// returning whether the change was possible.
    #[inline(always)]
    pub fn change_type(&mut self, node_type: NodeType) -> Result<bool> {
        let index = maybe_construct!(self);
        self.tree.as_mut().change_type(index, node_type)
    }

    /// Set flags on the node.
    #[inline(always)]
    pub fn set_type_flags(&mut self, more_flags: NodeType) -> Result<()> {
        let index = maybe_construct!(self);
        self.tree.as_mut().set_flags(index, more_flags)
    }

    /// Sets the node's key.
    #[inline(always)]
    pub fn set_key(&mut self, key: &str) -> Result<()> {
        let index = maybe_construct!(self);
        self.tree.as_mut().set_key(index, key)
    }

    /// Sets the node's value.
    #[inline(always)]
    pub fn set_val(&mut self, value: &str) -> Result<()> {
        let index = maybe_construct!(self);
        self.tree.as_mut().set_val(index, value)
    }

    /// Set the tag on the node key.
    #[inline(always)]
    pub fn set_key_tag(&mut self, v: &str) -> Result<()> {
        let index = maybe_construct!(self);
        self.tree.as_mut().set_key_tag(index, v)
    }

    /// Set the tag on the node val.
    #[inline(always)]
    pub fn set_val_tag(&mut self, v: &str) -> Result<()> {
        let index = maybe_construct!(self);
        self.tree.as_mut().set_val_tag(index, v)
    }

    /// Set the anchor on the node key.
    #[inline(always)]
    pub fn set_key_anchor(&mut self, v: &str) -> Result<()> {
        let index = maybe_construct!(self);
        self.tree.as_mut().set_key_anchor(index, v)
    }

    /// Set the anchor on the node val.
    #[inline(always)]
    pub fn set_val_anchor(&mut self, v: &str) -> Result<()> {
        let index = maybe_construct!(self);
        self.tree.as_mut().set_val_anchor(index, v)
    }

    /// Set the ref on the node key.
    #[inline(always)]
    pub fn set_key_ref(&mut self, v: &str) -> Result<()> {
        let index = maybe_construct!(self);
        self.tree.as_mut().set_key_ref(index, v)
    }

    /// Set the ref on the node val.
    #[inline(always)]
    pub fn set_val_ref(&mut self, v: &str) -> Result<()> {
        let index = maybe_construct!(self);
        self.tree.as_mut().set_val_ref(index, v)
    }

    /// Empties the node and removes any children.
    #[inline(always)]
    pub fn clear(&mut self) -> Result<()> {
        if let Seed::None = self.seed {
            self.tree.as_mut().clear_node(self.index)
        } else {
            Ok(())
        }
    }

    /// Clears the node key, if it exists.
    #[inline(always)]
    pub fn clear_key(&mut self) -> Result<()> {
        if let Seed::None = self.seed {
            self.tree.as_mut().clear_key(self.index)
        } else {
            Ok(())
        }
    }

    /// Clears the node value, if it exists.
    #[inline(always)]
    pub fn clear_val(&mut self) -> Result<()> {
        if let Seed::None = self.seed {
            self.tree.as_mut().clear_val(self.index)
        } else {
            Ok(())
        }
    }

    /// Get a mutable [`NodeRef`] to a child of this node by its given key (if
    /// this node is a map) or given position (if this node is a sequence).
    ///
    /// Unlike [`get`](#method.get), this method will succeed if the node does
    /// not exist and will retain the seed to lazy assignment. It will still
    /// return a `NodeNotFound` error if the current node does not exist.
    pub fn get_mut<'r, 'k2, S: Into<Seed<'k2>>>(
        &'r mut self,
        lookup: S,
    ) -> Result<NodeRef<'a, 't, 'k2, &'t mut Tree<'a>>> {
        if self.seed != Seed::None {
            return Err(Error::NodeNotFound);
        }
        let seed = lookup.into();
        let tree_ref = self.tree.as_mut() as *mut Tree;
        match seed {
            Seed::Index(child_pos) => match self.tree.as_ref().child_at(self.index, child_pos) {
                Ok(index) => Ok(NodeRef {
                    tree: unsafe { tree_ref.as_mut().unwrap() },
                    index,
                    seed: Seed::None,
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
            Seed::Key(child_key) => match self.tree.as_ref().find_child(self.index, child_key) {
                Ok(index) => Ok(NodeRef {
                    tree: unsafe { tree_ref.as_mut().unwrap() },
                    index,
                    seed: Seed::None,
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
}
