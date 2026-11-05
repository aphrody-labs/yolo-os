//! `yolo-pm`: read-only pacman sync-database tool.
//!
//! Subcommands: `sync`, `search`, `info`, `deps`, `stats`. Nothing is installed,
//! removed or written outside the cache directory.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::Instant;

use regex::Regex;
use serde_json::{json, Value};
use yolo_alpm::{closure, read_repo_db, Index, Package, RepoDb};

mod fetch;

/// Repositories searched by default, in priority order (earlier wins ties).
const DEFAULT_REPOS: &[(&str, &str)] = &[
    ("core", "https://geo.mirror.pkgbuild.com/core/os/x86_64/core.db"),
    ("extra", "https://geo.mirror.pkgbuild.com/extra/os/x86_64/extra.db"),
    ("cachyos", "https://mirror.cachyos.org/repo/x86_64/cachyos/cachyos.db"),
];

struct Opts {
    cache: PathBuf,
    json: bool,
    repos: Vec<String>,
    rest: Vec<String>,
}

fn usage() -> &'static str {
    "usage: yolo-pm [--cache DIR] [--repo NAME]... [--json] <command>\n\
     \n\
     commands:\n\
     \x20 sync            download sync databases into the cache\n\
     \x20 search <re>     regex search over package names and descriptions\n\
     \x20 info <pkg>      show one package (first repo in priority order wins)\n\
     \x20 deps <pkg>      transitive runtime dependency closure (read-only)\n\
     \x20 stats           packages parsed per repo, parse timings\n\
     \n\
     default repos: core extra cachyos (priority order)"
}

fn parse_args() -> Result<Opts, String> {
    let mut args = std::env::args().skip(1);
    let mut opts = Opts {
        cache: default_cache(),
        json: false,
        repos: Vec::new(),
        rest: Vec::new(),
    };
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--cache" => {
                opts.cache = PathBuf::from(args.next().ok_or("--cache needs a value")?);
            }
            "--repo" => {
                opts.repos.push(args.next().ok_or("--repo needs a value")?);
            }
            "--json" => opts.json = true,
            "-h" | "--help" => return Err(String::new()),
            _ => opts.rest.push(arg),
        }
    }
    if opts.repos.is_empty() {
        opts.repos = DEFAULT_REPOS.iter().map(|(n, _)| n.to_string()).collect();
    }
    Ok(opts)
}

fn default_cache() -> PathBuf {
    if let Some(dir) = std::env::var_os("XDG_CACHE_HOME") {
        return PathBuf::from(dir).join("yolo-pm");
    }
    let home = std::env::var_os("HOME").unwrap_or_else(|| ".".into());
    PathBuf::from(home).join(".cache").join("yolo-pm")
}

fn repo_url(name: &str) -> Option<&'static str> {
    DEFAULT_REPOS.iter().find(|(n, _)| *n == name).map(|(_, u)| *u)
}

fn main() -> ExitCode {
    let opts = match parse_args() {
        Ok(o) => o,
        Err(msg) => {
            if !msg.is_empty() {
                eprintln!("yolo-pm: {msg}");
            }
            eprintln!("{}", usage());
            return ExitCode::from(2);
        }
    };
    let Some(command) = opts.rest.first().cloned() else {
        eprintln!("{}", usage());
        return ExitCode::from(2);
    };
    let result = match command.as_str() {
        "sync" => cmd_sync(&opts),
        "search" => cmd_search(&opts),
        "info" => cmd_info(&opts),
        "deps" => cmd_deps(&opts),
        "stats" => cmd_stats(&opts),
        other => Err(format!("unknown command: {other}")),
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(msg) => {
            eprintln!("yolo-pm: {msg}");
            ExitCode::FAILURE
        }
    }
}

fn db_path(cache: &Path, repo: &str) -> PathBuf {
    cache.join(format!("{repo}.db"))
}

/// Loads every configured repo from the cache, in priority order.
fn load_repos(opts: &Opts) -> Result<Vec<(RepoDb, f64)>, String> {
    let mut out = Vec::new();
    for repo in &opts.repos {
        let path = db_path(&opts.cache, repo);
        let bytes = fs::read(&path).map_err(|e| {
            format!("{}: {e} (run `yolo-pm sync` first)", path.display())
        })?;
        let start = Instant::now();
        let db = read_repo_db(repo, &bytes).map_err(|e| format!("{repo}: {e}"))?;
        out.push((db, start.elapsed().as_secs_f64() * 1000.0));
    }
    Ok(out)
}

fn cmd_sync(opts: &Opts) -> Result<(), String> {
    fs::create_dir_all(&opts.cache).map_err(|e| format!("{}: {e}", opts.cache.display()))?;
    let mut records = Vec::new();
    for repo in &opts.repos {
        let url = repo_url(repo).ok_or_else(|| {
            format!("no default URL for repo {repo}; only core, extra, cachyos are built in")
        })?;
        let start = Instant::now();
        let bytes = fetch::get(url)?;
        let elapsed = start.elapsed().as_secs_f64() * 1000.0;

        let path = db_path(&opts.cache, repo);
        let tmp = path.with_extension("db.partial");
        fs::write(&tmp, &bytes).map_err(|e| format!("{}: {e}", tmp.display()))?;
        fs::rename(&tmp, &path).map_err(|e| format!("{}: {e}", path.display()))?;

        // Detached signature of the database, if the mirror serves one.
        // Presence is recorded; cryptographic verification is not done here.
        let sig_url = format!("{url}.sig");
        let sig = fetch::head_status(&sig_url);

        records.push(json!({
            "repo": repo,
            "url": url,
            "bytes": bytes.len(),
            "sha256": fetch::sha256_hex(&bytes),
            "fetch_ms": elapsed,
            "db_sig_status": sig,
        }));
        if !opts.json {
            println!(
                "{repo:<8} {:>10} bytes  {:>8.0} ms  sig:{sig}  sha256:{}",
                bytes.len(),
                elapsed,
                fetch::sha256_hex(&bytes)
            );
        }
    }
    fs::write(
        opts.cache.join("sync.json"),
        serde_json::to_string_pretty(&records).map_err(|e| e.to_string())?,
    )
    .map_err(|e| format!("sync.json: {e}"))?;
    if opts.json {
        println!("{}", serde_json::to_string_pretty(&records).map_err(|e| e.to_string())?);
    }
    Ok(())
}

fn cmd_stats(opts: &Opts) -> Result<(), String> {
    let repos = load_repos(opts)?;
    let mut rows = Vec::new();
    for (db, ms) in &repos {
        let with_sig = db.packages.iter().filter(|p| p.pgpsig).count();
        let with_md5 = db.packages.iter().filter(|p| p.md5sum.is_some()).count();
        rows.push(json!({
            "repo": db.name,
            "packages": db.packages.len(),
            "compression": db.compression,
            "files_db": db.has_files,
            "pgpsig": with_sig,
            "md5sum_v1": with_md5,
            "unknown_keys": db.unknown_keys,
            "parse_ms": ms,
        }));
    }
    if opts.json {
        println!("{}", serde_json::to_string_pretty(&rows).map_err(|e| e.to_string())?);
    } else {
        println!(
            "{:<8} {:>9} {:<6} {:>7} {:>8} {:>6} {:>8} {:>10}",
            "repo", "packages", "comp", "pgpsig", "md5(v1)", "unk", "files", "parse_ms"
        );
        for (db, ms) in &repos {
            println!(
                "{:<8} {:>9} {:<6} {:>7} {:>8} {:>6} {:>8} {:>10.1}",
                db.name,
                db.packages.len(),
                db.compression,
                db.packages.iter().filter(|p| p.pgpsig).count(),
                db.packages.iter().filter(|p| p.md5sum.is_some()).count(),
                db.unknown_keys,
                db.has_files,
                ms
            );
        }
    }
    Ok(())
}

fn cmd_search(opts: &Opts) -> Result<(), String> {
    let pattern = opts.rest.get(1).ok_or("search needs a regex")?;
    let re = Regex::new(pattern).map_err(|e| format!("bad regex: {e}"))?;
    let repos = load_repos(opts)?;
    let mut hits: Vec<Value> = Vec::new();
    let mut text = String::new();
    for (db, _) in &repos {
        for pkg in &db.packages {
            if re.is_match(&pkg.name) || re.is_match(&pkg.desc) {
                hits.push(json!({
                    "repo": pkg.repo,
                    "name": pkg.name,
                    "version": pkg.version,
                    "desc": pkg.desc,
                }));
                text.push_str(&format!("{}/{} {}\n    {}\n", pkg.repo, pkg.name, pkg.version, pkg.desc));
            }
        }
    }
    if opts.json {
        println!("{}", serde_json::to_string_pretty(&hits).map_err(|e| e.to_string())?);
    } else {
        print!("{text}");
        eprintln!("{} match(es)", hits.len());
    }
    Ok(())
}

/// Index over every loaded repo, keeping the database pages alive.
fn with_index<T>(
    opts: &Opts,
    f: impl FnOnce(&Index<'_>) -> Result<T, String>,
) -> Result<T, String> {
    let repos = load_repos(opts)?;
    let all: Vec<&Package> = repos.iter().flat_map(|(db, _)| db.packages.iter()).collect();
    let index = Index::new(all);
    f(&index)
}

fn cmd_info(opts: &Opts) -> Result<(), String> {
    let name = opts.rest.get(1).ok_or("info needs a package name")?;
    with_index(opts, |index| {
        let pkg = index
            .by_exact_name(name)
            .ok_or_else(|| format!("package not found: {name}"))?;
        if opts.json {
            println!("{}", serde_json::to_string_pretty(&pkg_json(pkg)).map_err(|e| e.to_string())?);
        } else {
            print_pkg(pkg);
        }
        Ok(())
    })
}

fn pkg_json(p: &Package) -> Value {
    json!({
        "repo": p.repo,
        "name": p.name,
        "base": p.base,
        "version": p.version,
        "arch": p.arch,
        "desc": p.desc,
        "url": p.url,
        "license": p.license,
        "groups": p.groups,
        "csize": p.csize,
        "isize": p.isize,
        "builddate": p.builddate,
        "packager": p.packager,
        "sha256sum": p.sha256sum,
        "pgpsig": p.pgpsig,
        "filename": p.filename,
        "provides": p.provides,
        "depends": p.depends,
        "optdepends": p.optdepends,
        "conflicts": p.conflicts,
        "replaces": p.replaces,
    })
}

fn print_pkg(p: &Package) {
    println!("Repository   : {}", p.repo);
    println!("Name         : {}", p.name);
    println!("Version      : {}", p.version);
    println!("Architecture : {}", p.arch);
    println!("Description  : {}", p.desc);
    println!("URL          : {}", p.url);
    println!("Licenses     : {}", p.license.join(", "));
    println!("Groups       : {}", p.groups.join(", "));
    println!("Download size: {} bytes", p.csize);
    println!("Installed    : {} bytes", p.isize);
    println!("Build date   : {}", p.builddate);
    println!("Packager     : {}", p.packager);
    println!("SHA-256      : {}", p.sha256sum.as_deref().unwrap_or("-"));
    println!("PGP sig      : {}", if p.pgpsig { "present (not verified)" } else { "absent" });
    println!("Provides     : {}", p.provides.join("  "));
    println!("Depends On   : {}", p.depends.join("  "));
    println!("Optional     : {}", p.optdepends.join("  "));
    println!("Conflicts    : {}", p.conflicts.join("  "));
    println!("Replaces     : {}", p.replaces.join("  "));
}

fn cmd_deps(opts: &Opts) -> Result<(), String> {
    let name = opts.rest.get(1).ok_or("deps needs a package name")?;
    with_index(opts, |index| {
        let c = closure(index, name).ok_or_else(|| format!("package not found: {name}"))?;
        if opts.json {
            let entries: Vec<Value> = c
                .entries
                .iter()
                .map(|e| json!({"name": e.name, "version": e.version, "repo": e.repo, "via": e.via}))
                .collect();
            let unresolved: Vec<Value> = c
                .unresolved
                .iter()
                .map(|(from, dep)| json!({"from": from, "relation": dep}))
                .collect();
            let out = json!({
                "root": name,
                "count": c.entries.len(),
                "edges": c.edges,
                "entries": entries,
                "unresolved": unresolved,
            });
            println!("{}", serde_json::to_string_pretty(&out).map_err(|e| e.to_string())?);
        } else {
            for e in &c.entries {
                println!("{:<24} {:<24} {:<8} via {}", e.name, e.version, e.repo, e.via);
            }
            println!(
                "{} package(s), {} edge(s), {} unresolved",
                c.entries.len(),
                c.edges,
                c.unresolved.len()
            );
            for (from, dep) in &c.unresolved {
                println!("  unresolved: {dep} (required by {from})");
            }
        }
        Ok(())
    })
}
