use std::time::SystemTime;

use crate::{FileTimes, FileType, Permissions, Result, UniFileType, UniMetadata};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MemoryMetadata {
    pub(super) file_type: FileType,
    pub(super) len: u64,
    pub(super) permissions: Permissions,
    pub(super) file_times: FileTimes,
}

impl UniMetadata for MemoryMetadata {
    type Permissions = Permissions;
    type FileType = FileType;

    fn file_type(&self) -> Self::FileType {
        self.file_type
    }

    fn is_dir(&self) -> bool {
        self.file_type.is_dir()
    }

    fn is_file(&self) -> bool {
        self.file_type.is_file()
    }

    fn is_symlink(&self) -> bool {
        self.file_type.is_symlink()
    }

    fn len(&self) -> u64 {
        self.len
    }

    fn permissions(&self) -> Self::Permissions {
        self.permissions.clone()
    }

    fn modified(&self) -> Result<SystemTime> {
        self.file_times.modified.ok_or_else(|| {
            std::io::Error::new(std::io::ErrorKind::NotFound, "Modified time not set")
        })
    }

    fn accessed(&self) -> Result<SystemTime> {
        self.file_times.accessed.ok_or_else(|| {
            std::io::Error::new(std::io::ErrorKind::NotFound, "Accessed time not set")
        })
    }

    fn created(&self) -> Result<SystemTime> {
        Ok(self.file_times.created)
    }
}
