#![feature(core_ffi_c)]
use std::marker::PhantomData;
use thiserror::Error;
mod inner;

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
    data: TreeData<'a>,
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
            data: TreeData::Owned,
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
            data: TreeData::Borrowed(PhantomData),
        })
    }

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
}
