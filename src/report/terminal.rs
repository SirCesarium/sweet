//! Terminal-based report rendering.

use crate::FileReport;
use console::{Emoji, style};
use std::io::{self, BufWriter, Write};

/// Renders a summary of file reports to the terminal.
pub fn print_summary(reports: &[FileReport], quiet: bool) {
    let stdout = io::stdout();
    let mut handle = BufWriter::new(stdout.lock());

    if quiet {
        print_quiet_summary(&mut handle, reports);
        let _ = handle.flush();
        return;
    }

    let candy = Emoji("🍬 ", "!");
    let lemon_emoji = Emoji("🍋 ", "X");

    let total = reports.len();
    let sweet_count = reports.iter().filter(|r| r.is_sweet).count();
    let bitter_count = total - sweet_count;

    let _ = writeln!(
        handle,
        "\n{}",
        style(" Results Summary ").bold().cyan().on_black()
    );
    let _ = writeln!(handle, "{}", style("─".repeat(60)).dim());

    for report in reports {
        render_file_row(&mut handle, report);
    }

    let _ = writeln!(handle, "{}", style("─".repeat(60)).dim());

    let summary_text =
        format!("Total: {total}  |  Sweet: {sweet_count}  |  Bitter: {bitter_count}");
    let icon = if bitter_count == 0 {
        candy
    } else {
        lemon_emoji
    };
    let _ = writeln!(handle, "\n{} {}\n", icon, style(summary_text).bold());

    render_final_message(&mut handle, bitter_count);
    let _ = handle.flush();
}

fn render_file_row<W: Write>(handle: &mut W, report: &FileReport) {
    let path_str = report.path.to_string_lossy();
    let (lemon_t, bitter_t) = report.config.as_ref().map_or((200, 400), |c| {
        (c.ui.lemon_threshold, c.ui.bitter_threshold)
    });

    let line_emoji = if report.lines > bitter_t {
        " 🤕"
    } else if report.lines > lemon_t {
        " 🍋"
    } else {
        ""
    };

    let stats = format!(
        "{} lines · {} imports · depth {} · {:.1}% repeat{}",
        report.lines, report.imports, report.max_depth, report.repetition, line_emoji
    );

    if report.is_sweet {
        let _ = writeln!(
            handle,
            "{} {:<30} {}",
            style(" ✦ ").green().bold(),
            style(path_str).white(),
            style(stats).dim()
        );
    } else {
        let _ = writeln!(
            handle,
            "{} {:<30} {}",
            style(" ✘ ").red().bold(),
            style(path_str).magenta().bold(),
            style(stats).dim()
        );
        render_issues(handle, report);
    }
}

fn render_issues<W: Write>(handle: &mut W, report: &FileReport) {
    let issue_count = report.issues.len();
    let dup_count = report.duplicates.len();

    for (i, issue) in report.issues.iter().enumerate() {
        let is_last = i == issue_count - 1 && dup_count == 0;
        let connector = if is_last { " ╰─ " } else { " ├─ " };
        let _ = writeln!(
            handle,
            "    {}{}",
            style(connector).dim(),
            style(issue).yellow().italic()
        );
    }

    for (i, dup) in report.duplicates.iter().enumerate() {
        let is_last = i == dup_count - 1;
        let connector = if is_last { " ╰─ " } else { " ├─ " };
        let _ = writeln!(
            handle,
            "    {}{}",
            style(connector).dim(),
            style(format!("Duplicate found at line {}:", dup.line))
                .red()
                .bold()
        );

        for line in dup.content.lines().take(3) {
            let _ = writeln!(handle, "        {}", style(line).dim().italic());
        }

        for (path, line) in &dup.occurrences {
            let _ = writeln!(
                handle,
                "        {} {}",
                style(" also in:").dim(),
                style(format!("{}:{line}", path.display())).cyan()
            );
        }
    }
    let _ = writeln!(handle);
}

fn print_quiet_summary<W: Write>(handle: &mut W, reports: &[FileReport]) {
    let bitter_count = reports.iter().filter(|r| !r.is_sweet).count();
    if bitter_count == 0 {
        return;
    }

    for report in reports.iter().filter(|r| !r.is_sweet) {
        let _ = writeln!(
            handle,
            "{} {}: {}",
            style("BITTER").red().bold(),
            style(report.path.display()).white(),
            style(report.issues.join(", ")).yellow().italic()
        );
    }

    let total = reports.len();
    let _ = writeln!(
        handle,
        "\nSummary: {total} files analyzed, {} sweet, {bitter_count} bitter",
        style(total - bitter_count).green(),
    );
}

fn render_final_message<W: Write>(handle: &mut W, bitter_count: usize) {
    if bitter_count == 0 {
        let _ = writeln!(
            handle,
            "{}",
            style(" ✨ Your code is perfectly sweet! ✨ ")
                .green()
                .bold()
                .italic()
        );
    } else {
        let lollipop = Emoji("🍭 ", "!");
        let _ = writeln!(
            handle,
            "{}",
            style(format!(
                " {lollipop} Some files need a little more sugar...",
            ))
            .magenta()
            .bold()
        );
    }
}
