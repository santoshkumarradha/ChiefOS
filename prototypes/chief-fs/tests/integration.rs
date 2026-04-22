use chief_fs::aliases::AliasStore;
use chief_fs::cas::CasStore;
use std::fs;
use tempfile::TempDir;

#[test]
fn cas_roundtrip() {
    let temp = TempDir::new().expect("temp dir");
    let cas = CasStore::open(temp.path()).expect("open cas");

    let content = b"hello, world";
    let stat = cas.put_bytes(content).expect("put bytes");

    // Get the content back
    let retrieved = cas.get(&stat.cid).expect("get");
    assert_eq!(retrieved, content);

    // Stat should work
    let stat_again = cas.stat(&stat.cid).expect("stat").expect("stat exists");
    assert_eq!(stat_again.cid, stat.cid);
    assert_eq!(stat_again.bytes, content.len() as u64);
}

#[test]
fn cas_determinism() {
    let temp = TempDir::new().expect("temp dir");
    let cas = CasStore::open(temp.path()).expect("open cas");

    let content = b"determinism test";
    let cid1 = cas.put_bytes(content).expect("put 1").cid;
    let cid2 = cas.put_bytes(content).expect("put 2").cid;

    // Same content must produce same CID
    assert_eq!(cid1, cid2);
}

#[test]
fn cas_different_content() {
    let temp = TempDir::new().expect("temp dir");
    let cas = CasStore::open(temp.path()).expect("open cas");

    let content1 = b"content one";
    let content2 = b"content two";

    let cid1 = cas.put_bytes(content1).expect("put 1").cid;
    let cid2 = cas.put_bytes(content2).expect("put 2").cid;

    // Different content must produce different CIDs
    assert_ne!(cid1, cid2);
}

#[test]
fn aliases_basic() {
    let temp = TempDir::new().expect("temp dir");
    let db_path = temp.path().join("aliases.sqlite");
    let mut aliases = AliasStore::open(&db_path).expect("open aliases");

    let cid = "a".repeat(64);
    let record = aliases.set_alias("my-file.txt", &cid).expect("set alias");

    assert_eq!(record.path, "my-file.txt");
    assert_eq!(record.cid, cid);

    // Get it back
    let retrieved = aliases
        .get_alias("my-file.txt")
        .expect("get alias")
        .expect("alias exists");
    assert_eq!(retrieved.cid, cid);
}

#[test]
fn aliases_rename_preserves_cid() {
    let temp = TempDir::new().expect("temp dir");
    let db_path = temp.path().join("aliases.sqlite");
    let mut aliases = AliasStore::open(&db_path).expect("open aliases");

    let cid = "b".repeat(64);
    let original = aliases
        .set_alias("original.txt", &cid)
        .expect("set original");
    assert_eq!(original.cid, cid);

    // Update alias to new path with same CID (simulating a rename)
    let updated = aliases.set_alias("renamed.txt", &cid).expect("set renamed");
    assert_eq!(updated.cid, cid);

    // Both should exist and point to same CID
    let orig_get = aliases
        .get_alias("original.txt")
        .expect("get original")
        .expect("original exists");
    assert_eq!(orig_get.cid, cid);

    let rename_get = aliases
        .get_alias("renamed.txt")
        .expect("get renamed")
        .expect("renamed exists");
    assert_eq!(rename_get.cid, cid);
}

#[test]
fn cas_stat_missing() {
    let temp = TempDir::new().expect("temp dir");
    let cas = CasStore::open(temp.path()).expect("open cas");

    let missing_cid = "0".repeat(64);
    let result = cas.stat(&missing_cid).expect("stat missing");
    assert!(result.is_none());
}

#[test]
fn cid_format_validation() {
    // Valid CID
    let valid = "a".repeat(64);
    assert!(chief_fs::cas::validate_cid(&valid).is_ok());

    // Too short
    let short = "a".repeat(63);
    assert!(chief_fs::cas::validate_cid(&short).is_err());

    // Too long
    let long = "a".repeat(65);
    assert!(chief_fs::cas::validate_cid(&long).is_err());

    // Non-hex characters
    let invalid = "g".repeat(64);
    assert!(chief_fs::cas::validate_cid(&invalid).is_err());

    // Uppercase (must be lowercase)
    let uppercase = "A".repeat(64);
    assert!(chief_fs::cas::validate_cid(&uppercase).is_err());
}

#[test]
fn cas_put_file_path() {
    let temp = TempDir::new().expect("temp dir");
    let cas = CasStore::open(temp.path()).expect("open cas");

    let test_file = temp.path().join("test.txt");
    fs::write(&test_file, b"test content").expect("write test file");

    let stat = cas.put_path(&test_file).expect("put path");
    let retrieved = cas.get(&stat.cid).expect("get");
    assert_eq!(retrieved, b"test content");
}
