//! Container reader for `<repo>.db` and `<repo>.files` archives.
//!
//! Compression is detected from magic bytes, not from the file name: Arch
//! mirrors serve gzip, CachyOS serves zstd. xz and bzip2 are recognised but
//! refused, because the xz package in extra is the only one needing xz and
//! the pure-Rust options are stale; see the receipts.

use std::collections::BTreeMap;
use std::io::{Cursor, Read};

use crate::desc::{parse_desc, parse_files};
use crate::{Error, Package};

/// A parsed sync database.
#[derive(Debug, Default)]
pub struct RepoDb {
    pub name: String,
    pub packages: Vec<Package>,
    /// `%KEY%` headers pacman does not know; pacman warns and ignores them.
    pub unknown_keys: usize,
    /// True when the archive carried `files` members.
    pub has_files: bool,
    /// Compression detected from magic bytes: `gzip`, `zstd` or `tar`.
    pub compression: &'static str,
}

/// Parses a whole database file held in memory.
pub fn read_repo_db(name: &str, bytes: &[u8]) -> Result<RepoDb, Error> {
    let (compression, tar_bytes): (&'static str, Vec<u8>) = if bytes.starts_with(&[0x1f, 0x8b]) {
        let mut out = Vec::new();
        flate2::read::GzDecoder::new(bytes)
            .read_to_end(&mut out)
            .map_err(|e| Error::Archive(format!("gzip: {e}")))?;
        ("gzip", out)
    } else if bytes.starts_with(&[0x28, 0xb5, 0x2f, 0xfd]) {
        let mut decoder = ruzstd::decoding::StreamingDecoder::new(bytes)
            .map_err(|e| Error::Archive(format!("zstd: {e}")))?;
        let mut out = Vec::new();
        decoder
            .read_to_end(&mut out)
            .map_err(|e| Error::Archive(format!("zstd: {e}")))?;
        ("zstd", out)
    } else if bytes.starts_with(&[0xfd, b'7', b'z', b'X', b'Z', 0x00]) {
        return Err(Error::Unsupported("xz"));
    } else if bytes.starts_with(b"BZh") {
        return Err(Error::Unsupported("bzip2"));
    } else {
        ("tar", bytes.to_vec())
    };

    let mut descs: BTreeMap<String, String> = BTreeMap::new();
    let mut files: BTreeMap<String, String> = BTreeMap::new();

    let mut archive = tar::Archive::new(Cursor::new(tar_bytes));
    let entries = archive
        .entries()
        .map_err(|e| Error::Archive(format!("tar: {e}")))?;
    for entry in entries {
        let mut entry = entry.map_err(|e| Error::Archive(format!("tar entry: {e}")))?;
        if !entry.header().entry_type().is_file() {
            continue;
        }
        let path = entry
            .path()
            .map_err(|e| Error::Archive(format!("tar path: {e}")))?
            .to_string_lossy()
            .into_owned();
        let Some((dir, member)) = path.split_once('/') else {
            continue;
        };
        // Only one level of nesting is valid: `<name>-<version>/<member>`.
        if member.contains('/') {
            continue;
        }
        let target = match member {
            "desc" => &mut descs,
            "files" => &mut files,
            _ => continue,
        };
        let mut text = String::new();
        entry
            .read_to_string(&mut text)
            .map_err(|e| Error::Archive(format!("tar member {path}: {e}")))?;
        target.insert(dir.to_owned(), text);
    }

    let has_files = !files.is_empty();
    let mut db = RepoDb {
        name: name.to_owned(),
        compression,
        has_files,
        ..RepoDb::default()
    };
    for (dir, text) in &descs {
        let (mut pkg, unknown) = parse_desc(name, text)
            .map_err(|e| Error::Desc(format!("{dir}: {e}")))?;
        db.unknown_keys += unknown;
        if let Some(list) = files.get(dir) {
            pkg.files = parse_files(list);
        }
        db.packages.push(pkg);
    }
    Ok(db)
}
