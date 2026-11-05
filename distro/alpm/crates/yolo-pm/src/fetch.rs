//! HTTP and hashing helpers for `sync`. Only `GET`/`HEAD` over HTTPS.

use sha2::{Digest, Sha256};

/// Upper bound for one database download. The largest default repo is well
/// under this; ureq's own default (10 MB) would cut off larger mirrors.
const MAX_BYTES: u64 = 512 * 1024 * 1024;

pub fn get(url: &str) -> Result<Vec<u8>, String> {
    let mut resp = ureq::get(url)
        .call()
        .map_err(|e| format!("GET {url}: {e}"))?;
    resp.body_mut()
        .with_config()
        .limit(MAX_BYTES)
        .read_to_vec()
        .map_err(|e| format!("read {url}: {e}"))
}

/// Returns the HTTP status for a HEAD request, or a short error label.
pub fn head_status(url: &str) -> String {
    match ureq::head(url).call() {
        Ok(resp) => resp.status().as_u16().to_string(),
        Err(ureq::Error::StatusCode(code)) => code.to_string(),
        Err(e) => format!("error: {e}"),
    }
}

pub fn sha256_hex(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}
