#![feature(core_ffi_c)]
use std::{borrow::Borrow, marker::PhantomData};
use thiserror::Error;
mod inner;
pub use inner::{NodeScalar, NodeType};

/// Error type for this crate
#[derive(Debug, Error)]
pub enum Error {
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

    /// Get the index to the root node.
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
    /// Dereferencing is opt-in; after parsing, [`Tree::resolve()`](#method.resolve)
    /// has to be called explicitly for obtaining resolved references in the
    /// tree. This method will resolve all references and substitute the
    /// anchored values in place of the reference.
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

    /// Get the type of the node at the given index, if it exists.
    #[inline(always)]
    pub fn node_type(&self, index: usize) -> Option<NodeType> {
        inner::ffi::tree_node_type(&self.inner, index).ok()
    }

    /// Get the type name of the node at the given index, if it exists.
    #[inline(always)]
    pub fn node_type_as_str(&self, index: usize) -> Option<&str> {
        let ptr = self.inner.type_str(index).ok()?;
        unsafe { std::ffi::CStr::from_ptr(ptr) }.to_str().ok()
    }

    /// Get the text of the node at the given index, if it exists and is a key.
    #[inline(always)]
    pub fn key(&self, index: usize) -> Option<&str> {
        self.inner.key(index).ok().map(|s| s.as_ref())
    }

    /// Get the text of the tag on the key at the given index, if it exists and
    /// is a tagged key.
    #[inline(always)]
    pub fn key_tag(&self, index: usize) -> Option<&str> {
        self.inner.key_tag(index).ok().map(|s| s.as_ref())
    }

    /// Get the text of the reference on the key at the given index, if it exists
    /// and is a reference.
    pub fn key_ref(&self, index: usize) -> Option<&str> {
        self.inner.key_ref(index).ok().map(|s| s.as_ref())
    }

    /// Get the text of the anchor on the key at the given index, if it exists
    /// and is an anchor.
    pub fn key_anchor(&self, index: usize) -> Option<&str> {
        self.inner.key_anchor(index).ok().map(|s| s.as_ref())
    }

    /// Get the whole scalar key at the given index, if it exists and is a
    /// scalar key.
    pub fn key_scalar(&self, index: usize) -> Option<&NodeScalar> {
        self.inner.keysc(index).ok()
    }

    /// Get the text of the node at the given index, if it exists and is a value.
    #[inline(always)]
    pub fn val(&self, index: usize) -> Option<&str> {
        self.inner.val(index).ok().map(|s| s.as_ref())
    }

    /// Get the text of the tag on the value at the given index, if it exists and
    /// is a tagged value.
    #[inline(always)]
    pub fn val_tag(&self, index: usize) -> Option<&str> {
        self.inner.val_tag(index).ok().map(|s| s.as_ref())
    }

    /// Get the text of the reference on the value at the given index, if it exists
    /// and is a reference.
    #[inline(always)]
    pub fn val_ref(&self, index: usize) -> Option<&str> {
        self.inner.val_ref(index).ok().map(|s| s.as_ref())
    }

    /// Get the text of the anchor on the value at the given index, if it exists
    /// and is an anchor.
    #[inline(always)]
    pub fn val_anchor(&self, index: usize) -> Option<&str> {
        self.inner.val_anchor(index).ok().map(|s| s.as_ref())
    }

    /// Get the whole scalar value at the given index, if it exists and is a
    /// scalar value.
    #[inline(always)]
    pub fn val_scalar(&self, index: usize) -> Option<&NodeScalar> {
        self.inner.valsc(index).ok()
    }

    /// If the node at the given index exists, returns true if it is a root.
    #[inline(always)]
    pub fn is_root(&self, index: usize) -> Option<bool> {
        self.inner.is_root(index).ok()
    }

    /// If the node at the given index exists, returns true if it is a stream.
    #[inline(always)]
    pub fn is_stream(&self, index: usize) -> Option<bool> {
        self.inner.is_stream(index).ok()
    }

    /// If the node at the given index exists, returns true if it is a doc.
    #[inline(always)]
    pub fn is_doc(&self, index: usize) -> Option<bool> {
        self.inner.is_doc(index).ok()
    }

    /// If the node at the given index exists, returns true if it is a container.
    #[inline(always)]
    pub fn is_container(&self, index: usize) -> Option<bool> {
        self.inner.is_container(index).ok()
    }

    /// If the node at the given index exists, returns true if it is a map.
    #[inline(always)]
    pub fn is_map(&self, index: usize) -> Option<bool> {
        self.inner.is_map(index).ok()
    }

    /// If the node at the given index exists, returns true if it is a seq.
    #[inline(always)]
    pub fn is_seq(&self, index: usize) -> Option<bool> {
        self.inner.is_seq(index).ok()
    }

    /// If the node at the given index exists, returns true if it has a value.
    #[inline(always)]
    pub fn has_val(&self, index: usize) -> Option<bool> {
        self.inner.has_val(index).ok()
    }

    /// If the node at the given index exists, returns true if it has a key.
    #[inline(always)]
    pub fn has_key(&self, index: usize) -> Option<bool> {
        self.inner.has_key(index).ok()
    }

    /// If the node at the given index exists, returns true if it is a value.
    #[inline(always)]
    pub fn is_val(&self, index: usize) -> Option<bool> {
        self.inner.is_val(index).ok()
    }

    /// If the node at the given index exists, returns true if it is a keyval.
    #[inline(always)]
    pub fn is_keyval(&self, index: usize) -> Option<bool> {
        self.inner.is_keyval(index).ok()
    }

    /// If the node at the given index exists, returns true if it has a tagged key.
    #[inline(always)]
    pub fn has_key_tag(&self, index: usize) -> Option<bool> {
        self.inner.has_key_tag(index).ok()
    }

    /// If the node at the given index exists, returns true if it has a tagged value.
    #[inline(always)]
    pub fn has_val_tag(&self, index: usize) -> Option<bool> {
        self.inner.has_val_tag(index).ok()
    }

    /// If the node at the given index exists, returns true if it has an anchor key.
    #[inline(always)]
    pub fn has_key_anchor(&self, index: usize) -> Option<bool> {
        self.inner.has_key_anchor(index).ok()
    }

    /// If the node at the given index exists, returns true if it has an anchor value.
    #[inline(always)]
    pub fn has_val_anchor(&self, index: usize) -> Option<bool> {
        self.inner.has_val_anchor(index).ok()
    }

    /// If the node at the given index exists, returns true if it is a key_ref.
    #[inline(always)]
    pub fn is_key_ref(&self, index: usize) -> Option<bool> {
        self.inner.is_key_ref(index).ok()
    }

    /// If the node at the given index exists, returns true if it is a val_ref.
    #[inline(always)]
    pub fn is_val_ref(&self, index: usize) -> Option<bool> {
        self.inner.is_val_ref(index).ok()
    }

    /// If the node at the given index exists, returns true if it is a ref.
    #[inline(always)]
    pub fn is_ref(&self, index: usize) -> Option<bool> {
        self.inner.is_ref(index).ok()
    }

    /// If the node at the given index exists, returns true if it is a anchor_or_ref.
    #[inline(always)]
    pub fn is_anchor_or_ref(&self, index: usize) -> Option<bool> {
        self.inner.is_anchor_or_ref(index).ok()
    }

    /// If the node at the given index exists, returns true if it is a key_quoted.
    #[inline(always)]
    pub fn is_key_quoted(&self, index: usize) -> Option<bool> {
        self.inner.is_key_quoted(index).ok()
    }

    /// If the node at the given index exists, returns true if it is a val_quoted.
    #[inline(always)]
    pub fn is_val_quoted(&self, index: usize) -> Option<bool> {
        self.inner.is_val_quoted(index).ok()
    }

    /// If the node at the given index exists, returns true if it is a quoted.
    #[inline(always)]
    pub fn is_quoted(&self, index: usize) -> Option<bool> {
        self.inner.is_quoted(index).ok()
    }

    /// If the node at the given index exists, returns true if it is a anchor.
    #[inline(always)]
    pub fn is_anchor(&self, index: usize) -> Option<bool> {
        self.inner.is_anchor(index).ok()
    }

    /// If the node at the given index exists, returns true if the parent is a
    /// sequence.
    #[inline(always)]
    pub fn parent_is_seq(&self, index: usize) -> Option<bool> {
        self.inner.parent_is_seq(index).ok()
    }

    /// If the node at the given index exists, returns true if the parent is a
    /// map.
    #[inline(always)]
    pub fn parent_is_map(&self, index: usize) -> Option<bool> {
        self.inner.parent_is_map(index).ok()
    }

    /// If the node at the given index exists, returns true if it is empty.
    #[inline(always)]
    pub fn is_node_empty(&self, index: usize) -> Option<bool> {
        self.inner.is_node_empty(index).ok()
    }

    /// If the node at the given index exists, returns true if it has an anchor.
    #[inline(always)]
    pub fn has_anchor(&self, index: usize, anchor: &str) -> Option<bool> {
        self.inner.has_anchor(index, anchor.into()).ok()
    }

    /// If the node at the given index exists, returns true if it has a parent.
    #[inline(always)]
    pub fn has_parent(&self, index: usize) -> Option<bool> {
        self.inner.has_parent(index).ok()
    }
    /// If the node at the given index exists, returns true if it has a child.
    #[inline(always)]
    pub fn has_child(&self, index: usize, key: &str) -> Option<bool> {
        self.inner.has_child(index, key.into()).ok()
    }
    /// If the node at the given index exists, returns true if it has children.
    #[inline(always)]
    pub fn has_children(&self, index: usize) -> Option<bool> {
        self.inner.has_children(index).ok()
    }
    /// If the node at the given index exists, returns true if it has a sibling.
    #[inline(always)]
    pub fn has_sibling(&self, index: usize, key: &str) -> Option<bool> {
        self.inner.has_sibling(index, key.into()).ok()
    }
    /// If the node at the given index exists, returns true if it has siblings.
    #[inline(always)]
    pub fn has_siblings(&self, index: usize) -> Option<bool> {
        self.inner.has_siblings(index).ok()
    }
    /// If the node at the given index exists, returns true if it has other siblings.
    #[inline(always)]
    pub fn has_other_siblings(&self, index: usize) -> Option<bool> {
        self.inner.has_other_siblings(index).ok()
    }

    /// If the node at the given index exists and has a parent, returns the
    /// parent index.
    #[inline(always)]
    pub fn parent(&self, index: usize) -> Option<usize> {
        self.inner.parent(index).ok()
    }

    /// If the node at the given index exists and has a previous sibling,
    /// returns the index of the previous sibling.
    #[inline(always)]
    pub fn prev_sibling(&self, index: usize) -> Option<usize> {
        self.inner.prev_sibling(index).ok()
    }

    /// If the node at the given index exists and has a next sibling,
    /// returns the index of the next sibling.
    #[inline(always)]
    pub fn next_sibling(&self, index: usize) -> Option<usize> {
        self.inner.next_sibling(index).ok()
    }

    /// If the node at the given index exists and has children, returns the
    /// number of children.
    #[inline(always)]
    pub fn num_children(&self, index: usize) -> Option<usize> {
        self.inner.num_children(index).ok()
    }

    /// If the node at the given index exists and has the given child, returns
    /// the position of the child in the parent node.
    #[inline(always)]
    pub fn child_pos(&self, index: usize, child: usize) -> Option<usize> {
        self.inner.child_pos(index, child).ok()
    }

    /// If the node at the given index exists and has children, returns the
    /// index to the first child.
    #[inline(always)]
    pub fn first_child(&self, index: usize) -> Option<usize> {
        self.inner.first_child(index).ok()
    }

    /// If the node at the given index exists and has children, returns the
    /// index to the last child.
    #[inline(always)]
    pub fn last_child(&self, index: usize) -> Option<usize> {
        self.inner.last_child(index).ok()
    }

    /// If the node at the given index exists and has a child at the given
    /// position, returns the absolute index to the child.
    #[inline(always)]
    pub fn child(&self, index: usize, pos: usize) -> Option<usize> {
        self.inner.child(index, pos).ok()
    }

    /// If the node at the given index exists and has a child at the given
    /// key, returns the index to the child.
    #[inline(always)]
    pub fn find_child(&self, index: usize, key: &str) -> Option<usize> {
        self.inner.find_child(index, &(key.into())).ok()
    }

    /// If the node at the given index exists and has siblings, returns the
    /// number of siblings.
    #[inline(always)]
    pub fn num_siblings(&self, index: usize) -> Option<usize> {
        self.inner.num_siblings(index).ok()
    }

    /// If the node at the given index exists and has other siblings, returns
    /// the number of other siblings.
    #[inline(always)]
    pub fn num_other_siblings(&self, index: usize) -> Option<usize> {
        self.inner.num_other_siblings(index).ok()
    }

    /// If the node at the given index exists and has the given sibling, get
    /// position of the sibling in in the parent.
    #[inline(always)]
    pub fn sibling_pos(&self, index: usize, sibling: usize) -> Option<usize> {
        self.inner.sibling_pos(index, sibling).ok()
    }

    /// If the node at the given index exists and has siblings, returns the
    /// index to the first sibling.
    #[inline(always)]
    pub fn first_sibling(&self, index: usize) -> Option<usize> {
        self.inner.first_sibling(index).ok()
    }

    /// If the node at the given index exists and has siblings, returns the
    /// index to the last sibling.
    #[inline(always)]
    pub fn last_sibling(&self, index: usize) -> Option<usize> {
        self.inner.last_sibling(index).ok()
    }
    /// If the node at the given index exists and has a sibling as the given
    /// position, returns the abolute index to the sibling.
    #[inline(always)]
    pub fn sibling(&self, index: usize, pos: usize) -> Option<usize> {
        self.inner.sibling(index, pos).ok()
    }

    /// If the node at the given index exists and has a sibling at the given
    /// key, returns the index to the sibling.
    #[inline(always)]
    pub fn find_sibling(&self, index: usize, key: &str) -> Option<usize> {
        self.inner.find_sibling(index, &(key.into())).ok()
    }

    /// Turn the given node into a key-value pair.
    #[inline(always)]
    pub fn to_keyval(&mut self, index: usize, key: &str, val: &str) -> Result<()> {
        Ok(self
            .inner
            .pin_mut()
            .to_keyval(index, key.into(), val.into(), 0)?)
    }

    /// Turn the given node into a key-value pair with additional flags.
    #[inline(always)]
    pub fn to_keyval_with_flags(
        &mut self,
        index: usize,
        key: &str,
        val: &str,
        more_flags: NodeType,
    ) -> Result<()> {
        Ok(self
            .inner
            .pin_mut()
            .to_keyval(index, key.into(), val.into(), more_flags as u64)?)
    }
}
