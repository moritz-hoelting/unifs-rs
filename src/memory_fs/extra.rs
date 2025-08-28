use std::path::Path;

use crate::{MemoryFs, UniDirEntry as _, UniFileType as _, UniFs, UniFsExt as _, UniMetadata as _};

impl MemoryFs {
    /// Load the contents of a directory from any filesystem implementing `UniFs`
    /// into a new `MemoryFs` instance.
    ///
    /// # Errors
    /// - if any I/O operation fails during the loading process.
    pub fn load_from_dir(fs: impl UniFs, path: impl AsRef<Path>) -> crate::Result<Self> {
        let path = path.as_ref();
        let canon_path = fs.canonicalize(path)?;
        let memory_fs = MemoryFs::new();

        for entry in fs.walk_dir(path) {
            let entry = entry?;
            let file_type = entry.metadata()?.file_type();
            let entry_path = entry.path();
            let copy_path = entry_path
                .strip_prefix(&canon_path)
                .map_err(|_| std::io::Error::other("failed stripping path prefix"))?;
            if file_type.is_file() {
                let mut original = fs.open_file(&entry_path)?;
                let mut copy = memory_fs.create_file(copy_path)?;
                std::io::copy(&mut original, &mut copy)?;
            } else if file_type.is_dir() {
                memory_fs.create_dir(copy_path)?;
            } else if file_type.is_symlink() {
                return Err(std::io::Error::other("symlink not supported"));
            }
        }

        Ok(memory_fs)
    }
}
