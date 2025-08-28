use std::{collections::HashSet, ffi::OsString};

use unifs::{MemoryFs, UniDirEntry, UniFs as _, UniMetadata};

#[test]
fn general_test() -> unifs::Result<()> {
    let fs = MemoryFs::default();
    fs.create_dir("/test")?;
    assert!(fs.exists("test")?);

    fs.create_dir_all("/test/sub/dir")?;
    assert!(fs.exists("test/sub/dir")?);

    fs.create_new_file("/test/file.txt")?;
    assert!(fs.exists("test/file.txt")?);

    fs.write("test/file.txt", b"Hello, World!")?;
    let content = fs.read("test/file.txt")?;
    assert_eq!(content, b"Hello, World!");

    let content = fs.read_to_string("test/file.txt")?;
    assert_eq!(content, "Hello, World!");

    fs.copy("/test/file.txt", "test/copy.txt")?;
    let copy_content = fs.read("test/copy.txt")?;
    assert_eq!(copy_content, b"Hello, World!");

    let directory_files = fs
        .read_dir("/test")?
        .flat_map(|entry| match entry {
            Ok(e) => Some(e.file_name()),
            Err(_) => None,
        })
        .collect::<HashSet<_>>();
    assert_eq!(
        directory_files,
        HashSet::<OsString>::from(["file.txt".into(), "copy.txt".into(), "sub".into()])
    );

    fs.remove_file("test/copy.txt")?;
    assert!(!fs.exists("test/copy.txt")?);

    assert!(fs.metadata("test/copy.txt").is_err());
    let metadata = fs.metadata("test/file.txt")?;
    assert!(metadata.is_file());
    assert!(!metadata.is_dir());

    fs.rename("test", "test2")?;
    assert!(!fs.exists("test")?);
    assert!(fs.exists("test2")?);

    let dir_metadata = fs.metadata("test2/sub/dir")?;
    assert!(dir_metadata.is_dir());
    assert!(!dir_metadata.is_file());

    assert!(fs.remove_dir("test2/sub").is_err());
    fs.remove_dir("test2/sub/dir")?;
    assert!(!fs.exists("test2/sub/dir")?);

    let loaded_fs = MemoryFs::load_from_dir(&fs, "test2")?;
    assert!(loaded_fs.exists("file.txt")?);

    fs.remove_dir_all("test2")?;
    assert!(!fs.exists("test2")?);
    assert!(fs.read_dir("/test2").is_err());

    Ok(())
}
