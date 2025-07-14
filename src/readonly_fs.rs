//! A Wrapper for a [`UniFs`] filesystem, making it read-only.

use std::{io::ErrorKind, path::Path};

use crate::{
    traits::{dir_builder::UniDirBuilder, open_options::UniOpenOptions},
    Result, UniDirEntry, UniFs, UniMetadata, UniPermissions,
};

/// The `ReadonlyFs` struct provides a read-only filesystem interface that wraps around another filesystem implementation.
pub struct ReadonlyFs<FS: UniFs>(FS);

pub struct ReadonlyMetadata<T: UniMetadata>(T);

#[derive(PartialEq, Eq)]
pub struct ReadonlyPermissions;

pub struct ReadonlyOpenOptions<T: UniOpenOptions>(T);

pub struct ReadonlyDirEntry<T: UniDirEntry>(T);

pub struct ReadonlyReadDir<FS: UniFs>(FS::ReadDir);

pub struct ReadonlyDirBuilder<T: UniDirBuilder>(T);

fn error(msg: &str) -> std::io::Error {
    std::io::Error::new(ErrorKind::ReadOnlyFilesystem, msg)
}

impl<FS: UniFs> ReadonlyFs<FS> {
    pub fn new(fs: FS) -> Self {
        ReadonlyFs(fs)
    }
}

impl<FS> UniFs for ReadonlyFs<FS>
where
    FS: UniFs,
{
    type DirEntry = ReadonlyDirEntry<FS::DirEntry>;
    type Metadata = ReadonlyMetadata<FS::Metadata>;
    type Permissions = ReadonlyPermissions;
    type ReadDir = ReadonlyReadDir<FS>;
    type File = FS::File;
    type OpenOptions = ReadonlyOpenOptions<FS::OpenOptions>;
    type DirBuilder = ReadonlyDirBuilder<FS::DirBuilder>;

    fn canonicalize<P: AsRef<Path>>(&self, path: P) -> crate::Result<std::path::PathBuf> {
        self.0.canonicalize(path)
    }

    /// Attempts to copy a file from one path to another.
    ///
    /// This function will return an error indicating that the filesystem is read-only.
    fn copy<P: AsRef<Path>, Q: AsRef<Path>>(&self, _from: P, _to: Q) -> crate::Result<u64> {
        Err(error("Cannot copy files in a read-only filesystem"))
    }

    /// Creates a new directory at the specified path.
    ///
    /// This function will return an error indicating that the filesystem is read-only.
    fn create_dir<P: AsRef<Path>>(&self, _path: P) -> crate::Result<()> {
        Err(error("Cannot create directories in a read-only filesystem"))
    }

    /// Recursively creates a directory and all of its parent components if they are missing.
    ///
    /// This function will return an error indicating that the filesystem is read-only.
    fn create_dir_all<P: AsRef<Path>>(&self, _path: P) -> crate::Result<()> {
        Err(error("Cannot create directories in a read-only filesystem"))
    }

    fn exists<P: AsRef<Path>>(&self, path: P) -> crate::Result<bool> {
        self.0.exists(path)
    }

    /// Attempts to create a hard link to an existing file.
    ///
    /// This function will return an error indicating that the filesystem is read-only.
    fn hard_link<P: AsRef<Path>, Q: AsRef<Path>>(
        &self,
        _original: P,
        _link: Q,
    ) -> crate::Result<()> {
        Err(error("Cannot create hard links in a read-only filesystem"))
    }

    fn metadata<P: AsRef<Path>>(&self, path: P) -> crate::Result<Self::Metadata> {
        self.0.metadata(path).map(ReadonlyMetadata)
    }

    fn read<P: AsRef<Path>>(&self, path: P) -> crate::Result<Vec<u8>> {
        self.0.read(path)
    }

    fn read_dir<P: AsRef<Path>>(&self, path: P) -> crate::Result<Self::ReadDir> {
        self.0.read_dir(path).map(ReadonlyReadDir)
    }

    fn read_link<P: AsRef<Path>>(&self, path: P) -> crate::Result<std::path::PathBuf> {
        self.0.read_link(path)
    }

    fn read_to_string<P: AsRef<Path>>(&self, path: P) -> crate::Result<String> {
        self.0.read_to_string(path)
    }

    /// Attempts to remove a directory at the specified path.
    ///
    /// This function will return an error indicating that the filesystem is read-only.
    fn remove_dir<P: AsRef<Path>>(&self, _path: P) -> crate::Result<()> {
        Err(error("Cannot remove directories in a read-only filesystem"))
    }

    /// Attempts to remove a directory and all of its contents recursively.
    ///
    /// This function will return an error indicating that the filesystem is read-only.
    fn remove_dir_all<P: AsRef<Path>>(&self, _path: P) -> crate::Result<()> {
        Err(error("Cannot remove directories in a read-only filesystem"))
    }

    /// Attempts to remove a file at the specified path.
    ///
    /// This function will return an error indicating that the filesystem is read-only.
    fn remove_file<P: AsRef<Path>>(&self, _path: P) -> crate::Result<()> {
        Err(error("Cannot remove files in a read-only filesystem"))
    }

    /// Attempts to rename a file or directory.
    ///
    /// This function will return an error indicating that the filesystem is read-only.
    fn rename<P: AsRef<Path>, Q: AsRef<Path>>(&self, _from: P, _to: Q) -> crate::Result<()> {
        Err(error("Cannot rename files in a read-only filesystem"))
    }

    /// Changes the permissions of a file or directory.
    ///
    /// This function will return an error indicating that the filesystem is read-only.
    fn set_permissions<P: AsRef<Path>>(
        &self,
        _path: P,
        _perm: Self::Permissions,
    ) -> crate::Result<()> {
        Err(error("Cannot set permissions in a read-only filesystem"))
    }

    fn symlink_metadata<P: AsRef<Path>>(&self, path: P) -> crate::Result<Self::Metadata> {
        self.0.symlink_metadata(path).map(ReadonlyMetadata)
    }

    /// Writes a slice as the entire contents of a file.
    ///
    /// This function will return an error indicating that the filesystem is read-only.
    fn write<P: AsRef<Path>, C: AsRef<[u8]>>(&self, _path: P, _contents: C) -> crate::Result<()> {
        Err(error("Cannot write to files in a read-only filesystem"))
    }

    fn open_file<P: AsRef<Path>>(&self, path: P) -> crate::Result<Self::File> {
        self.0.open_file(path)
    }

    fn create_file<P: AsRef<Path>>(&self, _path: P) -> crate::Result<Self::File> {
        Err(error("Cannot create file in a read-only filesystem"))
    }

    fn new_openoptions(&self) -> Self::OpenOptions {
        ReadonlyOpenOptions(self.0.new_openoptions())
    }

    fn new_dirbuilder(&self) -> Self::DirBuilder {
        ReadonlyDirBuilder(self.0.new_dirbuilder())
    }
}

impl<T: UniMetadata> UniMetadata for ReadonlyMetadata<T> {
    type Permissions = ReadonlyPermissions;
    type FileType = T::FileType;

    fn accessed(&self) -> crate::Result<std::time::SystemTime> {
        self.0.accessed()
    }

    fn created(&self) -> crate::Result<std::time::SystemTime> {
        self.0.created()
    }

    fn file_type(&self) -> Self::FileType {
        self.0.file_type()
    }

    fn is_dir(&self) -> bool {
        self.0.is_dir()
    }

    fn is_file(&self) -> bool {
        self.0.is_file()
    }

    fn is_symlink(&self) -> bool {
        self.0.is_symlink()
    }

    fn len(&self) -> u64 {
        self.0.len()
    }

    fn modified(&self) -> crate::Result<std::time::SystemTime> {
        self.0.modified()
    }

    fn permissions(&self) -> Self::Permissions {
        ReadonlyPermissions
    }
}

impl UniPermissions for ReadonlyPermissions {
    fn readonly(&self) -> bool {
        true
    }

    fn set_readonly(&mut self, _readonly: bool) {}
}

impl<T: UniOpenOptions> UniOpenOptions for ReadonlyOpenOptions<T> {
    type File = T::File;

    fn append(&mut self, _append: bool) -> &mut Self {
        self
    }

    fn create(&mut self, _create: bool) -> &mut Self {
        self
    }

    fn create_new(&mut self, _create_new: bool) -> &mut Self {
        self
    }

    fn open<P: AsRef<Path>>(&self, path: P) -> crate::Result<Self::File> {
        self.0.open(path)
    }

    fn read(&mut self, read: bool) -> &mut Self {
        self.0.read(read);
        self
    }

    fn truncate(&mut self, _truncate: bool) -> &mut Self {
        self
    }

    fn write(&mut self, _write: bool) -> &mut Self {
        self
    }
}

impl<T: UniDirEntry> UniDirEntry for ReadonlyDirEntry<T> {
    type FileType = T::FileType;
    type Metadata = ReadonlyMetadata<T::Metadata>;

    fn file_name(&self) -> std::ffi::OsString {
        self.0.file_name()
    }

    fn file_type(&self) -> Result<Self::FileType> {
        self.0.file_type()
    }

    fn metadata(&self) -> Result<Self::Metadata> {
        self.0.metadata().map(ReadonlyMetadata)
    }

    fn path(&self) -> std::path::PathBuf {
        self.0.path()
    }
}

impl<FS: UniFs> Iterator for ReadonlyReadDir<FS> {
    type Item = Result<ReadonlyDirEntry<FS::DirEntry>>;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|res| res.map(ReadonlyDirEntry))
    }
}

impl<T: UniDirBuilder> UniDirBuilder for ReadonlyDirBuilder<T> {
    fn create<P: AsRef<Path>>(&self, _path: P) -> Result<()> {
        Err(error("Cannot create directory in a read-only filesystem"))
    }

    fn recursive(&mut self, recursive: bool) -> &mut Self {
        self.0.recursive(recursive);
        self
    }
}

impl<T: UniFs> From<T> for ReadonlyFs<T> {
    fn from(fs: T) -> Self {
        ReadonlyFs::new(fs)
    }
}
