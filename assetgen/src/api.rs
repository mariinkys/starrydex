//! Thin layer over rustemon: client construction and concurrent, retrying bulk fetches.

use std::{collections::BTreeMap, future::Future, sync::Arc, time::Duration};

use futures::StreamExt;
use rustemon::client::{
    CacheMode, Environment, MokaManager, RustemonClient, RustemonClientBuilder,
};

pub type Client = Arc<RustemonClient>;

/// Default PokéAPI location.
pub const DEFAULT_BASE_URL: &str = "https://pokeapi.co/api/v2/";

/// Plain HTTP access for resources whose rustemon model no longer matches PokéAPI.
#[derive(Clone)]
pub struct RawApi {
    http: reqwest::Client,
    base: String,
}

impl RawApi {
    pub fn new(base_url: Option<&str>) -> Result<Self, anywho::Error> {
        let mut base = base_url
            .unwrap_or(DEFAULT_BASE_URL)
            .trim_end_matches('/')
            .to_string();
        base.push('/');
        Ok(Self {
            http: reqwest::Client::builder()
                .timeout(Duration::from_secs(60))
                .build()?,
            base,
        })
    }

    /// GETs `{base}{endpoint}/{id}/` and deserializes it.
    pub async fn get<T: serde::de::DeserializeOwned>(
        &self,
        endpoint: &str,
        id: i64,
    ) -> Result<T, anywho::Error> {
        self.get_patched(endpoint, id, |_| {}).await
    }

    /// Like [`RawApi::get`] but lets `patch` fix up the JSON before it is deserialized.
    pub async fn get_patched<T: serde::de::DeserializeOwned>(
        &self,
        endpoint: &str,
        id: i64,
        patch: impl Fn(&mut serde_json::Value),
    ) -> Result<T, anywho::Error> {
        let url = format!("{}{endpoint}/{id}/", self.base);
        let body = self
            .http
            .get(&url)
            .send()
            .await?
            .error_for_status()?
            .text()
            .await?;
        let mut value: serde_json::Value =
            serde_json::from_str(&body).map_err(|e| anywho::anywho!("{url}: {e}"))?;
        patch(&mut value);
        serde_json::from_value(value).map_err(|e| anywho::anywho!("{url}: {e}"))
    }
}

/// Recursively converts float values of the given key into integers
/// (PokéAPI sends `percentage_chance: 50.0` where rustemon expects an integer).
pub fn floats_to_ints(value: &mut serde_json::Value, key: &str) {
    match value {
        serde_json::Value::Object(map) => {
            for (k, v) in map.iter_mut() {
                if k == key
                    && let Some(f) = v.as_f64()
                {
                    *v = serde_json::Value::from(f.round() as i64);
                } else {
                    floats_to_ints(v, key);
                }
            }
        }
        serde_json::Value::Array(items) => items.iter_mut().for_each(|v| floats_to_ints(v, key)),
        _ => {}
    }
}

/// Builds the rustemon client. Caching is disabled: every resource is fetched exactly once
pub fn client(base_url: Option<&str>) -> Result<Client, anywho::Error> {
    let mut builder = RustemonClientBuilder::<MokaManager>::default().with_mode(CacheMode::NoStore);
    if let Some(url) = base_url {
        let mut url = url.trim_end_matches('/').to_string();
        url.push('/');
        builder = builder.with_environment(Environment::Custom(url));
    }
    builder
        .try_build()
        .map(Arc::new)
        .map_err(|e| anywho::anywho!("Failed to build PokéAPI client: {e}"))
}

/// Extracts the numeric id at the end of a PokéAPI url (`.../pokemon/25/` → 25).
pub fn id_of(url: &str) -> Option<i64> {
    url.trim_end_matches('/').rsplit('/').next()?.parse().ok()
}

/// [`id_of`] for tables using 32-bit ids. Panics never: unparsable urls map to 0.
pub fn id32(url: &str) -> u32 {
    id_of(url)
        .and_then(|id| u32::try_from(id).ok())
        .unwrap_or(0)
}

/// [`id_of`] for tables using 16-bit ids.
pub fn id16(url: &str) -> u16 {
    id_of(url)
        .and_then(|id| u16::try_from(id).ok())
        .unwrap_or(0)
}

/// Fetches many resources concurrently with retries, printing progress.
///
/// Resources that still fail after all retries are reported and skipped.
pub async fn fetch_many<T, E, F, Fut>(
    label: &str,
    ids: impl IntoIterator<Item = i64>,
    concurrency: usize,
    fetch: F,
) -> BTreeMap<i64, T>
where
    F: Fn(i64) -> Fut,
    Fut: Future<Output = Result<T, E>>,
    E: std::fmt::Display,
{
    let mut ids: Vec<i64> = ids.into_iter().collect();
    ids.sort_unstable();
    ids.dedup();
    let total = ids.len();
    if total == 0 {
        return BTreeMap::new();
    }
    println!("  Fetching {total} {label}...");

    let fetch = &fetch;
    let mut done = 0usize;
    let mut failed = 0usize;
    let step = (total / 10).max(1);
    let mut results = BTreeMap::new();

    let mut stream = futures::stream::iter(ids)
        .map(|id| async move {
            let mut attempt = 0u32;
            loop {
                match fetch(id).await {
                    Ok(value) => return (id, Ok(value)),
                    Err(e) if attempt >= 3 => return (id, Err(e)),
                    Err(_) => {
                        attempt += 1;
                        tokio::time::sleep(Duration::from_millis(500 * u64::from(attempt))).await;
                    }
                }
            }
        })
        .buffer_unordered(concurrency.max(1));

    while let Some((id, result)) = stream.next().await {
        done += 1;
        match result {
            Ok(value) => {
                results.insert(id, value);
            }
            Err(e) => {
                failed += 1;
                eprintln!("    ! {label} #{id}: {e}");
            }
        }
        if done.is_multiple_of(step) || done == total {
            println!("    {label}: {done}/{total}");
        }
    }
    if failed > 0 {
        eprintln!("  {failed} {label} could not be fetched");
    }
    results
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_ids() {
        assert_eq!(id_of("https://pokeapi.co/api/v2/pokemon/25/"), Some(25));
        assert_eq!(id_of("/api/v2/move/10001/"), Some(10001));
        assert_eq!(id_of("/api/v2/move/"), None);
    }
}
