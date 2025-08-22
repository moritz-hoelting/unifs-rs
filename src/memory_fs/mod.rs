//! This module provides an in-memory filesystem implementation.

use std::{
    collections::{HashMap, HashSet, VecDeque},
    ffi::OsString,
    io::{Error, ErrorKind},
    path::{Path, PathBuf},
    sync::Arc,
    time::SystemTime,
};

use crate::{
    memory_fs::{
        dir_builder::MemoryDirBuilder, file::MemoryFile, metadata::MemoryMetadata,
        open_options::MemoryOpenOptions,
    },
    rw_lock::RwLock,
    Permissions, UniDirEntry, UniFs,
};

mod dir_builder;
mod file;
mod metadata;
mod open_options;

/// The `MemoryFs` struct provides a filesystem interface that operates entirely in memory.
pub struct MemoryFs {
    inner: Arc<RwLock<MemoryFsInner>>,
}

impl MemoryFs {
    pub fn new() -> Self {
        MemoryFs {
            inner: Arc::new(RwLock::new(MemoryFsInner::new())),
        }
    }
}

impl Default for MemoryFs {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
struct MemoryFsInner {
    files: HashMap<PathBuf, MemoryEntry>,
}

impl MemoryFsInner {
    pub fn new() -> Self {
        let mut files = HashMap::new();

        // Create the root directory entry
        let root_path = PathBuf::from("/");
        let root_entry = MemoryEntry {
            file_type: MemoryEntryType::Directory(HashSet::new()),
            created: SystemTime::now(),
            modified: None,
            accessed: None,
            permissions: Permissions { readonly: false },
        };
        files.insert(root_path, root_entry);

        MemoryFsInner { files }
    }
}

#[derive(Debug, Clone)]
struct MemoryEntry {
    file_type: MemoryEntryType,
    created: SystemTime,
    modified: Option<SystemTime>,
    accessed: Option<SystemTime>,
    permissions: Permissions,
}

impl MemoryEntry {
    fn metadata(&self) -> MemoryMetadata {
        MemoryMetadata {
            file_type: self.file_type.clone().into(),
            len: match &self.file_type {
                MemoryEntryType::File(data) => data.read().len() as u64,
                _ => 0,
            },
            permissions: self.permissions.clone(),
            file_times: crate::FileTimes {
                created: self.created,
                modified: self.modified,
                accessed: self.accessed,
            },
        }
    }
}

#[derive(Debug, Clone)]
enum MemoryEntryType {
    File(Arc<RwLock<Vec<u8>>>),
    Directory(HashSet<OsString>),
    HardLink(PathBuf),
}

impl MemoryEntryType {
    fn as_directory_mut(&mut self) -> Option<&mut HashSet<OsString>> {
        if let MemoryEntryType::Directory(ref mut set) = self {
            Some(set)
        } else {
            None
        }
    }
}

impl From<MemoryEntryType> for crate::FileType {
    fn from(entry_type: MemoryEntryType) -> Self {
        match entry_type {
            MemoryEntryType::File(_) => crate::FileType::File,
            MemoryEntryType::Directory(_) => crate::FileType::Directory,
            MemoryEntryType::HardLink(_) => crate::FileType::Symlink,
        }
    }
}

fn canonicalize_inner<P: AsRef<Path>>(
    inner: &MemoryFsInner,
    path: P,
    resolve_hardlinks: bool,
) -> crate::Result<PathBuf> {
    use std::path::Component;

    let mut buf = PathBuf::new();

    for comp in path.as_ref().components() {
        match comp {
            Component::CurDir => {}
            Component::Normal(name) => {
                buf.push(name);
            }
            Component::ParentDir => {
                if !buf.pop() {
                    return Err(Error::new(ErrorKind::NotFound, "No parent directory"));
                }
            }
            Component::Prefix(_) | Component::RootDir => {
                buf.clear();
                buf.push("/");
            }
        }
    }

    if !buf.starts_with("/") {
        buf = Path::new("/").join(buf);
    }

    if resolve_hardlinks {
        let resolve = match inner.files.get(&buf) {
            Some(entry) if matches!(entry.file_type, MemoryEntryType::HardLink(_)) => true,
            None => true,
            _ => false,
        };
        if resolve {
            let mut current_path = PathBuf::from("/");
            for comp in buf.components() {
                match comp {
                    Component::Normal(name) => {
                        current_path.push(name);
                        if let Some(entry) = inner.files.get(&current_path) {
                            if let MemoryEntryType::HardLink(target) = &entry.file_type {
                                current_path = target.clone();
                            }
                        }
                    }
                    Component::ParentDir => {
                        if !current_path.pop() {
                            return Err(Error::new(ErrorKind::NotFound, "No parent directory"));
                        }
                    }
                    _ => {}
                }
            }
            buf = current_path;
        }
    }

    Ok(buf)
}

fn is_dir(inner: &MemoryFsInner, path: &Path) -> crate::Result<bool> {
    match inner.files.get(path) {
        Some(entry) => match &entry.file_type {
            MemoryEntryType::Directory(_) => Ok(true),
            _ => Ok(false),
        },
        None => Err(Error::new(
            ErrorKind::NotFound,
            format!("Path '{}' does not exist", path.display()),
        )),
    }
}

fn remove_recursive(path: &Path, inner: &mut MemoryFsInner) -> crate::Result<()> {
    if let Some(entry) = inner.files.get(path) {
        match &entry.file_type {
            MemoryEntryType::Directory(files) => {
                let files = files.clone();
                for file_name in files.iter() {
                    let file_path = path.join(file_name);
                    remove_recursive(&file_path, inner)?;
                }
            }
            MemoryEntryType::File(_) => {
                if let Some(parent) = path.parent() {
                    if let Some(parent_entry) = inner.files.get_mut(parent) {
                        if let Some(files) = parent_entry.file_type.as_directory_mut() {
                            files.remove(path.file_name().unwrap());
                        }
                    }
                }
            }
            MemoryEntryType::HardLink(_) => {}
        }
        inner.files.remove(path);
        Ok(())
    } else {
        Err(Error::new(
            ErrorKind::NotFound,
            format!("Path '{}' does not exist", path.display()),
        ))
    }
}

fn change_path_recursive(
    inner: &mut MemoryFsInner,
    from: &Path,
    to: &Path,
    subpath: &Path,
) -> crate::Result<()> {
    let from_path = from.join(subpath);
    let to_path = to.join(subpath);

    if let Some(mut entry) = inner.files.remove(&from_path) {
        match &entry.file_type {
            MemoryEntryType::Directory(files) => {
                let files = files.clone();
                for file_name in files.iter() {
                    let new_subpath = subpath.join(file_name);
                    change_path_recursive(inner, from, to, &new_subpath)?;
                }
            }
            MemoryEntryType::File(_) | MemoryEntryType::HardLink(_) => {
                entry.accessed = Some(SystemTime::now());
                entry.modified = Some(SystemTime::now());
            }
        }
        inner.files.insert(to_path, entry);
        Ok(())
    } else {
        Err(Error::new(
            ErrorKind::NotFound,
            format!("Path '{}' does not exist", from_path.display()),
        ))
    }
}

fn canonicalize<P: AsRef<Path>>(inner: &MemoryFsInner, path: P) -> crate::Result<PathBuf> {
    canonicalize_inner(inner, path, true)
}

fn copy<P: AsRef<Path>, Q: AsRef<Path>>(
    inner: &mut MemoryFsInner,
    from: P,
    to: Q,
) -> crate::Result<u64> {
    let from = canonicalize_inner(inner, from, true)?;
    let to = canonicalize_inner(inner, to, true)?;

    let from_entry = inner.files.get(&from).ok_or_else(|| {
        Error::new(
            ErrorKind::NotFound,
            format!("Source path '{}' does not exist", from.display()),
        )
    })?;

    let from_filetype = from_entry.file_type.to_owned();

    if let MemoryEntryType::File(data) = from_filetype {
        let data = data.read();
        let new_entry = MemoryEntry {
            file_type: MemoryEntryType::File(Arc::new(RwLock::new(data.clone()))),
            created: SystemTime::now(),
            modified: Some(SystemTime::now()),
            accessed: None,
            permissions: from_entry.permissions.clone(),
        };

        if let (Some(from_parent), Some(to_parent)) = (from.parent(), to.parent()) {
            if !inner.files.contains_key(from_parent) {
                return Err(Error::new(
                    ErrorKind::NotFound,
                    format!(
                        "Parent directory '{}' does not exist",
                        from_parent.display()
                    ),
                ));
            }

            if let Some(to_parent_entry) = inner.files.get_mut(to_parent) {
                if let MemoryEntryType::Directory(files) = &mut to_parent_entry.file_type {
                    files.insert(to.file_name().unwrap().to_os_string());
                } else {
                    return Err(Error::new(
                        ErrorKind::InvalidInput,
                        format!("Parent '{}' is not a directory", to_parent.display()),
                    ));
                }
            } else {
                return Err(Error::new(
                    ErrorKind::NotFound,
                    format!(
                        "Destination parent directory '{}' does not exist",
                        to_parent.display()
                    ),
                ));
            }
        }

        inner.files.insert(to, new_entry);
        Ok(data.len() as u64)
    } else {
        Err(Error::new(
            ErrorKind::InvalidInput,
            "Source path is not a file",
        ))
    }
}

fn create_dir<P: AsRef<Path>>(inner: &mut MemoryFsInner, path: P) -> crate::Result<()> {
    let path = canonicalize_inner(inner, path, false)?;

    if inner.files.contains_key(&path) {
        return Err(Error::new(
            ErrorKind::AlreadyExists,
            format!("Directory '{}' already exists", path.display()),
        ));
    }

    if let Some(parent) = path.parent() {
        if !inner.files.contains_key(parent) {
            return Err(Error::new(
                ErrorKind::NotFound,
                format!("Parent directory '{}' does not exist", parent.display()),
            ));
        }

        if let Some(parent_entry) = inner.files.get_mut(parent) {
            if let MemoryEntryType::Directory(files) = &mut parent_entry.file_type {
                files.insert(path.file_name().unwrap().to_os_string());
            } else {
                return Err(Error::new(
                    ErrorKind::InvalidInput,
                    format!("Parent '{}' is not a directory", parent.display()),
                ));
            }
        }
    }

    let new_entry = MemoryEntry {
        file_type: MemoryEntryType::Directory(HashSet::new()),
        created: SystemTime::now(),
        modified: Some(SystemTime::now()),
        accessed: None,
        permissions: Permissions { readonly: false },
    };
    inner.files.insert(path, new_entry);
    Ok(())
}

fn exists<P: AsRef<Path>>(inner: &MemoryFsInner, path: P) -> crate::Result<bool> {
    let path = canonicalize_inner(inner, path, true)?;
    Ok(inner.files.contains_key(&path))
}

fn hard_link<P: AsRef<Path>, Q: AsRef<Path>>(
    inner: &mut MemoryFsInner,
    original: P,
    link: Q,
) -> crate::Result<()> {
    let original = canonicalize_inner(inner, original, true)?;
    let link = canonicalize_inner(inner, link, false)?;

    if !inner.files.contains_key(&original) {
        return Err(Error::new(
            ErrorKind::NotFound,
            format!("Original path '{}' does not exist", original.display()),
        ));
    }

    if inner.files.contains_key(&link) {
        return Err(Error::new(
            ErrorKind::AlreadyExists,
            format!("Link path '{}' already exists", link.display()),
        ));
    }

    let link_parent = link.parent().ok_or_else(|| {
        Error::new(
            ErrorKind::InvalidInput,
            "Link path must have a parent directory",
        )
    })?;

    if !is_dir(inner, link_parent)? {
        return Err(Error::new(
            ErrorKind::NotFound,
            format!("Parent directory for '{}' does not exist", link.display()),
        ));
    }

    let new_entry = MemoryEntry {
        file_type: MemoryEntryType::HardLink(original.clone()),
        created: SystemTime::now(),
        modified: Some(SystemTime::now()),
        accessed: None,
        permissions: Permissions { readonly: false },
    };

    inner
        .files
        .get_mut(link_parent)
        .expect("Parent directory should exist")
        .file_type
        .as_directory_mut()
        .expect("Parent should be a directory")
        .insert(link.file_name().unwrap().to_os_string());

    inner.files.insert(link, new_entry);

    Ok(())
}

fn metadata<P: AsRef<Path>>(inner: &MemoryFsInner, path: P) -> crate::Result<MemoryMetadata> {
    let path = canonicalize_inner(inner, path, true)?;

    if let Some(entry) = inner.files.get(&path) {
        Ok(entry.metadata())
    } else {
        Err(Error::new(
            ErrorKind::NotFound,
            format!("Path '{}' does not exist", path.display()),
        ))
    }
}

fn read<P: AsRef<Path>>(inner: &MemoryFsInner, path: P) -> crate::Result<Vec<u8>> {
    let path = canonicalize_inner(inner, path, true)?;

    if let Some(entry) = inner.files.get(&path) {
        if let MemoryEntryType::File(data) = &entry.file_type {
            Ok(data.read().clone())
        } else {
            Err(Error::new(
                ErrorKind::InvalidInput,
                format!("Path '{}' is not a file", path.display()),
            ))
        }
    } else {
        Err(Error::new(
            ErrorKind::NotFound,
            format!("Path '{}' does not exist", path.display()),
        ))
    }
}

fn read_dir<P: AsRef<Path>>(inner: &MemoryFsInner, path: P) -> crate::Result<MemoryReadDir> {
    let path = canonicalize_inner(inner, path, true)?;

    if let Some(entry) = inner.files.get(&path) {
        if let MemoryEntryType::Directory(files) = &entry.file_type {
            let mut entries = files.iter().cloned().collect::<Vec<_>>();
            entries.sort();
            let entries = entries
                .into_iter()
                .map(|file_name| {
                    let path = path.join(&file_name);
                    let file_entry = inner.files.get(&path).ok_or_else(|| {
                        Error::new(
                            ErrorKind::NotFound,
                            format!("File '{}' does not exist", path.display()),
                        )
                    })?;
                    let metadata = Ok(file_entry.metadata());
                    Ok(MemoryDirEntry {
                        file_name,
                        path,
                        metadata,
                        file_type: Ok(file_entry.file_type.clone().into()),
                    })
                })
                .collect();
            Ok(MemoryReadDir { entries })
        } else {
            Err(Error::new(
                ErrorKind::InvalidInput,
                format!("Path '{}' is not a directory", path.display()),
            ))
        }
    } else {
        Err(Error::new(
            ErrorKind::NotFound,
            format!("Path '{}' does not exist", path.display()),
        ))
    }
}

fn read_link<P: AsRef<Path>>(_path: P) -> crate::Result<PathBuf> {
    Err(Error::new(
        ErrorKind::Unsupported,
        "MemoryFs does not support symbolic links",
    ))
}

fn read_to_string<P: AsRef<Path>>(inner: &MemoryFsInner, path: P) -> crate::Result<String> {
    let bytes = read(inner, path)?;
    String::from_utf8(bytes).map_err(|e| {
        Error::new(
            ErrorKind::InvalidData,
            format!("Failed to convert bytes to string: {}", e),
        )
    })
}

fn remove_dir<P: AsRef<Path>>(inner: &mut MemoryFsInner, path: P) -> crate::Result<()> {
    let path = canonicalize_inner(inner, path, true)?;

    if let Some(entry) = inner.files.get(&path) {
        if let MemoryEntryType::Directory(files) = &entry.file_type {
            if files.is_empty() {
                let parent = path.parent().ok_or_else(|| {
                    Error::new(ErrorKind::InvalidInput, "Cannot remove root directory")
                })?;
                if let Some(parent_entry) = inner.files.get_mut(parent) {
                    if let MemoryEntryType::Directory(files) = &mut parent_entry.file_type {
                        files.remove(path.file_name().unwrap());
                    } else {
                        return Err(Error::new(
                            ErrorKind::InvalidInput,
                            format!("Parent '{}' is not a directory", parent.display()),
                        ));
                    }
                }
                inner.files.remove(&path);
                Ok(())
            } else {
                Err(Error::new(
                    ErrorKind::DirectoryNotEmpty,
                    format!("Directory '{}' is not empty", path.display()),
                ))
            }
        } else {
            Err(Error::new(
                ErrorKind::InvalidInput,
                format!("Path '{}' is not a directory", path.display()),
            ))
        }
    } else {
        Err(Error::new(
            ErrorKind::NotFound,
            format!("Path '{}' does not exist", path.display()),
        ))
    }
}

fn remove_dir_all<P: AsRef<Path>>(inner: &mut MemoryFsInner, path: P) -> crate::Result<()> {
    let path = canonicalize_inner(inner, path, true)?;

    if let Some(entry) = inner.files.get(&path) {
        if let MemoryEntryType::Directory(files) = &entry.file_type {
            let files = files.clone();
            for file_name in files.iter() {
                let file_path = path.join(file_name);
                remove_recursive(&file_path, inner)?;
            }
            let parent = path.parent().ok_or_else(|| {
                Error::new(ErrorKind::InvalidInput, "Cannot remove root directory")
            })?;
            if let Some(parent_entry) = inner.files.get_mut(parent) {
                if let MemoryEntryType::Directory(files) = &mut parent_entry.file_type {
                    files.remove(path.file_name().unwrap());
                } else {
                    return Err(Error::new(
                        ErrorKind::InvalidInput,
                        format!("Parent '{}' is not a directory", parent.display()),
                    ));
                }
            }
            inner.files.remove(&path);
            Ok(())
        } else {
            Err(Error::new(
                ErrorKind::InvalidInput,
                format!("Path '{}' is not a directory", path.display()),
            ))
        }
    } else {
        Err(Error::new(
            ErrorKind::NotFound,
            format!("Path '{}' does not exist", path.display()),
        ))
    }
}

fn remove_file<P: AsRef<Path>>(inner: &mut MemoryFsInner, path: P) -> crate::Result<()> {
    let path = canonicalize_inner(inner, path, true)?;

    if let Some(entry) = inner.files.get(&path) {
        if let MemoryEntryType::File(_) = entry.file_type {
            if let Some(parent) = path.parent() {
                if let Some(parent_entry) = inner.files.get_mut(parent) {
                    if let Some(files) = parent_entry.file_type.as_directory_mut() {
                        files.remove(path.file_name().unwrap());
                    }
                }
            }

            inner.files.remove(&path);
            Ok(())
        } else {
            Err(Error::new(
                ErrorKind::InvalidInput,
                format!("Path '{}' is not a file", path.display()),
            ))
        }
    } else {
        Err(Error::new(
            ErrorKind::NotFound,
            format!("Path '{}' does not exist", path.display()),
        ))
    }
}

fn rename<P: AsRef<Path>, Q: AsRef<Path>>(
    inner: &mut MemoryFsInner,
    from: P,
    to: Q,
) -> crate::Result<()> {
    let from = canonicalize_inner(inner, from, true)?;
    let to = canonicalize_inner(inner, to, false)?;

    if !inner.files.contains_key(&from) {
        return Err(Error::new(
            ErrorKind::NotFound,
            format!("Source path '{}' does not exist", from.display()),
        ));
    }

    if let Some(entry) = inner.files.get(&to) {
        if let MemoryEntryType::Directory(_) = entry.file_type {
            return Err(Error::new(
                ErrorKind::AlreadyExists,
                format!("Destination path '{}' is a directory", to.display()),
            ));
        }
    }

    let from_parent = from.parent();
    let to_parent = to.parent();

    if let Some(mut entry) = inner.files.remove(&from) {
        match &entry.file_type {
            MemoryEntryType::Directory(files) => {
                for file_name in files.iter() {
                    change_path_recursive(inner, &from, &to, Path::new(file_name))?;
                }
            }
            MemoryEntryType::File(_) | MemoryEntryType::HardLink(_) => {}
        }

        if let (Some(from_parent), Some(to_parent)) = (from_parent, to_parent) {
            if from_parent != to_parent {
                if let Some(from_entry) = dbg!(inner.files.get_mut(from_parent)) {
                    if let Some(files) = from_entry.file_type.as_directory_mut() {
                        files.remove(from.file_name().unwrap());
                    }
                }
                if let Some(to_entry) = dbg!(inner.files.get_mut(to_parent)) {
                    if let Some(files) = to_entry.file_type.as_directory_mut() {
                        files.insert(to.file_name().unwrap().to_owned());
                    }
                }
            }
        }

        entry.accessed = Some(SystemTime::now());
        entry.modified = Some(SystemTime::now());

        inner.files.insert(to, entry);
    }

    Ok(())
}

fn set_permissions<P: AsRef<Path>>(
    inner: &mut MemoryFsInner,
    path: P,
    perm: Permissions,
) -> crate::Result<()> {
    let path = canonicalize_inner(inner, path, true)?;

    if let Some(entry) = inner.files.get_mut(&path) {
        entry.permissions = perm;
        entry.modified = Some(SystemTime::now());
        Ok(())
    } else {
        Err(Error::new(
            ErrorKind::NotFound,
            format!("Path '{}' does not exist", path.display()),
        ))
    }
}

fn symlink_metadata<P: AsRef<Path>>(_path: P) -> crate::Result<MemoryMetadata> {
    Err(Error::new(
        ErrorKind::Unsupported,
        "MemoryFs does not support symbolic links",
    ))
}

impl UniFs for MemoryFs {
    type Metadata = MemoryMetadata;
    type ReadDir = MemoryReadDir;
    type DirEntry = MemoryDirEntry;
    type Permissions = Permissions;
    type File = MemoryFile;
    type OpenOptions = MemoryOpenOptions;
    type DirBuilder = MemoryDirBuilder;

    fn canonicalize<P: AsRef<Path>>(&self, path: P) -> crate::Result<PathBuf> {
        let inner = self.inner.read();
        canonicalize(&inner, path)
    }

    fn copy<P: AsRef<Path>, Q: AsRef<Path>>(&self, from: P, to: Q) -> crate::Result<u64> {
        let mut inner = self.inner.write();
        copy(&mut inner, from, to)
    }

    fn create_dir<P: AsRef<Path>>(&self, path: P) -> crate::Result<()> {
        let mut inner = self.inner.write();
        create_dir(&mut inner, path)
    }

    fn exists<P: AsRef<Path>>(&self, path: P) -> crate::Result<bool> {
        let inner = self.inner.read();
        exists(&inner, path)
    }

    fn hard_link<P: AsRef<Path>, Q: AsRef<Path>>(&self, original: P, link: Q) -> crate::Result<()> {
        let mut inner = self.inner.write();
        hard_link(&mut inner, original, link)
    }

    fn metadata<P: AsRef<Path>>(&self, path: P) -> crate::Result<Self::Metadata> {
        let inner = self.inner.read();
        metadata(&inner, path)
    }

    fn read<P: AsRef<Path>>(&self, path: P) -> crate::Result<Vec<u8>> {
        let inner = self.inner.read();
        read(&inner, path)
    }

    fn read_dir<P: AsRef<Path>>(&self, path: P) -> crate::Result<Self::ReadDir> {
        let inner = self.inner.read();
        read_dir(&inner, path)
    }

    fn read_link<P: AsRef<Path>>(&self, path: P) -> crate::Result<PathBuf> {
        read_link(path)
    }

    fn read_to_string<P: AsRef<Path>>(&self, path: P) -> crate::Result<String> {
        let inner = self.inner.read();
        read_to_string(&inner, path)
    }

    fn remove_dir<P: AsRef<Path>>(&self, path: P) -> crate::Result<()> {
        let mut inner = self.inner.write();
        remove_dir(&mut inner, path)
    }

    fn remove_dir_all<P: AsRef<Path>>(&self, path: P) -> crate::Result<()> {
        let mut inner = self.inner.write();
        remove_dir_all(&mut inner, path)
    }

    fn remove_file<P: AsRef<Path>>(&self, path: P) -> crate::Result<()> {
        let mut inner = self.inner.write();
        remove_file(&mut inner, path)
    }

    fn rename<P: AsRef<Path>, Q: AsRef<Path>>(&self, from: P, to: Q) -> crate::Result<()> {
        let mut inner = self.inner.write();
        rename(&mut inner, from, to)
    }

    fn set_permissions<P: AsRef<Path>>(
        &self,
        path: P,
        perm: Self::Permissions,
    ) -> crate::Result<()> {
        let mut inner = self.inner.write();
        set_permissions(&mut inner, path, perm)
    }

    fn symlink_metadata<P: AsRef<Path>>(&self, path: P) -> crate::Result<Self::Metadata> {
        symlink_metadata(path)
    }

    fn new_openoptions(&self) -> Self::OpenOptions {
        let fs = MemoryFs {
            inner: self.inner.clone(),
        };

        MemoryOpenOptions::new(fs)
    }

    fn new_dirbuilder(&self) -> Self::DirBuilder {
        let fs = MemoryFs {
            inner: self.inner.clone(),
        };

        MemoryDirBuilder::new(fs)
    }
}

pub struct MemoryReadDir {
    entries: VecDeque<crate::Result<MemoryDirEntry>>,
}

impl Iterator for MemoryReadDir {
    type Item = crate::Result<MemoryDirEntry>;

    fn next(&mut self) -> Option<Self::Item> {
        self.entries.pop_front()
    }
}

pub struct MemoryDirEntry {
    file_name: OsString,
    path: PathBuf,
    metadata: crate::Result<MemoryMetadata>,
    file_type: crate::Result<crate::FileType>,
}

impl UniDirEntry for MemoryDirEntry {
    type Metadata = MemoryMetadata;
    type FileType = crate::FileType;

    fn file_name(&self) -> OsString {
        self.file_name.clone()
    }

    fn path(&self) -> PathBuf {
        self.path.clone()
    }

    fn metadata(&self) -> crate::Result<Self::Metadata> {
        match &self.metadata {
            Ok(metadata) => Ok(metadata.clone()),
            Err(ref e) => Err(Error::new(
                e.kind(),
                format!("Failed to get metadata: {}", e),
            )),
        }
    }

    fn file_type(&self) -> crate::Result<Self::FileType> {
        match self.file_type {
            Ok(file_type) => Ok(file_type),
            Err(ref e) => Err(Error::new(
                e.kind(),
                format!("Failed to get file type: {}", e),
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_canonicalize() {
        let fs = MemoryFs::new();
        let path = fs.canonicalize("/foo/../bar/./baz").unwrap();
        assert_eq!(path, PathBuf::from("/bar/baz"));

        let path = fs.canonicalize("foo/../../bar/./baz");
        assert!(path.is_err(), "Expected error for invalid path");

        fs.hard_link("/", "/link").unwrap();
        let link_path = fs.canonicalize("/link").unwrap();
        assert_eq!(link_path, PathBuf::from("/"));

        let path = fs.canonicalize("test").unwrap();
        assert_eq!(path, PathBuf::from("/test"));
    }
}
