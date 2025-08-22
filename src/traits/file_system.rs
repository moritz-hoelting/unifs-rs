use std::{
    io::Write as _,
    path::{Path, PathBuf},
};

use crate::{
    traits::{dir_builder::UniDirBuilder, open_options::UniOpenOptions},
    Result, UniDirEntry, UniFile, UniMetadata, UniPermissions,
};

/// A trait that represents a filesystem that can be used to perform
/// various file system operations.
///
/// This trait is designed to be a universal filesystem interface, allowing
/// for operations similar to those found in the standard library's [`std::fs`] module.
pub trait UniFs
where
    for<'a> &'a Self: UniFs,
{
    /// The metadata type returned by this filesystem.
    /// This type must implement the [`UniMetadata`] trait.
    type Metadata: UniMetadata;

    /// An iterator over the entries within a directory.
    type ReadDir: Iterator<Item = Result<Self::DirEntry>>;

    /// The type of directory entries returned by this filesystem.
    /// This type must implement the [`UniDirEntry`] trait.
    type DirEntry: UniDirEntry<Metadata = Self::Metadata>;

    /// The type of permissions used by this filesystem.
    /// This type must implement the [`UniPermissions`] trait.
    type Permissions: UniPermissions;

    /// The type of file this file system uses.
    /// This type must implement the [`UniFile`] trait.
    type File: UniFile;

    /// The type of OpenOptions this file system uses.
    /// This type must implement the [`UniOpenOptions`] trait.
    type OpenOptions: UniOpenOptions<File = Self::File>;

    /// The type of DirBuilder this file system uses.
    /// This type must implement the [`UniDirBuilder`] trait.
    type DirBuilder: UniDirBuilder;

    /// Returns the canonical, absolute form of a path with all intermediate
    /// components normalized and symbolic links resolved.
    ///
    /// This function mirrors the [`std::fs::canonicalize`] function.
    fn canonicalize<P: AsRef<Path>>(&self, path: P) -> Result<PathBuf>;

    /// Copies the contents of one file to another. This function will also
    /// copy the permission bits of the original file to the destination file.
    ///
    /// This function will **overwrite** the contents of `to`.
    ///
    /// On success, the total number of bytes copied is returned and it is equal to
    /// the length of the `to` file as reported by `metadata`.
    ///
    /// This function mirrors the [`std::fs::copy`] function.
    fn copy<P: AsRef<Path>, Q: AsRef<Path>>(&self, from: P, to: Q) -> Result<u64>;

    /// Creates a new, empty directory at the provided path
    ///
    /// This function mirrors the [`std::fs::create_dir`] function.
    fn create_dir<P: AsRef<Path>>(&self, path: P) -> Result<()>;

    /// Recursively create a directory and all of its parent components if they
    /// are missing.
    ///
    /// If this function returns an error, some of the parent components might have
    /// been created already.
    ///
    /// This function mirrors the [`std::fs::create_dir_all`] function.
    fn create_dir_all<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        self.new_dirbuilder().recursive(true).create(path.as_ref())
    }

    /// Returns `Ok(true)` if the path points at an existing entity.
    ///
    /// This function will traverse symbolic links to query information about the
    /// destination file. In case of broken symbolic links this will return `Ok(false)`.
    ///
    /// As opposed to the [`Path::exists`] method, this will only return `Ok(true)` or `Ok(false)`
    /// if the path was _verified_ to exist or not exist. If its existence can neither be confirmed
    /// nor denied, an `Err(_)` will be propagated instead. This can be the case if e.g. listing
    /// permission is denied on one of the parent directories.
    ///
    /// This function mirrors the [`std::fs::exists`] function.
    fn exists<P: AsRef<Path>>(&self, path: P) -> Result<bool>;

    /// Creates a new hard link on the filesystem.
    ///
    /// The `link` path will be a link pointing to the `original` path. Note that
    /// systems often require these two paths to both be located on the same
    /// filesystem.
    ///
    /// This function mirrors the [`std::fs::hard_link`] function.
    fn hard_link<P: AsRef<Path>, Q: AsRef<Path>>(&self, original: P, link: Q) -> Result<()>;

    /// Given a path, queries the file system to get information about a file,
    /// directory, etc.
    ///
    /// This function will traverse symbolic links to query information about the
    /// destination file.
    ///
    /// This function mirrors the [`std::fs::metadata`] function.
    fn metadata<P: AsRef<Path>>(&self, path: P) -> Result<Self::Metadata>;

    /// Reads the entire contents of a file into a bytes vector.
    ///
    /// This function mirrors the [`std::fs::read`] function.
    fn read<P: AsRef<Path>>(&self, path: P) -> Result<Vec<u8>>;

    /// Returns an iterator over the entries within a directory.
    ///
    /// The iterator will yield instances of <code>[std::io::Result]<[Self::DirEntry]></code>.
    /// New errors may be encountered after an iterator is initially constructed.
    /// Entries for the current and parent directories (typically `.` and `..`) are
    /// skipped.
    ///
    /// This function mirrors the [`std::fs::read_dir`] function.
    fn read_dir<P: AsRef<Path>>(&self, path: P) -> Result<Self::ReadDir>;

    /// Reads a symbolic link, returning the file that the link points to.
    ///
    /// This function mirrors the [`std::fs::read_link`] function.
    fn read_link<P: AsRef<Path>>(&self, path: P) -> Result<PathBuf>;

    /// Reads the entire contents of a file into a string.
    ///
    /// This function mirrors the [`std::fs::read_to_string`] function.
    fn read_to_string<P: AsRef<Path>>(&self, path: P) -> Result<String>;

    /// Removes an empty directory.
    ///
    /// If you want to remove a directory that is not empty, as well as all
    /// of its contents recursively, consider using [`UniFs::remove_dir_all`]
    /// instead.
    ///
    /// This function mirrors the [`std::fs::remove_dir`] function.
    fn remove_dir<P: AsRef<Path>>(&self, path: P) -> Result<()>;

    /// Removes a directory at this path, after removing all its contents. Use
    /// carefully!
    ///
    /// This function mirrors the [`std::fs::remove_dir_all`] function.
    fn remove_dir_all<P: AsRef<Path>>(&self, path: P) -> Result<()>;

    /// Removes a file from the filesystem.
    ///
    /// This function mirrors the [`std::fs::remove_file`] function.
    fn remove_file<P: AsRef<Path>>(&self, path: P) -> Result<()>;

    /// Renames a file or directory to a new name, replacing the original file if
    /// `to` already exists.
    ///
    /// This function mirrors the [`std::fs::rename`] function.
    fn rename<P: AsRef<Path>, Q: AsRef<Path>>(&self, from: P, to: Q) -> Result<()>;

    /// Changes the permissions found on a file or a directory.
    ///
    /// This function mirrors the [`std::fs::set_permissions`] function.
    fn set_permissions<P: AsRef<Path>>(&self, path: P, perm: Self::Permissions) -> Result<()>;

    /// Queries the metadata about a file without following symlinks.
    ///
    /// This function mirrors the [`std::fs::symlink_metadata`] function.
    fn symlink_metadata<P: AsRef<Path>>(&self, path: P) -> Result<Self::Metadata>;

    /// Writes a slice as the entire contents of a file.
    ///
    /// This function will create a file if it does not exist,
    /// and will entirely replace its contents if it does.
    ///
    /// This function mirrors the [`std::fs::write`] function.
    fn write<P: AsRef<Path>, C: AsRef<[u8]>>(&self, path: P, contents: C) -> Result<()> {
        self.new_openoptions()
            .write(true)
            .create(true)
            .truncate(true)
            .open(path.as_ref())?
            .write_all(contents.as_ref())
    }

    /// Attempts to open a file in read-only mode.
    ///
    ///See the [`UniOpenOptions::open`] method for more details.
    ///
    /// If you only need to read the entire file contents, consider [`UniFs::read()`] or [`UniFs::read_to_string()`] instead.
    ///
    /// Used instead of [`std::fs::File::open`] to allow using the [`UniFs`] trait.
    fn open_file<P: AsRef<Path>>(&self, path: P) -> Result<Self::File> {
        self.new_openoptions().read(true).open(path.as_ref())
    }

    /// Opens a file in write-only mode.
    ///
    /// This function will create a file if it does not exist, and will truncate it if it does.
    ///
    /// Used instead of [`std::fs::File::create`] to allow using the [`UniFs`] trait.
    fn create_file<P: AsRef<Path>>(&self, path: P) -> Result<Self::File> {
        self.new_openoptions()
            .write(true)
            .create(true)
            .truncate(true)
            .open(path.as_ref())
    }

    /// Creates a new file in read-write mode; error if the file exists.
    ///
    /// Used instead of [`std::fs::File::create_new`] to allow using the [`UniFs`] trait.
    fn create_new_file<P: AsRef<Path>>(&self, path: P) -> Result<Self::File> {
        self.new_openoptions()
            .read(true)
            .write(true)
            .create_new(true)
            .open(path.as_ref())
    }

    /// Creates a blank new set of options ready for configuration.
    ///
    /// All options are initially set to false.
    ///
    /// Used instead of [`std::fs::OpenOptions::new`] to allow using the [`UniFs`] trait.
    fn new_openoptions(&self) -> Self::OpenOptions;

    /// Creates a new set of options with default mode/security settings for all platforms and also non-recursive.
    ///
    /// Used instead of [`std::fs::DirBuilder::new`] to allow using the [`UniFs`] trait.
    fn new_dirbuilder(&self) -> Self::DirBuilder;
}

impl<T: UniFs + ?Sized> UniFs for &T {
    type Metadata = T::Metadata;
    type ReadDir = T::ReadDir;
    type DirEntry = T::DirEntry;
    type Permissions = T::Permissions;
    type File = T::File;
    type OpenOptions = T::OpenOptions;
    type DirBuilder = T::DirBuilder;

    fn canonicalize<P: AsRef<Path>>(&self, path: P) -> Result<PathBuf> {
        (**self).canonicalize(path)
    }

    fn copy<P: AsRef<Path>, Q: AsRef<Path>>(&self, from: P, to: Q) -> Result<u64> {
        (**self).copy(from, to)
    }

    fn create_dir<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        (**self).create_dir(path)
    }

    fn create_dir_all<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        (**self).create_dir_all(path)
    }

    fn exists<P: AsRef<Path>>(&self, path: P) -> Result<bool> {
        (**self).exists(path)
    }

    fn hard_link<P: AsRef<Path>, Q: AsRef<Path>>(&self, original: P, link: Q) -> Result<()> {
        (**self).hard_link(original, link)
    }

    fn metadata<P: AsRef<Path>>(&self, path: P) -> Result<Self::Metadata> {
        (**self).metadata(path)
    }

    fn read<P: AsRef<Path>>(&self, path: P) -> Result<Vec<u8>> {
        (**self).read(path)
    }

    fn read_dir<P: AsRef<Path>>(&self, path: P) -> Result<Self::ReadDir> {
        (**self).read_dir(path)
    }

    fn read_link<P: AsRef<Path>>(&self, path: P) -> Result<PathBuf> {
        (**self).read_link(path)
    }

    fn read_to_string<P: AsRef<Path>>(&self, path: P) -> Result<String> {
        (**self).read_to_string(path)
    }

    fn remove_dir<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        (**self).remove_dir(path)
    }

    fn remove_dir_all<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        (**self).remove_dir_all(path)
    }

    fn remove_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        (**self).remove_file(path)
    }

    fn rename<P: AsRef<Path>, Q: AsRef<Path>>(&self, from: P, to: Q) -> Result<()> {
        (**self).rename(from, to)
    }

    fn set_permissions<P: AsRef<Path>>(&self, path: P, perm: Self::Permissions) -> Result<()> {
        (**self).set_permissions(path, perm)
    }

    fn symlink_metadata<P: AsRef<Path>>(&self, path: P) -> Result<Self::Metadata> {
        (**self).symlink_metadata(path)
    }

    fn write<P: AsRef<Path>, C: AsRef<[u8]>>(&self, path: P, contents: C) -> Result<()> {
        (**self).write(path, contents)
    }

    fn open_file<P: AsRef<Path>>(&self, path: P) -> Result<Self::File> {
        (**self).open_file(path)
    }

    fn new_openoptions(&self) -> Self::OpenOptions {
        (**self).new_openoptions()
    }

    fn new_dirbuilder(&self) -> Self::DirBuilder {
        (**self).new_dirbuilder()
    }
}
