//! Relation parsing and a provides-aware, cycle-safe dependency closure.
//!
//! Resolution rules (a read-only model of pacman's, no installed set):
//! - A relation names a package or a provision, with an optional
//!   `<`, `<=`, `=`, `>=`, `>` version constraint.
//! - Candidates are exact-name packages first, then providers. Within each
//!   group, repositories earlier in the input order win, as in pacman.
//! - A provision without a version does not satisfy a versioned constraint.
//! - A relation already satisfied by a visited package is not re-expanded,
//!   so cycles terminate.
//! - Optional dependencies are never followed.

use std::cmp::Ordering;
use std::collections::{HashMap, HashSet, VecDeque};

use crate::vercmp::vercmp;
use crate::Package;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Op {
    Lt,
    Le,
    Eq,
    Ge,
    Gt,
}

impl Op {
    fn accepts(self, ord: Ordering) -> bool {
        match self {
            Op::Lt => ord == Ordering::Less,
            Op::Le => ord != Ordering::Greater,
            Op::Eq => ord == Ordering::Equal,
            Op::Ge => ord != Ordering::Less,
            Op::Gt => ord == Ordering::Greater,
        }
    }
}

/// A parsed `name[op version]` relation, from `%DEPENDS%` or `%PROVIDES%`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Relation {
    pub name: String,
    pub constraint: Option<(Op, String)>,
}

/// Parses `name`, `name=1.0`, `name>=2:3-1`, `lib:libx.so`, `libc.so=6-64`.
///
/// Description suffixes (`name: text`) are only valid in `%OPTDEPENDS%` and
/// are not stripped here.
pub fn parse_relation(input: &str) -> Relation {
    let name_end = input.find(['<', '>', '=']).unwrap_or(input.len());
    let name = input[..name_end].to_owned();
    let rest = &input[name_end..];
    let constraint = if rest.is_empty() {
        None
    } else {
        let (op, len) = if rest.starts_with("<=") {
            (Op::Le, 2)
        } else if rest.starts_with(">=") {
            (Op::Ge, 2)
        } else if rest.starts_with('<') {
            (Op::Lt, 1)
        } else if rest.starts_with('>') {
            (Op::Gt, 1)
        } else {
            (Op::Eq, 1)
        };
        Some((op, rest[len..].to_owned()))
    };
    Relation { name, constraint }
}

/// Lookup tables over a set of repositories, in priority order.
#[derive(Debug)]
pub struct Index<'a> {
    packages: Vec<&'a Package>,
    by_name: HashMap<&'a str, Vec<usize>>,
    /// provision name -> (package index, provided version if any)
    providers: HashMap<String, Vec<(usize, Option<String>)>>,
}

impl<'a> Index<'a> {
    /// `repos` is searched in the given order; earlier entries win ties.
    pub fn new<I>(packages: I) -> Self
    where
        I: IntoIterator<Item = &'a Package>,
    {
        let packages: Vec<&Package> = packages.into_iter().collect();
        let mut by_name: HashMap<&str, Vec<usize>> = HashMap::new();
        let mut providers: HashMap<String, Vec<(usize, Option<String>)>> = HashMap::new();
        for (idx, pkg) in packages.iter().enumerate() {
            by_name.entry(pkg.name.as_str()).or_default().push(idx);
            // A package always provides itself by name, with its own version.
            for provision in &pkg.provides {
                let rel = parse_relation(provision);
                let version = rel.constraint.map(|(_, v)| v);
                providers.entry(rel.name).or_default().push((idx, version));
            }
        }
        Index {
            packages,
            by_name,
            providers,
        }
    }

    /// Indices of exact-name packages, then providers, each in priority order.
    fn candidates(&self, rel: &Relation) -> Vec<usize> {
        let mut out: Vec<usize> = Vec::new();
        if let Some(idx) = self.by_name.get(rel.name.as_str()) {
            for &i in idx {
                if satisfies_package(self.packages[i], rel) {
                    out.push(i);
                }
            }
        }
        if let Some(provs) = self.providers.get(&rel.name) {
            for (i, provided) in provs {
                if out.contains(i) {
                    continue;
                }
                if satisfies_provision(provided.as_deref(), rel) {
                    out.push(*i);
                }
            }
        }
        out
    }

    /// The package with this exact name, earliest repository wins.
    pub fn by_exact_name(&self, name: &str) -> Option<&'a Package> {
        self.by_name
            .get(name)
            .and_then(|v| v.first())
            .map(|&i| self.packages[i])
    }

    /// All packages, in priority order.
    pub fn packages(&self) -> &[&'a Package] {
        &self.packages
    }

    /// Packages whose exact name equals `name` or that provide it.
    pub fn named_or_provided(&self, name: &str) -> Vec<&'a Package> {
        let rel = Relation {
            name: name.to_owned(),
            constraint: None,
        };
        self.candidates(&rel)
            .into_iter()
            .map(|i| self.packages[i])
            .collect()
    }
}

fn satisfies_package(pkg: &Package, rel: &Relation) -> bool {
    match &rel.constraint {
        None => true,
        Some((op, want)) => op.accepts(vercmp(&pkg.version, want)),
    }
}

fn satisfies_provision(provided: Option<&str>, rel: &Relation) -> bool {
    match (&rel.constraint, provided) {
        (None, _) => true,
        (Some(_), None) => false,
        (Some((op, want)), Some(have)) => op.accepts(vercmp(have, want)),
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClosureEntry {
    pub name: String,
    pub version: String,
    pub repo: String,
    /// The relation string that first pulled this package in.
    pub via: String,
}

/// Result of [`closure`].
#[derive(Debug, Default)]
pub struct Closure {
    /// Root first, then breadth-first order.
    pub entries: Vec<ClosureEntry>,
    /// `(from package, relation)` pairs with no candidate in the index.
    pub unresolved: Vec<(String, String)>,
    /// Number of dependency edges examined.
    pub edges: usize,
}

/// Transitive closure of runtime `%DEPENDS%` from `root`.
///
/// Returns `None` when neither an exact-name package nor a provider of `root`
/// exists in the index.
pub fn closure(index: &Index<'_>, root: &str) -> Option<Closure> {
    let start = index
        .candidates(&Relation {
            name: root.to_owned(),
            constraint: None,
        })
        .into_iter()
        .min_by_key(|&i| (index.packages()[i].name != root) as u8)?;

    let mut out = Closure::default();
    let mut visited: HashSet<usize> = HashSet::new();
    let mut queue: VecDeque<usize> = VecDeque::new();

    visited.insert(start);
    let pkg = index.packages()[start];
    out.entries.push(ClosureEntry {
        name: pkg.name.clone(),
        version: pkg.version.clone(),
        repo: pkg.repo.clone(),
        via: "(root)".into(),
    });
    queue.push_back(start);

    while let Some(current) = queue.pop_front() {
        let pkg = index.packages()[current];
        for dep in &pkg.depends {
            out.edges += 1;
            let candidates = index.candidates(&parse_relation(dep));
            // A candidate already in the closure satisfies the relation, so
            // nothing is added twice and cycles stop here.
            if candidates.iter().any(|i| visited.contains(i)) {
                continue;
            }
            match candidates.first() {
                Some(&next) => {
                    visited.insert(next);
                    let next_pkg = index.packages()[next];
                    out.entries.push(ClosureEntry {
                        name: next_pkg.name.clone(),
                        version: next_pkg.version.clone(),
                        repo: next_pkg.repo.clone(),
                        via: dep.clone(),
                    });
                    queue.push_back(next);
                }
                None => out.unresolved.push((pkg.name.clone(), dep.clone())),
            }
        }
    }
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pkg(repo: &str, name: &str, version: &str, provides: &[&str], depends: &[&str]) -> Package {
        Package {
            repo: repo.into(),
            name: name.into(),
            version: version.into(),
            provides: provides.iter().map(|s| s.to_string()).collect(),
            depends: depends.iter().map(|s| s.to_string()).collect(),
            ..Package::default()
        }
    }

    #[test]
    fn relation_parsing() {
        assert_eq!(parse_relation("glibc").constraint, None);
        assert_eq!(
            parse_relation("libcurl.so=4-64"),
            Relation {
                name: "libcurl.so".into(),
                constraint: Some((Op::Eq, "4-64".into()))
            }
        );
        assert_eq!(
            parse_relation("python>=3.12"),
            Relation {
                name: "python".into(),
                constraint: Some((Op::Ge, "3.12".into()))
            }
        );
        assert_eq!(parse_relation("x<2").constraint, Some((Op::Lt, "2".into())));
        assert_eq!(parse_relation("x<=2").constraint, Some((Op::Le, "2".into())));
        assert_eq!(parse_relation("x>2").constraint, Some((Op::Gt, "2".into())));
        assert_eq!(parse_relation("lib:libx.so").name, "lib:libx.so");
    }

    #[test]
    fn closure_is_provides_aware_and_cycle_safe() {
        let pkgs = vec![
            pkg("core", "app", "1.0-1", &[], &["libfoo.so=1-64", "bar"]),
            pkg("core", "libfoo", "2.0-1", &["libfoo.so=1-64"], &["app"]),
            pkg("core", "bar", "0.1-1", &[], &["missing-thing"]),
        ];
        let index = Index::new(pkgs.iter());
        let c = closure(&index, "app").unwrap();
        let names: Vec<_> = c.entries.iter().map(|e| e.name.as_str()).collect();
        assert_eq!(names, vec!["app", "libfoo", "bar"]);
        assert_eq!(c.unresolved, vec![("bar".into(), "missing-thing".into())]);
        assert_eq!(c.entries[1].via, "libfoo.so=1-64");
    }

    #[test]
    fn versioned_constraint_filters_candidates() {
        let pkgs = vec![
            pkg("core", "root", "1-1", &[], &["lib>=2"]),
            pkg("core", "lib", "1.9-1", &[], &[]),
            pkg("extra", "lib", "2.1-1", &[], &[]),
        ];
        let index = Index::new(pkgs.iter());
        let c = closure(&index, "root").unwrap();
        assert_eq!(c.entries[1].repo, "extra");
        assert_eq!(c.entries[1].version, "2.1-1");
    }

    #[test]
    fn earlier_repo_wins_for_equal_candidates() {
        let pkgs = vec![
            pkg("core", "root", "1-1", &[], &["lib"]),
            pkg("core", "lib", "1-1", &[], &[]),
            pkg("cachyos", "lib", "9-1", &[], &[]),
        ];
        let index = Index::new(pkgs.iter());
        let c = closure(&index, "root").unwrap();
        assert_eq!(c.entries[1].repo, "core");
    }

    #[test]
    fn unversioned_provision_does_not_satisfy_versioned_constraint() {
        let pkgs = vec![
            pkg("core", "root", "1-1", &[], &["virt>=1"]),
            pkg("core", "impl", "1-1", &["virt"], &[]),
        ];
        let index = Index::new(pkgs.iter());
        let c = closure(&index, "root").unwrap();
        assert_eq!(c.unresolved.len(), 1);
    }

    #[test]
    fn missing_root_is_none() {
        let pkgs: Vec<Package> = vec![];
        let index = Index::new(pkgs.iter());
        assert!(closure(&index, "nope").is_none());
    }
}
