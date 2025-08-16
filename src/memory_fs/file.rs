use std::{
    fmt::Debug,
    io::{Read, Seek, Write},
    path::PathBuf,
    sync::Arc,
};

use crate::{
    memory_fs::metadata::MemoryMetadata, rw_lock::RwLock, FileTimes, Permissions, UniFile,
};

pub struct MemoryFile {
    path: PathBuf,
    inner: Arc<RwLock<MemoryFileInner>>,
    write: bool,
    append: bool,
}

impl MemoryFile {
    pub(super) fn new(
        path: PathBuf,
        data: Arc<RwLock<Vec<u8>>>,
        metadata: MemoryMetadata,
        write: bool,
        append: bool,
    ) -> Self {
        Self {
            path,
            inner: Arc::new(RwLock::new(MemoryFileInner {
                data,
                position: 0,
                metadata,
            })),
            write,
            append,
        }
    }
}

struct MemoryFileInner {
    // The underlying data of the file, stored in memory.
    data: Arc<RwLock<Vec<u8>>>,
    // The current position in the file.
    position: usize,
    /// The file's metadata, such as creation time, modified time, etc.
    metadata: MemoryMetadata,
}

impl Debug for MemoryFile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MemoryFile")
            .field("path", &self.path.display())
            .finish()
    }
}

impl Read for MemoryFile {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let mut inner = self.inner.write();
        let bytes_to_read = {
            let data = inner.data.read();
            if inner.position >= data.len() {
                return Ok(0); // EOF
            }
            let bytes_to_read = std::cmp::min(buf.len(), data.len() - inner.position);
            buf[..bytes_to_read]
                .copy_from_slice(&data[inner.position..inner.position + bytes_to_read]);
            bytes_to_read
        };
        inner.position += bytes_to_read;
        Ok(bytes_to_read)
    }
}

impl Write for MemoryFile {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        if !self.write {
            return Err(std::io::Error::new(
                std::io::ErrorKind::PermissionDenied,
                "File is not open for writing",
            ));
        }

        let mut inner = self.inner.write();
        let bytes_written = {
            if self.append {
                let length = inner.data.read().len();
                inner.position = length;
            }
            let mut data = inner.data.write();
            let position = inner.position;
            if position + buf.len() > data.len() {
                data.resize(position + buf.len(), 0);
            }
            data[position..position + buf.len()].copy_from_slice(buf);

            buf.len()
        };
        inner.position += bytes_written;
        Ok(bytes_written)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

impl Seek for MemoryFile {
    fn seek(&mut self, pos: std::io::SeekFrom) -> std::io::Result<u64> {
        self.append = false;
        let mut inner = self.inner.write();
        let position = {
            let data = inner.data.read();
            match pos {
                std::io::SeekFrom::Start(offset) => offset as usize,
                std::io::SeekFrom::End(offset) => {
                    if (-offset as usize) > data.len() {
                        return Err(std::io::Error::new(
                            std::io::ErrorKind::InvalidInput,
                            "Seek position out of bounds",
                        ));
                    }
                    (data.len() as isize + offset as isize) as usize
                }
                std::io::SeekFrom::Current(offset) => {
                    if (inner.position as i64 + offset) < 0 {
                        return Err(std::io::Error::new(
                            std::io::ErrorKind::InvalidInput,
                            "Seek position out of bounds",
                        ));
                    }
                    (inner.position as i64 + offset) as usize
                }
            }
        };
        inner.position = position;
        Ok(position as u64)
    }
}

impl UniFile for MemoryFile {
    type Metadata = MemoryMetadata;
    type Permissions = Permissions;
    type FileTimes = FileTimes;

    fn sync_all(&self) -> crate::Result<()> {
        Ok(())
    }

    fn sync_data(&self) -> crate::Result<()> {
        Ok(())
    }

    fn set_len(&self, size: u64) -> crate::Result<()> {
        let mut inner = self.inner.write();
        {
            let mut data = inner.data.write();
            data.resize(size as usize, 0);
        }
        inner.metadata.file_times.modified = Some(std::time::SystemTime::now());
        Ok(())
    }

    fn metadata(&self) -> crate::Result<Self::Metadata> {
        let inner = self.inner.read();
        Ok(inner.metadata.clone())
    }

    fn try_clone(&self) -> crate::Result<Self> {
        Ok(Self {
            path: self.path.clone(),
            inner: self.inner.clone(),
            write: self.write,
            append: self.append,
        })
    }

    fn set_permissions(&self, perm: Self::Permissions) -> crate::Result<()> {
        let mut inner = self.inner.write();
        inner.metadata.permissions = perm;
        Ok(())
    }

    fn set_times(&self, times: Self::FileTimes) -> crate::Result<()> {
        let mut inner = self.inner.write();
        inner.metadata.file_times = times;
        Ok(())
    }
}
