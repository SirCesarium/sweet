//! Sweet CLI: Blazing-fast code health analyzer.

#![deny(clippy::pedantic)]

use clap::{Parser, Subcommand};
use console::style;
use std::fs;
use std::path::{Path, PathBuf};
use swt::Config;
use swt::analyzer::AnalysisEngine;

const ASCII: &str = r"
                            __ 
   ______      _____  ___  / /_
  / ___/ | /| / / _ \/ _ \/ __/
 /__  /| |/ |/ /  __/  __/ /_  
/____/ |__/|__/\___/\___/\__/  ";

/// Sweet CLI: High-performance code health and architectural integrity analyzer.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Subcommand to execute. If omitted, performs a standard health check on the path.
    #[command(subcommand)]
    command: Option<Commands>,

    /// Path to analyze (default: current directory).
    #[arg(default_value = ".")]
    path: PathBuf,

    /// Output report in JSON format.
    #[allow(clippy::option_option)]
    #[arg(long, value_name = "FILE")]
    json: Option<Option<PathBuf>>,

    /// Minimal output for CI environments.
    #[arg(short, long)]
    quiet: bool,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Check for updates and install if available.
    Update,
    /// Check for updates without installing.
    CheckUpdates,
    /// Detailed inspection of code duplication and repetition.
    Inspect {
        /// Path to inspect (default: current directory).
        #[arg(default_value = ".")]
        path: PathBuf,
    },
    /// Strip comments from a specific file.
    Uncomment {
        /// File to strip comments from.
        #[arg(value_name = "FILE")]
        path: PathBuf,

        /// Remove even doc comments (///, /**).
        #[arg(long, short)]
        aggressive: bool,
    },
}

fn main() -> std::process::ExitCode {
    let args = Args::parse();

    match args.command {
        Some(Commands::Update) => {
            return match swt::update::handle_update() {
                Ok(()) => std::process::ExitCode::SUCCESS,
                Err(_) => std::process::ExitCode::FAILURE,
            };
        }
        Some(Commands::CheckUpdates) => {
            swt::update::check_for_updates();
            return std::process::ExitCode::SUCCESS;
        }
        Some(Commands::Inspect { path }) => {
            return run_analysis(&path, args.json.as_ref(), args.quiet, true);
        }
        Some(Commands::Uncomment { path, aggressive }) => {
            if handle_uncomment(&path, aggressive) {
                return std::process::ExitCode::SUCCESS;
            }
            return std::process::ExitCode::FAILURE;
        }
        None => {}
    }

    run_analysis(&args.path, args.json.as_ref(), args.quiet, false)
}

fn run_analysis(
    path: &Path,
    #[allow(clippy::option_option)] json: Option<&Option<PathBuf>>,
    quiet: bool,
    inspect: bool,
) -> std::process::ExitCode {
    let config = Config::load(path);
    let engine = AnalysisEngine::new(path.to_path_buf(), config);

    if !quiet && json.is_none() {
        show_branding();
    }

    let reports = engine.run(quiet, json.is_none(), inspect);

    if reports.is_empty() {
        if !quiet {
            println!(
                "\n{}",
                style(" 📭 No supported files found.").yellow().bold()
            );
        }
        return std::process::ExitCode::SUCCESS;
    }

    let bitter_count = reports.iter().filter(|r| !r.is_sweet).count();

    if let Some(json_opt) = json {
        handle_json_reporting(&reports, json_opt.as_ref());
    } else {
        swt::report::print_reports(&reports, quiet, None);
    }

    if bitter_count > 0 {
        std::process::ExitCode::FAILURE
    } else {
        std::process::ExitCode::SUCCESS
    }
}

fn handle_json_reporting(reports: &[swt::FileReport], json_opt: Option<&PathBuf>) {
    if let Some(path) = json_opt {
        swt::report::json::write_json_report(reports, path);
    } else if let Ok(json) = serde_json::to_string_pretty(&reports) {
        println!("{json}");
    }
}

fn handle_uncomment(path: &Path, aggressive: bool) -> bool {
    match fs::read_to_string(path) {
        Ok(content) => {
            let extension = path.extension().and_then(|s| s.to_str()).unwrap_or("");
            let clean = swt::analyzer::uncomment::remove_comments(&content, extension, aggressive);
            if fs::write(path, clean).is_ok() {
                println!("{}", style("Uncommented!").cyan().bold());
                true
            } else {
                eprintln!("{}", style("Error: Could not write to file").red());
                false
            }
        }
        Err(e) => {
            eprintln!("Error: Could not read file {}: {}", path.display(), e);
            false
        }
    }
}

fn show_branding() {
    println!("{}", style(ASCII).magenta().bold());
    println!(
        "\n{}",
        style("— A blazing-fast code health analyzer :)")
            .italic()
            .cyan()
    );
}
