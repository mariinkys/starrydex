//! Downloads sprites/cries and packs them into `.tar.gz` archives.

use std::{
    collections::BTreeSet,
    io::Cursor,
    path::{Path, PathBuf},
    time::Duration,
};

use anywho::anywho;
use flate2::{Compression, write::GzEncoder};
use futures::StreamExt;

use crate::dataset::Download;

/// Size regular PokéAPI sprites have; fallback renders are downscaled to it.
const SPRITE_SIZE: u32 = 96;

/// Downloads every file of `downloads` below `root`. Returns the archive paths that failed.
pub async fn download_all(
    label: &str,
    downloads: &[Download],
    root: &Path,
    concurrency: usize,
) -> Result<BTreeSet<String>, anywho::Error> {
    let client = reqwest::Client::builder()
        .pool_max_idle_per_host(concurrency)
        .timeout(Duration::from_secs(60))
        .build()?;

    let total = downloads.len();
    println!("  Downloading {total} {label}...");
    let step = (total / 10).max(1);
    let mut done = 0usize;
    let mut failed = BTreeSet::new();

    let client = &client;
    let mut stream = futures::stream::iter(downloads)
        .map(|d| async move { (d, download_one(client, d, root).await) })
        .buffer_unordered(concurrency.max(1));

    while let Some((download, result)) = stream.next().await {
        done += 1;
        if let Err(e) = result {
            eprintln!("    ! {}: {e}", download.url);
            failed.insert(download.path.clone());
        }
        if done.is_multiple_of(step) || done == total {
            println!("    {label}: {done}/{total}");
        }
    }
    if !failed.is_empty() {
        eprintln!("  {} {label} could not be downloaded", failed.len());
    }
    Ok(failed)
}

async fn download_one(
    client: &reqwest::Client,
    d: &Download,
    root: &Path,
) -> Result<(), anywho::Error> {
    let target = root.join(&d.path);
    if tokio::fs::metadata(&target).await.is_ok() {
        return Ok(());
    }

    let mut attempt = 0u32;
    let bytes = loop {
        let result = async {
            let response = client.get(&d.url).send().await?.error_for_status()?;
            response.bytes().await
        }
        .await;
        match result {
            Ok(bytes) => break bytes,
            // 404s won't fix themselves
            Err(e) if attempt >= 3 || e.status().is_some_and(|s| s.is_client_error()) => {
                return Err(e.into());
            }
            Err(_) => {
                attempt += 1;
                tokio::time::sleep(Duration::from_millis(500 * u64::from(attempt))).await;
            }
        }
    };

    let bytes = if d.resize {
        downscale(&bytes)?
    } else {
        bytes.to_vec()
    };

    if let Some(parent) = target.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }
    tokio::fs::write(&target, bytes).await?;
    Ok(())
}

/// Downscales a large render (HOME/official artwork) to regular sprite size.
fn downscale(bytes: &[u8]) -> Result<Vec<u8>, anywho::Error> {
    let image = image::load_from_memory(bytes).map_err(|e| anywho!("decode failed: {e}"))?;
    let small = image.resize(
        SPRITE_SIZE,
        SPRITE_SIZE,
        image::imageops::FilterType::Lanczos3,
    );
    let mut out = Cursor::new(Vec::new());
    small
        .write_to(&mut out, image::ImageFormat::Png)
        .map_err(|e| anywho!("encode failed: {e}"))?;
    Ok(out.into_inner())
}

/// Packs `root/<dir>` into `archive` as `<dir>/...`.
pub fn pack(root: &Path, dir: &str, archive: &Path) -> Result<(), anywho::Error> {
    if let Some(parent) = archive.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let file = std::fs::File::create(archive)?;
    let mut tar = tar::Builder::new(GzEncoder::new(file, Compression::best()));
    let source = root.join(dir);
    if source.exists() {
        tar.append_dir_all(dir, &source)?;
    }
    tar.into_inner()?.finish()?;
    Ok(())
}

/// Scratch directory for downloads (reused between runs so interrupted runs can resume).
pub fn scratch_dir() -> PathBuf {
    std::env::temp_dir().join("starrydex-assetgen")
}
