//! TODO
#![deny(missing_docs)]
#![feature(core_ffi_c)]
use std::{marker::PhantomData, ops::Deref};
use thiserror::Error;
mod inner;
pub use inner::{NodeScalar, NodeType};

/// Error type for this crate
#[derive(Debug, Error)]
pub enum Error {
    /// Convenience for converting None options to results.
    #[error("None error: {0:?}")]
    None(#[from] std::option::Option<std::convert::Infallible>),
    /// A general exception thrown by rapidyaml over FFI.
    #[error(transparent)]
    Other(#[from] cxx::Exception),
}

type Result<T> = std::result::Result<T, Error>;

enum TreeData<'a> {
    Owned,
    Borrowed(PhantomData<&'a [u8]>),
}

/// Represents a parsed YAML tree
pub struct Tree<'a> {
    inner: cxx::UniquePtr<inner::ffi::Tree>,
    _data: TreeData<'a>,
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
        let mut buf = vec![0; self.inner.capacity()];
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
    pub fn root_id(&self) -> Option<usize> {
        self.inner.root_id().ok()
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
    pub fn node_type(&self, node: usize) -> Option<NodeType> {
        inner::ffi::tree_node_type(&self.inner, node).ok()
    }

    /// Get the type name of the given node, if it exists.
    #[inline(always)]
    pub fn node_type_as_str(&self, node: usize) -> Option<&str> {
        let ptr = self.inner.type_str(node).ok()?;
        unsafe { std::ffi::CStr::from_ptr(ptr) }.to_str().ok()
    }

    /// Get the text of the given node, if it exists and is a key.
    #[inline(always)]
    pub fn key(&self, node: usize) -> Option<&str> {
        self.inner.key(node).ok().map(|s| s.as_ref())
    }

    /// Get the text of the tag on the key of the given node, if it exists and
    /// is a tagged key.
    #[inline(always)]
    pub fn key_tag(&self, node: usize) -> Option<&str> {
        self.inner.key_tag(node).ok().map(|s| s.as_ref())
    }

    /// Get the text of the reference on the key of the given node, if it exists
    /// and is a reference.
    pub fn key_ref(&self, node: usize) -> Option<&str> {
        self.inner.key_ref(node).ok().map(|s| s.as_ref())
    }

    /// Get the text of the anchor on the key of the given node, if it exists
    /// and is an anchor.
    pub fn key_anchor(&self, node: usize) -> Option<&str> {
        self.inner.key_anchor(node).ok().map(|s| s.as_ref())
    }

    /// Get the whole scalar key of the given node, if it exists and is a
    /// scalar key.
    pub fn key_scalar(&self, node: usize) -> Option<&NodeScalar> {
        self.inner.keysc(node).ok()
    }

    /// Get the text of the given node, if it exists and is a value.
    #[inline(always)]
    pub fn val(&self, node: usize) -> Option<&str> {
        self.inner.val(node).ok().map(|s| s.as_ref())
    }

    /// Get the text of the tag on the value of the given node, if it exists and
    /// is a tagged value.
    #[inline(always)]
    pub fn val_tag(&self, node: usize) -> Option<&str> {
        self.inner.val_tag(node).ok().map(|s| s.as_ref())
    }

    /// Get the text of the reference on the value of the given node, if it
    /// exists and is a reference.
    #[inline(always)]
    pub fn val_ref(&self, node: usize) -> Option<&str> {
        self.inner.val_ref(node).ok().map(|s| s.as_ref())
    }

    /// Get the text of the anchor on the value of the given node, if it exists
    /// and is an anchor.
    #[inline(always)]
    pub fn val_anchor(&self, node: usize) -> Option<&str> {
        self.inner.val_anchor(node).ok().map(|s| s.as_ref())
    }

    /// Get the whole scalar value of the given node, if it exists and is a
    /// scalar value.
    #[inline(always)]
    pub fn val_scalar(&self, node: usize) -> Option<&NodeScalar> {
        self.inner.valsc(node).ok()
    }

    /// If the given node exists, returns true if it is a root.
    #[inline(always)]
    pub fn is_root(&self, node: usize) -> Option<bool> {
        self.inner.is_root(node).ok()
    }

    /// If the given node exists, returns true if it is a stream.
    #[inline(always)]
    pub fn is_stream(&self, node: usize) -> Option<bool> {
        self.inner.is_stream(node).ok()
    }

    /// If the given node exists, returns true if it is a doc.
    #[inline(always)]
    pub fn is_doc(&self, node: usize) -> Option<bool> {
        self.inner.is_doc(node).ok()
    }

    /// If the given node exists, returns true if it is a container.
    #[inline(always)]
    pub fn is_container(&self, node: usize) -> Option<bool> {
        self.inner.is_container(node).ok()
    }

    /// If the given node exists, returns true if it is a map.
    #[inline(always)]
    pub fn is_map(&self, node: usize) -> Option<bool> {
        self.inner.is_map(node).ok()
    }

    /// If the given node exists, returns true if it is a seq.
    #[inline(always)]
    pub fn is_seq(&self, node: usize) -> Option<bool> {
        self.inner.is_seq(node).ok()
    }

    /// If the given node exists, returns true if it has a value.
    #[inline(always)]
    pub fn has_val(&self, node: usize) -> Option<bool> {
        self.inner.has_val(node).ok()
    }

    /// If the given node exists, returns true if it has a key.
    #[inline(always)]
    pub fn has_key(&self, node: usize) -> Option<bool> {
        self.inner.has_key(node).ok()
    }

    /// If the given node exists, returns true if it is a value.
    #[inline(always)]
    pub fn is_val(&self, node: usize) -> Option<bool> {
        self.inner.is_val(node).ok()
    }

    /// If the given node exists, returns true if it is a keyval.
    #[inline(always)]
    pub fn is_keyval(&self, node: usize) -> Option<bool> {
        self.inner.is_keyval(node).ok()
    }

    /// If the given node exists, returns true if it has a tagged key.
    #[inline(always)]
    pub fn has_key_tag(&self, node: usize) -> Option<bool> {
        self.inner.has_key_tag(node).ok()
    }

    /// If the given node exists, returns true if it has a tagged value.
    #[inline(always)]
    pub fn has_val_tag(&self, node: usize) -> Option<bool> {
        self.inner.has_val_tag(node).ok()
    }

    /// If the given node exists, returns true if it has an anchor key.
    #[inline(always)]
    pub fn has_key_anchor(&self, node: usize) -> Option<bool> {
        self.inner.has_key_anchor(node).ok()
    }

    /// If the given node exists, returns true if it has an anchor value.
    #[inline(always)]
    pub fn has_val_anchor(&self, node: usize) -> Option<bool> {
        self.inner.has_val_anchor(node).ok()
    }

    /// If the given node exists, returns true if it is a key_ref.
    #[inline(always)]
    pub fn is_key_ref(&self, node: usize) -> Option<bool> {
        self.inner.is_key_ref(node).ok()
    }

    /// If the given node exists, returns true if it is a val_ref.
    #[inline(always)]
    pub fn is_val_ref(&self, node: usize) -> Option<bool> {
        self.inner.is_val_ref(node).ok()
    }

    /// If the given node exists, returns true if it is a ref.
    #[inline(always)]
    pub fn is_ref(&self, node: usize) -> Option<bool> {
        self.inner.is_ref(node).ok()
    }

    /// If the given node exists, returns true if it is a anchor_or_ref.
    #[inline(always)]
    pub fn is_anchor_or_ref(&self, node: usize) -> Option<bool> {
        self.inner.is_anchor_or_ref(node).ok()
    }

    /// If the given node exists, returns true if it is a key_quoted.
    #[inline(always)]
    pub fn is_key_quoted(&self, node: usize) -> Option<bool> {
        self.inner.is_key_quoted(node).ok()
    }

    /// If the given node exists, returns true if it is a val_quoted.
    #[inline(always)]
    pub fn is_val_quoted(&self, node: usize) -> Option<bool> {
        self.inner.is_val_quoted(node).ok()
    }

    /// If the given node exists, returns true if it is a quoted.
    #[inline(always)]
    pub fn is_quoted(&self, node: usize) -> Option<bool> {
        self.inner.is_quoted(node).ok()
    }

    /// If the given node exists, returns true if it is a anchor.
    #[inline(always)]
    pub fn is_anchor(&self, node: usize) -> Option<bool> {
        self.inner.is_anchor(node).ok()
    }

    /// If the given node exists, returns true if the parent is a
    /// sequence.
    #[inline(always)]
    pub fn parent_is_seq(&self, node: usize) -> Option<bool> {
        self.inner.parent_is_seq(node).ok()
    }

    /// If the given node exists, returns true if the parent is a
    /// map.
    #[inline(always)]
    pub fn parent_is_map(&self, node: usize) -> Option<bool> {
        self.inner.parent_is_map(node).ok()
    }

    /// If the given node exists, returns true if it is empty.
    #[inline(always)]
    pub fn is_node_empty(&self, node: usize) -> Option<bool> {
        self.inner.is_node_empty(node).ok()
    }

    /// If the given node exists, returns true if it has an anchor.
    #[inline(always)]
    pub fn has_anchor(&self, node: usize, anchor: &str) -> Option<bool> {
        self.inner.has_anchor(node, anchor.into()).ok()
    }

    /// If the given node exists, returns true if it has a parent.
    #[inline(always)]
    pub fn has_parent(&self, node: usize) -> Option<bool> {
        self.inner.has_parent(node).ok()
    }
    /// If the given node exists, returns true if it has a child.
    #[inline(always)]
    pub fn has_child(&self, node: usize, key: &str) -> Option<bool> {
        self.inner.has_child(node, key.into()).ok()
    }
    /// If the given node exists, returns true if it has children.
    #[inline(always)]
    pub fn has_children(&self, node: usize) -> Option<bool> {
        self.inner.has_children(node).ok()
    }
    /// If the given node exists, returns true if it has a sibling.
    #[inline(always)]
    pub fn has_sibling(&self, node: usize, key: &str) -> Option<bool> {
        self.inner.has_sibling(node, key.into()).ok()
    }
    /// If the given node exists, returns true if it has siblings.
    #[inline(always)]
    pub fn has_siblings(&self, node: usize) -> Option<bool> {
        self.inner.has_siblings(node).ok()
    }
    /// If the given node exists, returns true if it has other siblings.
    #[inline(always)]
    pub fn has_other_siblings(&self, node: usize) -> Option<bool> {
        self.inner.has_other_siblings(node).ok()
    }

    /// If the given node exists and has a parent, returns the
    /// parent node.
    #[inline(always)]
    pub fn parent(&self, node: usize) -> Option<usize> {
        self.inner.parent(node).ok()
    }

    /// If the given node exists and has a previous sibling, returns the index
    /// to the sibling node.
    #[inline(always)]
    pub fn prev_sibling(&self, node: usize) -> Option<usize> {
        self.inner.prev_sibling(node).ok()
    }

    /// If the given node exists and has a next sibling, returns the index to
    /// the sibling node.
    #[inline(always)]
    pub fn next_sibling(&self, node: usize) -> Option<usize> {
        self.inner.next_sibling(node).ok()
    }

    /// If the given node exists and has children, returns the
    /// number of children.
    #[inline(always)]
    pub fn num_children(&self, node: usize) -> Option<usize> {
        self.inner.num_children(node).ok()
    }

    /// If the given node exists and has the given child, returns
    /// the position of the child in the parent node.
    #[inline(always)]
    pub fn child_pos(&self, node: usize, child: usize) -> Option<usize> {
        self.inner.child_pos(node, child).ok()
    }

    /// If the given node exists and has children, returns the index of the
    /// first child node.
    #[inline(always)]
    pub fn first_child(&self, node: usize) -> Option<usize> {
        self.inner.first_child(node).ok()
    }

    /// If the given node exists and has children, returns the
    /// index to the last child node.
    #[inline(always)]
    pub fn last_child(&self, node: usize) -> Option<usize> {
        self.inner.last_child(node).ok()
    }

    /// If the given node exists and has a child at the given
    /// position, returns the index to the child node.
    #[inline(always)]
    pub fn child(&self, node: usize, pos: usize) -> Option<usize> {
        self.inner.child(node, pos).ok()
    }

    /// If the given node exists and has a child at the given
    /// key, returns the index to the child node.
    #[inline(always)]
    pub fn find_child(&self, node: usize, key: &str) -> Option<usize> {
        self.inner.find_child(node, &(key.into())).ok()
    }

    /// If the given node exists and has siblings, returns the
    /// number of siblings.
    #[inline(always)]
    pub fn num_siblings(&self, node: usize) -> Option<usize> {
        self.inner.num_siblings(node).ok()
    }

    /// If the given node exists and has other siblings, returns
    /// the number of other siblings.
    #[inline(always)]
    pub fn num_other_siblings(&self, node: usize) -> Option<usize> {
        self.inner.num_other_siblings(node).ok()
    }

    /// If the given node exists and has the given sibling, get
    /// position of the sibling in in the parent.
    #[inline(always)]
    pub fn sibling_pos(&self, node: usize, sibling: usize) -> Option<usize> {
        self.inner.sibling_pos(node, sibling).ok()
    }

    /// If the given node exists and has siblings, returns the
    /// index to the first sibling node.
    #[inline(always)]
    pub fn first_sibling(&self, node: usize) -> Option<usize> {
        self.inner.first_sibling(node).ok()
    }

    /// If the given node exists and has siblings, returns the
    /// index to the last sibling node.
    #[inline(always)]
    pub fn last_sibling(&self, node: usize) -> Option<usize> {
        self.inner.last_sibling(node).ok()
    }
    /// If the given node exists and has a sibling as the given
    /// position, returns the index to the sibling node.
    #[inline(always)]
    pub fn sibling(&self, node: usize, pos: usize) -> Option<usize> {
        self.inner.sibling(node, pos).ok()
    }

    /// If the given node exists and has a sibling at the given
    /// key, returns the index to the sibling node.
    #[inline(always)]
    pub fn find_sibling(&self, node: usize, key: &str) -> Option<usize> {
        self.inner.find_sibling(node, &(key.into())).ok()
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

    /// Set the tag on the given key node.
    #[inline(always)]
    pub fn set_key_tag(&mut self, node: usize, tag: &str) -> Result<()> {
        Ok(self.inner.pin_mut().set_key_tag(node, tag.into())?)
    }

    /// Set the anchor on the given key node.
    #[inline(always)]
    pub fn set_key_anchor(&mut self, node: usize, anchor: &str) -> Result<()> {
        Ok(self.inner.pin_mut().set_key_anchor(node, anchor.into())?)
    }

    /// Set the anchor on the given value node.
    #[inline(always)]
    pub fn set_val_anchor(&mut self, node: usize, anchor: &str) -> Result<()> {
        Ok(self.inner.pin_mut().set_val_anchor(node, anchor.into())?)
    }

    /// Set the ref on the given key node.
    #[inline(always)]
    pub fn set_key_ref(&mut self, node: usize, refr: &str) -> Result<()> {
        Ok(self.inner.pin_mut().set_key_ref(node, refr.into())?)
    }

    /// Set the ref on the given value node.
    #[inline(always)]
    pub fn set_val_ref(&mut self, node: usize, refr: &str) -> Result<()> {
        Ok(self.inner.pin_mut().set_val_ref(node, refr.into())?)
    }

    /// Set the tag on the given value node.
    pub fn set_val_tag(&mut self, node: usize, tag: &str) -> Result<()> {
        Ok(self.inner.pin_mut().set_val_tag(node, tag.into())?)
    }

    /// Remove the anchor on the given key node.
    pub fn rem_key_anchor(&mut self, node: usize) -> Result<()> {
        Ok(self.inner.pin_mut().rem_key_anchor(node)?)
    }

    /// Remove the anchor on the given value node.
    pub fn rem_val_anchor(&mut self, node: usize) -> Result<()> {
        Ok(self.inner.pin_mut().rem_val_anchor(node)?)
    }

    /// Remove the reference on the given key node.
    pub fn rem_key_ref(&mut self, node: usize) -> Result<()> {
        Ok(self.inner.pin_mut().rem_key_ref(node)?)
    }

    /// Remove the reference on the given value node.
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
    fn parse() {
        let tree = Tree::parse(SRC).unwrap();
        assert_eq!(78, tree.len());
        tree.val(47).unwrap();
    }
}
