use anyhow::{Context, Result, bail};
use chrono::{SecondsFormat, Utc};
use clap::{Args, Parser, Subcommand};
use reqwest::blocking::Client;
use reqwest::header::{HeaderMap, HeaderValue, LOCATION, USER_AGENT};
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet, VecDeque};
use std::fs;
use std::io::{Read, Write};
use std::net::IpAddr;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};
use url::Url;

const VERSION: &str = env!("CARGO_PKG_VERSION");
const USER_AGENT_VALUE: &str = "YOLO-LFS-Research/2026 (+https://github.com/yolo-rs/yolo-os)";
const MAX_BODY_BYTES: usize = 2 * 1024 * 1024;

#[derive(Parser, Debug)]
#[command(name = "cargo-lfs", version = VERSION, about = "Build and study Linux From Scratch with source transparency and Rust tooling")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Print the checked-in source registry.
    Sources,
    /// Crawl the registered web sources and save evidence plus a Markdown synthesis.
    Crawl(CrawlArgs),
    /// Rank the registered GitHub organizations and user repositories.
    Repos(RepoArgs),
    /// Record symbolic refs and commit IDs from source Git remotes.
    GitHeads(OutputArgs),
    /// Regenerate Markdown from a saved crawl JSON file.
    Synthesize {
        input: PathBuf,
        #[arg(long)]
        output: Option<PathBuf>,
    },
    /// Retrieve the exact package list and checksums published for an LFS book track.
    Book {
        #[command(subcommand)]
        command: BookCommand,
    },
    /// Operate the official ALFS/jhalfs extraction workflow.
    Alfs {
        #[command(subcommand)]
        command: AlfsCommand,
    },
    /// Inspect a book download manifest without downloading packages.
    Plan {
        #[arg(long)]
        book: PathBuf,
        #[arg(long)]
        sources: Option<PathBuf>,
    },
    /// Fetch the book's package list and check its official checksums.
    Fetch {
        #[arg(long)]
        book: PathBuf,
        #[arg(long)]
        sources: Option<PathBuf>,
        #[arg(long)]
        execute: bool,
    },
    /// Preview or run one reviewed ALFS instruction script with a digest journal.
    Run(RunArgs),
}

#[derive(Args, Debug)]
struct CrawlArgs {
    /// Explicit seed URLs. With none, uses the non-Git entries in sources.json.
    urls: Vec<String>,
    #[arg(long, default_value_t = 1)]
    depth: usize,
    #[arg(long, default_value_t = 60)]
    max_pages: usize,
    #[arg(long, default_value_t = 900)]
    delay_ms: u64,
    #[arg(long)]
    ignore_robots: bool,
    #[arg(long)]
    include_body: bool,
    #[arg(long)]
    output: Option<PathBuf>,
    #[arg(long)]
    markdown: Option<PathBuf>,
}

#[derive(Args, Debug)]
struct RepoArgs {
    #[arg(long, default_value_t = 5)]
    max_pages: usize,
    #[arg(long)]
    output: Option<PathBuf>,
}

#[derive(Args, Debug)]
struct OutputArgs {
    #[arg(long)]
    output: Option<PathBuf>,
}

#[derive(Subcommand, Debug)]
enum BookCommand {
    Sync {
        #[arg(long, default_value = "stable-systemd")]
        track: String,
        #[arg(long)]
        dir: Option<PathBuf>,
    },
}

#[derive(Subcommand, Debug)]
enum AlfsCommand {
    /// Clone official jhalfs into a new directory. --execute performs the clone.
    Setup {
        #[arg(long)]
        dir: PathBuf,
        #[arg(long)]
        execute: bool,
    },
    /// Check the Linux host command prerequisites used by jhalfs.
    Check {
        #[arg(long)]
        dir: PathBuf,
    },
    /// Generate the official scripts and Makefile, with package build/download gates off.
    Generate {
        #[arg(long)]
        dir: PathBuf,
        #[arg(long)]
        execute: bool,
    },
}

#[derive(Args, Debug)]
struct RunArgs {
    #[arg(long)]
    script: PathBuf,
    #[arg(long)]
    step_id: Option<String>,
    #[arg(long)]
    journal: Option<PathBuf>,
    #[arg(long)]
    execute: bool,
}

#[derive(Deserialize)]
struct SourceRegistry {
    sources: Vec<Source>,
}
#[derive(Deserialize, Clone)]
struct Source {
    name: String,
    url: String,
    kind: String,
}

#[derive(Serialize, Deserialize, Clone)]
struct CrawlPage {
    url: String,
    final_url: String,
    title: String,
    description: String,
    text: String,
    links: Vec<String>,
    status: u16,
    content_type: String,
    fetched_at: String,
    sha256: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    raw_body: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

#[derive(Serialize, Deserialize)]
struct CrawlPolicy {
    max_depth: usize,
    max_pages: usize,
    delay_ms: u64,
    respect_robots: bool,
    origins: Vec<String>,
}
#[derive(Serialize, Deserialize)]
struct CrawlResult {
    schema_version: u8,
    generated_at: String,
    crawler: String,
    policy: CrawlPolicy,
    pages: Vec<CrawlPage>,
}

#[derive(Clone)]
struct RobotsRule {
    allow: bool,
    path: String,
}

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap_or_else(|_| PathBuf::from("."))
}
fn now() -> String {
    Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true)
}
fn sha256(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
}
fn write_json(path: &Path, value: &impl Serialize) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    let tmp = path.with_extension(format!(
        "{}.tmp",
        path.extension().and_then(|s| s.to_str()).unwrap_or("json")
    ));
    fs::write(&tmp, serde_json::to_vec_pretty(value)?)?;
    // `rename` does not replace an existing destination on Windows. Remove
    // it first so repeated CLI runs can update manifests and journals there.
    if path.exists() {
        fs::remove_file(path).with_context(|| format!("remove old {}", path.display()))?;
    }
    fs::rename(&tmp, path).with_context(|| format!("replace {}", path.display()))?;
    Ok(())
}
fn read_sources() -> Result<SourceRegistry> {
    Ok(serde_json::from_str(&fs::read_to_string(
        root().join("sources.json"),
    )?)?)
}
fn default_artifact(name: &str) -> PathBuf {
    root().join("artifacts").join(name)
}

fn validate_public_url(raw: &str, base: Option<&Url>) -> Result<Url> {
    let mut url = match base {
        Some(base) => base.join(raw)?,
        None => Url::parse(raw)?,
    };
    if !matches!(url.scheme(), "https" | "http") {
        bail!("unsupported URL scheme: {}", url.scheme());
    }
    if !url.username().is_empty() || url.password().is_some() {
        bail!("embedded URL credentials are not allowed");
    }
    let host = url
        .host_str()
        .unwrap_or_default()
        .trim_matches(['[', ']'])
        .to_ascii_lowercase();
    if host == "localhost" || host.ends_with(".localhost") || host.ends_with(".local") {
        bail!("local host is not allowed: {host}");
    }
    if let Ok(ip) = host.parse::<IpAddr>() {
        let private = match ip {
            IpAddr::V4(v4) => {
                v4.is_private()
                    || v4.is_loopback()
                    || v4.is_link_local()
                    || v4.is_broadcast()
                    || v4.is_unspecified()
            }
            IpAddr::V6(v6) => {
                v6.is_loopback()
                    || v6.is_unique_local()
                    || v6.is_unicast_link_local()
                    || v6.is_unspecified()
            }
        };
        if private {
            bail!("private or loopback IP is not allowed: {ip}");
        }
    }
    url.set_fragment(None);
    Ok(url)
}

fn client(timeout: Duration, redirects: bool) -> Result<Client> {
    let mut headers = HeaderMap::new();
    headers.insert(USER_AGENT, HeaderValue::from_static(USER_AGENT_VALUE));
    Ok(Client::builder()
        .default_headers(headers)
        .timeout(timeout)
        .redirect(if redirects {
            reqwest::redirect::Policy::limited(5)
        } else {
            reqwest::redirect::Policy::none()
        })
        .build()?)
}

struct ResponseMeta {
    status: u16,
    content_type: String,
}

fn get_limited(
    client: &Client,
    raw_url: &str,
    max_bytes: usize,
) -> Result<(ResponseMeta, Url, Vec<u8>)> {
    let initial = validate_public_url(raw_url, None)?;
    let mut current = initial.clone();
    for redirect in 0..=5 {
        let response = client.get(current.clone()).send()?;
        if response.status().is_redirection() {
            let location = response
                .headers()
                .get(LOCATION)
                .context("redirect without Location")?
                .to_str()?;
            if redirect == 5 {
                bail!("redirect limit exceeded: {raw_url}");
            }
            let next = validate_public_url(location, Some(&current))?;
            if initial.scheme() == "https" && next.scheme() != "https" {
                bail!("refusing HTTPS downgrade: {raw_url}");
            }
            current = next;
            continue;
        }
        let meta = ResponseMeta {
            status: response.status().as_u16(),
            content_type: response
                .headers()
                .get(reqwest::header::CONTENT_TYPE)
                .and_then(|v| v.to_str().ok())
                .unwrap_or("")
                .to_string(),
        };
        let mut body = Vec::new();
        response
            .take((max_bytes + 1) as u64)
            .read_to_end(&mut body)?;
        if body.len() > max_bytes {
            bail!("response exceeds {max_bytes} bytes: {current}");
        }
        return Ok((meta, current, body));
    }
    bail!("redirect limit exceeded: {raw_url}")
}

fn robots_for(client: &Client, origin: &str) -> Vec<RobotsRule> {
    let Ok(url) = Url::parse(&format!("{origin}/robots.txt")) else {
        return vec![RobotsRule {
            allow: false,
            path: "/".into(),
        }];
    };
    match client.get(url).send() {
        Ok(response) if response.status().as_u16() == 404 || response.status().as_u16() == 410 => {
            vec![]
        }
        Ok(response)
            if response.status().is_client_error()
                && ![401, 403, 429].contains(&response.status().as_u16()) =>
        {
            vec![]
        }
        Ok(response) if response.status().is_success() => {
            parse_robots(&response.text().unwrap_or_default())
        }
        _ => vec![RobotsRule {
            allow: false,
            path: "/".into(),
        }],
    }
}

fn parse_robots(body: &str) -> Vec<RobotsRule> {
    let mut rules = Vec::new();
    let mut wildcard_group = false;
    let mut saw_rule = false;
    for line in body.lines() {
        let line = line.split('#').next().unwrap_or("").trim();
        let Some((key, value)) = line.split_once(':') else {
            continue;
        };
        let key = key.trim().to_ascii_lowercase();
        let value = value.trim();
        if key == "user-agent" {
            if saw_rule {
                wildcard_group = false;
                saw_rule = false;
            }
            if value == "*" {
                wildcard_group = true;
            }
        } else if wildcard_group && (key == "allow" || key == "disallow") {
            saw_rule = true;
            if !value.is_empty() {
                rules.push(RobotsRule {
                    allow: key == "allow",
                    path: value.to_string(),
                });
            }
        }
    }
    rules
}

fn robots_allows(url: &Url, rules: &[RobotsRule]) -> bool {
    let path = format!(
        "{}{}",
        url.path(),
        url.query().map(|q| format!("?{q}")).unwrap_or_default()
    );
    let matching = rules
        .iter()
        .filter(|rule| path.starts_with(&rule.path.replace('$', "")))
        .max_by_key(|rule| (rule.path.trim_end_matches('$').len(), rule.allow));
    matching.map(|r| r.allow).unwrap_or(true)
}

fn page_from_html(
    url: &Url,
    final_url: &Url,
    status: u16,
    content_type: &str,
    body: &[u8],
    include_body: bool,
) -> CrawlPage {
    let html = String::from_utf8_lossy(body).to_string();
    let doc = Html::parse_document(&html);
    let title_sel = Selector::parse("title").unwrap();
    let desc_sel = Selector::parse("meta[name='description']").unwrap();
    let link_sel = Selector::parse("a[href]").unwrap();
    let title = doc
        .select(&title_sel)
        .next()
        .map(|e| e.text().collect::<String>())
        .unwrap_or_default()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    let description = doc
        .select(&desc_sel)
        .next()
        .and_then(|e| e.value().attr("content"))
        .unwrap_or("")
        .to_string();
    let text = doc
        .root_element()
        .text()
        .collect::<Vec<_>>()
        .join(" ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    let mut links = HashSet::new();
    for element in doc.select(&link_sel) {
        if let Some(href) = element.value().attr("href") {
            if let Ok(link) = validate_public_url(href, Some(final_url)) {
                if link.origin().ascii_serialization() == final_url.origin().ascii_serialization() {
                    links.insert(link.to_string());
                }
            }
        }
    }
    let mut links: Vec<_> = links.into_iter().collect();
    links.sort();
    CrawlPage {
        url: url.to_string(),
        final_url: final_url.to_string(),
        title,
        description,
        text,
        links,
        status,
        content_type: content_type.into(),
        fetched_at: now(),
        sha256: sha256(body),
        raw_body: include_body.then(|| html),
        error: None,
    }
}

fn crawl(args: CrawlArgs) -> Result<()> {
    if args.depth > 8 || args.max_pages == 0 || args.max_pages > 1000 {
        bail!("depth must be 0..8 and max-pages must be 1..1000");
    }
    let registry = read_sources()?;
    let seeds = if args.urls.is_empty() {
        registry
            .sources
            .into_iter()
            .filter(|s| s.kind != "git")
            .map(|s| s.url)
            .collect()
    } else {
        args.urls.clone()
    };
    let urls = seeds
        .iter()
        .map(|s| validate_public_url(s, None))
        .collect::<Result<Vec<_>>>()?;
    let origins: Vec<String> = urls
        .iter()
        .map(|u| u.origin().ascii_serialization())
        .collect::<HashSet<_>>()
        .into_iter()
        .collect();
    let allowed: HashSet<String> = origins.iter().cloned().collect();
    let http = client(Duration::from_secs(20), false)?;
    let mut robots_cache = HashMap::<String, Vec<RobotsRule>>::new();
    let mut queue: VecDeque<(Url, usize)> = urls.into_iter().map(|u| (u, 0)).collect();
    let mut seen = HashSet::new();
    let mut pages = Vec::new();
    let mut last_request: Option<Instant> = None;
    while let Some((url, depth)) = queue.pop_front() {
        if pages.len() >= args.max_pages || !seen.insert(url.to_string()) {
            continue;
        }
        let origin = url.origin().ascii_serialization();
        if !allowed.contains(&origin) {
            continue;
        }
        if !args.ignore_robots {
            let rules = robots_cache
                .entry(origin.clone())
                .or_insert_with(|| robots_for(&http, &origin));
            if !robots_allows(&url, rules) {
                continue;
            }
        }
        if let Some(last) = last_request {
            let due = last + Duration::from_millis(args.delay_ms);
            if due > Instant::now() {
                thread::sleep(due - Instant::now());
            }
        }
        last_request = Some(Instant::now());
        match get_limited(&http, url.as_str(), MAX_BODY_BYTES) {
            Ok((response, final_url, body)) => {
                let page = page_from_html(
                    &url,
                    &final_url,
                    response.status,
                    &response.content_type,
                    &body,
                    args.include_body,
                );
                if depth < args.depth {
                    for link in &page.links {
                        if let Ok(next) = Url::parse(link) {
                            if allowed.contains(&next.origin().ascii_serialization())
                                && !seen.contains(link)
                            {
                                queue.push_back((next, depth + 1));
                            }
                        }
                    }
                }
                pages.push(page);
            }
            Err(error) => pages.push(CrawlPage {
                url: url.to_string(),
                final_url: url.to_string(),
                title: String::new(),
                description: String::new(),
                text: String::new(),
                links: vec![],
                status: 0,
                content_type: String::new(),
                fetched_at: now(),
                sha256: String::new(),
                raw_body: None,
                error: Some(error.to_string()),
            }),
        }
    }
    let result = CrawlResult {
        schema_version: 1,
        generated_at: now(),
        crawler: "cargo-lfs Rust crawler".into(),
        policy: CrawlPolicy {
            max_depth: args.depth,
            max_pages: args.max_pages,
            delay_ms: args.delay_ms,
            respect_robots: !args.ignore_robots,
            origins,
        },
        pages,
    };
    let out = args.output.unwrap_or_else(|| {
        default_artifact(&format!("research-{}.json", Utc::now().format("%Y-%m-%d")))
    });
    let md = args.markdown.unwrap_or_else(|| out.with_extension("md"));
    write_json(&out, &result)?;
    fs::write(&md, synthesize(&result, "YOLO OS / LFS web reconnaissance"))?;
    println!(
        "{}",
        json!({"pages": result.pages.len(), "output": out, "markdown": md, "origins": result.policy.origins})
    );
    Ok(())
}

fn synthesize(result: &CrawlResult, title: &str) -> String {
    let mut out = format!("# {title}\n\nCollected: {}\n\n", result.generated_at);
    for page in &result.pages {
        out.push_str(&format!(
            "## [{}]({})\n\n",
            if page.title.is_empty() {
                &page.url
            } else {
                &page.title
            },
            page.final_url
        ));
        out.push_str(&format!(
            "- Status: {}\n- Retrieved: {}\n- SHA-256: `{}`\n\n",
            page.status, page.fetched_at, page.sha256
        ));
        if let Some(error) = &page.error {
            out.push_str(&format!("Capture error: {error}\n\n"));
            continue;
        }
        if !page.description.is_empty() {
            out.push_str(&format!("{}\n\n", page.description));
        }
        let excerpt: String = page.text.chars().take(1800).collect();
        if !excerpt.is_empty() {
            out.push_str(&format!(
                "{}{}\n\n",
                excerpt,
                if page.text.chars().count() > 1800 {
                    "…"
                } else {
                    ""
                }
            ));
        }
    }
    out
}

#[derive(Serialize)]
struct Repo {
    full_name: String,
    html_url: String,
    description: Option<String>,
    stars: u64,
    forks: u64,
    language: Option<String>,
    archived: bool,
    pushed_at: Option<String>,
    default_branch: String,
    license: Option<Value>,
}

fn github_repos(args: RepoArgs) -> Result<()> {
    if args.max_pages == 0 || args.max_pages > 10 {
        bail!("max-pages must be 1..10");
    }
    let registry = read_sources()?;
    let http = client(Duration::from_secs(30), true)?;
    let token = std::env::var("GITHUB_TOKEN").ok();
    let mut listings = Vec::new();
    for source in registry
        .sources
        .iter()
        .filter(|s| s.kind == "github-org" || s.kind == "github-user")
    {
        let parsed = Url::parse(&source.url)?;
        let parts: Vec<_> = parsed
            .path_segments()
            .map(|segments| segments.filter(|s| !s.is_empty()).collect())
            .unwrap_or_default();
        let (owner_type, owner) = if source.kind == "github-org" {
            (
                "org",
                parts
                    .get(1)
                    .copied()
                    .context("GitHub org seed missing owner")?,
            )
        } else {
            (
                "user",
                parts
                    .first()
                    .copied()
                    .context("GitHub user seed missing owner")?,
            )
        };
        let mut repos = Vec::<Repo>::new();
        let mut pages = Vec::new();
        let mut truncated = false;
        for page in 1..=args.max_pages {
            let api = format!(
                "https://api.github.com/{owner_type}s/{owner}/repos?per_page=100&page={page}&sort=stargazers&direction=desc"
            );
            let mut request = http
                .get(&api)
                .header("Accept", "application/vnd.github+json");
            if let Some(token) = &token {
                request = request.bearer_auth(token);
            }
            let response = request.send()?.error_for_status()?;
            let etag = response
                .headers()
                .get("etag")
                .and_then(|v| v.to_str().ok())
                .map(str::to_string);
            let body = response.bytes()?.to_vec();
            let values: Vec<Value> = serde_json::from_slice(&body)?;
            pages.push(json!({"page": page, "fetched_at": now(), "item_count": values.len(), "sha256": sha256(&body), "etag": etag, "raw_body": String::from_utf8_lossy(&body)}));
            for item in values {
                repos.push(Repo {
                    full_name: item["full_name"].as_str().unwrap_or_default().into(),
                    html_url: item["html_url"].as_str().unwrap_or_default().into(),
                    description: item["description"].as_str().map(str::to_string),
                    stars: item["stargazers_count"].as_u64().unwrap_or_default(),
                    forks: item["forks_count"].as_u64().unwrap_or_default(),
                    language: item["language"].as_str().map(str::to_string),
                    archived: item["archived"].as_bool().unwrap_or(false),
                    pushed_at: item["pushed_at"].as_str().map(str::to_string),
                    default_branch: item["default_branch"].as_str().unwrap_or("main").into(),
                    license: item.get("license").cloned().filter(|v| !v.is_null()),
                });
            }
            if pages
                .last()
                .and_then(|p| p["item_count"].as_u64())
                .unwrap_or(0)
                < 100
            {
                break;
            }
            if page == args.max_pages {
                truncated = true;
            }
        }
        repos.sort_by(|a, b| {
            b.stars
                .cmp(&a.stars)
                .then_with(|| a.full_name.cmp(&b.full_name))
        });
        listings.push(json!({"schema_version":1,"source":format!("https://api.github.com/{owner_type}s/{owner}/repos"),"owner":owner,"owner_type":owner_type,"sort":"stargazers","direction":"desc","generated_at":now(),"page_count":pages.len(),"truncated":truncated,"pages":pages,"repositories":repos}));
        eprintln!("{}: {} repositories", source.name, repos.len());
    }
    let snapshot = json!({"schema_version":1,"generated_at":now(),"listings":listings});
    let output = args.output.unwrap_or_else(|| {
        default_artifact(&format!(
            "github-repositories-{}.json",
            Utc::now().format("%Y-%m-%d")
        ))
    });
    write_json(&output, &snapshot)?;
    let mut md = format!("# GitHub repository rankings\n\nCollected: {}\n\n", now());
    for listing in snapshot["listings"].as_array().unwrap_or(&vec![]) {
        let repos = listing["repositories"]
            .as_array()
            .cloned()
            .unwrap_or_default();
        md.push_str(&format!("## {} ({}, {} repositories{})\n\n| Rank | Repository | Stars | Language | Archived |\n| ---: | --- | ---: | --- | :---: |\n", listing["owner_type"], listing["owner"], repos.len(), if listing["truncated"] == true { ", partial listing" } else { "" }));
        for (i, repo) in repos.iter().enumerate() {
            md.push_str(&format!(
                "| {} | [{}]({}) | {} | {} | {} |\n",
                i + 1,
                repo["full_name"],
                repo["html_url"],
                repo["stars"],
                repo["language"].as_str().unwrap_or("—"),
                if repo["archived"] == true {
                    "yes"
                } else {
                    "no"
                }
            ));
        }
        md.push('\n');
    }
    fs::write(output.with_extension("md"), md)?;
    println!("{}", output.display());
    Ok(())
}

fn git_heads(args: OutputArgs) -> Result<()> {
    let registry = read_sources()?;
    let mut remotes = Vec::new();
    for source in registry.sources.iter().filter(|s| s.kind == "git") {
        let output = Command::new("git")
            .args(["ls-remote", "--symref", &source.url, "HEAD"])
            .output()
            .context("git is required for git-heads")?;
        if !output.status.success() {
            bail!("git ls-remote failed for {}", source.url);
        }
        let raw = String::from_utf8_lossy(&output.stdout).to_string();
        let symbolic = raw.lines().find_map(|line| {
            line.strip_prefix("ref: ")
                .and_then(|s| s.split_once('\t'))
                .map(|(r, _)| r.to_string())
        });
        let commit = raw
            .lines()
            .find(|line| !line.starts_with("ref:"))
            .and_then(|l| l.split_whitespace().next())
            .unwrap_or_default();
        remotes.push(json!({"label":source.name,"schema_version":1,"remote":source.url,"symbolic_head":symbolic,"commit":commit,"fetched_at":now(),"sha256":sha256(raw.as_bytes()),"raw_output":raw}));
    }
    let snapshot = json!({"schema_version":1,"generated_at":now(),"remotes":remotes});
    let output = args.output.unwrap_or_else(|| {
        default_artifact(&format!(
            "git-remotes-{}.json",
            Utc::now().format("%Y-%m-%d")
        ))
    });
    write_json(&output, &snapshot)?;
    let mut md = format!("# Git remote heads\n\nCollected: {}\n\n", now());
    for item in snapshot["remotes"].as_array().unwrap() {
        md.push_str(&format!(
            "- [{}]({}): `{}` at `{}`\n",
            item["label"],
            item["remote"],
            item["symbolic_head"].as_str().unwrap_or("detached"),
            item["commit"]
        ));
    }
    fs::write(output.with_extension("md"), md)?;
    println!("{}", output.display());
    Ok(())
}

fn book_sync(track: String, directory: Option<PathBuf>) -> Result<()> {
    let base = match track.as_str() {
        "stable-systemd" => "https://www.linuxfromscratch.org/lfs/downloads/stable-systemd/",
        "development" => "https://www.linuxfromscratch.org/lfs/downloads/development/",
        "stable" | "stable-sysv" => "https://www.linuxfromscratch.org/lfs/downloads/stable/",
        _ => bail!("unsupported LFS track: {track}"),
    };
    let directory = directory.unwrap_or_else(|| root().join("artifacts/lfs-book").join(&track));
    fs::create_dir_all(&directory)?;
    let http = client(Duration::from_secs(30), false)?;
    let mut files = serde_json::Map::new();
    for name in ["wget-list", "md5sums"] {
        let (response, final_url, body) = get_limited(&http, &format!("{base}{name}"), 1_000_000)?;
        if !(200..300).contains(&response.status)
            || final_url.host_str() != Some("www.linuxfromscratch.org")
        {
            bail!(
                "failed to retrieve official {name} from {base} (HTTP {})",
                response.status
            );
        }
        fs::write(directory.join(name), &body)?;
        files.insert(
            name.into(),
            json!({"url":format!("{base}{name}"),"bytes":body.len(),"sha256":sha256(&body)}),
        );
    }
    let meta = json!({"schema_version":1,"project":"Linux From Scratch","track":track,"retrieved_at":now(),"files":files});
    write_json(&directory.join("book-source.json"), &meta)?;
    println!("{}", serde_json::to_string_pretty(&meta)?);
    Ok(())
}

fn package_plan(book: &Path, sources: Option<PathBuf>) -> Result<Value> {
    let source_cache = sources.unwrap_or_else(|| default_artifact("lfs-sources"));
    let list = fs::read_to_string(book.join("wget-list"))
        .with_context(|| format!("no wget-list in {}", book.display()))?;
    let urls: Vec<String> = list
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .map(str::to_string)
        .collect();
    let mut names = HashMap::<String, String>::new();
    for raw in &urls {
        let url = validate_public_url(raw, None)?;
        let name = url
            .path_segments()
            .and_then(|mut p| p.next_back())
            .context("package URL has no filename")?
            .to_string();
        if let Some(previous) = names.insert(name.clone(), raw.clone()) {
            if previous != *raw {
                bail!("package filename collision {name}: {previous} and {raw}");
            }
        }
    }
    let md5 = book.join("md5sums");
    let checksums = if md5.exists() {
        Some(
            fs::read_to_string(&md5)?
                .lines()
                .filter(|l| !l.trim().is_empty())
                .count(),
        )
    } else {
        None
    };
    Ok(
        json!({"book_root":book,"source_cache":source_cache,"package_urls":urls.len(),"checksums_listed":checksums,"execution_enabled":false}),
    )
}

fn fetch_packages(book: &Path, sources: Option<PathBuf>, execute: bool) -> Result<()> {
    let plan = package_plan(book, sources.clone())?;
    if !execute || !cfg!(target_os = "linux") {
        println!("{}", serde_json::to_string_pretty(&plan)?);
        if execute && !cfg!(target_os = "linux") {
            bail!("package retrieval with checksum validation requires Linux");
        }
        return Ok(());
    }
    let cache = sources.unwrap_or_else(|| default_artifact("lfs-sources"));
    fs::create_dir_all(&cache)?;
    let list = fs::read_to_string(book.join("wget-list"))?;
    let http = client(Duration::from_secs(120), false)?;
    for raw in list
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
    {
        let url = validate_public_url(raw, None)?;
        let name = url
            .path_segments()
            .and_then(|mut p| p.next_back())
            .context("package URL has no name")?;
        let target = cache.join(name);
        if target.exists() {
            continue;
        }
        let (response, _, body) = get_limited(&http, raw, 2 * 1024 * 1024 * 1024)?;
        if !(200..300).contains(&response.status) {
            bail!("HTTP {} downloading {raw}", response.status);
        }
        fs::write(target, body)?;
        println!("Fetched {name}");
    }
    let md5 = book.join("md5sums");
    if md5.exists() {
        let status = Command::new("md5sum")
            .args(["--check"])
            .arg(&md5)
            .current_dir(&cache)
            .status()
            .context("md5sum is required")?;
        if !status.success() {
            bail!("official checksum validation failed");
        }
    }
    Ok(())
}

fn alfs(command: AlfsCommand) -> Result<()> {
    const UPSTREAM: &str = "https://git.linuxfromscratch.org/jhalfs.git";
    match command {
        AlfsCommand::Setup { dir, execute } => {
            let dir = if dir.is_absolute() {
                dir
            } else {
                std::env::current_dir()?.join(dir)
            };
            if !execute {
                println!(
                    "{}",
                    json!({"planned":true,"command":["git","clone","--depth","1",UPSTREAM,dir]})
                );
                return Ok(());
            }
            if dir.join(".git/HEAD").exists() {
                let origin = Command::new("git")
                    .arg("-C")
                    .arg(&dir)
                    .args(["remote", "get-url", "origin"])
                    .output()?;
                let value = String::from_utf8_lossy(&origin.stdout).trim().to_string();
                if value != UPSTREAM {
                    bail!("refusing checkout with unexpected origin {value}");
                }
                println!(
                    "{}",
                    json!({"existing":true,"directory":dir,"origin":value})
                );
                return Ok(());
            }
            if dir.exists() {
                bail!("refusing to clone over existing path {}", dir.display());
            }
            if let Some(parent) = dir.parent() {
                fs::create_dir_all(parent)?;
            }
            let status = Command::new("git")
                .args(["clone", "--depth", "1", UPSTREAM])
                .arg(&dir)
                .status()?;
            if !status.success() {
                bail!("jhalfs clone failed");
            }
            let commit = Command::new("git")
                .arg("-C")
                .arg(&dir)
                .args(["rev-parse", "HEAD"])
                .output()?;
            write_json(
                &dir.join("yolo-alfs-source.json"),
                &json!({"upstream":UPSTREAM,"commit":String::from_utf8_lossy(&commit.stdout).trim(),"cloned_at":now()}),
            )?;
            println!(
                "{}",
                json!({"directory":dir,"commit":String::from_utf8_lossy(&commit.stdout).trim()})
            );
            Ok(())
        }
        AlfsCommand::Check { dir } => {
            for file in [
                "jhalfs",
                "configuration",
                "common/libs/func_check_version.sh",
            ] {
                if !dir.join(file).exists() {
                    bail!(
                        "expected configured jhalfs checkout: missing {}",
                        dir.join(file).display()
                    );
                }
            }
            let mut tools = Vec::new();
            for name in [
                "bash", "git", "make", "python3", "sudo", "wget", "xmllint", "xsltproc",
            ] {
                let found = Command::new("bash")
                    .args(["-lc", &format!("command -v {name}")])
                    .output()
                    .map(|o| o.status.success())
                    .unwrap_or(false);
                tools.push(json!({"tool":name,"found":found}));
            }
            let missing: Vec<_> = tools.iter().filter(|t| t["found"] != true).collect();
            println!(
                "{}",
                serde_json::to_string_pretty(
                    &json!({"directory":dir,"linux_host":cfg!(target_os="linux"),"tools":tools,"note":"Also verify libxml2/libxslt versions, DocBook XML DTD 4.5, DocBook XSL, the selected book requirements, sudo access, and an empty dedicated target."})
                )?
            );
            if !cfg!(target_os = "linux") || !missing.is_empty() {
                bail!("host prerequisites need attention");
            }
            Ok(())
        }
        AlfsCommand::Generate { dir, execute } => {
            let config_path = dir.join("configuration");
            let config = fs::read_to_string(&config_path)?;
            for (index, line) in config.lines().enumerate() {
                let line = line.trim();
                if line.is_empty() || line.starts_with('#') {
                    continue;
                }
                let (key, value) = line
                    .split_once('=')
                    .context("non-Kconfig shell in jhalfs configuration")?;
                if key.is_empty()
                    || !key.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
                    || !safe_kconfig_value(value)
                {
                    bail!(
                        "refusing non-Kconfig shell syntax at {}:{}",
                        config_path.display(),
                        index + 1
                    );
                }
            }
            for option in ["RUNMAKE", "GETPKG", "CLEAN"] {
                let prefix = format!("{option}=");
                for line in config
                    .lines()
                    .map(str::trim)
                    .filter(|line| line.starts_with(&prefix))
                {
                    if !matches!(line[prefix.len()..].trim(), "n" | "\"n\"" | "'n'") {
                        bail!(
                            "{option} must be unset or disabled in {}",
                            config_path.display()
                        );
                    }
                }
            }
            if !execute {
                println!(
                    "{}",
                    json!({"planned":true,"cwd":dir,"command":["./jhalfs","run"],"run_make":"n (jhalfs default; enabled setting rejected)"})
                );
                return Ok(());
            }
            if !cfg!(target_os = "linux") {
                bail!("jhalfs generation requires Linux");
            }
            let mut child = Command::new("bash")
                .args(["./jhalfs", "run"])
                .current_dir(&dir)
                .env("RUNMAKE", "n")
                .stdin(Stdio::piped())
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit())
                .spawn()?;
            if let Some(mut input) = child.stdin.take() {
                input.write_all(b"yes\nyes\n")?;
            }
            let status = child.wait()?;
            if !status.success() {
                bail!("jhalfs generation exited with {status}");
            }
            Ok(())
        }
    }
}

fn safe_kconfig_value(value: &str) -> bool {
    let value = value.trim();
    if matches!(value, "y" | "n") || value.parse::<i64>().is_ok() {
        return true;
    }
    if value.starts_with('"') && value.ends_with('"') {
        return !value[1..value.len() - 1].contains(['"', '`', '\\'])
            && !value.contains("$(")
            && !value.contains("${");
    }
    if value.starts_with('\'') && value.ends_with('\'') {
        return !value[1..value.len() - 1].contains(['\'', '\r', '\n']);
    }
    false
}

#[derive(Serialize, Deserialize)]
struct Journal {
    schema_version: u8,
    events: Vec<JournalEvent>,
}
#[derive(Serialize, Deserialize, Clone)]
struct JournalEvent {
    kind: String,
    step_id: String,
    script: String,
    sha256: String,
    at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    exit_code: Option<i32>,
}
fn run_script(args: RunArgs) -> Result<()> {
    let script = args.script.canonicalize().context("script not found")?;
    let body = fs::read(&script)?;
    let digest = sha256(&body);
    let id = args.step_id.unwrap_or_else(|| {
        script
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("step")
            .into()
    });
    let journal_path = args
        .journal
        .unwrap_or_else(|| default_artifact("lfs-build-state.json"));
    if !args.execute {
        println!(
            "{}",
            serde_json::to_string_pretty(
                &json!({"planned":true,"step_id":id,"script":script,"sha256":digest,"command":["bash",script],"journal":journal_path})
            )?
        );
        return Ok(());
    }
    if !cfg!(target_os = "linux") {
        bail!("LFS build scripts require Linux");
    }
    let mut journal: Journal = if journal_path.exists() {
        serde_json::from_slice(&fs::read(&journal_path)?)?
    } else {
        Journal {
            schema_version: 1,
            events: vec![],
        }
    };
    if journal.schema_version != 1 {
        bail!("unsupported build journal version");
    }
    if journal.events.iter().rev().any(|e| {
        e.kind == "step-finish" && e.step_id == id && e.sha256 == digest && e.exit_code == Some(0)
    }) {
        println!("{}", json!({"skipped":true,"step_id":id,"sha256":digest}));
        return Ok(());
    }
    if let Some(parent) = journal_path.parent() {
        fs::create_dir_all(parent)?;
    }
    let record = |kind: &str, code: Option<i32>| JournalEvent {
        kind: kind.into(),
        step_id: id.clone(),
        script: script.display().to_string(),
        sha256: digest.clone(),
        at: now(),
        exit_code: code,
    };
    journal.events.push(record("step-start", None));
    write_json(&journal_path, &journal)?;
    let status = Command::new("bash")
        .arg("--noprofile")
        .arg("--norc")
        .arg(&script)
        .status()?;
    journal
        .events
        .push(record("step-finish", Some(status.code().unwrap_or(1))));
    write_json(&journal_path, &journal)?;
    if !status.success() {
        bail!("step {id} exited with {status}");
    }
    Ok(())
}

fn main() -> Result<()> {
    match Cli::parse().command.unwrap_or(Commands::Sources) {
        Commands::Sources => println!("{}", fs::read_to_string(root().join("sources.json"))?),
        Commands::Crawl(args) => crawl(args)?,
        Commands::Repos(args) => github_repos(args)?,
        Commands::GitHeads(args) => git_heads(args)?,
        Commands::Synthesize { input, output } => {
            let result: CrawlResult = serde_json::from_slice(&fs::read(&input)?)?;
            let path = output.unwrap_or_else(|| input.with_extension("md"));
            fs::write(
                &path,
                synthesize(&result, "YOLO OS / LFS research synthesis"),
            )?;
            println!("{}", path.display());
        }
        Commands::Book {
            command: BookCommand::Sync { track, dir },
        } => book_sync(track, dir)?,
        Commands::Alfs { command } => alfs(command)?,
        Commands::Plan { book, sources } => println!(
            "{}",
            serde_json::to_string_pretty(&package_plan(&book, sources)?)?
        ),
        Commands::Fetch {
            book,
            sources,
            execute,
        } => fetch_packages(&book, sources, execute)?,
        Commands::Run(args) => run_script(args)?,
    }
    Ok(())
}
