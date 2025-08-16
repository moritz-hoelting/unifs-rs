use std::{
    fmt::Debug,
    io::{Read, Seek, Write},
    time::SystemTime,
};

use crate::{traits::UniFileTimes, Result, UniMetadata, UniPermissions};

/// A trait representing a unified file type that can be used across different filesystems.
pub trait UniFile: Debug + Read + Seek + Write + Sized
where
    for<'a> &'a mut Self: Read + Seek + Write,
{
    /// The Metadata type of the file.
    type Metadata: UniMetadata;

    /// The Permissions type of the file.
    type Permissions: UniPermissions;

    /// The FileTimes type of the file.
    type FileTimes: UniFileTimes;

    /// Attempts to sync all OS-internal file content and metadata to disk.
    ///
    /// This function will attempt to ensure that all in-memory data reaches the
    /// filesystem before returning.
    ///
    /// This can be used to handle errors that would otherwise only be caught when
    /// the File is closed, as dropping a File will ignore all errors. Note, however,
    /// that sync_all is generally more expensive than closing a file by dropping it,
    /// because the latter is not required to block until the data has been written
    /// to the filesystem.
    ///
    /// If synchronizing the metadata is not required, use [`UniFile::sync_data`] instead.
    ///
    /// This functions mirrors the [`std::fs::File::sync_all`] function.
    fn sync_all(&self) -> Result<()>;

    /// This function is similar to [`UniFile::sync_all`], except that it might not synchronize
    /// file metadata to the filesystem.
    ///
    /// This is intended for use cases that must synchronize content, but don’t need
    /// the metadata on disk. The goal of this method is to reduce disk operations.
    ///
    /// Note that some platforms may simply implement this in terms of [`UniFile::sync_all`].
    ///
    /// This functions mirrors the [`std::fs::File::sync_data`] function.
    fn sync_data(&self) -> Result<()>;

    /// Truncates or extends the underlying file, updating the size of this file to become size.
    ///
    /// This function mirrors the [`std::fs::File::set_len`] function.
    fn set_len(&self, size: u64) -> Result<()>;

    /// Queries metadata about the underlying file.
    fn metadata(&self) -> Result<Self::Metadata>;

    /// Creates a new File instance that shares the same underlying file handle as the existing
    /// File instance. Reads, writes, and seeks will affect both File instances simultaneously.
    fn try_clone(&self) -> Result<Self>;

    /// Changes the permissions on the underlying file.
    fn set_permissions(&self, perm: Self::Permissions) -> Result<()>;

    /// Changes the timestamps of the underlying file.
    fn set_times(&self, times: Self::FileTimes) -> Result<()>;

    /// Changes the modification time of the underlying file.
    ///
    /// This is an alias for set_times(FileTimes::new().set_modified(time))
    fn set_modified(&self, time: SystemTime) -> Result<()> {
        self.set_times(Self::FileTimes::default().set_modified(time))
    }
}
