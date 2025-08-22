//! This module provides an alternative root directory for a filesystem.

use std::{
    borrow::Cow,
    io::ErrorKind,
    path::{Path, PathBuf},
};

use crate::{Result, UniDirBuilder, UniDirEntry, UniFs, UniMetadata, UniOpenOptions};

/// Wraps a filesystem to provide an alternative root directory.
pub struct AltrootFs<FS: UniFs> {
    root: PathBuf,
    fs: FS,
}

pub struct AltrootDirEntry<T: UniDirEntry> {
    root: PathBuf,
    entry: T,
}

pub struct AltrootReadDir<FS: UniFs> {
    root: PathBuf,
    inner: FS::ReadDir,
}

pub struct AltrootOpenOptions<O: UniOpenOptions> {
    root: PathBuf,
    inner: O,
}

pub struct AltrootDirBuilder<T: UniDirBuilder> {
    root: PathBuf,
    inner: T,
}

fn get_real_path<P: AsRef<Path>, Q: AsRef<Path>>(root: P, path: Q) -> PathBuf {
    let path = path.as_ref();

    let path = if path
        .components()
        .next()
        .is_some_and(|comp| matches!(comp, std::path::Component::RootDir))
    {
        Cow::Owned(path.components().skip(1).collect::<PathBuf>())
    } else {
        Cow::Borrowed(path)
    };

    root.as_ref().join(path)
}

impl<FS: UniFs> AltrootFs<FS> {
    pub fn new<P: Into<PathBuf>>(root: P, fs: FS) -> Result<Self> {
        let root = root.into();
        if let Ok(metadata) = fs.metadata(&root) {
            if metadata.is_dir() {
                Ok(Self { root, fs })
            } else {
                Err(std::io::Error::new(
                    ErrorKind::NotADirectory,
                    format!("Root path is not a directory: {}", root.display()),
                ))
            }
        } else {
            Err(std::io::Error::new(
                ErrorKind::NotFound,
                format!("Root path does not exist: {}", root.display()),
            ))
        }
    }

    pub fn new_or_create<P: Into<PathBuf>>(root: P, fs: FS) -> Result<Self> {
        let root = root.into();
        if !fs.exists(&root).unwrap_or_default() {
            fs.create_dir_all(&root)?;
        }
        Self::new(root, fs)
    }

    fn get_real_path<P: AsRef<Path>>(&self, path: P) -> PathBuf {
        get_real_path(&self.root, path)
    }
}

impl<FS: UniFs> UniFs for AltrootFs<FS> {
    type Metadata = FS::Metadata;
    type ReadDir = AltrootReadDir<FS>;
    type DirEntry = AltrootDirEntry<FS::DirEntry>;
    type Permissions = FS::Permissions;
    type File = FS::File;
    type OpenOptions = AltrootOpenOptions<FS::OpenOptions>;
    type DirBuilder = AltrootDirBuilder<FS::DirBuilder>;

    fn canonicalize<P: AsRef<std::path::Path>>(&self, path: P) -> Result<PathBuf> {
        let original = self.fs.canonicalize(path)?;
        let root = self.fs.canonicalize(&self.root)?;
        original
            .strip_prefix(root)
            .map(|p| p.to_path_buf())
            .map_err(|e| std::io::Error::new(ErrorKind::NotFound, format!("Path not found: {}", e)))
    }

    fn copy<P: AsRef<std::path::Path>, Q: AsRef<std::path::Path>>(
        &self,
        from: P,
        to: Q,
    ) -> Result<u64> {
        let from = self.get_real_path(from);
        let to = self.get_real_path(to);

        self.fs.copy(from, to)
    }

    fn create_dir<P: AsRef<std::path::Path>>(&self, path: P) -> Result<()> {
        let path = self.get_real_path(path);
        self.fs.create_dir(path)
    }

    fn create_dir_all<P: AsRef<std::path::Path>>(&self, path: P) -> Result<()> {
        let path = self.get_real_path(path);

        self.fs.create_dir_all(path)
    }

    fn exists<P: AsRef<std::path::Path>>(&self, path: P) -> Result<bool> {
        let path = self.get_real_path(path);

        self.fs.exists(path)
    }

    fn hard_link<P: AsRef<std::path::Path>, Q: AsRef<std::path::Path>>(
        &self,
        original: P,
        link: Q,
    ) -> Result<()> {
        let original = self.get_real_path(original);
        let link = self.get_real_path(link);

        self.fs.hard_link(original, link)
    }

    fn metadata<P: AsRef<std::path::Path>>(&self, path: P) -> Result<Self::Metadata> {
        let path = self.get_real_path(path);

        self.fs.metadata(path)
    }

    fn read<P: AsRef<std::path::Path>>(&self, path: P) -> Result<Vec<u8>> {
        let path = self.get_real_path(path);

        self.fs.read(path)
    }

    fn read_dir<P: AsRef<std::path::Path>>(&self, path: P) -> Result<Self::ReadDir> {
        let path = self.get_real_path(path);

        self.fs.read_dir(path).map(|r| AltrootReadDir {
            root: self.root.clone(),
            inner: r,
        })
    }

    fn read_link<P: AsRef<std::path::Path>>(&self, path: P) -> Result<PathBuf> {
        let path = self.get_real_path(path);

        self.fs.read_link(path)
    }

    fn read_to_string<P: AsRef<std::path::Path>>(&self, path: P) -> Result<String> {
        let path = self.get_real_path(path);

        self.fs.read_to_string(path)
    }

    fn remove_dir<P: AsRef<std::path::Path>>(&self, path: P) -> Result<()> {
        let path = self.get_real_path(path);

        self.fs.remove_dir(path)
    }

    fn remove_dir_all<P: AsRef<std::path::Path>>(&self, path: P) -> Result<()> {
        let path = self.get_real_path(path);

        self.fs.remove_dir_all(path)
    }

    fn remove_file<P: AsRef<std::path::Path>>(&self, path: P) -> Result<()> {
        let path = self.get_real_path(path);

        self.fs.remove_file(path)
    }

    fn rename<P: AsRef<std::path::Path>, Q: AsRef<std::path::Path>>(
        &self,
        from: P,
        to: Q,
    ) -> Result<()> {
        let from = self.get_real_path(from);
        let to = self.get_real_path(to);

        self.fs.rename(from, to)
    }

    fn set_permissions<P: AsRef<std::path::Path>>(
        &self,
        path: P,
        perm: Self::Permissions,
    ) -> Result<()> {
        let path = self.get_real_path(path);

        self.fs.set_permissions(path, perm)
    }

    fn symlink_metadata<P: AsRef<std::path::Path>>(&self, path: P) -> Result<Self::Metadata> {
        let path = self.get_real_path(path);

        self.fs.symlink_metadata(path)
    }

    fn write<P: AsRef<std::path::Path>, C: AsRef<[u8]>>(&self, path: P, contents: C) -> Result<()> {
        let path = self.get_real_path(path);

        self.fs.write(path, contents)
    }

    fn open_file<P: AsRef<std::path::Path>>(&self, path: P) -> Result<Self::File> {
        let path = self.get_real_path(path);

        self.fs.open_file(path)
    }

    fn new_openoptions(&self) -> Self::OpenOptions {
        AltrootOpenOptions {
            root: self.root.clone(),
            inner: self.fs.new_openoptions(),
        }
    }

    fn new_dirbuilder(&self) -> Self::DirBuilder {
        AltrootDirBuilder {
            root: self.root.clone(),
            inner: self.fs.new_dirbuilder(),
        }
    }
}

impl<T: UniDirEntry> UniDirEntry for AltrootDirEntry<T> {
    type Metadata = T::Metadata;
    type FileType = T::FileType;

    fn path(&self) -> PathBuf {
        let path = self.entry.path();
        if let Ok(stripped) = path.strip_prefix(&self.root) {
            stripped.to_path_buf()
        } else {
            path
        }
    }

    fn metadata(&self) -> Result<Self::Metadata> {
        self.entry.metadata()
    }

    fn file_type(&self) -> Result<Self::FileType> {
        self.entry.file_type()
    }

    fn file_name(&self) -> std::ffi::OsString {
        self.entry.file_name()
    }
}

impl<O: UniOpenOptions> UniOpenOptions for AltrootOpenOptions<O> {
    type File = O::File;

    fn read(&mut self, read: bool) -> &mut Self {
        self.inner.read(read);
        self
    }

    fn write(&mut self, write: bool) -> &mut Self {
        self.inner.write(write);
        self
    }

    fn append(&mut self, append: bool) -> &mut Self {
        self.inner.append(append);
        self
    }

    fn truncate(&mut self, truncate: bool) -> &mut Self {
        self.inner.truncate(truncate);
        self
    }

    fn create(&mut self, create: bool) -> &mut Self {
        self.inner.create(create);
        self
    }

    fn create_new(&mut self, create_new: bool) -> &mut Self {
        self.inner.create_new(create_new);
        self
    }

    fn open<P: AsRef<std::path::Path>>(&self, path: P) -> Result<Self::File> {
        let path = get_real_path(&self.root, path);
        self.inner.open(path)
    }
}

impl<T: UniDirBuilder> UniDirBuilder for AltrootDirBuilder<T> {
    fn recursive(&mut self, recursive: bool) -> &mut Self {
        self.inner.recursive(recursive);
        self
    }

    fn create<P: AsRef<std::path::Path>>(&self, path: P) -> Result<()> {
        let path = get_real_path(&self.root, path);
        self.inner.create(path)
    }
}

impl<FS> Iterator for AltrootReadDir<FS>
where
    FS: UniFs,
{
    type Item = Result<AltrootDirEntry<FS::DirEntry>>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.inner.next() {
            Some(Ok(entry)) => {
                let root = self.root.clone();
                Some(Ok(AltrootDirEntry { root, entry }))
            }
            Some(Err(e)) => Some(Err(e)),
            None => None,
        }
    }
}
