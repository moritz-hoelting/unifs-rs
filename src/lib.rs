#![deny(clippy::all)]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]

mod traits;

mod rw_lock;

#[cfg(feature = "fs_access")]
mod physical_fs;

#[cfg(feature = "memory_fs")]
pub mod memory_fs;

pub mod altroot_fs;
pub mod readonly_fs;

use std::{fmt::Debug, time::SystemTime};

#[doc(inline)]
pub use traits::{
    dir_builder::UniDirBuilder, file::UniFile, file_system::UniFs, file_system_ext::UniFsExt,
    open_options::UniOpenOptions, UniDirEntry, UniFileTimes, UniFileType, UniMetadata,
    UniPermissions,
};

#[doc(inline)]
#[cfg(feature = "fs_access")]
pub use physical_fs::PhysicalFs;

#[doc(inline)]
#[cfg(feature = "memory_fs")]
pub use memory_fs::MemoryFs;

#[doc(inline)]
pub use altroot_fs::AltrootFs;
#[doc(inline)]
pub use readonly_fs::ReadonlyFs;

pub type Result<T> = std::result::Result<T, std::io::Error>;

/// A unified file type that can represent different file types in a filesystem.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileType {
    /// Represents a directory.
    Directory,
    /// Represents a regular file.
    File,
    /// Represents a symbolic link.
    Symlink,
}

impl UniFileType for FileType {
    fn is_dir(&self) -> bool {
        matches!(self, FileType::Directory)
    }

    fn is_file(&self) -> bool {
        matches!(self, FileType::File)
    }

    fn is_symlink(&self) -> bool {
        matches!(self, FileType::Symlink)
    }
}

/// A unified permissions type that can represent file permissions in a filesystem.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Permissions {
    readonly: bool,
}

impl UniPermissions for Permissions {
    fn readonly(&self) -> bool {
        self.readonly
    }

    fn set_readonly(&mut self, readonly: bool) {
        self.readonly = readonly;
    }
}

/// A unified file times structure that can represent file timestamps in a filesystem.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileTimes {
    created: SystemTime,
    modified: Option<SystemTime>,
    accessed: Option<SystemTime>,
}

impl Default for FileTimes {
    fn default() -> Self {
        FileTimes {
            created: SystemTime::now(),
            modified: None,
            accessed: None,
        }
    }
}

impl UniFileTimes for FileTimes {
    fn set_accessed(self, t: SystemTime) -> Self {
        FileTimes {
            accessed: Some(t),
            ..self
        }
    }

    fn set_modified(self, t: SystemTime) -> Self {
        FileTimes {
            modified: Some(t),
            ..self
        }
    }
}
