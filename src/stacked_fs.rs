//! Stacked file system module

use std::{
    fmt::Debug,
    io::{Read, Seek, Write},
    path::{Path, PathBuf},
};

use crate::{
    UniDirBuilder, UniDirEntry, UniFile, UniFileTimes, UniFileType, UniFs, UniMetadata,
    UniOpenOptions, UniPermissions,
};

/// A file system that allows stacking multiple file systems on top of each other.
pub struct StackedFs<B, O>
where
    B: UniFs,
    O: UniFs,
{
    base_fs: B,
    overlay_fs: O,
    mount_point: PathBuf,
}

/// Metadata for a stacked file system, which can represent metadata from either the base or overlay file system.
pub enum StackedMetadata<B, O>
where
    B: UniMetadata,
    O: UniMetadata,
{
    /// Metadata from the base file system.
    Base(B),
    /// Metadata from the overlay file system, along with the mount point.
    Overlay {
        /// The metadata from the overlay file system.
        data: O,
        /// The mount point where the overlay file system is mounted.
        mount_point: PathBuf,
    },
}

/// Permissions for a stacked file system, which can represent permissions from either the base or overlay file system.
pub enum StackedPermissions<B, O>
where
    B: UniPermissions,
    O: UniPermissions,
{
    /// Permissions from the base file system.
    Base(B),
    /// Permissions from the overlay file system.
    Overlay(O),
}

/// File type for a stacked file system, which can represent file types from either the base or overlay file system.
pub enum StackedFileType<B, O>
where
    B: UniMetadata,
    O: UniMetadata,
{
    /// File type from the base file system.
    Base(B::FileType),
    /// File type from the overlay file system.
    Overlay(O::FileType),
}

/// Directory entry for a stacked file system, which can represent directory entries from either the base or overlay file system.
pub enum StackedDirEntry<B, O>
where
    B: UniDirEntry,
    O: UniDirEntry,
{
    /// Directory entry from the base file system.
    Base(B),
    /// Directory entry from the overlay file system, along with the mount point.
    Overlay {
        /// The directory entry from the overlay file system.
        data: O,
        /// The mount point where the overlay file system is mounted.
        mount_point: PathBuf,
    },
}

/// Read directory iterator for a stacked file system, which can represent read directory iterators from either the base or overlay file system.
pub enum StackedReadDir<B, O>
where
    B: UniFs,
    O: UniFs,
{
    /// Read directory iterator from the base file system.
    Base(B::ReadDir),
    /// Read directory iterator from the overlay file system, along with the mount point.
    Overlay {
        /// The read directory iterator from the overlay file system.
        data: O::ReadDir,
        /// The mount point where the overlay file system is mounted.
        mount_point: PathBuf,
    },
}

/// File for a stacked file system, which can represent files from either the base or overlay file system.
pub enum StackedFile<B, O>
where
    B: UniFs,
    O: UniFs,
{
    /// File from the base file system.
    Base(B::File),
    /// File from the overlay file system, along with the mount point.
    Overlay {
        /// The file from the overlay file system.
        data: O::File,
        /// The mount point where the overlay file system is mounted.
        mount_point: PathBuf,
    },
}

/// File times for a stacked file system, which can represent file times from either the base or overlay file system.
pub enum StackedFileTimes<B, O>
where
    B: UniFileTimes,
    O: UniFileTimes,
{
    /// File times from the base file system.
    Base(B),
    /// File times from the overlay file system.
    Overlay(O),
}

/// Open options for a stacked file system, which contains open options for both the base and overlay file systems.
pub struct StackedOpenOptions<B, O>
where
    B: UniFs,
    O: UniFs,
{
    base: B::OpenOptions,
    overlay: O::OpenOptions,
    mount_point: PathBuf,
}

/// Directory builder for a stacked file system, which contains directory builders for both the base and overlay file systems.
pub struct StackedDirBuilder<B, O>
where
    B: UniFs,
    O: UniFs,
{
    base: B::DirBuilder,
    overlay: O::DirBuilder,
    mount_point: PathBuf,
}

impl<B, O> StackedFs<B, O>
where
    B: UniFs,
    O: UniFs,
{
    /// Creates a new stacked file system with the given base and overlay file systems.
    pub fn new<P: Into<PathBuf>>(base_fs: B, overlay_fs: O, mount_point: P) -> Self {
        Self {
            base_fs,
            overlay_fs,
            mount_point: mount_point.into(),
        }
    }
}

impl<B, O> UniFs for StackedFs<B, O>
where
    B: UniFs,
    O: UniFs,
{
    type Metadata = StackedMetadata<B::Metadata, O::Metadata>;
    type ReadDir = StackedReadDir<B, O>;
    type DirEntry = StackedDirEntry<B::DirEntry, O::DirEntry>;
    type Permissions = StackedPermissions<B::Permissions, O::Permissions>;
    type File = StackedFile<B, O>;
    type OpenOptions = StackedOpenOptions<B, O>;
    type DirBuilder = StackedDirBuilder<B, O>;

    fn canonicalize<P: AsRef<Path>>(&self, path: P) -> crate::Result<PathBuf> {
        let path = path.as_ref();
        if let Ok(path) = path.strip_prefix(&self.mount_point) {
            Ok(self.mount_point.join(self.overlay_fs.canonicalize(path)?))
        } else {
            self.base_fs.canonicalize(path)
        }
    }

    fn copy<P: AsRef<Path>, Q: AsRef<Path>>(&self, from: P, to: Q) -> crate::Result<u64> {
        let from = from.as_ref();
        let to = to.as_ref();
        match (
            from.strip_prefix(&self.mount_point),
            to.strip_prefix(&self.mount_point),
        ) {
            (Ok(from), Ok(to)) => self.overlay_fs.copy(from, to),
            (Err(_), Err(_)) => self.base_fs.copy(from, to),
            (Ok(from), Err(_)) => {
                let mut from_file = self.overlay_fs.new_openoptions().read(true).open(from)?;
                let mut to_file = self
                    .base_fs
                    .new_openoptions()
                    .write(true)
                    .create(true)
                    .open(to)?;

                std::io::copy(&mut from_file, &mut to_file)
            }
            (Err(_), Ok(to)) => {
                let mut from_file = self.base_fs.new_openoptions().read(true).open(from)?;
                let mut to_file = self
                    .overlay_fs
                    .new_openoptions()
                    .write(true)
                    .create(true)
                    .open(to)?;

                std::io::copy(&mut from_file, &mut to_file)
            }
        }
    }

    fn create_dir<P: AsRef<Path>>(&self, path: P) -> crate::Result<()> {
        let path = path.as_ref();
        if let Ok(path) = path.strip_prefix(&self.mount_point) {
            self.overlay_fs.create_dir(path)
        } else {
            self.base_fs.create_dir(path)
        }
    }

    fn exists<P: AsRef<Path>>(&self, path: P) -> crate::Result<bool> {
        let path = path.as_ref();
        if let Ok(path) = path.strip_prefix(&self.mount_point) {
            if self.overlay_fs.exists(path)? {
                return Ok(true);
            }
        }

        self.base_fs.exists(path)
    }

    fn hard_link<P: AsRef<Path>, Q: AsRef<Path>>(&self, original: P, link: Q) -> crate::Result<()> {
        let original = original.as_ref();
        let link = link.as_ref();
        match (
            original.strip_prefix(&self.mount_point),
            link.strip_prefix(&self.mount_point),
        ) {
            (Ok(original), Ok(link)) => self.overlay_fs.hard_link(original, link),
            (Err(_), Err(_)) => self.base_fs.hard_link(original, link),
            _ => Err(std::io::Error::other(
                "Cannot create hard link across filesystems",
            )),
        }
    }

    fn metadata<P: AsRef<Path>>(&self, path: P) -> crate::Result<Self::Metadata> {
        let path = path.as_ref();
        if let Ok(path) = path.strip_prefix(&self.mount_point) {
            if self.overlay_fs.exists(path)? {
                let metadata = self.overlay_fs.metadata(path)?;
                return Ok(StackedMetadata::Overlay {
                    data: metadata,
                    mount_point: self.mount_point.clone(),
                });
            }
        }

        let metadata = self.base_fs.metadata(path)?;
        Ok(StackedMetadata::Base(metadata))
    }

    fn read<P: AsRef<Path>>(&self, path: P) -> crate::Result<Vec<u8>> {
        let path = path.as_ref();
        if let Ok(path) = path.strip_prefix(&self.mount_point) {
            if self.overlay_fs.exists(path)? {
                return self.overlay_fs.read(path);
            }
        }

        self.base_fs.read(path)
    }

    fn read_dir<P: AsRef<Path>>(&self, path: P) -> crate::Result<Self::ReadDir> {
        let path = path.as_ref();
        if let Ok(path) = path.strip_prefix(&self.mount_point) {
            let overlay_read_dir = self.overlay_fs.read_dir(path)?;
            return Ok(StackedReadDir::Overlay {
                data: overlay_read_dir,
                mount_point: self.mount_point.clone(),
            });
        }

        let base_read_dir = self.base_fs.read_dir(path)?;
        Ok(StackedReadDir::Base(base_read_dir))
    }

    fn read_link<P: AsRef<Path>>(&self, path: P) -> crate::Result<PathBuf> {
        let path = path.as_ref();
        if let Ok(path) = path.strip_prefix(&self.mount_point) {
            if self.overlay_fs.exists(path)? {
                return self.overlay_fs.read_link(path);
            }
        }

        self.base_fs.read_link(path)
    }

    fn read_to_string<P: AsRef<Path>>(&self, path: P) -> crate::Result<String> {
        let path = path.as_ref();
        if let Ok(path) = path.strip_prefix(&self.mount_point) {
            if self.overlay_fs.exists(path)? {
                return self.overlay_fs.read_to_string(path);
            }
        }

        self.base_fs.read_to_string(path)
    }

    fn remove_dir<P: AsRef<Path>>(&self, path: P) -> crate::Result<()> {
        let path = path.as_ref();
        if let Ok(path) = path.strip_prefix(&self.mount_point) {
            if self.overlay_fs.exists(path)? {
                return self.overlay_fs.remove_dir(path);
            }
        }

        self.base_fs.remove_dir(path)
    }

    fn remove_dir_all<P: AsRef<Path>>(&self, path: P) -> crate::Result<()> {
        let path = path.as_ref();
        if let Ok(path) = path.strip_prefix(&self.mount_point) {
            if self.overlay_fs.exists(path)? {
                return self.overlay_fs.remove_dir_all(path);
            }
        }

        self.base_fs.remove_dir_all(path)
    }

    fn remove_file<P: AsRef<Path>>(&self, path: P) -> crate::Result<()> {
        let path = path.as_ref();
        if let Ok(path) = path.strip_prefix(&self.mount_point) {
            if self.overlay_fs.exists(path)? {
                return self.overlay_fs.remove_file(path);
            }
        }

        self.base_fs.remove_file(path)
    }

    fn rename<P: AsRef<Path>, Q: AsRef<Path>>(&self, from: P, to: Q) -> crate::Result<()> {
        let from = from.as_ref();
        let to = to.as_ref();
        match (
            from.strip_prefix(&self.mount_point),
            to.strip_prefix(&self.mount_point),
        ) {
            (Ok(from), Ok(to)) => self.overlay_fs.rename(from, to),
            (Err(_), Err(_)) => self.base_fs.rename(from, to),
            (Ok(from), Err(_)) => {
                let mut from_file = self.overlay_fs.new_openoptions().read(true).open(from)?;
                let mut to_file = self
                    .base_fs
                    .new_openoptions()
                    .write(true)
                    .create(true)
                    .open(to)?;

                std::io::copy(&mut from_file, &mut to_file)?;
                self.overlay_fs.remove_file(from)
            }
            (Err(_), Ok(to)) => {
                let mut from_file = self.base_fs.new_openoptions().read(true).open(from)?;
                let mut to_file = self
                    .overlay_fs
                    .new_openoptions()
                    .write(true)
                    .create(true)
                    .open(to)?;

                std::io::copy(&mut from_file, &mut to_file)?;
                self.base_fs.remove_file(from)
            }
        }
    }

    fn set_permissions<P: AsRef<Path>>(
        &self,
        path: P,
        perm: Self::Permissions,
    ) -> crate::Result<()> {
        let path = path.as_ref();
        if let Ok(path) = path.strip_prefix(&self.mount_point) {
            if self.overlay_fs.exists(path)? {
                return self.overlay_fs.set_permissions(
                    path,
                    match perm {
                        StackedPermissions::Overlay(p) => p,
                        _ => {
                            return Err(std::io::Error::other(
                                "Permission type does not match filesystem type",
                            ))
                        }
                    },
                );
            }
        }

        self.base_fs.set_permissions(
            path,
            match perm {
                StackedPermissions::Base(p) => p,
                _ => {
                    return Err(std::io::Error::other(
                        "Permission type does not match filesystem type",
                    ))
                }
            },
        )
    }

    fn symlink_metadata<P: AsRef<Path>>(&self, path: P) -> crate::Result<Self::Metadata> {
        let path = path.as_ref();
        if let Ok(path) = path.strip_prefix(&self.mount_point) {
            if self.overlay_fs.exists(path)? {
                let metadata = self.overlay_fs.symlink_metadata(path)?;
                return Ok(StackedMetadata::Overlay {
                    data: metadata,
                    mount_point: self.mount_point.clone(),
                });
            }
        }

        let metadata = self.base_fs.symlink_metadata(path)?;
        Ok(StackedMetadata::Base(metadata))
    }

    fn new_openoptions(&self) -> Self::OpenOptions {
        StackedOpenOptions {
            base: self.base_fs.new_openoptions(),
            overlay: self.overlay_fs.new_openoptions(),
            mount_point: self.mount_point.clone(),
        }
    }

    fn new_dirbuilder(&self) -> Self::DirBuilder {
        StackedDirBuilder {
            base: self.base_fs.new_dirbuilder(),
            overlay: self.overlay_fs.new_dirbuilder(),
            mount_point: self.mount_point.clone(),
        }
    }
}

impl<B, O> UniMetadata for StackedMetadata<B, O>
where
    B: UniMetadata,
    O: UniMetadata,
{
    type Permissions = StackedPermissions<B::Permissions, O::Permissions>;
    type FileType = StackedFileType<B, O>;

    fn file_type(&self) -> Self::FileType {
        match self {
            StackedMetadata::Base(meta) => StackedFileType::Base(meta.file_type()),
            StackedMetadata::Overlay { data, .. } => StackedFileType::Overlay(data.file_type()),
        }
    }

    fn is_dir(&self) -> bool {
        match self {
            StackedMetadata::Base(meta) => meta.is_dir(),
            StackedMetadata::Overlay { data, .. } => data.is_dir(),
        }
    }

    fn is_file(&self) -> bool {
        match self {
            StackedMetadata::Base(meta) => meta.is_file(),
            StackedMetadata::Overlay { data, .. } => data.is_file(),
        }
    }

    fn is_symlink(&self) -> bool {
        match self {
            StackedMetadata::Base(meta) => meta.is_symlink(),
            StackedMetadata::Overlay { data, .. } => data.is_symlink(),
        }
    }

    fn len(&self) -> u64 {
        match self {
            StackedMetadata::Base(meta) => meta.len(),
            StackedMetadata::Overlay { data, .. } => data.len(),
        }
    }

    fn permissions(&self) -> Self::Permissions {
        match self {
            StackedMetadata::Base(meta) => StackedPermissions::Base(meta.permissions()),
            StackedMetadata::Overlay { data, .. } => {
                StackedPermissions::Overlay(data.permissions())
            }
        }
    }

    fn modified(&self) -> crate::Result<std::time::SystemTime> {
        match self {
            StackedMetadata::Base(meta) => meta.modified(),
            StackedMetadata::Overlay { data, .. } => data.modified(),
        }
    }

    fn accessed(&self) -> crate::Result<std::time::SystemTime> {
        match self {
            StackedMetadata::Base(meta) => meta.accessed(),
            StackedMetadata::Overlay { data, .. } => data.accessed(),
        }
    }

    fn created(&self) -> crate::Result<std::time::SystemTime> {
        match self {
            StackedMetadata::Base(meta) => meta.created(),
            StackedMetadata::Overlay { data, .. } => data.created(),
        }
    }
}

impl<B, O> PartialEq for StackedPermissions<B, O>
where
    B: UniPermissions,
    O: UniPermissions,
{
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (StackedPermissions::Base(a), StackedPermissions::Base(b)) => a == b,
            (StackedPermissions::Overlay(a), StackedPermissions::Overlay(b)) => a == b,
            _ => false,
        }
    }
}

impl<B, O> Eq for StackedPermissions<B, O>
where
    B: UniPermissions,
    O: UniPermissions,
{
}

impl<B, O> UniPermissions for StackedPermissions<B, O>
where
    B: UniPermissions,
    O: UniPermissions,
{
    fn readonly(&self) -> bool {
        match self {
            StackedPermissions::Base(perm) => perm.readonly(),
            StackedPermissions::Overlay(perm) => perm.readonly(),
        }
    }

    fn set_readonly(&mut self, readonly: bool) {
        match self {
            StackedPermissions::Base(perm) => perm.set_readonly(readonly),
            StackedPermissions::Overlay(perm) => perm.set_readonly(readonly),
        }
    }
}

impl<B, O> UniFileType for StackedFileType<B, O>
where
    B: UniMetadata,
    O: UniMetadata,
{
    fn is_dir(&self) -> bool {
        match self {
            StackedFileType::Base(ft) => ft.is_dir(),
            StackedFileType::Overlay(ft) => ft.is_dir(),
        }
    }

    fn is_file(&self) -> bool {
        match self {
            StackedFileType::Base(ft) => ft.is_file(),
            StackedFileType::Overlay(ft) => ft.is_file(),
        }
    }

    fn is_symlink(&self) -> bool {
        match self {
            StackedFileType::Base(ft) => ft.is_symlink(),
            StackedFileType::Overlay(ft) => ft.is_symlink(),
        }
    }
}

impl<B, O> UniDirEntry for StackedDirEntry<B, O>
where
    B: UniDirEntry,
    O: UniDirEntry,
{
    type Metadata = StackedMetadata<B::Metadata, O::Metadata>;
    type FileType = StackedFileType<B::Metadata, O::Metadata>;

    fn path(&self) -> PathBuf {
        match self {
            StackedDirEntry::Base(entry) => entry.path(),
            StackedDirEntry::Overlay { data, mount_point } => mount_point.join(data.path()),
        }
    }

    fn metadata(&self) -> crate::Result<Self::Metadata> {
        match self {
            StackedDirEntry::Base(entry) => Ok(StackedMetadata::Base(entry.metadata()?)),
            StackedDirEntry::Overlay { data, mount_point } => {
                let metadata = data.metadata()?;
                Ok(StackedMetadata::Overlay {
                    data: metadata,
                    mount_point: mount_point.clone(),
                })
            }
        }
    }

    fn file_type(&self) -> crate::Result<Self::FileType> {
        match self {
            StackedDirEntry::Base(entry) => Ok(StackedFileType::Base(entry.file_type()?)),
            StackedDirEntry::Overlay { data, .. } => {
                let file_type = data.file_type()?;
                Ok(StackedFileType::Overlay(file_type))
            }
        }
    }

    fn file_name(&self) -> std::ffi::OsString {
        match self {
            StackedDirEntry::Base(entry) => entry.file_name(),
            StackedDirEntry::Overlay { data, .. } => data.file_name(),
        }
    }
}

impl<B, O> Iterator for StackedReadDir<B, O>
where
    B: UniFs,
    O: UniFs,
{
    type Item = crate::Result<StackedDirEntry<B::DirEntry, O::DirEntry>>;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            StackedReadDir::Base(iter) => iter.next().map(|res| res.map(StackedDirEntry::Base)),
            StackedReadDir::Overlay { data, mount_point } => data.next().map(|res| {
                res.map(|entry| StackedDirEntry::Overlay {
                    data: entry,
                    mount_point: mount_point.clone(),
                })
            }),
        }
    }
}

impl<B, O> Debug for StackedFile<B, O>
where
    B: UniFs,
    O: UniFs,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StackedFile::Base(file) => f.debug_tuple("Base").field(file).finish(),
            StackedFile::Overlay { data, mount_point } => f
                .debug_struct("Overlay")
                .field("data", data)
                .field("mount_point", mount_point)
                .finish(),
        }
    }
}

impl<B, O> Read for StackedFile<B, O>
where
    B: UniFs,
    O: UniFs,
{
    fn read(&mut self, buf: &mut [u8]) -> crate::Result<usize> {
        match self {
            StackedFile::Base(file) => file.read(buf),
            StackedFile::Overlay { data, .. } => data.read(buf),
        }
    }
}

impl<B, O> Write for StackedFile<B, O>
where
    B: UniFs,
    O: UniFs,
{
    fn write(&mut self, buf: &[u8]) -> crate::Result<usize> {
        match self {
            StackedFile::Base(file) => file.write(buf),
            StackedFile::Overlay { data, .. } => data.write(buf),
        }
    }

    fn flush(&mut self) -> crate::Result<()> {
        match self {
            StackedFile::Base(file) => file.flush(),
            StackedFile::Overlay { data, .. } => data.flush(),
        }
    }
}

impl<B, O> Seek for StackedFile<B, O>
where
    B: UniFs,
    O: UniFs,
{
    fn seek(&mut self, pos: std::io::SeekFrom) -> crate::Result<u64> {
        match self {
            StackedFile::Base(file) => file.seek(pos),
            StackedFile::Overlay { data, .. } => data.seek(pos),
        }
    }
}

impl<B, O> UniFile for StackedFile<B, O>
where
    B: UniFs,
    O: UniFs,
{
    type Metadata = StackedMetadata<B::Metadata, O::Metadata>;
    type Permissions = StackedPermissions<B::Permissions, O::Permissions>;
    type FileTimes =
        StackedFileTimes<<B::File as UniFile>::FileTimes, <O::File as UniFile>::FileTimes>;

    fn sync_all(&self) -> crate::Result<()> {
        match self {
            StackedFile::Base(file) => file.sync_all(),
            StackedFile::Overlay { data, .. } => data.sync_all(),
        }
    }

    fn sync_data(&self) -> crate::Result<()> {
        match self {
            StackedFile::Base(file) => file.sync_data(),
            StackedFile::Overlay { data, .. } => data.sync_data(),
        }
    }

    fn set_len(&self, size: u64) -> crate::Result<()> {
        match self {
            StackedFile::Base(file) => file.set_len(size),
            StackedFile::Overlay { data, .. } => data.set_len(size),
        }
    }

    fn metadata(&self) -> crate::Result<Self::Metadata> {
        match self {
            StackedFile::Base(file) => Ok(StackedMetadata::Base(file.metadata()?)),
            StackedFile::Overlay { data, mount_point } => {
                let metadata = data.metadata()?;
                Ok(StackedMetadata::Overlay {
                    data: metadata,
                    mount_point: mount_point.clone(),
                })
            }
        }
    }

    fn try_clone(&self) -> crate::Result<Self> {
        match self {
            StackedFile::Base(file) => file.try_clone().map(StackedFile::Base),
            StackedFile::Overlay { data, mount_point } => {
                data.try_clone().map(|file| StackedFile::Overlay {
                    data: file,
                    mount_point: mount_point.clone(),
                })
            }
        }
    }

    fn set_permissions(&self, perm: Self::Permissions) -> crate::Result<()> {
        match (self, perm) {
            (StackedFile::Base(file), StackedPermissions::Base(perm)) => file.set_permissions(perm),
            (StackedFile::Overlay { data, .. }, StackedPermissions::Overlay(perm)) => {
                data.set_permissions(perm)
            }
            _ => Err(std::io::Error::other(
                "Permission type does not match file type",
            )),
        }
    }

    fn set_times(&self, times: Self::FileTimes) -> crate::Result<()> {
        match (self, times) {
            (StackedFile::Base(file), StackedFileTimes::Base(times)) => file.set_times(times),
            (StackedFile::Overlay { data, .. }, StackedFileTimes::Overlay(times)) => {
                data.set_times(times)
            }
            _ => Err(std::io::Error::other(
                "FileTimes type does not match file type",
            )),
        }
    }
}

impl<B, O> Default for StackedFileTimes<B, O>
where
    B: UniFileTimes + Default,
    O: UniFileTimes,
{
    fn default() -> Self {
        StackedFileTimes::Base(B::default())
    }
}

impl<B, O> UniFileTimes for StackedFileTimes<B, O>
where
    B: UniFileTimes,
    O: UniFileTimes,
{
    fn set_accessed(self, t: std::time::SystemTime) -> Self {
        match self {
            Self::Base(bt) => Self::Base(bt.set_accessed(t)),
            Self::Overlay(ot) => Self::Overlay(ot.set_accessed(t)),
        }
    }

    fn set_modified(self, t: std::time::SystemTime) -> Self {
        match self {
            Self::Base(bt) => Self::Base(bt.set_modified(t)),
            Self::Overlay(ot) => Self::Overlay(ot.set_modified(t)),
        }
    }
}

impl<B, O> UniOpenOptions for StackedOpenOptions<B, O>
where
    B: UniFs,
    O: UniFs,
{
    type File = StackedFile<B, O>;

    fn read(&mut self, read: bool) -> &mut Self {
        self.base.read(read);
        self.overlay.read(read);

        self
    }

    fn write(&mut self, write: bool) -> &mut Self {
        self.base.write(write);
        self.overlay.write(write);

        self
    }

    fn append(&mut self, append: bool) -> &mut Self {
        self.base.append(append);
        self.overlay.append(append);

        self
    }

    fn truncate(&mut self, truncate: bool) -> &mut Self {
        self.base.truncate(truncate);
        self.overlay.truncate(truncate);

        self
    }

    fn create(&mut self, create: bool) -> &mut Self {
        self.base.create(create);
        self.overlay.create(create);

        self
    }

    fn create_new(&mut self, create_new: bool) -> &mut Self {
        self.base.create_new(create_new);
        self.overlay.create_new(create_new);

        self
    }

    fn open<P: AsRef<Path>>(&self, path: P) -> crate::Result<Self::File> {
        let path = path.as_ref();
        if let Ok(path) = path.strip_prefix(&self.mount_point) {
            self.overlay.open(path).map(|file| StackedFile::Overlay {
                data: file,
                mount_point: self.mount_point.clone(),
            })
        } else {
            self.base.open(path).map(StackedFile::Base)
        }
    }
}

impl<B, O> UniDirBuilder for StackedDirBuilder<B, O>
where
    B: UniFs,
    O: UniFs,
{
    fn create<P: AsRef<Path>>(&self, path: P) -> crate::Result<()> {
        let path = path.as_ref();
        if let Ok(path) = path.strip_prefix(&self.mount_point) {
            self.overlay.create(path)
        } else {
            self.base.create(path)
        }
    }

    fn recursive(&mut self, recursive: bool) -> &mut Self {
        self.base.recursive(recursive);
        self.overlay.recursive(recursive);

        self
    }
}
