use std::{
    ffi::OsString,
    fs::{self, FileTimes},
    path::{Path, PathBuf},
    time::SystemTime,
};

use crate::{
    traits::{
        dir_builder::UniDirBuilder, open_options::UniOpenOptions, UniDirEntry, UniFileTimes,
        UniFileType, UniMetadata, UniPermissions,
    },
    Result, UniFile, UniFs,
};

/// The `PhysicalFs` struct provides a filesystem interface that operates on the root filesystem of the operating system.
/// It implements the [`UniFs`] trait, allowing for various filesystem operations such as reading, writing,
/// creating directories, and managing file metadata.
///
/// Methods of the [`UniFs`] trait behave exactly like calls to the standard library's `std::fs` module.
///
/// # Example
///
/// ```no_run
/// use unifs::PhysicalFs;
///
/// # fn main() -> unifs::Result<()> {
/// PhysicalFs::create_dir("example_dir")?;
/// PhysicalFs::write("example_dir/example_file.txt", "Hello, World!")?;
/// # }
/// ```
pub struct PhysicalFs;

impl UniFs for PhysicalFs {
    type Metadata = fs::Metadata;
    type ReadDir = fs::ReadDir;
    type Permissions = fs::Permissions;
    type DirEntry = fs::DirEntry;
    type File = fs::File;
    type OpenOptions = fs::OpenOptions;
    type DirBuilder = fs::DirBuilder;

    #[inline(always)]
    fn canonicalize<P: AsRef<Path>>(&self, path: P) -> Result<PathBuf> {
        fs::canonicalize(path)
    }

    #[inline(always)]
    fn copy<P: AsRef<Path>, Q: AsRef<Path>>(&self, from: P, to: Q) -> Result<u64> {
        fs::copy(from, to)
    }

    #[inline(always)]
    fn create_dir<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        fs::create_dir(path)
    }

    #[inline(always)]
    fn create_dir_all<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        fs::create_dir_all(path)
    }

    #[inline(always)]
    fn exists<P: AsRef<Path>>(&self, path: P) -> Result<bool> {
        fs::exists(path)
    }

    #[inline(always)]
    fn hard_link<P: AsRef<Path>, Q: AsRef<Path>>(&self, original: P, link: Q) -> Result<()> {
        fs::hard_link(original, link)
    }

    #[inline(always)]
    fn metadata<P: AsRef<Path>>(&self, path: P) -> Result<fs::Metadata> {
        fs::metadata(path)
    }

    #[inline(always)]
    fn read<P: AsRef<Path>>(&self, path: P) -> Result<Vec<u8>> {
        fs::read(path)
    }

    #[inline(always)]
    fn read_dir<P: AsRef<Path>>(&self, path: P) -> Result<Self::ReadDir> {
        fs::read_dir(path)
    }

    #[inline(always)]
    fn read_link<P: AsRef<Path>>(&self, path: P) -> Result<PathBuf> {
        fs::read_link(path)
    }

    #[inline(always)]
    fn read_to_string<P: AsRef<Path>>(&self, path: P) -> Result<String> {
        fs::read_to_string(path)
    }

    #[inline(always)]
    fn remove_dir<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        fs::remove_dir(path)
    }

    #[inline(always)]
    fn remove_dir_all<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        fs::remove_dir_all(path)
    }

    #[inline(always)]
    fn remove_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        fs::remove_file(path)
    }

    #[inline(always)]
    fn rename<P: AsRef<Path>, Q: AsRef<Path>>(&self, from: P, to: Q) -> Result<()> {
        fs::rename(from, to)
    }

    #[inline(always)]
    fn set_permissions<P: AsRef<Path>>(&self, path: P, perm: Self::Permissions) -> Result<()> {
        fs::set_permissions(path, perm)
    }

    #[inline(always)]
    fn symlink_metadata<P: AsRef<Path>>(&self, path: P) -> Result<fs::Metadata> {
        fs::symlink_metadata(path)
    }

    #[inline(always)]
    fn write<P: AsRef<Path>, C: AsRef<[u8]>>(&self, path: P, contents: C) -> Result<()> {
        fs::write(path, contents)
    }

    #[inline(always)]
    fn open_file<P: AsRef<Path>>(&self, path: P) -> Result<Self::File> {
        fs::File::open(path)
    }

    #[inline(always)]
    fn new_openoptions(&self) -> Self::OpenOptions {
        fs::OpenOptions::new()
    }

    #[inline(always)]
    fn new_dirbuilder(&self) -> Self::DirBuilder {
        fs::DirBuilder::new()
    }
}

impl UniMetadata for fs::Metadata {
    type FileType = fs::FileType;

    type Permissions = fs::Permissions;

    #[inline(always)]
    fn accessed(&self) -> Result<SystemTime> {
        self.accessed()
    }

    #[inline(always)]
    fn created(&self) -> Result<SystemTime> {
        self.created()
    }

    #[inline(always)]
    fn file_type(&self) -> Self::FileType {
        self.file_type()
    }

    #[inline(always)]
    fn is_dir(&self) -> bool {
        self.is_dir()
    }

    #[inline(always)]
    fn is_file(&self) -> bool {
        self.is_file()
    }

    #[inline(always)]
    fn is_symlink(&self) -> bool {
        self.is_symlink()
    }

    #[inline(always)]
    fn len(&self) -> u64 {
        self.len()
    }

    #[inline(always)]
    fn modified(&self) -> Result<SystemTime> {
        self.modified()
    }

    #[inline(always)]
    fn permissions(&self) -> Self::Permissions {
        self.permissions()
    }
}

impl UniPermissions for fs::Permissions {
    #[inline(always)]
    fn readonly(&self) -> bool {
        self.readonly()
    }

    #[inline(always)]
    fn set_readonly(&mut self, readonly: bool) {
        self.set_readonly(readonly);
    }
}

impl UniFileType for fs::FileType {
    #[inline(always)]
    fn is_dir(&self) -> bool {
        self.is_dir()
    }

    #[inline(always)]
    fn is_file(&self) -> bool {
        self.is_file()
    }

    #[inline(always)]
    fn is_symlink(&self) -> bool {
        self.is_symlink()
    }
}

impl UniDirEntry for fs::DirEntry {
    type FileType = fs::FileType;

    type Metadata = fs::Metadata;

    #[inline(always)]
    fn path(&self) -> PathBuf {
        self.path()
    }

    #[inline(always)]
    fn file_type(&self) -> Result<Self::FileType> {
        self.file_type()
    }

    #[inline(always)]
    fn metadata(&self) -> Result<Self::Metadata> {
        self.metadata()
    }

    #[inline(always)]
    fn file_name(&self) -> OsString {
        self.file_name()
    }
}

impl UniFile for fs::File {
    type Metadata = fs::Metadata;
    type FileTimes = fs::FileTimes;
    type Permissions = fs::Permissions;

    #[inline(always)]
    fn metadata(&self) -> Result<Self::Metadata> {
        self.metadata()
    }

    #[inline(always)]
    fn set_len(&self, size: u64) -> Result<()> {
        self.set_len(size)
    }

    #[inline(always)]
    fn set_permissions(&self, perm: Self::Permissions) -> Result<()> {
        self.set_permissions(perm)
    }

    #[inline(always)]
    fn set_times(&self, times: Self::FileTimes) -> Result<()> {
        self.set_times(times)
    }

    #[inline(always)]
    fn sync_all(&self) -> Result<()> {
        self.sync_all()
    }

    #[inline(always)]
    fn sync_data(&self) -> Result<()> {
        self.sync_data()
    }

    #[inline(always)]
    fn try_clone(&self) -> Result<Self> {
        self.try_clone()
    }
}

impl UniOpenOptions for fs::OpenOptions {
    type File = fs::File;

    #[inline(always)]
    fn append(&mut self, append: bool) -> &mut Self {
        self.append(append)
    }

    #[inline(always)]
    fn create(&mut self, create: bool) -> &mut Self {
        self.create(create)
    }

    #[inline(always)]
    fn create_new(&mut self, create_new: bool) -> &mut Self {
        self.create_new(create_new)
    }

    #[inline(always)]
    fn open<P: AsRef<Path>>(&self, path: P) -> Result<Self::File> {
        self.open(path)
    }

    #[inline(always)]
    fn read(&mut self, read: bool) -> &mut Self {
        self.read(read)
    }

    #[inline(always)]
    fn truncate(&mut self, truncate: bool) -> &mut Self {
        self.truncate(truncate)
    }

    #[inline(always)]
    fn write(&mut self, write: bool) -> &mut Self {
        self.write(write)
    }
}

impl UniFileTimes for FileTimes {
    #[inline(always)]
    fn set_accessed(self, t: SystemTime) -> Self {
        self.set_accessed(t)
    }

    #[inline(always)]
    fn set_modified(self, t: SystemTime) -> Self {
        self.set_modified(t)
    }
}

impl UniDirBuilder for fs::DirBuilder {
    #[inline(always)]
    fn create<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        self.create(path)
    }

    #[inline(always)]
    fn recursive(&mut self, recursive: bool) -> &mut Self {
        self.recursive(recursive)
    }
}
