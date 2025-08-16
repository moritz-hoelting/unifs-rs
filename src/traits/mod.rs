use std::{ffi::OsString, path::PathBuf, time::SystemTime};

use crate::Result;

pub(crate) mod dir_builder;
pub(crate) mod file;
pub(crate) mod file_system;
pub(crate) mod open_options;

/// A trait that represents metadata about a file or directory.
///
/// Similar to the [`std::fs::Metadata`] type.
#[expect(clippy::len_without_is_empty)]
pub trait UniMetadata {
    /// The type of permissions used by this metadata.
    type Permissions: UniPermissions;

    /// The type of file type used by this metadata.
    type FileType: UniFileType;

    /// Returns the file type for this metadata.
    ///
    /// This function mirrors the [`std::fs::Metadata::file_type`] function.
    fn file_type(&self) -> Self::FileType;

    /// Returns `true` if this metadata is for a directory. The
    /// result is mutually exclusive to the result of
    /// [`UniMetadata::is_file`], and will be false for symlink metadata
    /// obtained from [`crate::UniFs::symlink_metadata`].
    ///
    /// This function mirrors the [`std::fs::Metadata::is_dir`] function.
    fn is_dir(&self) -> bool;

    /// Returns `true` if this metadata is for a regular file. The
    /// result is mutually exclusive to the result of
    /// [`UniMetadata::is_dir`], and will be false for symlink metadata
    /// obtained from [`crate::UniFs::symlink_metadata`].
    ///
    /// This function mirrors the [`std::fs::Metadata::is_file`] function.
    fn is_file(&self) -> bool;

    /// Returns `true` if this metadata is for a symbolic link.
    ///
    /// This function mirrors the [`std::fs::Metadata::is_symlink`] function.
    fn is_symlink(&self) -> bool;

    /// Returns the size of the file, in bytes, this metadata is for.
    ///
    /// This function mirrors the [`std::fs::Metadata::len`] function.
    fn len(&self) -> u64;

    /// Returns the permissions of the file this metadata is for.
    ///
    /// This function mirrors the [`std::fs::Metadata::permissions`] function.
    fn permissions(&self) -> Self::Permissions;

    /// Returns the last modification time listed in this metadata.
    ///
    /// This function mirrors the [`std::fs::Metadata::modified`] function.
    fn modified(&self) -> Result<std::time::SystemTime>;

    /// Returns the last access time of this metadata.
    ///
    /// This function mirrors the [`std::fs::Metadata::accessed`] function.
    fn accessed(&self) -> Result<std::time::SystemTime>;

    /// Returns the creation time listed in this metadata.
    ///
    /// This function mirrors the [`std::fs::Metadata::created`] function.
    fn created(&self) -> Result<std::time::SystemTime>;
}

/// A trait that represents permissions for a file or directory.
///
/// Similar to the [`std::fs::Permissions`] type.
pub trait UniPermissions: PartialEq + Eq {
    /// Returns `true` if these permissions describe a readonly (unwritable) file.
    ///
    /// This function mirrors the [`std::fs::Permissions::readonly`] function.
    fn readonly(&self) -> bool;

    /// Returns `true` if these permissions describe a writable file.
    ///
    /// This function mirrors the [`std::fs::Permissions::readonly`] function.
    fn set_readonly(&mut self, readonly: bool);
}

/// A trait that represents the type of a file or directory.
///
/// Similar to the [`std::fs::FileType`] type.
pub trait UniFileType {
    /// Tests whether this file type represents a directory. The
    /// result is mutually exclusive to the results of
    /// [`UniFileType::is_file`] and [`UniFileType::is_symlink`]; only zero or one of these
    /// tests may pass.
    ///
    /// This function mirrors the [`std::fs::FileType::is_dir`] function.
    fn is_dir(&self) -> bool;

    /// Tests whether this file type represents a regular file.
    /// The result is mutually exclusive to the results of
    /// [`UniFileType::is_dir`] and [`UniFileType::is_symlink`]; only zero or one of these
    /// tests may pass.
    ///
    /// This function mirrors the [`std::fs::FileType::is_file`] function.
    fn is_file(&self) -> bool;

    /// Tests whether this file type represents a symbolic link.
    /// The result is mutually exclusive to the results of
    /// [`UniFileType::is_dir`] and [`UniFileType::is_file`]; only zero or one of these
    /// tests may pass.
    ///
    /// This function mirrors the [`std::fs::FileType::is_symlink`] function.
    fn is_symlink(&self) -> bool;
}

/// A trait that represents a directory entry in a filesystem.
///
/// Similar to the [`std::fs::DirEntry`] type.
pub trait UniDirEntry {
    /// The type of metadata returned by this directory entry.
    type Metadata: UniMetadata;

    /// The type of file type returned by this directory entry.
    type FileType: UniFileType;

    /// Returns the full path to the file that this entry represents.
    ///
    /// The full path is created by joining the original path to `read_dir`
    /// with the filename of this entry.
    ///
    /// This function mirrors the [`std::fs::DirEntry::path`] function.
    fn path(&self) -> PathBuf;

    /// Returns the metadata for the file that this entry points at.
    ///
    /// This function mirrors the [`std::fs::DirEntry::metadata`] function.
    fn metadata(&self) -> Result<Self::Metadata>;

    /// Returns the file type for the file that this entry points at.
    ///
    /// This function mirrors the [`std::fs::DirEntry::file_type`] function.
    fn file_type(&self) -> Result<Self::FileType>;

    /// Returns the file name of this directory entry without any
    /// leading path component(s).
    ///
    /// As an example,
    /// the output of the function will result in "foo" for all the following paths:
    /// - "./foo"
    /// - "/the/foo"
    /// - "../../foo"
    ///
    /// This function mirrors the [`std::fs::DirEntry::file_name`] function.
    fn file_name(&self) -> OsString;
}

/// A trait that abstracts over file times.
pub trait UniFileTimes: Default {
    /// Set the last access time of a file.
    fn set_accessed(self, t: SystemTime) -> Self;

    /// Set the last modified time of a file.
    fn set_modified(self, t: SystemTime) -> Self;
}
