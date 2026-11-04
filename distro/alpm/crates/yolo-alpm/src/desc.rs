//! Parser for the `desc` and `files` members of a sync database
//! (`alpm-repo-desc` v1 and v2, `alpm-repo-files`).
//!
//! Grammar: a `%KEY%` header line followed by value lines, until the next
//! header or an empty line. Unknown keys are counted, not fatal, matching
//! pacman's warning-only behaviour.

use crate::{Error, Package};

/// Parses one `desc` member into a [`Package`] (without `files`).
///
/// `repo` is the section name the database came from.
pub fn parse_desc(repo: &str, text: &str) -> Result<(Package, usize), Error> {
    let mut pkg = Package {
        repo: repo.to_owned(),
        ..Package::default()
    };
    let mut unknown_keys = 0usize;
    let mut current: Option<&str> = None;
    let mut seen_name = false;

    for raw in text.lines() {
        let line = raw.trim_end_matches('\r');
        if line.is_empty() {
            current = None;
            continue;
        }
        if let Some(key) = header(line) {
            current = Some(key);
            if key == "NAME" {
                seen_name = true;
            }
            if !is_known_key(key) {
                unknown_keys += 1;
                current = None;
            }
            continue;
        }
        let Some(key) = current else {
            // Value line without a header: pacman ignores it too.
            continue;
        };
        apply(&mut pkg, key, line)?;
    }

    if !seen_name || pkg.name.is_empty() {
        return Err(Error::Desc("missing %NAME%".into()));
    }
    Ok((pkg, unknown_keys))
}

/// Parses a `files` member: the `%FILES%` list plus `%BACKUP%` lines
/// (`path<TAB>md5`) are ignored in favour of the file list.
pub fn parse_files(text: &str) -> Vec<String> {
    let mut files = Vec::new();
    let mut in_files = false;
    for line in text.lines() {
        if let Some(key) = header(line) {
            in_files = key == "FILES";
            continue;
        }
        if in_files && !line.is_empty() {
            files.push(line.to_owned());
        }
    }
    files
}

fn header(line: &str) -> Option<&str> {
    let inner = line.strip_prefix('%')?.strip_suffix('%')?;
    if inner.is_empty() || !inner.bytes().all(|b| b.is_ascii_uppercase() || b.is_ascii_digit() || b == b'_') {
        return None;
    }
    Some(inner)
}

fn is_known_key(key: &str) -> bool {
    matches!(
        key,
        "FILENAME"
            | "NAME"
            | "BASE"
            | "VERSION"
            | "DESC"
            | "GROUPS"
            | "CSIZE"
            | "ISIZE"
            | "MD5SUM"
            | "SHA256SUM"
            | "PGPSIG"
            | "URL"
            | "LICENSE"
            | "ARCH"
            | "BUILDDATE"
            | "PACKAGER"
            | "REPLACES"
            | "CONFLICTS"
            | "PROVIDES"
            | "DEPENDS"
            | "OPTDEPENDS"
            | "MAKEDEPENDS"
            | "CHECKDEPENDS"
    )
}

fn apply(pkg: &mut Package, key: &str, value: &str) -> Result<(), Error> {
    let num = |v: &str| -> Result<u64, Error> {
        v.parse()
            .map_err(|_| Error::Desc(format!("%{key}% is not a number: {v:?}")))
    };
    match key {
        "FILENAME" => pkg.filename = value.to_owned(),
        "NAME" => pkg.name = value.to_owned(),
        "BASE" => pkg.base = value.to_owned(),
        "VERSION" => pkg.version = value.to_owned(),
        "DESC" => pkg.desc = value.to_owned(),
        "GROUPS" => pkg.groups.push(value.to_owned()),
        "CSIZE" => pkg.csize = num(value)?,
        "ISIZE" => pkg.isize = num(value)?,
        "MD5SUM" => pkg.md5sum = Some(value.to_owned()),
        "SHA256SUM" => pkg.sha256sum = Some(value.to_owned()),
        // The signature itself is base64 OpenPGP data. Presence only; no
        // cryptographic verification is done by this crate.
        "PGPSIG" => pkg.pgpsig = true,
        "URL" => pkg.url = value.to_owned(),
        "LICENSE" => pkg.license.push(value.to_owned()),
        "ARCH" => pkg.arch = value.to_owned(),
        "BUILDDATE" => pkg.builddate = num(value)?,
        "PACKAGER" => pkg.packager = value.to_owned(),
        "REPLACES" => pkg.replaces.push(value.to_owned()),
        "CONFLICTS" => pkg.conflicts.push(value.to_owned()),
        "PROVIDES" => pkg.provides.push(value.to_owned()),
        "DEPENDS" => pkg.depends.push(value.to_owned()),
        "OPTDEPENDS" => pkg.optdepends.push(value.to_owned()),
        "MAKEDEPENDS" => pkg.makedepends.push(value.to_owned()),
        "CHECKDEPENDS" => pkg.checkdepends.push(value.to_owned()),
        _ => {}
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const DESC: &str = "\
%NAME%
pacman

%VERSION%
7.1.0.r9.g54d9411-3

%BASE%
pacman

%CSIZE%
993348

%ISIZE%
5284710

%SHA256SUM%
0000000000000000000000000000000000000000000000000000000000000000

%PGPSIG%
iQEz

%ARCH%
x86_64

%BUILDDATE%
1792052823

%PROVIDES%
libalpm.so=16-64

%DEPENDS%
libcurl.so=4-64
glibc

%SOMETHING_NEW%
x

%MD5SUM%
abc
";

    #[test]
    fn parses_typed_fields_and_counts_unknown_keys() {
        let (pkg, unknown) = parse_desc("core", DESC).unwrap();
        assert_eq!(pkg.name, "pacman");
        assert_eq!(pkg.version, "7.1.0.r9.g54d9411-3");
        assert_eq!(pkg.repo, "core");
        assert_eq!(pkg.csize, 993_348);
        assert_eq!(pkg.isize, 5_284_710);
        assert_eq!(pkg.builddate, 1_792_052_823);
        assert!(pkg.pgpsig);
        assert_eq!(pkg.md5sum.as_deref(), Some("abc"));
        assert_eq!(pkg.provides, vec!["libalpm.so=16-64"]);
        assert_eq!(pkg.depends, vec!["libcurl.so=4-64", "glibc"]);
        assert_eq!(unknown, 1);
    }

    #[test]
    fn missing_name_is_an_error() {
        assert!(parse_desc("core", "%VERSION%\n1.0-1\n").is_err());
    }

    #[test]
    fn non_numeric_size_is_an_error() {
        assert!(parse_desc("core", "%NAME%\nx\n%CSIZE%\nabc\n").is_err());
    }

    #[test]
    fn files_list_is_root_relative() {
        let files = parse_files("%FILES%\nusr/\nusr/bin/pacman\n\n%BACKUP%\netc/x\tabc\n");
        assert_eq!(files, vec!["usr/", "usr/bin/pacman"]);
    }
}
