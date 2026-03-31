//! Diagnostic generation logic for Sweet LSP.

use serde_json::to_value;
use swt::{Config, FileReport, Severity};
use tower_lsp::lsp_types::{
    Diagnostic, DiagnosticRelatedInformation, DiagnosticSeverity, Location, Position, Range, Url,
};

/// Generates a list of diagnostics from a file report.
pub fn generate(report: &FileReport, config: &Config) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();

    for issue in &report.issues {
        let rule = if issue.message.contains("File too long") {
            "max-lines"
        } else if issue.message.contains("Too many imports") {
            "max-imports"
        } else if issue.message.contains("Excessive nesting") {
            "max-depth"
        } else if issue.message.contains("repetition") {
            "max-repetition"
        } else {
            "unknown"
        };

        let severity = match config.thresholds.severities.get(rule) {
            Severity::Error => DiagnosticSeverity::ERROR,
            Severity::Warning => DiagnosticSeverity::WARNING,
        };

        diagnostics.push(Diagnostic {
            range: Range::new(Position::new(0, 0), Position::new(0, 80)),
            severity: Some(severity),
            message: format!("🍬 Sweet: {}", issue.message),
            source: Some("sweet".to_string()),
            data: Some(to_value(rule).unwrap_or_default()),
            ..Default::default()
        });
    }

    for (line, depth) in &report.deep_lines {
        #[allow(clippy::cast_possible_truncation)]
        let l = (*line as u32).saturating_sub(1);
        let severity = match config.thresholds.severities.get("max-depth") {
            Severity::Error => DiagnosticSeverity::ERROR,
            Severity::Warning => DiagnosticSeverity::WARNING,
        };

        diagnostics.push(Diagnostic {
            range: Range::new(Position::new(l, 0), Position::new(l, 80)),
            severity: Some(severity),
            message: format!("🍬 Sweet: Excessive nesting depth: {depth}"),
            source: Some("sweet".to_string()),
            data: Some(to_value("max-depth").unwrap_or_default()),
            ..Default::default()
        });
    }

    report_duplicates(report, config, &mut diagnostics);
    diagnostics
}

/// Append duplication diagnostics to the list.
fn report_duplicates(report: &FileReport, config: &Config, diagnostics: &mut Vec<Diagnostic>) {
    for duplicate in &report.duplicates {
        #[allow(clippy::cast_possible_truncation)]
        let start_line = (duplicate.line as u32).saturating_sub(1);
        #[allow(clippy::cast_possible_truncation)]
        let line_count = duplicate.content.lines().count() as u32;
        let end_line = start_line + line_count.saturating_sub(1);

        let mut related_information = Vec::new();
        for (other_path, other_line) in &duplicate.occurrences {
            if let Ok(other_uri) = Url::from_file_path(other_path) {
                #[allow(clippy::cast_possible_truncation)]
                let l = (*other_line as u32).saturating_sub(1);
                related_information.push(DiagnosticRelatedInformation {
                    location: Location::new(
                        other_uri,
                        Range::new(Position::new(l, 0), Position::new(l, 80)),
                    ),
                    message: format!("Duplicate found here (line {other_line})"),
                });
            }
        }

        let severity = match config.thresholds.severities.get("max-repetition") {
            Severity::Error => DiagnosticSeverity::ERROR,
            Severity::Warning => DiagnosticSeverity::WARNING,
        };

        diagnostics.push(Diagnostic {
            range: Range::new(Position::new(start_line, 0), Position::new(end_line, 80)),
            severity: Some(severity),
            message: format!(
                "🍬 Sweet: Code duplication detected! (repeated in {} other places)",
                duplicate.occurrences.len()
            ),
            source: Some("sweet".to_string()),
            related_information: Some(related_information),
            data: Some(to_value("max-repetition").unwrap_or_default()),
            ..Default::default()
        });
    }
}
