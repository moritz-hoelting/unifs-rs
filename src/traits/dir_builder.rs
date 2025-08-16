use std::path::Path;

use crate::Result;

/// A trait for building and creating directories in a filesystem.
pub trait UniDirBuilder {
    /// Indicates that directories should be created recursively,
    /// creating all parent directories. Parents that do not exist
    /// are created with the same security and permissions settings.
    ///
    /// This option defaults to `false`.
    fn recursive(&mut self, recursive: bool) -> &mut Self;

    /// Creates the specified directory with the options configured
    /// in this builder.
    ///
    /// It is considered an error if the directory already exists
    /// unless recursive mode is enabled.
    fn create<P: AsRef<Path>>(&self, path: P) -> Result<()>;
}
