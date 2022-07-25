#![feature(core_ffi_c)]
use std::marker::PhantomData;
use thiserror::Error;
mod inner;
pub use inner::NodeType;

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
        inner::ffi::init_ryml_once();
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
        inner::ffi::init_ryml_once();
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
        inner::ffi::init_ryml_once();
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
        inner::ffi::init_ryml_once();
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
        inner::ffi::init_ryml_once();
        let written =
            inner::ffi::emit_to_rwriter(&self.inner, Box::new(inner::RWriter { writer }))?;
        Ok(written)
    }

    /// Get the index to the root node.
    #[inline(always)]
    pub fn root_id(&self) -> Option<usize> {
        inner::ffi::init_ryml_once();
        self.inner.root_id().ok()
    }

    /// Get the total number of nodes.
    #[inline(always)]
    pub fn len(&self) -> usize {
        inner::ffi::init_ryml_once();
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
    pub fn arena_len(&self) -> usize {
        self.inner.arena_size()
    }

    /// Returns true is the internal string arena is empty.
    pub fn arena_is_empty(&self) -> bool {
        self.arena_len() == 0
    }

    /// Get the capacity of the internal string arena.
    pub fn arena_capacity(&self) -> usize {
        self.inner.arena_capacity()
    }

    /// Get the unused capacity of the internal string arena.
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
    pub fn reserve_arena(&mut self, arena_capacity: usize) {
        self.inner.pin_mut().reserve_arena(arena_capacity);
    }

    /// Clear the tree and zero every node.
    ///
    /// **Note**: Does **not** clear the arena.
    /// See also [`clear_arena`](#method.clear_arena).
    pub fn clear(&mut self) {
        self.inner.pin_mut().clear();
    }

    /// Clear the internal string arena.
    pub fn clear_arena(&mut self) {
        self.inner.pin_mut().clear_arena();
    }

    /// Get the type of the node at the given index, if it exists.
    pub fn node_type(&self, index: usize) -> Option<NodeType> {
        inner::ffi::init_ryml_once();
        inner::ffi::tree_node_type(&self.inner, index).ok()
    }

    /// Get the type name of the node at the given index, if it exists.
    pub fn node_type_as_str(&self, index: usize) -> Option<&str> {
        let ptr = self.inner.type_str(index).ok()?;
        unsafe { std::ffi::CStr::from_ptr(ptr) }.to_str().ok()
    }

    /// Get the text of the node at the given index, if it exists and is a key.
    pub fn key(&self, index: usize) -> Option<&str> {
        self.inner.key(index).ok().map(|s| s.as_ref())
    }

    /// Get the text of the tag on the key at the given index, if it exists and
    /// is a tagged key.
    pub fn key_tag(&self, index: usize) -> Option<&str> {
        self.inner.key_tag(index).ok().map(|s| s.as_ref())
    }
}
