//! High-level analysis orchestration and parallel file processing.

use crate::analyzer::{RawMetrics, collect_issues, repetition};
use crate::{Config, FileReport, RepetitionDetail};
use dashmap::DashMap;
use ignore::WalkBuilder;
use indicatif::{ProgressBar, ProgressStyle};
use rayon::prelude::*;
use std::collections::{HashMap, HashSet};
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use std::str;
use std::sync::Arc;
use std::sync::mpsc::channel;
use std::time::Duration;

type ChunkMap = DashMap<u64, Vec<(PathBuf, usize)>>;
type Duplicates = Vec<(u64, Vec<(PathBuf, usize)>)>;

pub struct AnalysisEngine {
    root: PathBuf,
    config: Config,
}

#[allow(clippy::struct_excessive_bools)]
#[derive(Default, Clone, Copy)]
struct RunOptions {
    quiet: bool,
    show_progress: bool,
    inspect: bool,
    cross_file: bool,
}

impl AnalysisEngine {
    #[must_use]
    pub const fn new(root: PathBuf, config: Config) -> Self {
        Self { root, config }
    }

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

        let (tx, rx) = channel();
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

    #[allow(clippy::fn_params_excessive_bools)]
    #[must_use]
    pub fn run(
        &self,
        quiet: bool,
        show_progress: bool,
        inspect: bool,
        cross_file: bool,
    ) -> Vec<FileReport> {
        let entries = self.collect_files(quiet);
        if entries.is_empty() {
            return Vec::new();
        }

        let options = RunOptions {
            quiet,
            show_progress,
            inspect,
            cross_file,
        };

        let pb = Self::create_progress_bar(entries.len(), options.quiet, options.show_progress);
        let global_chunks: Arc<ChunkMap> = Arc::new(DashMap::new());
        let use_global = options.cross_file || self.config.cross_file_repetition;

        let mut reports: Vec<FileReport> = entries
            .par_iter()
            .filter_map(|path| {
                let res = self.analyze_and_collect(path, &global_chunks, options, use_global);
                if let Some(ref pb) = pb {
                    pb.inc(1);
                    if let Some(ref r) = res {
                        pb.set_message(r.path.to_string_lossy().to_string());
                    }
                }
                res
            })
            .collect();

        if let Some(pb) = pb {
            pb.finish_and_clear();
        }

        if use_global {
            Self::finalize_global_analysis(&mut reports, &global_chunks);
        } else if options.inspect {
            Self::finalize_local_inspection(&mut reports);
        }

        Self::sort_reports(&mut reports);
        reports
    }

    fn analyze_and_collect(
        &self,
        path: &Path,
        global_chunks: &ChunkMap,
        options: RunOptions,
        use_global: bool,
    ) -> Option<FileReport> {
        let (report, content) = super::analyze_file(path, &self.config, options.inspect)?;

        if use_global {
            let extension = path.extension()?.to_str()?;
            let thresholds = self.config.get_thresholds(extension);
            let window_size = thresholds.min_duplicate_lines;

            if report.hashes.len() >= window_size {
                let raw_hashes: Vec<u64> = report.hashes.iter().map(|(_, h)| *h).collect();
                let chunks = repetition::get_chunks(&raw_hashes, window_size);
                for (chunk, indices) in chunks {
                    for idx in indices {
                        let original_line = report.hashes[idx].0;
                        global_chunks
                            .entry(chunk)
                            .or_default()
                            .push((path.to_path_buf(), original_line));
                    }
                }
            }
        }

        drop(content);
        Some(report)
    }

    fn finalize_global_analysis(reports: &mut [FileReport], global_chunks: &ChunkMap) {
        let mut file_to_duplicated_lines: HashMap<PathBuf, HashSet<usize>> = HashMap::new();
        let mut global_duplicates: Duplicates = Vec::new();

        for entry in global_chunks {
            let (hash, occurrences) = entry.pair();
            if occurrences.len() > 1 {
                global_duplicates.push((*hash, occurrences.clone()));

                for (path, line) in occurrences {
                    file_to_duplicated_lines
                        .entry(path.clone())
                        .or_default()
                        .insert(*line);
                }
            }
        }

        for report in reports {
            let path = &report.path;
            let config = report.config.clone().unwrap_or_default();
            let extension = path.extension().and_then(OsStr::to_str).unwrap_or("");
            let thresholds = config.get_thresholds(extension);
            let window_size = thresholds.min_duplicate_lines;

            if let Some(duplicated_start_lines) = file_to_duplicated_lines.get(path) {
                let mut total_duplicated_indices = HashSet::new();
                for &start in duplicated_start_lines {
                    for i in 0..window_size {
                        total_duplicated_indices.insert(start + i);
                    }
                }

                if report.lines > 0 {
                    #[allow(clippy::cast_precision_loss)]
                    let new_percentage =
                        (total_duplicated_indices.len() as f64 / report.lines as f64) * 100.0;
                    report.repetition = new_percentage.min(100.0);
                }
            }

            let disabled_rules = fs::read(path).map_or_else(
                |_| HashSet::new(),
                |content| super::ignore::get_disabled_rules(&content),
            );

            let metrics = RawMetrics {
                lines: report.lines,
                imports: report.imports,
                max_depth: report.max_depth,
                repetition: report.repetition,
            };

            report.issues = collect_issues(&metrics, &thresholds, &config, &disabled_rules);
            report.is_sweet = report
                .issues
                .iter()
                .all(|i| i.severity != crate::Severity::Error);

            if !global_duplicates.is_empty()
                && let Ok(content) = fs::read_to_string(path)
            {
                let file_lines: Vec<&str> = content.lines().collect();

                for (_, occurrences) in &global_duplicates {
                    let local_pos: Vec<usize> = occurrences
                        .iter()
                        .filter(|(p, _)| p == path)
                        .map(|(_, l)| *l)
                        .collect();

                    for &start_line in &local_pos {
                        if report.duplicates.iter().any(|d| d.line == start_line) {
                            continue;
                        }

                        if start_line > 0 && start_line + window_size <= file_lines.len() {
                            let snippet =
                                file_lines[start_line - 1..start_line - 1 + window_size].join("\n");
                            let others: Vec<_> = occurrences
                                .iter()
                                .filter(|(p, l)| p != path || *l != start_line)
                                .cloned()
                                .collect();

                            if !others.is_empty() {
                                report.duplicates.push(RepetitionDetail {
                                    content: snippet,
                                    line: start_line,
                                    occurrences: others,
                                });
                            }
                        }
                    }
                }
            }
        }
    }

    fn finalize_local_inspection(reports: &mut [FileReport]) {
        for report in reports {
            if report.duplicates.is_empty() && !report.hashes.is_empty() {
                let window_size = report.config.as_ref().map_or(6, |c| {
                    let ext = report
                        .path
                        .extension()
                        .and_then(OsStr::to_str)
                        .unwrap_or("");
                    c.get_thresholds(ext).min_duplicate_lines
                });

                let raw_hashes: Vec<u64> = report.hashes.iter().map(|(_, h)| *h).collect();
                let chunks = repetition::get_chunks(&raw_hashes, window_size);
                if let Ok(content) = fs::read_to_string(&report.path) {
                    let file_lines: Vec<&str> = content.lines().collect();
                    for indices in chunks.values().filter(|v| v.len() > 1) {
                        for &idx in indices {
                            let start_line = report.hashes[idx].0;
                            if start_line > 0 && start_line + window_size <= file_lines.len() {
                                let snippet = file_lines
                                    [start_line - 1..start_line - 1 + window_size]
                                    .join("\n");
                                let others: Vec<_> = indices
                                    .iter()
                                    .filter(|&&i| i != idx)
                                    .map(|&i| (report.path.clone(), report.hashes[i].0))
                                    .collect();

                                report.duplicates.push(RepetitionDetail {
                                    content: snippet,
                                    line: start_line,
                                    occurrences: others,
                                });
                            }
                        }
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
            b.is_sweet.cmp(&a.is_sweet).then_with(|| {
                b.issues
                    .len()
                    .cmp(&a.issues.len())
                    .then_with(|| b.lines.cmp(&a.lines))
            })
        });
    }
}
