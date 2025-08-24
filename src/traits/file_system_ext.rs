use std::path::Path;

use crate::{UniDirEntry, UniFileType, UniFs};

/// Extends the `UniFs` trait with additional methods for filesystem operations.
pub trait UniFsExt: UniFs {
    /// Recursively walks through the directory at the specified path,
    /// yielding each directory entry found.
    fn walk_dir<'a, P>(
        &'a self,
        path: P,
    ) -> impl Iterator<Item = crate::Result<Self::DirEntry>> + 'a
    where
        P: AsRef<Path>,
        Self: Sized,
    {
        WalkDirIterator::new(self, path.as_ref())
    }
}

impl<T: UniFs> UniFsExt for T {}

struct WalkDirIterator<'a, F: UniFs> {
    fs: &'a F,
    stack: Vec<F::DirEntry>,
    error: Option<std::io::Error>,
}

impl<'a, F: UniFs> WalkDirIterator<'a, F> {
    fn new(fs: &'a F, path: &Path) -> Self {
        let mut stack = Vec::new();
        if let Ok(entries) = fs.read_dir(path) {
            for entry in entries {
                match entry {
                    Ok(e) => stack.push(e),
                    Err(err) => {
                        return Self {
                            fs,
                            stack: Vec::new(),
                            error: Some(err),
                        };
                    }
                }
            }
        }
        Self {
            fs,
            stack,
            error: None,
        }
    }
}

impl<'a, F> Iterator for WalkDirIterator<'a, F>
where
    F: UniFs,
{
    type Item = crate::Result<F::DirEntry>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(err) = std::mem::take(&mut self.error) {
            self.stack.clear();
            self.stack.shrink_to_fit();
            return Some(Err(err));
        }

        if let Some(entry) = self.stack.pop() {
            match entry.file_type() {
                Ok(file_type) => {
                    if file_type.is_dir() {
                        if let Ok(entries) = self.fs.read_dir(entry.path()) {
                            for e in entries {
                                match e {
                                    Ok(e) => {
                                        self.stack.push(e);
                                    }
                                    Err(err) => {
                                        self.stack.clear();
                                        self.stack.shrink_to_fit();
                                        return Some(Err(err));
                                    }
                                }
                            }
                        }
                    }
                    Some(Ok(entry))
                }
                Err(err) => {
                    self.stack.clear();
                    self.stack.shrink_to_fit();
                    Some(Err(err))
                }
            }
        } else {
            None
        }
    }
}
