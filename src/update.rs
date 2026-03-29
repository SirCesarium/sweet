//! Logic for self-updating and version checking.

use console::style;
use indicatif::{ProgressBar, ProgressStyle};
use std::fs;

/// Checks for updates and caches the result.
/// Performs a synchronous network check only if the cache is missing or expired (24h).
pub fn check_for_updates() {
    let cache_dir = std::env::temp_dir().join("sweet_update_cache");
    let current_version = env!("CARGO_PKG_VERSION");

    // 1. Try to use existing cache
    if let Ok(cached_version) = fs::read_to_string(&cache_dir) {
        let is_fresh = fs::metadata(&cache_dir)
            .and_then(|m| m.modified())
            .map(|m| m.elapsed().map(|e| e.as_secs() < 86400).unwrap_or(false))
            .unwrap_or(false);

        if is_fresh {
            if self_update::version::bump_is_greater(current_version, &cached_version)
                .unwrap_or(false)
            {
                print_update_msg(&cached_version, current_version);
            }
            return;
        }
    }

    // 2. Cache missing or expired: Perform synchronous check
    let releases = self_update::backends::github::ReleaseList::configure()
        .repo_owner("SirCesarium")
        .repo_name("sweet")
        .build();

    if let Some(latest_release) = releases
        .and_then(self_update::backends::github::ReleaseList::fetch)
        .ok()
        .and_then(|latest| latest.into_iter().next())
    {
        let _ = fs::write(&cache_dir, &latest_release.version);

        if self_update::version::bump_is_greater(current_version, &latest_release.version)
            .unwrap_or(false)
        {
            print_update_msg(&latest_release.version, current_version);
        }
    }
}

/// Performs the update process with a beautiful progress bar.
///
/// # Errors
///
/// Returns an error if the network request fails, the binary cannot be extracted,
/// or the current executable cannot be replaced.
pub fn handle_update() -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", style("🔍 Checking for updates...").cyan());
    let current_version = env!("CARGO_PKG_VERSION");
    let target = self_update::get_target();

    let releases = self_update::backends::github::ReleaseList::configure()
        .repo_owner("SirCesarium")
        .repo_name("sweet")
        .build()?
        .fetch()?;

    let latest = releases
        .iter()
        .find(|r| {
            self_update::version::bump_is_greater(current_version, &r.version).unwrap_or(false)
        })
        .ok_or("Sweet is already up to date.")?;

    let asset = latest.asset_for(target, None).ok_or_else(|| {
        format!(
            "No compatible binary found for {target} in v{}",
            latest.version
        )
    })?;

    println!(
        " 🚀 {} v{current_version} -> v{}",
        style("Updating Sweet:").bold(),
        style(&latest.version).green().bold()
    );

    let tmp_dir = std::env::temp_dir().join("sweet_update");
    if !tmp_dir.exists() {
        fs::create_dir_all(&tmp_dir)?;
    }
    let tmp_file_path = tmp_dir.join(&asset.name);
    let mut tmp_file = fs::File::create(&tmp_file_path)?;

    // Download with beautiful progress bar
    let client = reqwest::blocking::Client::new();
    let mut response = client.get(&asset.download_url).send()?;
    let total_size = response
        .content_length()
        .ok_or("Failed to get content length")?;

    let pb = ProgressBar::new(total_size);
    pb.set_style(
        ProgressStyle::with_template(
            "{prefix:>12.cyan.bold} [{bar:40.magenta/dim}] {bytes}/{total_bytes} ({bytes_per_sec}, {eta})",
        )?
        .progress_chars("⭓⭔-"),
    );
    pb.set_prefix("Downloading");

    let mut downloaded: u64 = 0;
    let mut buffer = [0; 8192];
    while let Ok(n) = std::io::Read::read(&mut response, &mut buffer) {
        if n == 0 {
            break;
        }
        std::io::Write::write_all(&mut tmp_file, &buffer[..n])?;
        downloaded += n as u64;
        pb.set_position(downloaded);
    }
    pb.finish_with_message("Download complete");

    println!(
        " {} Extracting and replacing binary...",
        style("📦").magenta()
    );

    self_update::Extract::from_source(&tmp_file_path).extract_into(&tmp_dir)?;

    let new_bin = tmp_dir.join(if cfg!(windows) { "swt.exe" } else { "swt" });
    self_update::self_replace::self_replace(new_bin)?;

    println!(
        "\n ✨ {}",
        style(format!("Successfully updated to v{}!", latest.version))
            .green()
            .bold()
    );

    // Clean up
    let _ = fs::remove_dir_all(&tmp_dir);

    Ok(())
}

fn print_update_msg(latest: &str, current: &str) {
    println!(
        "\n{}",
        style(format!(
            " 🚀 A new version of Sweet is available: v{latest} (current: v{current})"
        ))
        .yellow()
        .bold()
    );
    println!(
        "    Run {} to update.\n",
        style("swt update").cyan().italic()
    );
}
