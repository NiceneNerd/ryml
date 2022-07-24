use std::ops::Deref;

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
        unsafe { std::str::from_utf8_unchecked(std::slice::from_raw_parts(self.ptr, self.len)) }
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

impl std::ops::Deref for Substr {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        unsafe { std::str::from_utf8_unchecked(std::slice::from_raw_parts(self.ptr, self.len)) }
    }
}

impl std::ops::DerefMut for Substr {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { std::mem::transmute(*self) }
    }
}

unsafe impl cxx::ExternType for Substr {
    type Id = cxx::type_id!("c4::substr");
    type Kind = cxx::kind::Trivial;
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u64)]
pub enum NodeType {
    /// no type is set
    NoType = 0,
    /// a leaf node, has a (possibly empty) value
    Val = (1 << 0),
    /// is member of a map, must have non-empty key
    Key = (1 << 1),
    /// a map: a parent of keyvals
    Map = (1 << 2),
    /// a seq: a parent of vals
    Seq = (1 << 3),
    /// a document
    Doc = (1 << 4),
    /// a stream: a seq of docs
    Stream = (1 << 5) | (1 << 3),
    /// a *reference: the key references an &anchor
    KeyRef = (1 << 6),
    /// a *reference: the val references an &anchor
    ValRef = (1 << 7),
    /// the key has an &anchor
    KeyAnch = (1 << 8),
    /// the val has an &anchor
    ValAnch = (1 << 9),
    /// the key has an explicit tag/type
    KeyTag = (1 << 10),
    /// the val has an explicit tag/type
    ValTag = (1 << 11),
}

unsafe impl cxx::ExternType for NodeType {
    type Id = cxx::type_id!("c4::yml::NodeType");
    type Kind = cxx::kind::Trivial;
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NodeScalar {
    tag: CSubstr,
    scalar: CSubstr,
    anchor: CSubstr,
}

unsafe impl cxx::ExternType for NodeScalar {
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

trait WriteSeek: std::io::Write + std::io::Seek {}
impl<T: std::io::Write + std::io::Seek> WriteSeek for T {}

pub struct RWriter {
    writer: Box<dyn WriteSeek>,
}

impl RWriter {
    #[inline(always)]
    fn _get(&mut self, _error_on_excess: bool) -> std::io::Result<Substr> {
        Ok(Substr {
            ptr: std::ptr::null_mut(),
            len: self.writer.stream_position()? as usize,
        })
    }

    #[inline(always)]
    fn _do_write(&mut self, s: CSubstr) -> std::io::Result<()> {
        if s.is_empty() {
            return Ok(());
        }
        self.writer.write_all(s.as_bytes())?;
        Ok(())
    }

    #[inline(always)]
    fn _do_write_slice(&mut self, slice: &[core::ffi::c_char]) -> std::io::Result<()> {
        for c in slice {
            self.writer.write_all(&[*c as u8])?;
        }
        Ok(())
    }

    #[inline(always)]
    fn _do_write_char(&mut self, c: core::ffi::c_char) -> std::io::Result<()> {
        self.writer.write_all(&[c as u8])?;
        Ok(())
    }

    #[inline(always)]
    fn _do_write_repc(&mut self, recp: RepC) -> std::io::Result<()> {
        for _ in 0..recp.num_times {
            self.writer.write_all(&[recp.char as u8])?;
        }
        Ok(())
    }
}

#[cxx::bridge]
pub(crate) mod ffi {
    #[namespace = "c4"]
    unsafe extern "C++" {
        type csubstr = super::CSubstr;
        type substr = super::Substr;
        #[namespace = "c4::yml"]
        type RepC = super::RepC;
    }
    #[namespace = "shimmy"]
    extern "Rust" {
        type RWriter;
        fn _get(self: &mut RWriter, error_on_excess: bool) -> Result<substr>;
        fn _do_write(self: &mut RWriter, s: csubstr) -> Result<()>;
        fn _do_write_slice(self: &mut RWriter, slice: &[c_char]) -> Result<()>;
        fn _do_write_char(self: &mut RWriter, c: c_char) -> Result<()>;
        fn _do_write_repc(self: &mut RWriter, recp: RepC) -> Result<()>;
    }
    #[namespace = "c4::yml"]
    unsafe extern "C++" {
        include!("ryml/include/ryml.h");
        /// NodeType and NodeType_e
        type NodeType = super::NodeType;
        type NodeScalar = super::NodeScalar;
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
        fn size(self: &Tree) -> usize;
        fn capacity(self: &Tree) -> usize;
        fn slack(self: &Tree) -> Result<usize>;

        fn arena_size(self: &Tree) -> usize;
        fn arena_capacity(self: &Tree) -> usize;
        fn arena_slack(self: &Tree) -> Result<usize>;

        fn resolve(self: Pin<&mut Tree>) -> Result<()>;

        fn type_str(self: &Tree, node: usize) -> *const c_char;

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
        fn empty(self: &Tree, node: usize) -> Result<bool>;
        fn has_anchor(self: &Tree, node: usize, a: csubstr) -> Result<bool>;

        fn has_parent(self: &Tree, node: usize) -> Result<bool>;
        fn has_child(self: &Tree, node: usize, key: csubstr) -> Result<bool>;
        fn has_children(self: &Tree, node: usize) -> Result<bool>;
        fn has_sibling(self: &Tree, node: usize, key: csubstr) -> Result<bool>;
        fn has_siblings(self: &Tree, node: usize) -> Result<bool>;
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
        fn to_map(self: Pin<&mut Tree>, node: usize, key: csubstr, more_flags: u64) -> Result<()>;
        fn to_seq(self: Pin<&mut Tree>, node: usize, key: csubstr, more_flags: u64) -> Result<()>;
        fn to_val(self: Pin<&mut Tree>, node: usize, val: csubstr, more_flags: u64) -> Result<()>;
        fn to_stream(self: Pin<&mut Tree>, node: usize, more_flags: u64) -> Result<()>;
        #[cxx_name = "to_map"]
        fn to_map_from_node(self: Pin<&mut Tree>, node: usize, more_flags: u64) -> Result<()>;
        #[cxx_name = "to_seq"]
        fn to_seq_from_node(self: Pin<&mut Tree>, node: usize, more_flags: u64) -> Result<()>;
        fn to_doc(self: Pin<&mut Tree>, node: usize, more_flags: u64) -> Result<()>;

        fn set_key_tag(self: Pin<&mut Tree>, node: usize, tag: csubstr) -> Result<()>;
        fn set_key_anchor(self: Pin<&mut Tree>, node: usize, anchor: csubstr) -> Result<()>;
        fn set_val_anchor(self: Pin<&mut Tree>, node: usize, anchor: csubstr) -> Result<()>;
        fn set_key_ref(self: Pin<&mut Tree>, node: usize, refr: csubstr) -> Result<()>;
        fn set_val_ref(self: Pin<&mut Tree>, node: usize, refr: csubstr) -> Result<()>;

        fn _set_key(self: Pin<&mut Tree>, node: usize, key: csubstr, more_flags: u64)
            -> Result<()>;
        fn _set_val(self: Pin<&mut Tree>, node: usize, val: csubstr, more_flags: u64)
            -> Result<()>;

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

        /// remove an entire branch at once: ie remove the children and the node itself
        fn remove(self: Pin<&mut Tree>, node: usize) -> Result<()>;
        /// remove all the node's children, but keep the node itself
        fn remove_children(self: Pin<&mut Tree>, node: usize) -> Result<()>;

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

        /// duplicate the node's children (but not the node) in a new parent, but
        /// omit repetitions where a duplicated node has the same key (in maps) or
        /// value (in seqs). If one of the duplicated children has the same key
        /// (in maps) or value (in seqs) as one of the parent's children, the one
        /// that is placed closest to the end will prevail.
        fn duplicate_children_no_rep(
            self: Pin<&mut Tree>,
            node: usize,
            parent: usize,
            after: usize,
        ) -> Result<usize>;

        fn emit(tree: &Tree, buffer: substr, error_on_excess: bool) -> Result<substr>;
        fn emit_json(tree: &Tree, buffer: substr, error_on_excess: bool) -> Result<substr>;
    }

    #[namespace = "shimmy"]
    unsafe extern "C++" {
        include!("ryml/include/shim.h");
        fn parse(text: &str) -> Result<UniquePtr<Tree>>;
        unsafe fn parse_in_place(text: *mut c_char, len: usize) -> Result<UniquePtr<Tree>>;
        fn emit_to_rwriter(tree: &Tree, writer: Box<RWriter>) -> Result<usize>;

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

        fn init_ryml_once();
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
        ffi::init_ryml_once();
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
        println!("{}", unsafe { std::str::from_utf8_unchecked(&buf) });
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
        ffi::init_ryml_once();
        let tree = ffi::parse(SRC)?;
        tree.is_doc(555).expect_err("is_doc should fail");
        Ok(())
    }

    #[test]
    fn emit_into_buffer() -> Result<(), cxx::Exception> {
        ffi::init_ryml_once();
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
        println!("{}", unsafe { std::str::from_utf8_unchecked(&buf) });
        Ok(())
    }
}
