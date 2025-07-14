#![cfg_attr(docsrs, feature(doc_auto_cfg))]

mod traits;

#[cfg(feature = "fs_access")]
mod physical_fs;

pub mod readonly_fs;

#[doc(inline)]
pub use traits::{
    dir_builder::UniDirBuilder, file::UniFile, file_system::UniFs, open_options::UniOpenOptions,
    UniDirEntry, UniFileTimes, UniFileType, UniMetadata, UniPermissions,
};

#[doc(inline)]
#[cfg(feature = "fs_access")]
pub use physical_fs::PhysicalFs;

pub use readonly_fs::ReadonlyFs;

pub type Result<T> = std::result::Result<T, std::io::Error>;
