//! Pure-Rust, read-only reader for pacman sync databases.
//!
//! Scope: parse `<repo>.db` / `<repo>.files` archives (gzip, zstd or plain
//! tar), their `desc` and `files` members, version comparison identical to
//! pacman's `vercmp`, and a provides-aware, cycle-safe dependency closure.
//! No installation, no transactions, no signature verification, no libalpm.

#![forbid(unsafe_code)]

use std::fmt;

mod db;
mod deps;
mod desc;
mod vercmp;

pub use db::{read_repo_db, RepoDb};
pub use deps::{closure, parse_relation, Closure, Index, Op, Relation};
pub use desc::{parse_desc, parse_files};
pub use vercmp::{rpmvercmp, vercmp};

/// One package entry from a sync database.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Package {
    /// Name of the section the database was read from (`core`, `extra`, ...).
    pub repo: String,
    pub filename: String,
    pub name: String,
    pub base: String,
    /// Full `[epoch:]pkgver-pkgrel` version.
    pub version: String,
    pub desc: String,
    pub groups: Vec<String>,
    /// Compressed package size in bytes (`%CSIZE%`).
    pub csize: u64,
    /// Installed size in bytes (`%ISIZE%`).
    pub isize: u64,
    /// Present only in `desc` v1 entries.
    pub md5sum: Option<String>,
    pub sha256sum: Option<String>,
    /// True when `%PGPSIG%` is present. The signature is not verified.
    pub pgpsig: bool,
    pub url: String,
    pub license: Vec<String>,
    pub arch: String,
    /// Unix time.
    pub builddate: u64,
    pub packager: String,
    pub replaces: Vec<String>,
    pub conflicts: Vec<String>,
    pub provides: Vec<String>,
    pub depends: Vec<String>,
    pub optdepends: Vec<String>,
    pub makedepends: Vec<String>,
    pub checkdepends: Vec<String>,
    /// Filled only when the database is a `.files` database.
    pub files: Vec<String>,
}

#[derive(Debug)]
pub enum Error {
    /// The archive or a member could not be read.
    Archive(String),
    /// A `desc` member is malformed.
    Desc(String),
    /// The compression format is recognised but not implemented in pure Rust.
    Unsupported(&'static str),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Archive(msg) => write!(f, "archive: {msg}"),
            Error::Desc(msg) => write!(f, "desc: {msg}"),
            Error::Unsupported(what) => write!(f, "unsupported: {what}"),
        }
    }
}

impl std::error::Error for Error {}
