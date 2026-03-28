//! High-level analysis orchestration and parallel file processing.

use crate::analyzer::repetition;
use crate::{Config, FileReport, RepetitionDetail};
use dashmap::DashMap;
use ignore::WalkBuilder;
use indicatif::{ProgressBar, ProgressStyle};
use rayon::prelude::*;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

/// Type alias for mapping code chunks to their project-wide occurrences.
type ChunkMap = DashMap<Vec<u64>, Vec<(PathBuf, usize)>>;
/// Type alias for a list of duplicated chunks and their occurrences.
type Duplicates = Vec<(Vec<u64>, Vec<(PathBuf, usize)>)>;

/// The `AnalysisEngine` orchestrates the collection and parallel analysis of project files.
pub struct AnalysisEngine {
    root: PathBuf,
    config: Config,
}

impl AnalysisEngine {
    /// Creates a new `AnalysisEngine` instance.
    #[must_use]
    pub const fn new(root: PathBuf, config: Config) -> Self {
        Self { root, config }
    }

    /// Recursively collects all supported files within the root directory.
    ///
    /// Utilizes a parallel walker for high-speed file system discovery.
    #[must_use]
    pub fn collect_files(&self, quiet: bool) -> Vec<PathBuf> {
        let spinner = if quiet {
            None
        } else {
            let sp = ProgressBar::new_spinner();
            let style = ProgressStyle::with_template("{spinner:.magenta} {msg}")
                .unwrap_or_else(|_| ProgressStyle::default_spinner())
                .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"]);
            sp.set_style(style);
            sp.set_message("Discovering project files...");
            sp.enable_steady_tick(Duration::from_millis(80));
            Some(sp)
        };

        let mut walk_builder = WalkBuilder::new(&self.root);
        for exclude in &self.config.exclude {
            walk_builder.add_ignore(exclude);
        }

        let (tx, rx) = std::sync::mpsc::channel();
        walk_builder.build_parallel().run(|| {
            let tx = tx.clone();
            Box::new(move |result| {
                if let Some(entry) = result.ok().filter(|e| Config::is_supported_file(e.path())) {
                    let _ = tx.send(entry.path().to_path_buf());
                }
                ignore::WalkState::Continue
            })
        });

        drop(tx);
        let entries: Vec<PathBuf> = rx.into_iter().collect();

        if let Some(sp) = spinner {
            sp.finish_and_clear();
        }

        entries
    }

    /// Executes the analysis phase and performs global repetition inspection if requested.
    #[must_use]
    pub fn run(&self, quiet: bool, show_progress: bool, inspect: bool) -> Vec<FileReport> {
        let entries = self.collect_files(quiet);
        if entries.is_empty() {
            return Vec::new();
        }

        let pb = Self::create_progress_bar(entries.len(), quiet, show_progress);
        let global_chunks: Arc<ChunkMap> = Arc::new(DashMap::new());

        let mut reports: Vec<FileReport> = entries
            .par_iter()
            .filter_map(|path| {
                let res = self.analyze_and_collect(path, &global_chunks, inspect);
                if let Some(ref pb) = pb {
                    pb.inc(1);
                    if let Some(ref r) = res {
                        pb.set_message(format!("{}", r.path.display()));
                    }
                }
                res
            })
            .collect();

        if let Some(pb) = pb {
            pb.finish_and_clear();
        }

        if inspect {
            Self::finalize_inspection(&mut reports, &global_chunks);
        }

        Self::sort_reports(&mut reports);
        reports
    }

    /// Analyzes a file and optionally collects chunk hashes for global duplication detection.
    fn analyze_and_collect(
        &self,
        path: &Path,
        global_chunks: &ChunkMap,
        inspect: bool,
    ) -> Option<FileReport> {
        let content = std::fs::read_to_string(path).ok()?;
        let report = super::analyze_file(path, &self.config)?;

        if inspect {
            let extension = path.extension()?.to_str()?;
            let clean = super::uncomment::remove_comments(&content, extension, true);
            let rep_res = repetition::analyze_repetition(&clean);

            let window_size = 4;
            if rep_res.hashes.len() >= window_size {
                for i in 0..=rep_res.hashes.len() - window_size {
                    let chunk = rep_res.hashes[i..i + window_size].to_vec();
                    global_chunks
                        .entry(chunk)
                        .or_default()
                        .push((path.to_path_buf(), i + 1));
                }
            }
        }

        Some(report)
    }

    /// Finalizes the inspection by mapping duplicated chunks back to source files.
    fn finalize_inspection(reports: &mut [FileReport], global_chunks: &ChunkMap) {
        let duplicates: Duplicates = global_chunks
            .iter()
            .filter(|entry| entry.value().len() > 1)
            .map(|entry| (entry.key().clone(), entry.value().clone()))
            .collect();

        for report in reports {
            for (_, occurrences) in &duplicates {
                if let Some(pos) = occurrences.iter().find(|(p, _)| p == &report.path)
                    && let Ok(content) = std::fs::read_to_string(&report.path)
                {
                    let lines: Vec<String> = content
                        .lines()
                        .map(std::string::ToString::to_string)
                        .collect();
                    let start_line = pos.1;
                    if start_line > 0 && start_line + 3 <= lines.len() {
                        let snippet = lines[start_line - 1..start_line + 3].join("\n");

                        report.duplicates.push(RepetitionDetail {
                            content: snippet,
                            line: start_line,
                            occurrences: occurrences
                                .iter()
                                .filter(|(p, l)| p != &report.path || l != &start_line)
                                .cloned()
                                .collect(),
                        });
                    }
                }
            }
        }
    }

    fn create_progress_bar(len: usize, quiet: bool, show_progress: bool) -> Option<ProgressBar> {
        if quiet || !show_progress {
            return None;
        }
        let pb = ProgressBar::new(len as u64);
        let style = ProgressStyle::with_template(
            "{prefix:>12.cyan.bold} [{bar:40.magenta/dim}] {pos}/{len} {msg}",
        )
        .unwrap_or_else(|_| ProgressStyle::default_bar())
        .progress_chars("⭓⭔-");

        pb.set_style(style);
        pb.set_prefix("Analyzing");
        Some(pb)
    }

    fn sort_reports(reports: &mut [FileReport]) {
        reports.sort_by(|a, b| {
            b.is_sweet
                .cmp(&a.is_sweet)
                .then_with(|| b.issues.len().cmp(&a.issues.len()))
                .then_with(|| b.lines.cmp(&a.lines))
        });
    }
}
