use std::{path::Path, sync::Arc, time::SystemTime};

use crate::{
    memory_fs::{
        file::MemoryFile, metadata::MemoryMetadata, MemoryEntry, MemoryEntryType, MemoryFs,
    },
    rw_lock::RwLock,
    FileType, UniOpenOptions,
};

pub struct MemoryOpenOptions {
    fs: MemoryFs,

    read: bool,
    write: bool,
    append: bool,
    truncate: bool,
    create: bool,
    create_new: bool,
}

impl MemoryOpenOptions {
    pub(crate) fn new(fs: MemoryFs) -> Self {
        Self {
            fs,
            read: false,
            write: false,
            append: false,
            truncate: false,
            create: false,
            create_new: false,
        }
    }
}

impl UniOpenOptions for MemoryOpenOptions {
    type File = MemoryFile;

    fn read(&mut self, read: bool) -> &mut Self {
        self.read = read;
        self
    }

    fn write(&mut self, write: bool) -> &mut Self {
        self.write = write;
        self
    }

    fn append(&mut self, append: bool) -> &mut Self {
        self.write |= append;
        self.append = append;
        self
    }

    fn truncate(&mut self, truncate: bool) -> &mut Self {
        self.truncate = truncate;
        self
    }

    fn create(&mut self, create: bool) -> &mut Self {
        self.create = create;
        self
    }

    fn create_new(&mut self, create_new: bool) -> &mut Self {
        self.create |= create_new;
        self.create_new = create_new;
        self
    }

    fn open<P: AsRef<Path>>(&self, path: P) -> crate::Result<Self::File> {
        let mut inner = self.fs.inner.write();
        let path = super::canonicalize_inner(&inner, path, true)?;

        if self.create_new && super::exists(&inner, &path)? {
            return Err(std::io::Error::new(
                std::io::ErrorKind::AlreadyExists,
                "File already exists",
            ));
        }

        if !self.create && !super::exists(&inner, &path)? {
            return Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "File not found",
            ));
        }

        if let Some(entry) = inner.files.get(&path) {
            match &entry.file_type {
                MemoryEntryType::Directory(_) => Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    "Cannot open a directory as a file",
                )),
                MemoryEntryType::HardLink(_) => Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    "Cannot open a symlink as a file",
                )),
                MemoryEntryType::File(data) => {
                    if self.truncate {
                        let mut data = data.write();
                        data.clear();
                        data.shrink_to_fit();
                    }
                    Ok(MemoryFile::new(
                        path,
                        data.clone(),
                        entry.metadata(),
                        self.write,
                        self.append,
                    ))
                }
            }
        } else {
            if !self.create || !self.write {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "File not found",
                ));
            }

            let metadata = MemoryMetadata {
                file_type: FileType::File,
                permissions: crate::Permissions { readonly: false },
                file_times: Default::default(),
                len: 0,
            };
            let data = Arc::new(RwLock::new(Vec::new()));
            let file_type = MemoryEntryType::File(data.clone());

            let entry = MemoryEntry {
                accessed: None,
                created: SystemTime::now(),
                modified: None,
                file_type,
                permissions: metadata.permissions.clone(),
            };

            let parent = path.parent().ok_or_else(|| {
                std::io::Error::new(std::io::ErrorKind::InvalidInput, "No parent path")
            })?;
            if let Some(parent_entry) = inner.files.get_mut(parent) {
                if let MemoryEntryType::Directory(files) = &mut parent_entry.file_type {
                    files.insert(path.file_name().unwrap().to_os_string());
                } else {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidInput,
                        "Parent is not a directory",
                    ));
                }
            } else {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "Parent directory not found",
                ));
            }

            inner.files.insert(path.clone(), entry);

            Ok(MemoryFile::new(
                path,
                data,
                metadata,
                self.write,
                self.append,
            ))
        }
    }
}
