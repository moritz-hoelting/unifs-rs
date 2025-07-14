use std::path::Path;

use crate::{Result, UniFile};

/// Options and flags which can be used to configure how a file is opened.
///
/// This builder exposes the ability to configure how a File is opened and what
/// operations are permitted on the open file. The [`crate::UniFs::open_file`] and
/// [`crate::UniFs::create_file`] methods are aliases for commonly used options using this builder.
pub trait UniOpenOptions {
    /// The File type of the OpenOptions
    type File: UniFile;

    fn read(&mut self, read: bool) -> &mut Self;

    fn write(&mut self, write: bool) -> &mut Self;

    fn append(&mut self, append: bool) -> &mut Self;

    fn truncate(&mut self, truncate: bool) -> &mut Self;

    fn create(&mut self, create: bool) -> &mut Self;

    fn create_new(&mut self, create_new: bool) -> &mut Self;

    fn open<P: AsRef<Path>>(&self, path: P) -> Result<Self::File>;
}
