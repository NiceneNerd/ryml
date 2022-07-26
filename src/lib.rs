//! TODO
#![deny(missing_docs)]
#![feature(core_ffi_c)]
use std::{marker::PhantomData, ops::Deref};
use thiserror::Error;
mod inner;
mod node;
pub use inner::{NodeData, NodeScalar, NodeType};
pub use node::NodeRef;

/// Represents the pseudo-index of a node that does not exist.
pub const NONE: usize = usize::MAX;

macro_rules! not_none {
    ($result:expr) => {
        match $result {
            NONE => return Err(Error::NodeNotFound),
            v => Ok(v),
        }
    };
}

/// Error type for this crate
#[derive(Debug, Error)]
pub enum Error {
    /// Thrown when a node lookup turns up empty.
    #[error("Node does not exist")]
    NodeNotFound,
    /// A general exception thrown by rapidyaml over FFI.
    #[error(transparent)]
    Other(#[from] cxx::Exception),
}

type Result<T> = std::result::Result<T, Error>;

enum TreeData<'a> {
    Owned,
    Borrowed(PhantomData<&'a mut [u8]>),
}

/// Represents a parsed YAML tree
pub struct Tree<'a> {
    inner: cxx::UniquePtr<inner::ffi::Tree>,
    _data: TreeData<'a>,
}

impl PartialEq for Tree<'_> {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self.inner.deref(), other.inner.deref())
    }
}

impl Clone for Tree<'_> {
    fn clone(&self) -> Self {
        Self {
            inner: inner::ffi::clone_tree(self.inner.deref()),
            _data: TreeData::Borrowed(PhantomData),
        }
    }
}

impl<'s, 't> AsRef<Tree<'s>> for Tree<'s>
where
    's: 't,
{
    fn as_ref(&self) -> &Tree<'s> {
        self
    }
}

impl<'s, 't> AsMut<Tree<'s>> for Tree<'s>
where
    's: 't,
{
    fn as_mut(&mut self) -> &mut Tree<'s> {
        self
    }
}

impl Eq for Tree<'_> {}

impl core::fmt::Debug for Tree<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Tree")
            .field("len", &self.len())
            .field("capacity", &self.capacity())
            .field("arena_capacity", &self.arena_capacity())
            .finish()
    }
}

impl Default for Tree<'_> {
    fn default() -> Self {
        Self {
            inner: inner::ffi::new_tree(),
            _data: TreeData::Owned,
        }
    }
}

impl<'a> Tree<'a> {
    /// Create a new tree and parse into its root.  
    /// The immutable YAML source is first copied to the tree's arena, and
    /// parsed from there.
    #[inline(always)]
    pub fn parse(text: impl AsRef<str>) -> Result<Tree<'a>> {
        let tree = inner::ffi::parse(text.as_ref())?;
        Ok(Self {
            inner: tree,
            _data: TreeData::Owned,
        })
    }

    /// Create a new tree and parse into its root.  
    /// A mutable reference to the YAML source is passed to the tree parser,
    /// and parsed in-situ.
    #[inline(always)]
    pub fn parse_in_place(mut text: impl AsMut<str> + 'a) -> Result<Tree<'a>> {
        let tree = unsafe {
            inner::ffi::parse_in_place(text.as_mut().as_mut_ptr() as *mut i8, text.as_mut().len())
        }?;
        Ok(Self {
            inner: tree,
            _data: TreeData::Borrowed(PhantomData),
        })
    }

    /// Emit YAML to an owned string.
    #[inline(always)]
    pub fn emit(&self) -> Result<String> {
        let mut buf = vec![0; self.inner.capacity() * 32 + self.inner.arena_capacity()];
        let written = inner::ffi::emit(
            self.inner.as_ref().unwrap(),
            inner::Substr {
                ptr: buf.as_mut_ptr(),
                len: buf.len(),
            },
            true,
        )?;
        Ok(written.to_string())
    }

    /// Emit YAML to the given buffer. Returns the number of bytes written.
    #[inline(always)]
    pub fn emit_to_buffer(&self, buf: &mut [u8]) -> Result<usize> {
        let written = inner::ffi::emit(
            self.inner.as_ref().unwrap(),
            inner::Substr {
                ptr: buf.as_mut_ptr(),
                len: buf.len(),
            },
            true,
        )?;
        Ok(written.len)
    }

    /// Emit YAML to the given writer. Returns the number of bytes written.
    #[inline(always)]
    pub fn emit_to_writer<W: std::io::Write + std::io::Seek>(
        &self,
        writer: &mut W,
    ) -> Result<usize> {
        let written =
            inner::ffi::emit_to_rwriter(&self.inner, Box::new(inner::RWriter { writer }))?;
        Ok(written)
    }

    /// Get the node to the root node.
    #[inline(always)]
    pub fn root_id(&self) -> Result<usize> {
        Ok(self.inner.root_id()?)
    }

    /// Get a [`NodeRef`] to the root node.
    #[inline(always)]
    pub fn root_ref<'t>(&'t self) -> Result<NodeRef<'a, 't, '_, &'t Self>> {
        Ok(NodeRef::new_exists(self, self.root_id()?))
    }

    /// Get a mutable [`NodeRef`] to the root node.
    #[inline(always)]
    pub fn root_ref_mut<'t>(&'t mut self) -> Result<NodeRef<'a, 't, '_, &'t mut Tree<'a>>> {
        Ok(NodeRef::new_exists_mut(self, self.root_id()?))
    }

    /// Get a [`NodeRef`] to the given node, if it exists.
    #[inline(always)]
    pub fn get<'t>(&'t self, index: usize) -> Result<NodeRef<'a, 't, '_, &'t Self>> {
        if index < self.inner.size() {
            Ok(NodeRef::new_exists(self, index))
        } else {
            Err(Error::NodeNotFound)
        }
    }

    /// Get a mutable [`NodeRef`] to the given node, if it exists.
    #[inline(always)]
    pub fn get_mut<'t>(&'t mut self, index: usize) -> Result<NodeRef<'a, 't, '_, &'t mut Self>> {
        if index < self.inner.size() {
            Ok(NodeRef::new_exists(self, index))
        } else {
            Err(Error::NodeNotFound)
        }
    }

    /// Get the total number of nodes.
    #[inline(always)]
    pub fn len(&self) -> usize {
        self.inner.size()
    }

    /// Returns true if the tree is empty.
    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.inner.empty()
    }

    /// Get the capacity of the tree.
    #[inline(always)]
    pub fn capacity(&self) -> usize {
        self.inner.capacity()
    }

    /// Get the unused capacity of the tree.
    #[inline(always)]
    pub fn slack(&self) -> Result<usize> {
        Ok(self.inner.slack()?)
    }

    /// Get the size of the internal string arena.
    #[inline(always)]
    pub fn arena_len(&self) -> usize {
        self.inner.arena_size()
    }

    /// Returns true is the internal string arena is empty.
    #[inline(always)]
    pub fn arena_is_empty(&self) -> bool {
        self.arena_len() == 0
    }

    /// Get the capacity of the internal string arena.
    #[inline(always)]
    pub fn arena_capacity(&self) -> usize {
        self.inner.arena_capacity()
    }

    /// Get the unused capacity of the internal string arena.
    #[inline(always)]
    pub fn arena_slack(&self) -> Result<usize> {
        Ok(self.inner.arena_slack()?)
    }

    /// Reserves capacity to hold at least `capacity` nodes.
    #[inline(always)]
    pub fn reserve(&mut self, node_capacity: usize) {
        self.inner.pin_mut().reserve(node_capacity);
    }

    /// Ensures the tree's internal string arena is at least the given
    /// capacity.
    ///
    /// **Note**: Growing the arena may cause relocation of the entire existing
    /// arena, and thus change the contents of individual nodes.
    #[inline(always)]
    pub fn reserve_arena(&mut self, arena_capacity: usize) {
        self.inner.pin_mut().reserve_arena(arena_capacity);
    }

    /// Clear the tree and zero every node.
    ///
    /// **Note**: Does **not** clear the arena.
    /// See also [`clear_arena`](#method.clear_arena).
    #[inline(always)]
    pub fn clear(&mut self) {
        self.inner.pin_mut().clear();
    }

    /// Clear the internal string arena.
    #[inline(always)]
    pub fn clear_arena(&mut self) {
        self.inner.pin_mut().clear_arena();
    }

    /// Resolve references (aliases <- anchors) in the tree.
    ///
    /// Dereferencing is opt-in; after parsing,
    /// [`Tree::resolve()`](#method.resolve) has to be called explicitly for
    /// obtaining resolved references in the tree. This method will resolve
    /// all references and substitute the anchored values in place of the
    /// reference.
    ///
    /// This method first does a full traversal of the tree to gather all
    /// anchors and references in a separate collection, then it goes through
    /// that collection to locate the names, which it does by obeying the YAML
    /// standard diktat that "an alias node refers to the most recent node in
    /// the serialization having the specified anchor"
    ///
    /// So, depending on the number of anchor/alias nodes, thit is a
    /// potentially expensive operation, with a best-case linear complexity
    /// (from the initial traversal). This potential cost is the reason for
    /// requiring an explicit call.
    #[inline(always)]
    pub fn resolve(&mut self) -> Result<()> {
        Ok(self.inner.pin_mut().resolve()?)
    }

    /// Get the type of the given node, if it exists.
    #[inline(always)]
    pub fn node_type(&self, node: usize) -> Result<NodeType> {
        Ok(inner::ffi::tree_node_type(&self.inner, node)?)
    }

    /// Get the type name of the given node, if it exists.
    #[inline(always)]
    pub fn node_type_as_str(&self, node: usize) -> Result<&str> {
        let ptr = self.inner.type_str(node)?;
        Ok(unsafe { std::ffi::CStr::from_ptr(ptr).to_str().unwrap_unchecked() })
    }

    /// Get the text of the given node, if it exists and is a key.
    #[inline(always)]
    pub fn key(&self, node: usize) -> Result<&str> {
        Ok(self.inner.key(node).map(|s| s.as_ref())?)
    }

    /// Get the text of the tag on the key of the given node, if it exists and
    /// is a tagged key.
    #[inline(always)]
    pub fn key_tag(&self, node: usize) -> Result<&str> {
        Ok(self.inner.key_tag(node).map(|s| s.as_ref())?)
    }

    /// Get the text of the reference on the key of the given node, if it exists
    /// and is a reference.
    pub fn key_ref(&self, node: usize) -> Result<&str> {
        Ok(self.inner.key_ref(node).map(|s| s.as_ref())?)
    }

    /// Get the text of the anchor on the key of the given node, if it exists
    /// and is an anchor.
    pub fn key_anchor(&self, node: usize) -> Result<&str> {
        Ok(self.inner.key_anchor(node).map(|s| s.as_ref())?)
    }

    /// Get the whole scalar key of the given node, if it exists and is a
    /// scalar key.
    pub fn key_scalar(&self, node: usize) -> Result<&NodeScalar> {
        Ok(self.inner.keysc(node)?)
    }

    /// Get the text of the given node, if it exists and is a value.
    #[inline(always)]
    pub fn val(&self, node: usize) -> Result<&str> {
        Ok(self.inner.val(node).map(|s| s.as_ref())?)
    }

    /// Get the text of the tag on the value of the given node, if it exists and
    /// is a tagged value.
    #[inline(always)]
    pub fn val_tag(&self, node: usize) -> Result<&str> {
        Ok(self.inner.val_tag(node).map(|s| s.as_ref())?)
    }

    /// Get the text of the reference on the value of the given node, if it
    /// exists and is a reference.
    #[inline(always)]
    pub fn val_ref(&self, node: usize) -> Result<&str> {
        Ok(self.inner.val_ref(node).map(|s| s.as_ref())?)
    }

    /// Get the text of the anchor on the value of the given node, if it exists
    /// and is an anchor.
    #[inline(always)]
    pub fn val_anchor(&self, node: usize) -> Result<&str> {
        Ok(self.inner.val_anchor(node).map(|s| s.as_ref())?)
    }

    /// Get the whole scalar value of the given node, if it exists and is a
    /// scalar value.
    #[inline(always)]
    pub fn val_scalar(&self, node: usize) -> Result<&NodeScalar> {
        Ok(self.inner.valsc(node)?)
    }

    /// If the given node exists, returns true if it is a root.
    #[inline(always)]
    pub fn is_root(&self, node: usize) -> Result<bool> {
        Ok(self.inner.is_root(node)?)
    }

    /// If the given node exists, returns true if it is a stream.
    #[inline(always)]
    pub fn is_stream(&self, node: usize) -> Result<bool> {
        Ok(self.inner.is_stream(node)?)
    }

    /// If the given node exists, returns true if it is a doc.
    #[inline(always)]
    pub fn is_doc(&self, node: usize) -> Result<bool> {
        Ok(self.inner.is_doc(node)?)
    }

    /// If the given node exists, returns true if it is a container.
    #[inline(always)]
    pub fn is_container(&self, node: usize) -> Result<bool> {
        Ok(self.inner.is_container(node)?)
    }

    /// If the given node exists, returns true if it is a map.
    #[inline(always)]
    pub fn is_map(&self, node: usize) -> Result<bool> {
        Ok(self.inner.is_map(node)?)
    }

    /// If the given node exists, returns true if it is a seq.
    #[inline(always)]
    pub fn is_seq(&self, node: usize) -> Result<bool> {
        Ok(self.inner.is_seq(node)?)
    }

    /// If the given node exists, returns true if it has a value.
    #[inline(always)]
    pub fn has_val(&self, node: usize) -> Result<bool> {
        Ok(self.inner.has_val(node)?)
    }

    /// If the given node exists, returns true if it has a key.
    #[inline(always)]
    pub fn has_key(&self, node: usize) -> Result<bool> {
        Ok(self.inner.has_key(node)?)
    }

    /// If the given node exists, returns true if it is a value.
    #[inline(always)]
    pub fn is_val(&self, node: usize) -> Result<bool> {
        Ok(self.inner.is_val(node)?)
    }

    /// If the given node exists, returns true if it is a keyval.
    #[inline(always)]
    pub fn is_keyval(&self, node: usize) -> Result<bool> {
        Ok(self.inner.is_keyval(node)?)
    }

    /// If the given node exists, returns true if it has a tagged key.
    #[inline(always)]
    pub fn has_key_tag(&self, node: usize) -> Result<bool> {
        Ok(self.inner.has_key_tag(node)?)
    }

    /// If the given node exists, returns true if it has a tagged value.
    #[inline(always)]
    pub fn has_val_tag(&self, node: usize) -> Result<bool> {
        Ok(self.inner.has_val_tag(node)?)
    }

    /// If the given node exists, returns true if it has an anchor key.
    #[inline(always)]
    pub fn has_key_anchor(&self, node: usize) -> Result<bool> {
        Ok(self.inner.has_key_anchor(node)?)
    }

    /// If the given node exists, returns true if it has an anchor value.
    #[inline(always)]
    pub fn has_val_anchor(&self, node: usize) -> Result<bool> {
        Ok(self.inner.has_val_anchor(node)?)
    }

    /// If the given node exists, returns true if it is a key_ref.
    #[inline(always)]
    pub fn is_key_ref(&self, node: usize) -> Result<bool> {
        Ok(self.inner.is_key_ref(node)?)
    }

    /// If the given node exists, returns true if it is a val_ref.
    #[inline(always)]
    pub fn is_val_ref(&self, node: usize) -> Result<bool> {
        Ok(self.inner.is_val_ref(node)?)
    }

    /// If the given node exists, returns true if it is a ref.
    #[inline(always)]
    pub fn is_ref(&self, node: usize) -> Result<bool> {
        Ok(self.inner.is_ref(node)?)
    }

    /// If the given node exists, returns true if it is a anchor_or_ref.
    #[inline(always)]
    pub fn is_anchor_or_ref(&self, node: usize) -> Result<bool> {
        Ok(self.inner.is_anchor_or_ref(node)?)
    }

    /// If the given node exists, returns true if it is a key_quoted.
    #[inline(always)]
    pub fn is_key_quoted(&self, node: usize) -> Result<bool> {
        Ok(self.inner.is_key_quoted(node)?)
    }

    /// If the given node exists, returns true if it is a val_quoted.
    #[inline(always)]
    pub fn is_val_quoted(&self, node: usize) -> Result<bool> {
        Ok(self.inner.is_val_quoted(node)?)
    }

    /// If the given node exists, returns true if it is a quoted.
    #[inline(always)]
    pub fn is_quoted(&self, node: usize) -> Result<bool> {
        Ok(self.inner.is_quoted(node)?)
    }

    /// If the given node exists, returns true if it is a anchor.
    #[inline(always)]
    pub fn is_anchor(&self, node: usize) -> Result<bool> {
        Ok(self.inner.is_anchor(node)?)
    }

    /// If the given node exists, returns true if the parent is a
    /// sequence.
    #[inline(always)]
    pub fn parent_is_seq(&self, node: usize) -> Result<bool> {
        Ok(self.inner.parent_is_seq(node)?)
    }

    /// If the given node exists, returns true if the parent is a
    /// map.
    #[inline(always)]
    pub fn parent_is_map(&self, node: usize) -> Result<bool> {
        Ok(self.inner.parent_is_map(node)?)
    }

    /// If the given node exists, returns true if it is empty.
    #[inline(always)]
    pub fn is_node_empty(&self, node: usize) -> Result<bool> {
        Ok(self.inner.is_node_empty(node)?)
    }

    /// If the given node exists, returns true if it has an anchor.
    #[inline(always)]
    pub fn has_anchor(&self, node: usize, anchor: &str) -> Result<bool> {
        Ok(self.inner.has_anchor(node, anchor.into())?)
    }

    /// If the given node exists, returns true if it has a parent.
    #[inline(always)]
    pub fn has_parent(&self, node: usize) -> Result<bool> {
        Ok(self.inner.has_parent(node)?)
    }

    /// If the given node exists, returns true if it has a child.
    #[inline(always)]
    pub fn has_child(&self, node: usize, key: &str) -> Result<bool> {
        Ok(self.inner.has_child(node, key.into())?)
    }

    /// If the given node exists, returns true if it has children.
    #[inline(always)]
    pub fn has_children(&self, node: usize) -> Result<bool> {
        Ok(self.inner.has_children(node)?)
    }

    /// If the given node exists, returns true if it has a sibling.
    #[inline(always)]
    pub fn has_sibling(&self, node: usize, key: &str) -> Result<bool> {
        Ok(self.inner.has_sibling(node, key.into())?)
    }

    /// If the given node exists, returns true if it has siblings.
    ///
    /// **Note**: This corresponds to `has_other_siblings()` in the C++ API, as
    /// the plain `has_siblings()` function always returns true by counting the
    /// node itself, which seems rather pointless and is not the obvious meaning
    /// of a method by the name.
    #[inline(always)]
    pub fn has_siblings(&self, node: usize) -> Result<bool> {
        Ok(self.inner.has_other_siblings(node)?)
    }

    /// If the given node exists and has a parent, returns the
    /// parent node.
    #[inline(always)]
    pub fn parent(&self, node: usize) -> Result<usize> {
        not_none!(self.inner.parent(node)?)
    }

    /// If the given node exists and has a previous sibling, returns the index
    /// to the sibling node.
    #[inline(always)]
    pub fn prev_sibling(&self, node: usize) -> Result<usize> {
        not_none!(self.inner.prev_sibling(node)?)
    }

    /// If the given node exists and has a next sibling, returns the index to
    /// the sibling node.
    #[inline(always)]
    pub fn next_sibling(&self, node: usize) -> Result<usize> {
        not_none!(self.inner.next_sibling(node)?)
    }

    /// If the given node exists and has children, returns the
    /// number of children.
    #[inline(always)]
    pub fn num_children(&self, node: usize) -> Result<usize> {
        Ok(self.inner.num_children(node)?)
    }

    /// If the given node exists and has the given child, returns
    /// the position of the child in the parent node.
    #[inline(always)]
    pub fn child_pos(&self, node: usize, child: usize) -> Result<usize> {
        Ok(self.inner.child_pos(node, child)?)
    }

    /// If the given node exists and has children, returns the index of the
    /// first child node.
    #[inline(always)]
    pub fn first_child(&self, node: usize) -> Result<usize> {
        not_none!(self.inner.first_child(node)?)
    }

    /// If the given node exists and has children, returns the
    /// index to the last child node.
    #[inline(always)]
    pub fn last_child(&self, node: usize) -> Result<usize> {
        not_none!(self.inner.last_child(node)?)
    }

    /// If the given node exists and has a child at the given
    /// position, returns the index to the child node.
    #[inline(always)]
    pub fn child_at(&self, node: usize, pos: usize) -> Result<usize> {
        not_none!(self.inner.child(node, pos)?)
    }

    /// If the given node exists and has a child at the given
    /// key, returns the index to the child node.
    #[inline(always)]
    pub fn find_child(&self, node: usize, key: &str) -> Result<usize> {
        not_none!(self.inner.find_child(node, &(key.into()))?)
    }

    /// If the given node exists and has siblings, returns the
    /// number of siblings.
    #[inline(always)]
    pub fn num_siblings(&self, node: usize) -> Result<usize> {
        Ok(self.inner.num_siblings(node)?)
    }

    /// If the given node exists and has other siblings, returns
    /// the number of other siblings.
    #[inline(always)]
    pub fn num_other_siblings(&self, node: usize) -> Result<usize> {
        Ok(self.inner.num_other_siblings(node)?)
    }

    /// If the given node exists and has the given sibling, get
    /// position of the sibling in in the parent.
    #[inline(always)]
    pub fn sibling_pos(&self, node: usize, sibling: usize) -> Result<usize> {
        Ok(self.inner.sibling_pos(node, sibling)?)
    }

    /// If the given node exists and has siblings, returns the
    /// index to the first sibling node.
    #[inline(always)]
    pub fn first_sibling(&self, node: usize) -> Result<usize> {
        not_none!(self.inner.first_sibling(node)?)
    }

    /// If the given node exists and has siblings, returns the
    /// index to the last sibling node.
    #[inline(always)]
    pub fn last_sibling(&self, node: usize) -> Result<usize> {
        not_none!(self.inner.last_sibling(node)?)
    }
    /// If the given node exists and has a sibling as the given
    /// position, returns the index to the sibling node.
    #[inline(always)]
    pub fn sibling_at(&self, node: usize, pos: usize) -> Result<usize> {
        not_none!(self.inner.sibling(node, pos)?)
    }

    /// If the given node exists and has a sibling at the given
    /// key, returns the index to the sibling node.
    #[inline(always)]
    pub fn find_sibling(&self, node: usize, key: &str) -> Result<usize> {
        not_none!(self.inner.find_sibling(node, &(key.into()))?)
    }

    /// Turn the given node into a key-value pair.
    #[inline(always)]
    pub fn to_keyval(&mut self, node: usize, key: &str, val: &str) -> Result<()> {
        Ok(self
            .inner
            .pin_mut()
            .to_keyval(node, key.into(), val.into(), 0)?)
    }

    /// Turn the given node into a key-value pair with additional flags.
    #[inline(always)]
    pub fn to_keyval_with_flags(
        &mut self,
        node: usize,
        key: &str,
        val: &str,
        more_flags: NodeType,
    ) -> Result<()> {
        Ok(self
            .inner
            .pin_mut()
            .to_keyval(node, key.into(), val.into(), more_flags as u64)?)
    }

    /// Turn the given node with the given key into a map.
    pub fn to_map_by_key(&mut self, node: usize, key: &str) -> Result<()> {
        Ok(self.inner.pin_mut().to_map_with_key(node, key.into(), 0)?)
    }

    /// Turn the given node with the given key into a map with additional flags.
    pub fn to_map_with_key_and_flags(
        &mut self,
        node: usize,
        key: &str,
        more_flags: NodeType,
    ) -> Result<()> {
        Ok(self
            .inner
            .pin_mut()
            .to_map_with_key(node, key.into(), more_flags as u64)?)
    }

    /// Turn the given node with the given key into a sequence.
    pub fn to_seq_by_key(&mut self, node: usize, key: &str) -> Result<()> {
        Ok(self.inner.pin_mut().to_seq_with_key(node, key.into(), 0)?)
    }

    /// Turn the given node with the given key into a sequence with additional
    /// flags.
    pub fn to_seq_with_key_and_flags(
        &mut self,
        node: usize,
        key: &str,
        more_flags: NodeType,
    ) -> Result<()> {
        Ok(self
            .inner
            .pin_mut()
            .to_seq_with_key(node, key.into(), more_flags as u64)?)
    }

    /// Turn the given node into a value.
    #[inline(always)]
    pub fn to_val(&mut self, node: usize, val: &str) -> Result<()> {
        Ok(self.inner.pin_mut().to_val(node, val.into(), 0)?)
    }

    /// Turn the given node into a value with additional flags.
    #[inline(always)]
    pub fn to_val_with_flags(
        &mut self,
        node: usize,
        val: &str,
        more_flags: NodeType,
    ) -> Result<()> {
        Ok(self
            .inner
            .pin_mut()
            .to_val(node, val.into(), more_flags as u64)?)
    }

    /// Turn the given node into a stream.
    #[inline(always)]
    pub fn to_stream(&mut self, node: usize) -> Result<()> {
        Ok(self.inner.pin_mut().to_stream(node, 0)?)
    }

    /// Turn the given node into a stream with additional flags.
    #[inline(always)]
    pub fn to_stream_with_flags(&mut self, node: usize, more_flags: NodeType) -> Result<()> {
        Ok(self.inner.pin_mut().to_stream(node, more_flags as u64)?)
    }

    /// Turn the given node into a map.
    #[inline(always)]
    pub fn to_map(&mut self, node: usize) -> Result<()> {
        Ok(self.inner.pin_mut().to_map(node, 0)?)
    }

    /// Turn the given node into a map with additional flags.
    #[inline(always)]
    pub fn to_map_with_flags(&mut self, node: usize, more_flags: NodeType) -> Result<()> {
        Ok(self.inner.pin_mut().to_map(node, more_flags as u64)?)
    }

    /// Turn the given node into a sequence.
    #[inline(always)]
    pub fn to_seq(&mut self, node: usize) -> Result<()> {
        Ok(self.inner.pin_mut().to_seq(node, 0)?)
    }

    /// Turn the given node into a sequence with additional flags.
    #[inline(always)]
    pub fn to_seq_with_flags(&mut self, node: usize, more_flags: NodeType) -> Result<()> {
        Ok(self.inner.pin_mut().to_seq(node, more_flags as u64)?)
    }

    /// Turn the given node into a doc.
    #[inline(always)]
    pub fn to_doc(&mut self, node: usize) -> Result<()> {
        Ok(self.inner.pin_mut().to_doc(node, 0)?)
    }

    /// Turn the given node into a doc with additional flags.
    #[inline(always)]
    pub fn to_doc_with_flags(&mut self, node: usize, more_flags: NodeType) -> Result<()> {
        Ok(self.inner.pin_mut().to_doc(node, more_flags as u64)?)
    }

    /// Set the tag on the key of the given node.
    #[inline(always)]
    pub fn set_key_tag(&mut self, node: usize, tag: &str) -> Result<()> {
        Ok(self.inner.pin_mut().set_key_tag(node, tag.into())?)
    }

    /// Set the anchor on the key of the given node.
    #[inline(always)]
    pub fn set_key_anchor(&mut self, node: usize, anchor: &str) -> Result<()> {
        Ok(self.inner.pin_mut().set_key_anchor(node, anchor.into())?)
    }

    /// Set the anchor on the value of the given node.
    #[inline(always)]
    pub fn set_val_anchor(&mut self, node: usize, anchor: &str) -> Result<()> {
        Ok(self.inner.pin_mut().set_val_anchor(node, anchor.into())?)
    }

    /// Set the ref on the key of the given node.
    #[inline(always)]
    pub fn set_key_ref(&mut self, node: usize, refr: &str) -> Result<()> {
        Ok(self.inner.pin_mut().set_key_ref(node, refr.into())?)
    }

    /// Set the ref on the value of the given node.
    #[inline(always)]
    pub fn set_val_ref(&mut self, node: usize, refr: &str) -> Result<()> {
        Ok(self.inner.pin_mut().set_val_ref(node, refr.into())?)
    }

    /// Set the tag on the value of the given node.
    pub fn set_val_tag(&mut self, node: usize, tag: &str) -> Result<()> {
        Ok(self.inner.pin_mut().set_val_tag(node, tag.into())?)
    }

    /// Remove the anchor on the key of the given node.
    pub fn rem_key_anchor(&mut self, node: usize) -> Result<()> {
        Ok(self.inner.pin_mut().rem_key_anchor(node)?)
    }

    /// Remove the anchor on the value of the given node.
    pub fn rem_val_anchor(&mut self, node: usize) -> Result<()> {
        Ok(self.inner.pin_mut().rem_val_anchor(node)?)
    }

    /// Remove the reference on the key of the given node.
    pub fn rem_key_ref(&mut self, node: usize) -> Result<()> {
        Ok(self.inner.pin_mut().rem_key_ref(node)?)
    }

    /// Remove the reference on the value of the given node.
    pub fn rem_val_ref(&mut self, node: usize) -> Result<()> {
        Ok(self.inner.pin_mut().rem_val_ref(node)?)
    }

    /// Remove the reference on the anchor of the given node.
    pub fn rem_anchor_ref(&mut self, node: usize) -> Result<()> {
        Ok(self.inner.pin_mut().rem_anchor_ref(node)?)
    }

    /// Insert a new node as the child of the given parent at the given
    /// position, returning its index.
    #[inline(always)]
    pub fn insert_child(&mut self, parent: usize, after: usize) -> Result<usize> {
        Ok(self.inner.pin_mut().insert_child(parent, after)?)
    }

    /// Insert a new node as the first child of the given parent, returning
    /// its index.
    #[inline(always)]
    pub fn prepend_child(&mut self, parent: usize) -> Result<usize> {
        Ok(self.inner.pin_mut().prepend_child(parent)?)
    }

    /// Insert a new node as the last child of the given parent, returning
    /// its index.
    #[inline(always)]
    pub fn append_child(&mut self, parent: usize) -> Result<usize> {
        Ok(self.inner.pin_mut().append_child(parent)?)
    }

    /// Insert a new node as the sibling of the given node, returning its index.
    #[inline(always)]
    pub fn insert_sibling(&mut self, node: usize, after: usize) -> Result<usize> {
        Ok(self.inner.pin_mut().insert_sibling(node, after)?)
    }

    /// Insert a new node as the first sibling of the given node, returning its
    /// index.
    #[inline(always)]
    pub fn prepend_sibling(&mut self, node: usize) -> Result<usize> {
        Ok(self.inner.pin_mut().prepend_sibling(node)?)
    }

    /// Insert a new node as the last sibling of the given node, returning its
    /// index.
    #[inline(always)]
    pub fn append_sibling(&mut self, node: usize) -> Result<usize> {
        Ok(self.inner.pin_mut().append_sibling(node)?)
    }

    /// Remove the given node from its parent, including any children.
    #[inline(always)]
    pub fn remove(&mut self, node: usize) -> Result<()> {
        Ok(self.inner.pin_mut().remove(node)?)
    }

    /// Remove all children from a given node, leaving the node itself.
    #[inline(always)]
    pub fn remove_children(&mut self, node: usize) -> Result<()> {
        Ok(self.inner.pin_mut().remove_children(node)?)
    }

    /// Reorder the tree in memory so that all the nodes are stored in a linear
    /// sequence when visited in depth-first order. This will invalidate
    /// existing ids/indicies, since the node id is its position in the node
    /// array.
    #[inline(always)]
    pub fn reorder(&mut self) -> Result<()> {
        Ok(self.inner.pin_mut().reorder()?)
    }

    /// Change the type of a node, resetting its contents if necessary and
    /// returning whether the change was possible.
    #[inline(always)]
    pub fn change_type(&mut self, node: usize, new_type: NodeType) -> Result<bool> {
        Ok(self.inner.pin_mut().change_type(node, new_type as u64)?)
    }

    #[inline(always)]
    fn set_flags(&mut self, node: usize, new_type: NodeType) -> Result<()> {
        Ok(self.inner.pin_mut()._set_flags(node, new_type as u64)?)
    }

    #[inline(always)]
    fn set_key(&mut self, node: usize, key: &str) -> Result<()> {
        Ok(self.inner.pin_mut()._set_key(node, key.into(), 0)?)
    }

    #[inline(always)]
    fn set_val(&mut self, node: usize, val: &str) -> Result<()> {
        Ok(self.inner.pin_mut()._set_val(node, val.into(), 0)?)
    }

    #[inline(always)]
    fn clear_node(&mut self, node: usize) -> Result<()> {
        Ok(self.inner.pin_mut()._clear(node)?)
    }

    #[inline(always)]
    fn clear_key(&mut self, node: usize) -> Result<()> {
        Ok(self.inner.pin_mut()._clear_key(node)?)
    }

    #[inline(always)]
    fn clear_val(&mut self, node: usize) -> Result<()> {
        Ok(self.inner.pin_mut()._clear_val(node)?)
    }

    /// Recursively duplicate the given node, returning the index to the
    /// duplicate.
    #[inline(always)]
    pub fn duplicate(&mut self, node: usize, new_parent: usize, after: usize) -> Result<usize> {
        Ok(self.inner.pin_mut().duplicate(node, new_parent, after)?)
    }

    /// Recursively duplicate the given node from a different tree, returning
    /// the index to the duplicate.
    #[inline(always)]
    pub fn duplicate_from_tree(
        &mut self,
        tree: &Self,
        node: usize,
        parent: usize,
        after: usize,
    ) -> Result<usize> {
        Ok(unsafe {
            self.inner.pin_mut().duplicate_from_tree(
                tree.inner.deref() as *const inner::ffi::Tree,
                node,
                parent,
                after,
            )?
        })
    }

    /// Recursively duplicate the children of the given node (but not the node
    /// itself), returning the index of the last duplicated child.
    #[inline(always)]
    pub fn duplicate_children(
        &mut self,
        node: usize,
        parent: usize,
        after: usize,
    ) -> Result<usize> {
        Ok(self
            .inner
            .pin_mut()
            .duplicate_children(node, parent, after)?)
    }

    /// Recursively duplicate the children of the given node (but not the node
    /// itself) from a different tree, returning the index of the last
    /// duplicated child.
    #[inline(always)]
    pub fn duplicate_children_from_tree(
        &mut self,
        tree: &Self,
        node: usize,
        parent: usize,
        after: usize,
    ) -> Result<usize> {
        Ok(unsafe {
            self.inner.pin_mut().duplicate_children_from_tree(
                tree.inner.deref() as *const inner::ffi::Tree,
                node,
                parent,
                after,
            )?
        })
    }

    /// Duplicate the contents of a given node to the given index.
    #[inline(always)]
    pub fn duplicate_contents(&mut self, node: usize, dest_index: usize) -> Result<()> {
        Ok(self.inner.pin_mut().duplicate_contents(node, dest_index)?)
    }

    /// Duplicate the contents of a given node from another tree to the given
    /// index.
    #[inline(always)]
    pub fn duplicate_contents_from_tree(
        &mut self,
        tree: &Self,
        node: usize,
        dest_index: usize,
    ) -> Result<()> {
        unsafe {
            self.inner.pin_mut().duplicate_contents_from_tree(
                tree.inner.deref() as *const inner::ffi::Tree,
                node,
                dest_index,
            )?
        };
        Ok(())
    }

    /// Duplicate the node's children (but not the node) in a new parent, but
    /// omit repetitions where a duplicated node has the same key (in maps) or
    /// value (in sequences). If one of the duplicated children has the same key
    /// (in maps) or value (in sequences) as one of the parent's children, the
    /// one that is placed closest to the end will prevail.
    #[inline(always)]
    pub fn duplicate_children_no_rep(
        &mut self,
        node: usize,
        parent: usize,
        after: usize,
    ) -> Result<usize> {
        Ok(self
            .inner
            .pin_mut()
            .duplicate_children_no_rep(node, parent, after)?)
    }

    /// Change the node's position in the parent.
    #[inline(always)]
    pub fn move_node(&mut self, node: usize, after: usize) -> Result<()> {
        Ok(inner::ffi::move_node(self.inner.pin_mut(), node, after)?)
    }

    /// Change the node's parent and position.
    #[inline(always)]
    pub fn move_node_to_new_parent(
        &mut self,
        node: usize,
        new_parent: usize,
        after: usize,
    ) -> Result<()> {
        Ok(inner::ffi::move_node_to_new_parent(
            self.inner.pin_mut(),
            node,
            new_parent,
            after,
        )?)
    }

    /// Change the node's parent (in a different tree) and position.
    #[inline(always)]
    pub fn move_node_from_tree(
        &mut self,
        tree: &mut Self,
        node: usize,
        new_parent: usize,
        after: usize,
    ) -> Result<()> {
        inner::ffi::move_node_from_tree(
            self.inner.pin_mut(),
            tree.inner.pin_mut(),
            node,
            new_parent,
            after,
        )?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    static SRC: &str = include_str!("../test/AIScheduleAnchor.aiprog.yml");

    #[test]
    fn parse() -> Result<()> {
        let tree = Tree::parse(SRC)?;
        assert_eq!(78, tree.len());
        let root = tree.root_id()?;
        assert_eq!("!io", tree.val_tag(root)?);
        assert_eq!("0", tree.val(tree.find_child(root, "version")?)?);
        let param_root = tree.find_child(root, "param_root")?;
        assert_eq!("!list", tree.val_tag(param_root)?);
        assert_eq!(2, tree.num_children(param_root)?);
        let root_lists = tree.find_child(param_root, "lists")?;
        assert_eq!(4, tree.num_children(root_lists)?);
        let actions = tree.find_child(root_lists, "Action")?;
        assert_eq!("!list", tree.val_tag(actions)?);
        let action_lists = tree.find_child(actions, "lists")?;
        for i in 0..tree.num_children(action_lists)? {
            let action = tree.child_at(action_lists, i)?;
            assert_eq!(tree.key(action)?, format!("Action_{}", i));
        }
        Ok(())
    }

    #[test]
    fn empties() -> Result<()> {
        let src = "key: value";
        let tree = Tree::parse(src).unwrap();
        let root = tree.root_id()?;
        assert!(tree.find_child(root, "fish").is_err());
        assert!(tree.parent(root).is_err());
        assert!(tree.last_child(2).is_err());
        tree.child_at(888, 4444).unwrap();
        Ok(())
    }

    #[test]
    fn construct_tree() -> Result<()> {
        let mut tree = Tree::default();
        assert_eq!(tree.capacity(), 0);
        tree.reserve(32);
        tree.to_map(0)?;
        let new_node = tree.append_child(0)?;
        tree.to_keyval(new_node, "hello", "world")?;
        tree.set_val_tag(1, "!str")?;
        assert_eq!("hello: !str world\n", &tree.emit()?);
        Ok(())
    }

    #[test]
    fn node_ref() {
        let mut tree = Tree::parse(SRC).unwrap();
        {
            let root_ref = tree.root_ref().unwrap();
            dbg!(root_ref.data().unwrap());
            assert_eq!("!io", root_ref.data().unwrap().value.tag);
            let demos = root_ref
                .get("param_root")
                .unwrap()
                .get("objects")
                .unwrap()
                .get("DemoAIActionIdx")
                .unwrap();
            assert_eq!(demos.num_children().unwrap(), 6);
        }
        {
            let mut root_ref_mut = tree.root_ref_mut().unwrap();
            let mut new_demo = root_ref_mut
                .get_mut("param_root")
                .unwrap()
                .get_mut("objects")
                .unwrap()
                .get_mut("DemoAIActionIdx")
                .unwrap()
                .get_mut("FakeDemo")
                .unwrap();
            new_demo.set_val("888").unwrap();
        }
        assert_eq!(
            tree.root_ref()
                .unwrap()
                .get("param_root")
                .unwrap()
                .get("objects")
                .unwrap()
                .get("DemoAIActionIdx")
                .unwrap()
                .get("FakeDemo")
                .unwrap()
                .data()
                .unwrap()
                .value
                .scalar,
            "888"
        );
    }
}
