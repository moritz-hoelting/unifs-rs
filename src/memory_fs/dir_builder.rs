use std::{io::Error, path::Path};

use crate::{memory_fs::MemoryFs, UniDirBuilder};

pub struct MemoryDirBuilder {
    fs: MemoryFs,
    recursive: bool,
}

impl MemoryDirBuilder {
    pub(super) fn new(fs: MemoryFs) -> Self {
        Self {
            fs,
            recursive: false,
        }
    }
}

impl UniDirBuilder for MemoryDirBuilder {
    fn recursive(&mut self, recursive: bool) -> &mut Self {
        self.recursive = recursive;
        self
    }

    fn create<P: AsRef<Path>>(&self, path: P) -> crate::Result<()> {
        let mut inner = self.fs.inner.write();
        let path = super::canonicalize_inner(&inner, path, true)?;

        if super::exists(&inner, &path)? {
            if self.recursive {
                Ok(())
            } else {
                Err(Error::new(
                    std::io::ErrorKind::AlreadyExists,
                    format!("Directory already exists: {}", path.display()),
                ))
            }
        } else {
            if self.recursive {
                let mut parts = Vec::new();
                let mut current = path.as_path();

                while !super::exists(&inner, current)? {
                    if let Some(parent) = current.parent() {
                        parts.push(
                            current
                                .file_name()
                                .expect("path has parent and was canonicalized"),
                        );
                        current = parent;
                    } else {
                        break;
                    }
                }

                if parts.is_empty() {
                    return Err(Error::new(
                        std::io::ErrorKind::InvalidInput,
                        "Cannot create directory at root",
                    ));
                }

                let mut current = current.to_path_buf();
                for part in parts.into_iter().rev() {
                    current.push(part);
                    if !super::exists(&inner, &current)? {
                        super::create_dir(&mut inner, &current)?;
                    }
                }
            } else {
                super::create_dir(&mut inner, &path)?;
            }

            Ok(())
        }
    }
}
