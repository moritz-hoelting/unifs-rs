use std::io::{Cursor, Seek, Write};

use zip::{write::FileOptions, ZipWriter};

use crate::{MemoryFs, UniDirEntry as _, UniFileType as _, UniFs as _, UniFsExt as _};

impl MemoryFs {
    /// Write the contents of the filesystem into a zip archive.
    pub fn zip_into<I>(&self, zip_data: I) -> std::io::Result<()>
    where
        I: Write + Seek,
    {
        let mut zip_writer = ZipWriter::new(zip_data);

        for entry in self.walk_dir(".") {
            let entry = entry?;

            let path = entry.path();
            let file_type = entry.file_type()?;

            if file_type.is_file() {
                let data = self.read(&path)?;

                zip_writer
                    .start_file_from_path::<(), _>(&path, FileOptions::default())
                    .map_err(|err| {
                        std::io::Error::other(format!("Failed to start file in zip: {}", err))
                    })?;
                zip_writer.write_all(&data).map_err(|err| {
                    std::io::Error::other(format!("Failed to write file data to zip: {}", err))
                })?;
            } else if file_type.is_dir() {
                zip_writer
                    .add_directory_from_path::<(), _>(&path, FileOptions::default())
                    .map_err(|err| {
                        std::io::Error::other(format!("Failed to add directory to zip: {}", err))
                    })?;
            }
        }

        zip_writer.finish().map_err(|err| {
            std::io::Error::other(format!("Failed to finish zip writing: {}", err))
        })?;

        Ok(())
    }

    /// Create a zip archive from the filesystem and return it as a byte vector.
    pub fn zip(&self) -> std::io::Result<Vec<u8>> {
        let mut buffer = Cursor::new(Vec::new());
        self.zip_into(&mut buffer)?;

        Ok(buffer.into_inner())
    }
}
