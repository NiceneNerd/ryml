#![allow(missing_docs, non_upper_case_globals)]
#[cfg(not(feature = "std"))]
use acid_io as io;
use auto_enum::enum_flags;
use core::ops::Deref;
#[cfg(feature = "std")]
use std::io;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct CSubstr {
    pub ptr: *const u8,
    pub len: usize,
}

impl PartialEq for CSubstr {
    fn eq(&self, other: &Self) -> bool {
        self.deref().eq(other.deref())
    }
}

impl Deref for CSubstr {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        unsafe { core::str::from_utf8_unchecked(core::slice::from_raw_parts(self.ptr, self.len)) }
    }
}

impl AsRef<str> for CSubstr {
    fn as_ref(&self) -> &str {
        self.deref()
    }
}

impl From<&str> for CSubstr {
    fn from(s: &str) -> Self {
        CSubstr {
            ptr: s.as_ptr(),
            len: s.len(),
        }
    }
}

impl From<Substr> for CSubstr {
    fn from(s: Substr) -> Self {
        CSubstr {
            ptr: s.as_ptr(),
            len: s.len(),
        }
    }
}

unsafe impl cxx::ExternType for CSubstr {
    type Id = cxx::type_id!("c4::csubstr");
    type Kind = cxx::kind::Trivial;
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Substr {
    pub ptr: *mut u8,
    pub len: usize,
}

impl PartialEq for Substr {
    fn eq(&self, other: &Self) -> bool {
        self.deref().eq(other.deref())
    }
}

impl core::ops::Deref for Substr {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        unsafe { core::str::from_utf8_unchecked(core::slice::from_raw_parts(self.ptr, self.len)) }
    }
}

impl core::ops::DerefMut for Substr {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { core::mem::transmute(*self) }
    }
}

unsafe impl cxx::ExternType for Substr {
    type Id = cxx::type_id!("c4::substr");
    type Kind = cxx::kind::Trivial;
}

/// An entry in the [`Tree`](super::Tree) representing all the data for a YAML
/// node.
///
/// Note that the relational indicies stored here (e.g.
/// [`first_child`](#structfield.first_child)) may refer to no existing node, in
/// which case the value will be the constant [`NONE`](super::NONE). Where it is
/// not a serious performance concern, it can be safer to use the getters
/// instead, which return [`Option`]s.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeData<'t> {
    /// The node type flags.
    pub node_type: NodeType,
    /// The node key.
    pub key: NodeScalar<'t>,
    /// The node value.
    pub value: NodeScalar<'t>,
    /// The index to the parent node.
    pub parent: usize,
    /// The index to the first child node.
    pub first_child: usize,
    /// The index to the last child node.
    pub last_child: usize,
    /// The index to the next sibling node.
    pub next_sibling: usize,
    /// The index to the previous sibling node.
    pub prev_sibling: usize,
}

impl NodeData<'_> {
    /// Get the index to the parent node, if one exists.
    #[inline(always)]
    pub fn parent(&self) -> Option<usize> {
        (self.parent != super::NONE).then_some(self.parent)
    }

    /// Get the index to the first child node, if one exists.
    #[inline(always)]
    pub fn first_child(&self) -> Option<usize> {
        (self.first_child != super::NONE).then_some(self.first_child)
    }
    /// Get the index to the last_child node, if one exists.
    #[inline(always)]
    pub fn last_child(&self) -> Option<usize> {
        (self.last_child != super::NONE).then_some(self.last_child)
    }

    /// Get the index to the next sibling node, if one exists.
    #[inline(always)]
    pub fn next_sibling(&self) -> Option<usize> {
        (self.next_sibling != super::NONE).then_some(self.next_sibling)
    }

    /// Get the index to the previous sibling node, if one exists.
    #[inline(always)]
    pub fn prev_sibling(&self) -> Option<usize> {
        (self.prev_sibling != super::NONE).then_some(self.prev_sibling)
    }
}

unsafe impl cxx::ExternType for NodeData<'_> {
    type Id = cxx::type_id!("c4::yml::NodeData");
    type Kind = cxx::kind::Trivial;
}

#[cfg(not(feature = "std"))]
use core as std;

#[enum_flags(u64)]
/// A bitmask for marking node types.
pub enum NodeType {
    /// no type is set
    NoType = 0,
    /// a leaf node, has a (possibly empty) value
    Val = 1 << 0,
    /// is member of a map, must have non-empty key
    Key = 1 << 1,
    /// a map: a parent of keyvals
    Map = 1 << 2,
    /// a seq: a parent of vals
    Seq = 1 << 3,
    /// a document
    Doc = 1 << 4,
    /// a stream: a seq of docs
    Stream = 1 << 5 | 1 << 3,
    /// a *reference: the key references an &anchor
    KeyRef = 1 << 6,
    /// a *reference: the val references an &anchor
    ValRef = 1 << 7,
    /// the key has an &anchor
    KeyAnch = 1 << 8,
    /// the val has an &anchor
    ValAnch = 1 << 9,
    /// the key has an explicit tag/type
    KeyTag = 1 << 10,
    /// the val has an explicit tag/type
    ValTag = 1 << 11,
    // these flags are from a work in progress and should not be used yet
    /// mark container with single-line flow format (seqs as '[val1,val2], maps
    /// as '{key: val, key2: val2}')
    WipStyleFlowSl = 1 << 14,
    /// mark container with multi-line flow format (seqs as '[val1,\nval2], maps
    /// as '{key: val,\nkey2: val2}')
    WipStyleFlowMl = 1 << 15,
    /// mark container with block format (seqs as '- val\n', maps as 'key: val')
    WipStyleBlock = 1 << 16,
    /// mark key scalar as multiline, block literal |
    WipKeyLiteral = 1 << 17,
    /// mark val scalar as multiline, block literal |
    WipValLiteral = 1 << 18,
    /// mark key scalar as multiline, block folded >
    WipKeyFolded = 1 << 19,
    /// mark val scalar as multiline, block folded >
    WipValFolded = 1 << 20,
    /// mark key scalar as single quoted
    WipKeySquo = 1 << 21,
    /// mark val scalar as single quoted
    WipValSquo = 1 << 22,
    /// mark key scalar as double quoted
    WipKeyDquo = 1 << 23,
    /// mark val scalar as double quoted
    WipValDquo = 1 << 24,
    /// mark key scalar as plain scalar (unquoted, even when multiline)
    WipKeyPlain = 1 << 25,
    /// mark val scalar as plain scalar (unquoted, even when multiline)
    WipValPlain = 1 << 26,
    /// ?
    WipKeyStyle = 1 << 17 | 1 << 19 | 1 << 21 | 1 << 23 | 1 << 25,
    /// ?
    WipValStyle = 1 << 18 | 1 << 20 | 1 << 22 | 1 << 24 | 1 << 26,
    /// features: mark key scalar as having \n in its contents
    WipKeyFtNl = 1 << 27,
    /// features: mark val scalar as having \n in its contents
    WipValFtNl = 1 << 28,
    /// features: mark key scalar as having single quotes in its contents
    WipKeyFtSq = 1 << 29,
    /// features: mark val scalar as having single quotes in its contents
    WipValFtSq = 1 << 30,
    /// features: mark key scalar as having double quotes in its contents
    WipKeyFtDq = 1 << 31,
    /// features: mark val scalar as having double quotes in its contents
    WipValFtDq = 1 << 32,
}

unsafe impl cxx::ExternType for NodeType {
    type Id = cxx::type_id!("c4::yml::NodeType");
    type Kind = cxx::kind::Trivial;
}

/// A view of scalar data for a node, containing the tag, anchor, and scalar
/// value.
///
/// Note that, for FFI simplicity, each value allows
/// blank string slices instead of being optional.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeScalar<'a> {
    /// The tag associated with this node.
    pub tag: &'a str,
    /// The text of the node scalar value.
    pub scalar: &'a str,
    /// The text of the anchor associated with this node.
    pub anchor: &'a str,
}

unsafe impl cxx::ExternType for NodeScalar<'_> {
    type Id = cxx::type_id!("c4::yml::NodeScalar");
    type Kind = cxx::kind::Trivial;
}

#[repr(C)]
pub struct RepC {
    char: core::ffi::c_char,
    num_times: usize,
}

unsafe impl cxx::ExternType for RepC {
    type Id = cxx::type_id!("c4::yml::RepC");
    type Kind = cxx::kind::Trivial;
}

pub trait WriteSeek: io::Write + io::Seek {}
impl<T: io::Write + io::Seek> WriteSeek for T {}

pub struct RWriter<'a> {
    pub writer: &'a mut dyn WriteSeek,
}

impl RWriter<'_> {
    #[inline(always)]
    fn _get(&mut self, _error_on_excess: bool) -> io::Result<Substr> {
        Ok(Substr {
            ptr: core::ptr::null_mut(),
            len: self.writer.stream_position()? as usize,
        })
    }

    #[inline(always)]
    fn _do_write(&mut self, s: CSubstr) -> io::Result<()> {
        if s.is_empty() {
            return Ok(());
        }
        self.writer.write_all(s.as_bytes())?;
        Ok(())
    }

    #[inline(always)]
    fn _do_write_slice(&mut self, slice: &[core::ffi::c_char]) -> io::Result<()> {
        for c in slice {
            self.writer.write_all(&[*c as u8])?;
        }
        Ok(())
    }

    #[inline(always)]
    fn _do_write_char(&mut self, c: core::ffi::c_char) -> io::Result<()> {
        self.writer.write_all(&[c as u8])?;
        Ok(())
    }

    #[inline(always)]
    fn _do_write_repc(&mut self, recp: RepC) -> io::Result<()> {
        for _ in 0..recp.num_times {
            self.writer.write_all(&[recp.char as u8])?;
        }
        Ok(())
    }
}

#[allow(clippy::needless_lifetimes)] // Needed because of some weirdness at line 539
#[cxx::bridge]
pub(crate) mod ffi {
    #[namespace = "c4"]
    unsafe extern "C++" {
        type csubstr = super::CSubstr;
        type substr = super::Substr;
        #[namespace = "c4::yml"]
        type RepC = super::RepC;
        #[namespace = "c4::yml"]
        type NodeData<'a> = super::NodeData<'a>;
    }
    #[namespace = "shimmy"]
    extern "Rust" {
        type RWriter<'a>;
        fn _get(self: &mut RWriter, error_on_excess: bool) -> Result<substr>;
        fn _do_write(self: &mut RWriter, s: csubstr) -> Result<()>;
        fn _do_write_slice(self: &mut RWriter, slice: &[c_char]) -> Result<()>;
        fn _do_write_char(self: &mut RWriter, c: c_char) -> Result<()>;
        fn _do_write_repc(self: &mut RWriter, recp: RepC) -> Result<()>;
    }
    #[namespace = "c4::yml"]
    #[allow(missing_docs)]
    unsafe extern "C++" {
        include!("ryml/include/ryml.h");
        /// NodeType and NodeType_e
        type NodeType = super::NodeType;
        type NodeScalar<'a> = super::NodeScalar<'a>;
        fn set(self: Pin<&mut NodeType>, t: u64);
        fn add(self: Pin<&mut NodeType>, t: u64);
        fn rem(self: Pin<&mut NodeType>, t: u64);
        fn is_stream(self: &NodeType) -> bool;
        fn is_doc(self: &NodeType) -> bool;
        fn is_container(self: &NodeType) -> bool;
        fn is_map(self: &NodeType) -> bool;
        fn is_seq(self: &NodeType) -> bool;
        fn has_val(self: &NodeType) -> bool;
        fn has_key(self: &NodeType) -> bool;
        fn is_val(self: &NodeType) -> bool;
        fn is_keyval(self: &NodeType) -> bool;
        fn has_key_tag(self: &NodeType) -> bool;
        fn has_val_tag(self: &NodeType) -> bool;
        fn has_key_anchor(self: &NodeType) -> bool;
        fn is_key_anchor(self: &NodeType) -> bool;
        fn has_val_anchor(self: &NodeType) -> bool;
        fn is_val_anchor(self: &NodeType) -> bool;
        fn has_anchor(self: &NodeType) -> bool;
        fn is_anchor(self: &NodeType) -> bool;
        fn is_key_ref(self: &NodeType) -> bool;
        fn is_val_ref(self: &NodeType) -> bool;
        fn is_ref(self: &NodeType) -> bool;
        fn is_anchor_or_ref(self: &NodeType) -> bool;
        fn is_key_quoted(self: &NodeType) -> bool;
        fn is_val_quoted(self: &NodeType) -> bool;
        fn is_quoted(self: &NodeType) -> bool;

        /// Tree
        type Tree;
        fn reserve(self: Pin<&mut Tree>, node_capacity: usize);
        fn reserve_arena(self: Pin<&mut Tree>, node_capacity: usize);
        fn clear(self: Pin<&mut Tree>);
        fn clear_arena(self: Pin<&mut Tree>);
        fn empty(self: &Tree) -> bool;
        fn size(self: &Tree) -> usize;
        fn capacity(self: &Tree) -> usize;
        fn slack(self: &Tree) -> Result<usize>;

        fn arena_size(self: &Tree) -> usize;
        fn arena_capacity(self: &Tree) -> usize;
        fn arena_slack(self: &Tree) -> Result<usize>;

        fn get(self: &Tree, i: usize) -> Result<*const NodeData>;
        #[cxx_name = "get"]
        fn get_mut(self: Pin<&mut Tree>, i: usize) -> Result<*mut NodeData>;

        fn resolve(self: Pin<&mut Tree>) -> Result<()>;

        fn type_str(self: &Tree, node: usize) -> Result<*const c_char>;

        fn key(self: &Tree, node: usize) -> Result<&csubstr>;
        fn key_tag(self: &Tree, node: usize) -> Result<&csubstr>;
        fn key_ref(self: &Tree, node: usize) -> Result<&csubstr>;
        fn key_anchor(self: &Tree, node: usize) -> Result<&csubstr>;
        fn keysc(self: &Tree, node: usize) -> Result<&NodeScalar>;

        fn val(self: &Tree, node: usize) -> Result<&csubstr>;
        fn val_tag(self: &Tree, node: usize) -> Result<&csubstr>;
        fn val_ref(self: &Tree, node: usize) -> Result<&csubstr>;
        fn val_anchor(self: &Tree, node: usize) -> Result<&csubstr>;
        fn valsc(self: &Tree, node: usize) -> Result<&NodeScalar>;

        fn is_root(self: &Tree, node: usize) -> Result<bool>;
        fn is_stream(self: &Tree, node: usize) -> Result<bool>;
        fn is_doc(self: &Tree, node: usize) -> Result<bool>;
        fn is_container(self: &Tree, node: usize) -> Result<bool>;
        fn is_map(self: &Tree, node: usize) -> Result<bool>;
        fn is_seq(self: &Tree, node: usize) -> Result<bool>;
        fn has_val(self: &Tree, node: usize) -> Result<bool>;
        fn has_key(self: &Tree, node: usize) -> Result<bool>;
        fn is_val(self: &Tree, node: usize) -> Result<bool>;
        fn is_keyval(self: &Tree, node: usize) -> Result<bool>;
        fn has_key_tag(self: &Tree, node: usize) -> Result<bool>;
        fn has_val_tag(self: &Tree, node: usize) -> Result<bool>;
        fn has_key_anchor(self: &Tree, node: usize) -> Result<bool>;
        fn has_val_anchor(self: &Tree, node: usize) -> Result<bool>;
        fn is_key_ref(self: &Tree, node: usize) -> Result<bool>;
        fn is_val_ref(self: &Tree, node: usize) -> Result<bool>;
        fn is_ref(self: &Tree, node: usize) -> Result<bool>;
        fn is_anchor_or_ref(self: &Tree, node: usize) -> Result<bool>;
        fn is_key_quoted(self: &Tree, node: usize) -> Result<bool>;
        fn is_val_quoted(self: &Tree, node: usize) -> Result<bool>;
        fn is_quoted(self: &Tree, node: usize) -> Result<bool>;
        fn is_anchor(self: &Tree, node: usize) -> Result<bool>;
        fn parent_is_seq(self: &Tree, node: usize) -> Result<bool>;
        fn parent_is_map(self: &Tree, node: usize) -> Result<bool>;
        #[cxx_name = "empty"]
        fn is_node_empty(self: &Tree, node: usize) -> Result<bool>;
        fn has_anchor(self: &Tree, node: usize, a: csubstr) -> Result<bool>;

        fn has_parent(self: &Tree, node: usize) -> Result<bool>;
        fn has_child(self: &Tree, node: usize, key: csubstr) -> Result<bool>;
        fn has_children(self: &Tree, node: usize) -> Result<bool>;
        fn has_sibling(self: &Tree, node: usize, key: csubstr) -> Result<bool>;
        // fn has_siblings(self: &Tree, node: usize) -> Result<bool>;
        fn has_other_siblings(self: &Tree, node: usize) -> Result<bool>;

        fn root_id(self: &Tree) -> Result<usize>;
        fn parent(self: &Tree, node: usize) -> Result<usize>;
        fn prev_sibling(self: &Tree, node: usize) -> Result<usize>;
        fn next_sibling(self: &Tree, node: usize) -> Result<usize>;
        fn num_children(self: &Tree, node: usize) -> Result<usize>;
        fn child_pos(self: &Tree, node: usize, ch: usize) -> Result<usize>;
        fn first_child(self: &Tree, node: usize) -> Result<usize>;
        fn last_child(self: &Tree, node: usize) -> Result<usize>;
        fn child(self: &Tree, node: usize, pos: usize) -> Result<usize>;
        fn find_child(self: &Tree, node: usize, key: &csubstr) -> Result<usize>;
        fn num_siblings(self: &Tree, node: usize) -> Result<usize>;
        fn num_other_siblings(self: &Tree, node: usize) -> Result<usize>;
        fn sibling_pos(self: &Tree, node: usize, sib: usize) -> Result<usize>;
        fn first_sibling(self: &Tree, node: usize) -> Result<usize>;
        fn last_sibling(self: &Tree, node: usize) -> Result<usize>;
        fn sibling(self: &Tree, node: usize, pos: usize) -> Result<usize>;
        fn find_sibling(self: &Tree, node: usize, key: &csubstr) -> Result<usize>;

        fn to_keyval(
            self: Pin<&mut Tree>,
            node: usize,
            key: csubstr,
            val: csubstr,
            more_flags: u64,
        ) -> Result<()>;
        #[cxx_name = "to_map"]
        fn to_map_with_key(
            self: Pin<&mut Tree>,
            node: usize,
            key: csubstr,
            more_flags: u64,
        ) -> Result<()>;
        #[cxx_name = "to_seq"]
        fn to_seq_with_key(
            self: Pin<&mut Tree>,
            node: usize,
            key: csubstr,
            more_flags: u64,
        ) -> Result<()>;
        fn to_val(self: Pin<&mut Tree>, node: usize, val: csubstr, more_flags: u64) -> Result<()>;
        fn to_stream(self: Pin<&mut Tree>, node: usize, more_flags: u64) -> Result<()>;
        fn to_map(self: Pin<&mut Tree>, node: usize, more_flags: u64) -> Result<()>;
        fn to_seq(self: Pin<&mut Tree>, node: usize, more_flags: u64) -> Result<()>;
        fn to_doc(self: Pin<&mut Tree>, node: usize, more_flags: u64) -> Result<()>;

        fn set_key_tag(self: Pin<&mut Tree>, node: usize, tag: csubstr) -> Result<()>;
        fn set_key_anchor(self: Pin<&mut Tree>, node: usize, anchor: csubstr) -> Result<()>;
        fn set_val_anchor(self: Pin<&mut Tree>, node: usize, anchor: csubstr) -> Result<()>;
        fn set_key_ref(self: Pin<&mut Tree>, node: usize, refr: csubstr) -> Result<()>;
        fn set_val_ref(self: Pin<&mut Tree>, node: usize, refr: csubstr) -> Result<()>;

        fn _set_flags(self: Pin<&mut Tree>, node: usize, flags: u64) -> Result<()>;
        fn _set_key(self: Pin<&mut Tree>, node: usize, key: csubstr, more_flags: u64)
            -> Result<()>;
        fn _set_val(self: Pin<&mut Tree>, node: usize, val: csubstr, more_flags: u64)
            -> Result<()>;
        fn _clear(self: Pin<&mut Tree>, node: usize) -> Result<()>;
        fn _clear_key(self: Pin<&mut Tree>, node: usize) -> Result<()>;
        fn _clear_val(self: Pin<&mut Tree>, node: usize) -> Result<()>;

        fn set_val_tag(self: Pin<&mut Tree>, node: usize, tag: csubstr) -> Result<()>;
        fn rem_key_anchor(self: Pin<&mut Tree>, node: usize) -> Result<()>;
        fn rem_val_anchor(self: Pin<&mut Tree>, node: usize) -> Result<()>;
        fn rem_key_ref(self: Pin<&mut Tree>, node: usize) -> Result<()>;
        fn rem_val_ref(self: Pin<&mut Tree>, node: usize) -> Result<()>;
        fn rem_anchor_ref(self: Pin<&mut Tree>, node: usize) -> Result<()>;

        fn insert_child(self: Pin<&mut Tree>, parent: usize, after: usize) -> Result<usize>;
        fn prepend_child(self: Pin<&mut Tree>, parent: usize) -> Result<usize>;
        fn append_child(self: Pin<&mut Tree>, parent: usize) -> Result<usize>;

        fn insert_sibling(self: Pin<&mut Tree>, node: usize, after: usize) -> Result<usize>;
        fn prepend_sibling(self: Pin<&mut Tree>, node: usize) -> Result<usize>;
        fn append_sibling(self: Pin<&mut Tree>, node: usize) -> Result<usize>;

        /// remove an entire branch at once: ie remove the children and the node
        /// itself
        fn remove(self: Pin<&mut Tree>, node: usize) -> Result<()>;
        /// remove all the node's children, but keep the node itself
        fn remove_children(self: Pin<&mut Tree>, node: usize) -> Result<()>;

        fn change_type(self: Pin<&mut Tree>, node: usize, new_type: u64) -> Result<bool>;

        fn reorder(self: Pin<&mut Tree>) -> Result<()>;

        /// recursively duplicate the node
        fn duplicate(
            self: Pin<&mut Tree>,
            node: usize,
            new_parent: usize,
            after: usize,
        ) -> Result<usize>;
        /// recursively duplicate a node from a different tree
        #[cxx_name = "duplicate"]
        unsafe fn duplicate_from_tree(
            self: Pin<&mut Tree>,
            src: *const Tree,
            node: usize,
            new_parent: usize,
            after: usize,
        ) -> Result<usize>;

        /// recursively duplicate the node's children (but not the node)
        fn duplicate_children(
            self: Pin<&mut Tree>,
            node: usize,
            parent: usize,
            after: usize,
        ) -> Result<usize>;
        /// recursively duplicate the node's children (but not the node), where
        /// the node is from a different tree
        #[cxx_name = "duplicate_children"]
        unsafe fn duplicate_children_from_tree(
            self: Pin<&mut Tree>,
            src: *const Tree,
            node: usize,
            parent: usize,
            after: usize,
        ) -> Result<usize>;

        fn duplicate_contents(self: Pin<&mut Tree>, node: usize, loc: usize) -> Result<()>;
        #[cxx_name = "duplicate_contents"]
        unsafe fn duplicate_contents_from_tree(
            self: Pin<&mut Tree>,
            tree: *const Tree,
            node: usize,
            loc: usize,
        ) -> Result<()>;

        /// duplicate the node's children (but not the node) in a new parent,
        /// but omit repetitions where a duplicated node has the same
        /// key (in maps) or value (in seqs). If one of the duplicated
        /// children has the same key (in maps) or value (in seqs) as
        /// one of the parent's children, the one that is placed closest
        /// to the end will prevail.
        fn duplicate_children_no_rep(
            self: Pin<&mut Tree>,
            node: usize,
            parent: usize,
            after: usize,
        ) -> Result<usize>;

        fn copy_to_arena(self: Pin<&mut Tree>, s: csubstr) -> Result<substr>;

        fn emit(tree: &Tree, buffer: substr, error_on_excess: bool) -> Result<substr>;
        fn emit_json(tree: &Tree, buffer: substr, error_on_excess: bool) -> Result<substr>;
    }

    #[namespace = "shimmy"]
    unsafe extern "C++" {
        include!("ryml/include/shim.h");
        fn new_tree() -> UniquePtr<Tree>;
        fn clone_tree(tree: &Tree) -> UniquePtr<Tree>;
        fn parse(text: &str) -> Result<UniquePtr<Tree>>;
        unsafe fn parse_in_place(text: *mut c_char, len: usize) -> Result<UniquePtr<Tree>>;
        #[cfg(all(not(windows), feature = "std"))]
        fn emit_to_rwriter(tree: &Tree, writer: Box<RWriter>, json: bool) -> Result<usize>;

        fn tree_node_type(tree: &Tree, node: usize) -> Result<NodeType>;

        // /** change the node's position in the parent */
        fn move_node(tree: Pin<&mut Tree>, node: usize, after: usize) -> Result<()>;

        // /** change the node's parent and position */
        fn move_node_to_new_parent(
            tree: Pin<&mut Tree>,
            node: usize,
            new_parent: usize,
            after: usize,
        ) -> Result<()>;
        // /** change the node's parent and position */
        fn move_node_from_tree(
            tree: Pin<&mut Tree>,
            src: Pin<&mut Tree>,
            node: usize,
            new_parent: usize,
            after: usize,
        ) -> Result<usize>;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn val_types() {
        assert!(NodeType::Val.is_val());
        assert!(NodeType::Map.is_map());
        assert!(NodeType::ValRef.is_ref());
    }

    static SRC: &str = r#"  HELLO: a
foo: |
           b
bar:   "c"
baz: !str64 d
seq: [0 ,  1, 2, 3]"#;

    #[test]
    fn check_tree() -> Result<(), cxx::Exception> {
        let tree = ffi::parse(SRC)?;
        assert_eq!(tree.size(), 10);
        assert_eq!(tree.root_id()?, 0);
        assert_eq!(tree.first_child(0)?, 1);
        assert_eq!(tree.next_sibling(1)?, 2);
        for i in 0..tree.num_children(5)? {
            let child = tree.child(5, i)?;
            println!("{}", child);
            assert_eq!(tree.parent(child)?, 5);
            println!("{}", tree.val(child)?.deref());
        }
        assert_eq!(tree.find_child(0, &("foo".into()))?, 2);
        let baz_val = tree.find_child(0, &("baz".into()))?;
        assert_eq!(tree.val(baz_val)?.deref(), "d");
        assert!(tree.has_val_tag(baz_val)?);
        println!("Baz value tag: {}", tree.val_tag(baz_val)?.deref());
        println!("{}", tree.num_children(0)?);
        assert_eq!(tree.last_sibling(1)?, 5);
        Ok(())
    }

    #[test]
    fn mut_tree() -> Result<(), cxx::Exception> {
        let mut src = SRC.to_string();
        let mut tree = unsafe { ffi::parse_in_place(src.as_mut_ptr() as *mut i8, src.len())? };
        let bar_val = tree.find_child(0, &("bar".into()))?;
        tree.pin_mut()._set_val(bar_val, "r353".into(), 0)?;
        println!("{}", &src);
        tree.pin_mut().resolve()?;
        let mut buf = vec![0; src.len() * 2];
        ffi::emit(
            &tree,
            Substr {
                ptr: buf.as_mut_ptr(),
                len: buf.len(),
            },
            true,
        )?;
        println!("{}", unsafe { core::str::from_utf8_unchecked(&buf) });
        ffi::emit_json(
            &tree,
            Substr {
                ptr: buf.as_mut_ptr(),
                len: buf.len(),
            },
            true,
        )
        .expect_err("JSON doesn't support tags");
        assert_eq!(SRC, src);
        Ok(())
    }

    #[test]
    fn test_exceptions() -> Result<(), cxx::Exception> {
        let tree = ffi::parse(SRC)?;
        tree.is_doc(555).expect_err("is_doc should fail");
        Ok(())
    }

    #[test]
    fn emit_into_buffer() -> Result<(), cxx::Exception> {
        let tree = ffi::parse(SRC)?;
        let mut buf = vec![0; SRC.len() * 2];
        ffi::emit(
            &tree,
            Substr {
                ptr: buf.as_mut_ptr(),
                len: buf.len(),
            },
            true,
        )?;
        println!("{}", unsafe { core::str::from_utf8_unchecked(&buf) });
        Ok(())
    }
}
