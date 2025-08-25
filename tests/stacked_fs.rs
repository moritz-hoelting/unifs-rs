use std::{collections::HashSet, ffi::OsString};

use unifs::{MemoryFs, StackedFs, UniDirEntry as _, UniFs as _, UniMetadata as _};

#[test]
fn general_test() -> unifs::Result<()> {
    let base = MemoryFs::default();
    let overlay = MemoryFs::default();

    let fs = StackedFs::new(&base, &overlay, "/stacked");

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

    fs.remove_dir_all("test2")?;
    assert!(!fs.exists("test2")?);
    assert!(fs.read_dir("/test2").is_err());

    // Test overlay shadowing

    fs.create_dir("/stacked/test")?;
    assert!(fs.exists("/stacked/test")?);
    assert!(overlay.exists("test")?);
    assert!(!base.exists("/stacked/test")?);

    fs.create_dir_all("/stacked/test/sub/dir")?;
    assert!(fs.exists("/stacked/test/sub/dir")?);
    assert!(overlay.exists("test/sub/dir")?);
    assert!(!base.exists("/stacked/test/sub/dir")?);

    fs.create_new_file("/stacked/test/file.txt")?;
    assert!(fs.exists("/stacked/test/file.txt")?);
    assert!(overlay.exists("test/file.txt")?);
    assert!(!base.exists("/stacked/test/file.txt")?);

    fs.write("/stacked/test/file.txt", b"Hello, World!")?;
    let content = fs.read("/stacked/test/file.txt")?;
    assert_eq!(content, b"Hello, World!");
    assert!(base.read("/stacked/test/file.txt").is_err());
    assert_eq!(overlay.read("test/file.txt")?, b"Hello, World!");

    let content = fs.read_to_string("/stacked/test/file.txt")?;
    assert_eq!(content, "Hello, World!");
    assert!(base.read_to_string("/stacked/test/file.txt").is_err());
    assert_eq!(overlay.read_to_string("test/file.txt")?, "Hello, World!");

    fs.create_dir("/test")?;
    fs.copy("/stacked/test/file.txt", "/test/copy-stacked.txt")?;
    let copy_content = fs.read("test/copy-stacked.txt")?;
    assert_eq!(copy_content, b"Hello, World!");
    fs.copy("/stacked/test/file.txt", "/stacked/test/copy.txt")?;
    let copy_content = fs.read("/stacked/test/copy.txt")?;
    assert_eq!(copy_content, b"Hello, World!");

    let directory_files = fs
        .read_dir("/stacked/test")?
        .flat_map(|entry| match entry {
            Ok(e) => Some(e.file_name()),
            Err(_) => None,
        })
        .collect::<HashSet<_>>();
    assert_eq!(
        directory_files,
        HashSet::<OsString>::from(["file.txt".into(), "copy.txt".into(), "sub".into()])
    );

    fs.remove_file("/stacked/test/copy.txt")?;
    assert!(!fs.exists("/stacked/test/copy.txt")?);

    assert!(fs.metadata("/stacked/test/copy.txt").is_err());
    let metadata = fs.metadata("/stacked/test/file.txt")?;
    assert!(metadata.is_file());
    assert!(!metadata.is_dir());

    fs.rename("/stacked/test", "/stacked/test2")?;
    assert!(!fs.exists("/stacked/test")?);
    assert!(fs.exists("/stacked/test2")?);

    let dir_metadata = fs.metadata("/stacked/test2/sub/dir")?;
    assert!(dir_metadata.is_dir());
    assert!(!dir_metadata.is_file());

    assert!(fs.remove_dir("/stacked/test2/sub").is_err());
    fs.remove_dir("/stacked/test2/sub/dir")?;
    assert!(!fs.exists("/stacked/test2/sub/dir")?);

    fs.remove_dir_all("/stacked/test2")?;
    assert!(!fs.exists("/stacked/test2")?);
    assert!(fs.read_dir("/test2").is_err());

    Ok(())
}
