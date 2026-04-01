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
        let rule = match_rule(&issue.message);
        let severity = to_lsp_severity(config.thresholds.severities.get(rule));

        diagnostics.push(create_diagnostic(
            Range::new(Position::new(0, 0), Position::new(0, 80)),
            severity,
            format!("🍬 Sweet: {}", issue.message),
            rule,
            None,
        ));
    }

    for (line, depth) in &report.deep_lines {
        #[allow(clippy::cast_possible_truncation)]
        let l = (*line as u32).saturating_sub(1);
        let severity = to_lsp_severity(config.thresholds.severities.get("max-depth"));

        diagnostics.push(create_diagnostic(
            Range::new(Position::new(l, 0), Position::new(l, 80)),
            severity,
            format!("🍬 Sweet: Excessive nesting depth: {depth}"),
            "max-depth",
            None,
        ));
    }

    report_duplicates(report, config, &mut diagnostics);
    diagnostics
}

fn match_rule(message: &str) -> &'static str {
    if message.contains("File too long") {
        "max-lines"
    } else if message.contains("Too many imports") {
        "max-imports"
    } else if message.contains("Excessive nesting") {
        "max-depth"
    } else if message.contains("repetition") {
        "max-repetition"
    } else {
        "unknown"
    }
}

const fn to_lsp_severity(severity: Severity) -> DiagnosticSeverity {
    match severity {
        Severity::Error => DiagnosticSeverity::ERROR,
        Severity::Warning => DiagnosticSeverity::WARNING,
    }
}

fn create_diagnostic(
    range: Range,
    severity: DiagnosticSeverity,
    message: String,
    rule: &str,
    related: Option<Vec<DiagnosticRelatedInformation>>,
) -> Diagnostic {
    Diagnostic {
        range,
        severity: Some(severity),
        message,
        source: Some("sweet".to_string()),
        related_information: related,
        data: Some(to_value(rule).unwrap_or_default()),
        ..Default::default()
    }
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

        let severity = to_lsp_severity(config.thresholds.severities.get("max-repetition"));

        diagnostics.push(create_diagnostic(
            Range::new(Position::new(start_line, 0), Position::new(end_line, 80)),
            severity,
            format!(
                "🍬 Sweet: Code duplication detected! (repeated in {} other places)",
                duplicate.occurrences.len()
            ),
            "max-repetition",
            Some(related_information),
        ));
    }
}
