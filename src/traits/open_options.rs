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

    /// Sets the option for read access.
    ///
    /// This option, when true, will indicate that the file should be `read`-able if opened.
    ///
    /// This function mirrors the [`std::fs::OpenOptions::read`] function.
    fn read(&mut self, read: bool) -> &mut Self;

    /// Sets the option for write access.
    ///
    /// This option, when true, will indicate that the file should be `write`-able if opened.
    ///
    /// If the file already exists, any write calls on it will overwrite its contents, without truncating it.
    ///
    /// This function mirrors the [`std::fs::OpenOptions::write`] function.
    fn write(&mut self, write: bool) -> &mut Self;

    /// Sets the option for the append mode.
    ///
    /// This option, when true, means that writes will append to a file instead of overwriting previous contents.
    /// Note that setting `.write(true).append(true)` has the same effect as setting only `.append(true)`.
    ///
    /// This function mirrors the [`std::fs::OpenOptions::append`] function.
    fn append(&mut self, append: bool) -> &mut Self;

    /// Sets the option for truncating a previous file.
    ///
    /// If a file is successfully opened with this option set to true, it will truncate the file to 0 length if it already exists.
    ///
    /// The file must be opened with write access for truncate to work.
    ///
    /// This function mirrors the [`std::fs::OpenOptions::truncate`] function.
    fn truncate(&mut self, truncate: bool) -> &mut Self;

    /// Sets the option to create a new file, or open it if it already exists.
    ///
    /// In order for the file to be created, [`UniOpenOptions::write`] or [`UniOpenOptions::append`] access must be used.
    ///
    /// See also [`crate::UniFs::write()`] for a simple function to create a file with some given data.
    ///
    /// This function mirrors the [`std::fs::OpenOptions::create`] function.
    fn create(&mut self, create: bool) -> &mut Self;

    /// Sets the option to create a new file, failing if it already exists.
    ///
    /// No file is allowed to exist at the target location, also no (dangling) symlink. In this way, if the call succeeds,
    /// the file returned is guaranteed to be new. If a file exists at the target location, creating a new file will fail
    /// with [`std::io::ErrorKind::AlreadyExists`] or another error based on the situation. See [`UniOpenOptions::open`]
    /// for a non-exhaustive list of likely errors.
    ///
    /// This function mirrors the [`std::fs::OpenOptions::create_new`] function.
    fn create_new(&mut self, create_new: bool) -> &mut Self;

    /// Opens a file at `path` with the options specified by `self`.
    ///
    /// This function mirrors the [`std::fs::OpenOptions::open`] function.
    fn open<P: AsRef<Path>>(&self, path: P) -> Result<Self::File>;
}
