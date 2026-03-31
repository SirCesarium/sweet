//! Logic for self-updating and version checking.

use console::style;
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::blocking::Client;
use self_update::backends::github::ReleaseList;
use self_update::self_replace;
use self_update::version::bump_is_greater;
use std::env;
use std::error;
use std::fs;
use std::io;
use std::path::Path;

/// Check for updates and prints a message if a new version is available.
pub fn check_for_updates() {
    let current_version = env!("CARGO_PKG_VERSION");

    println!("{}", style("🔍 Checking for updates...").cyan());

    let releases = ReleaseList::configure()
        .repo_owner("SirCesarium")
        .repo_name("sweet")
        .build();

    if let Some(latest_release) = releases
        .and_then(ReleaseList::fetch)
        .ok()
        .and_then(|latest| {
            latest
                .into_iter()
                .find(|r| bump_is_greater(current_version, &r.version).unwrap_or(false))
        })
    {
        print_update_msg(&latest_release.version, current_version);
    } else {
        println!("{}", style("Sweet is already up to date.").green());
    }
}

/// Perform the update process with a beautiful progress bar.
///
/// # Errors
///
/// Return an error if the network request fails, the binary cannot be extracted,
/// or the current executable cannot be replaced.
pub fn handle_update() -> Result<(), Box<dyn error::Error>> {
    println!("{}", style("🔍 Checking for updates...").cyan());
    let current_version = env!("CARGO_PKG_VERSION");
    let target = self_update::get_target();

    let releases = ReleaseList::configure()
        .repo_owner("SirCesarium")
        .repo_name("sweet")
        .build()?
        .fetch()?;

    let latest = releases
        .iter()
        .find(|r| bump_is_greater(current_version, &r.version).unwrap_or(false))
        .ok_or("Sweet is already up to date.")?;

    let asset = latest
        .assets
        .iter()
        .find(|a| a.name.starts_with("swt") && a.name.contains(target))
        .ok_or_else(|| {
            format!(
                "No compatible 'swt' binary found for {target} in v{}",
                latest.version
            )
        })?;

    println!(
        " 🚀 {} v{current_version} -> v{}",
        style("Updating Sweet:").bold(),
        style(&latest.version).green().bold()
    );

    let tmp_dir = env::temp_dir().join("sweet_update");
    if !tmp_dir.exists() {
        fs::create_dir_all(&tmp_dir)?;
    }
    let tmp_file_path = tmp_dir.join(&asset.name);

    download_asset(&asset.download_url, &tmp_file_path)?;

    replace_binary(
        &tmp_file_path,
        &tmp_dir,
        &latest.version,
        asset.name.contains(".tar") || asset.name.contains(".zip"),
    )?;

    let _ = fs::remove_dir_all(&tmp_dir);

    Ok(())
}

fn download_asset(url: &str, dest: &Path) -> Result<(), Box<dyn error::Error>> {
    let mut tmp_file = fs::File::create(dest)?;
    let client = Client::builder().user_agent("sweet-updater").build()?;

    let response = client
        .get(url)
        .header("Accept", "application/octet-stream")
        .send()?;

    let total_size = response.content_length();
    let pb = create_progress_bar(total_size)?;

    let mut source = pb.wrap_read(response);
    let downloaded = io::copy(&mut source, &mut tmp_file)?;
    pb.finish_with_message("Download complete");

    tmp_file.sync_all()?;
    drop(tmp_file);

    if downloaded == 0 {
        return Err(
            "Downloaded file is empty. Check your connection or GitHub release assets.".into(),
        );
    }
    Ok(())
}

fn create_progress_bar(total_size: Option<u64>) -> Result<ProgressBar, Box<dyn error::Error>> {
    let pb = if let Some(size) = total_size {
        let pb = ProgressBar::new(size);
        pb.set_style(
            ProgressStyle::with_template(
                "{prefix:>12.cyan.bold} [{bar:40.magenta/dim}] {bytes}/{total_bytes} ({bytes_per_sec}, {eta})",
            )?
            .progress_chars("⭓⭔-"),
        );
        pb
    } else {
        let pb = ProgressBar::new_spinner();
        pb.set_style(ProgressStyle::default_spinner());
        pb
    };
    pb.set_prefix("Downloading");
    Ok(pb)
}

fn replace_binary(
    tmp_file_path: &Path,
    tmp_dir: &Path,
    version: &str,
    is_archive: bool,
) -> Result<(), Box<dyn error::Error>> {
    if is_archive {
        println!(" {} Extracting package...", style("📦").magenta());
        let _ = self_update::Extract::from_source(tmp_file_path).extract_into(tmp_dir);
    }

    let mut new_bin = tmp_dir.join(if cfg!(windows) { "swt.exe" } else { "swt" });

    if !new_bin.exists() {
        if tmp_file_path.exists() {
            new_bin.clone_from(&tmp_file_path.to_path_buf());
        } else if let Ok(entries) = fs::read_dir(tmp_dir) {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                if name.starts_with("swt") && !name.contains(".tar") && !name.contains(".zip") {
                    new_bin = entry.path();
                    break;
                }
            }
        }
    }

    let new_bin_size = fs::metadata(&new_bin)?.len();
    if new_bin_size == 0 {
        return Err(format!(
            "The retrieved binary {} is empty (0 bytes).",
            new_bin.display()
        )
        .into());
    }

    println!(
        " {} Replacing binary (Source size: {} bytes)...",
        style("🚀").magenta(),
        new_bin_size
    );

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&new_bin)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&new_bin, perms)?;
    }

    self_replace::self_replace(&new_bin)?;

    println!(
        "\n ✨ {}",
        style(format!("Successfully updated to v{version}!"))
            .green()
            .bold()
    );

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
