use anyhow::Result;
use std::path::Path;

#[cfg(target_os = "linux")]
mod linux {
    use super::Result;
    use crate::aliases::AliasStore;
    use crate::cas::CasStore;
    use anyhow::{anyhow, Context};
    use fuser::{
        FileAttr, FileType, Filesystem, MountOption, ReplyAttr, ReplyData, ReplyDirectory,
        ReplyEntry, ReplyOpen, Request,
    };
    use std::collections::BTreeMap;
    use std::ffi::{OsStr, OsString};
    use std::fs;
    use std::path::{Component, Path};
    use std::time::{Duration, SystemTime};

    const TTL: Duration = Duration::from_secs(1);
    const ROOT_INO: u64 = 1;
    const FILE_PERM: u16 = 0o444;
    const DIR_PERM: u16 = 0o555;

    #[derive(Debug, Clone)]
    struct FileNode {
        ino: u64,
        name: OsString,
        cid: String,
        size: u64,
        modified: SystemTime,
    }

    #[derive(Debug)]
    struct ReadOnlyFs {
        cas: CasStore,
        nodes_by_name: BTreeMap<OsString, FileNode>,
        nodes_by_ino: BTreeMap<u64, FileNode>,
    }

    impl ReadOnlyFs {
        fn new(chief_home: impl AsRef<Path>) -> Result<Self> {
            let chief_home = chief_home.as_ref();
            let cas = CasStore::open(chief_home).context("opening CAS store")?;
            let aliases_path = chief_home.join("aliases.sqlite");
            let aliases = AliasStore::open(&aliases_path).context("opening alias store")?;

            let mut nodes_by_name = BTreeMap::new();
            let mut nodes_by_ino = BTreeMap::new();

            for (index, alias) in aliases.list_aliases()?.into_iter().enumerate() {
                let stat = cas.stat(&alias.cid)?.ok_or_else(|| {
                    anyhow!("alias {} points at missing blob {}", alias.path, alias.cid)
                })?;
                let name = flat_name_for_alias(alias.path.as_str(), &alias.cid);
                let ino = index as u64 + 2;
                let node = FileNode {
                    ino,
                    name: name.clone(),
                    cid: alias.cid,
                    size: stat.bytes,
                    modified: SystemTime::now(),
                };
                nodes_by_ino.insert(ino, node.clone());
                nodes_by_name.insert(name, node);
            }

            Ok(Self {
                cas,
                nodes_by_name,
                nodes_by_ino,
            })
        }

        fn root_attr(&self) -> FileAttr {
            let now = SystemTime::now();
            FileAttr {
                ino: ROOT_INO,
                size: 0,
                blocks: 0,
                atime: now,
                mtime: now,
                ctime: now,
                crtime: now,
                kind: FileType::Directory,
                perm: DIR_PERM,
                nlink: 2 + self.nodes_by_name.len() as u32,
                uid: unsafe { libc::geteuid() },
                gid: unsafe { libc::getegid() },
                rdev: 0,
                blksize: 4096,
                flags: 0,
            }
        }

        fn file_attr(&self, node: &FileNode) -> FileAttr {
            FileAttr {
                ino: node.ino,
                size: node.size,
                blocks: node.size.div_ceil(512),
                atime: node.modified,
                mtime: node.modified,
                ctime: node.modified,
                crtime: node.modified,
                kind: FileType::RegularFile,
                perm: FILE_PERM,
                nlink: 1,
                uid: unsafe { libc::geteuid() },
                gid: unsafe { libc::getegid() },
                rdev: 0,
                blksize: 4096,
                flags: 0,
            }
        }
    }

    impl Filesystem for ReadOnlyFs {
        fn lookup(&mut self, _req: &Request<'_>, parent: u64, name: &OsStr, reply: ReplyEntry) {
            if parent != ROOT_INO {
                reply.error(libc::ENOENT);
                return;
            }

            match self.nodes_by_name.get(name) {
                Some(node) => reply.entry(&TTL, &self.file_attr(node), 0),
                None => reply.error(libc::ENOENT),
            }
        }

        fn getattr(&mut self, _req: &Request<'_>, ino: u64, _fh: Option<u64>, reply: ReplyAttr) {
            if ino == ROOT_INO {
                reply.attr(&TTL, &self.root_attr());
                return;
            }

            match self.nodes_by_ino.get(&ino) {
                Some(node) => reply.attr(&TTL, &self.file_attr(node)),
                None => reply.error(libc::ENOENT),
            }
        }

        fn readdir(
            &mut self,
            _req: &Request<'_>,
            ino: u64,
            _fh: u64,
            offset: i64,
            mut reply: ReplyDirectory,
        ) {
            if ino != ROOT_INO {
                reply.error(libc::ENOENT);
                return;
            }

            let mut entries: Vec<(u64, FileType, OsString)> =
                Vec::with_capacity(2 + self.nodes_by_name.len());
            entries.push((ROOT_INO, FileType::Directory, OsString::from(".")));
            entries.push((ROOT_INO, FileType::Directory, OsString::from("..")));
            entries.extend(
                self.nodes_by_name
                    .values()
                    .map(|node| (node.ino, FileType::RegularFile, node.name.clone())),
            );

            let start = if offset < 0 { 0 } else { offset as usize };
            for (i, entry) in entries.into_iter().enumerate().skip(start) {
                let next_offset = (i + 1) as i64;
                if reply.add(entry.0, next_offset, entry.1, entry.2) {
                    break;
                }
            }
            reply.ok();
        }

        fn open(&mut self, _req: &Request<'_>, ino: u64, flags: i32, reply: ReplyOpen) {
            if flags & libc::O_ACCMODE != libc::O_RDONLY {
                reply.error(libc::EROFS);
                return;
            }

            if ino == ROOT_INO || self.nodes_by_ino.contains_key(&ino) {
                reply.opened(0, 0);
            } else {
                reply.error(libc::ENOENT);
            }
        }

        fn read(
            &mut self,
            _req: &Request<'_>,
            ino: u64,
            _fh: u64,
            offset: i64,
            size: u32,
            reply: ReplyData,
        ) {
            let Some(node) = self.nodes_by_ino.get(&ino) else {
                reply.error(libc::ENOENT);
                return;
            };

            let Ok(bytes) = self.cas.get(&node.cid) else {
                reply.error(libc::EIO);
                return;
            };

            let start = offset.max(0) as usize;
            if start >= bytes.len() {
                reply.data(&[]);
                return;
            }

            let end = bytes.len().min(start + size as usize);
            reply.data(&bytes[start..end]);
        }
    }

    pub fn mount_read_only(
        chief_home: impl AsRef<Path>,
        mountpoint: impl AsRef<Path>,
    ) -> Result<()> {
        let mountpoint = mountpoint.as_ref();
        fs::create_dir_all(mountpoint)
            .with_context(|| format!("creating mountpoint {}", mountpoint.display()))?;

        let fs = ReadOnlyFs::new(chief_home.as_ref())?;
        let options = vec![
            MountOption::RO,
            MountOption::FSName("chief-fs".to_string()),
            MountOption::DefaultPermissions,
        ];
        fuser::mount2(fs, mountpoint, &options)
            .with_context(|| format!("mounting readonly FUSE at {}", mountpoint.display()))?;
        Ok(())
    }

    fn flat_name_for_alias(path: &str, cid: &str) -> OsString {
        let mut encoded = String::new();
        let path = Path::new(path);
        for (index, component) in path.components().enumerate() {
            if index > 0 {
                encoded.push_str("__");
            }
            match component {
                Component::RootDir => encoded.push_str("root"),
                Component::CurDir => encoded.push('.'),
                Component::ParentDir => encoded.push_str("parent"),
                Component::Normal(part) => encode_component(&part.to_string_lossy(), &mut encoded),
                Component::Prefix(prefix) => {
                    encode_component(&prefix.as_os_str().to_string_lossy(), &mut encoded)
                }
            }
        }

        if encoded.is_empty() {
            encoded.push_str("alias");
        }
        encoded.push_str("--");
        encoded.push_str(&cid[..cid.len().min(12)]);
        OsString::from(encoded)
    }

    fn encode_component(value: &str, out: &mut String) {
        for byte in value.bytes() {
            match byte {
                b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                    out.push(byte as char)
                }
                _ => {
                    out.push('%');
                    out.push_str(&format!("{byte:02X}"));
                }
            }
        }
    }
}

#[cfg(target_os = "linux")]
pub use linux::mount_read_only;

#[cfg(not(target_os = "linux"))]
pub fn mount_read_only(_chief_home: impl AsRef<Path>, _mountpoint: impl AsRef<Path>) -> Result<()> {
    Err(anyhow::anyhow!(
        "chief-fs read-only FUSE mount is only supported on Linux in this prototype"
    ))
}
