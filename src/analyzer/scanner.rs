//! High-performance single-pass file analysis scanner.

use crate::languages::{Language, LanguageRegistry};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::str;

/// Type alias for 1-based line numbers.
pub type LineNumber = usize;
/// Type alias for byte offsets within a file buffer.
pub type ByteOffset = usize;

/// Results gathered from a single-pass scan of a source file.
pub struct ScanResult {
    /// Total number of source lines.
    pub lines: usize,
    /// Total number of import/include statements found.
    pub imports: usize,
    /// Maximum nesting depth based on indentation.
    pub max_depth: usize,
    /// Line numbers where the nesting depth exceeds a threshold.
    pub deep_lines: Vec<(LineNumber, usize)>,
    /// Pre-computed hashes of lines that contain actual code.
    pub hashes: Vec<(LineNumber, u64)>,
}

struct ScannerState<'a> {
    import_keywords: &'a [&'a str],
    depth_threshold: usize,
    indent_size: usize,
    line_comment: Option<&'a [u8]>,
    block_start: Option<&'a [u8]>,
    block_end: Option<&'a [u8]>,
}

/// Perform a single-pass analysis on the provided content.
#[must_use]
pub fn scan(
    content: &[u8],
    extension: &str,
    depth_threshold: usize,
    indent_size: usize,
) -> ScanResult {
    let lang = LanguageRegistry::get().get_by_extension(extension);
    let state = ScannerState {
        import_keywords: lang.map_or(&[] as &[&str], Language::import_keywords),
        line_comment: lang.and_then(Language::line_comment).map(str::as_bytes),
        block_start: lang
            .and_then(Language::block_comment)
            .map(|(s, _)| s.as_bytes()),
        block_end: lang
            .and_then(Language::block_comment)
            .map(|(_, e)| e.as_bytes()),
        depth_threshold,
        indent_size,
    };

    let mut res = ScanResult {
        lines: 0,
        imports: 0,
        max_depth: 0,
        deep_lines: Vec::new(),
        hashes: Vec::new(),
    };

    let (mut flags, mut line_data) = (ParseFlags::default(), LineData::new());
    let mut i: ByteOffset = 0;

    while i < content.len() {
        let current = content[i];
        if current == b'\n' {
            process_line_end(content, &line_data, i, &mut res, &state);
            res.lines += 1;
            line_data.reset(i + 1);
            if !flags.in_block_comment {
                flags.in_line_comment = false;
            }
            i += 1;
            continue;
        }
        if flags.in_string {
            handle_string_content(content, &mut i, &mut flags, &mut line_data.clean_line);
            continue;
        }
        if let Some(end) = state.block_end
            && flags.in_block_comment
            && content[i..].starts_with(end)
        {
            flags.in_block_comment = false;
            i += end.len();
            continue;
        }
        if flags.in_block_comment || flags.in_line_comment {
            i += 1;
            continue;
        }
        if line_data.is_at_start {
            if current == b' ' || current == b'\t' {
                line_data.leading_whitespace += if current == b' ' {
                    1
                } else {
                    state.indent_size
                };
                i += 1;
                continue;
            }
            line_data.is_at_start = false;
            line_data.start_offset = i;
        }
        if let Some(prefix) = state.line_comment
            && content[i..].starts_with(prefix)
        {
            flags.in_line_comment = true;
            i += prefix.len();
            continue;
        }
        if let Some(start) = state.block_start
            && content[i..].starts_with(start)
        {
            flags.in_block_comment = true;
            i += start.len();
            continue;
        }
        if current == b'\"' || current == b'\'' || current == b'`' {
            flags.in_string = true;
            flags.string_char = current;
            line_data.clean_line.push(current);
            i += 1;
            continue;
        }
        line_data.clean_line.push(current);
        i += 1;
    }
    if !line_data.is_at_start || i > line_data.start_offset {
        process_line_end(content, &line_data, i, &mut res, &state);
        res.lines += 1;
    }
    res
}

#[derive(Default)]
struct ParseFlags {
    in_string: bool,
    string_char: u8,
    in_block_comment: bool,
    in_line_comment: bool,
}

struct LineData {
    is_at_start: bool,
    leading_whitespace: usize,
    start_offset: ByteOffset,
    num: LineNumber,
    clean_line: Vec<u8>,
}

impl LineData {
    const fn new() -> Self {
        Self {
            is_at_start: true,
            leading_whitespace: 0,
            start_offset: 0,
            num: 1,
            clean_line: Vec::new(),
        }
    }

    fn reset(&mut self, next_start: ByteOffset) {
        self.is_at_start = true;
        self.leading_whitespace = 0;
        self.start_offset = next_start;
        self.num += 1;
        self.clean_line.clear();
    }
}

fn handle_string_content(
    content: &[u8],
    i: &mut ByteOffset,
    flags: &mut ParseFlags,
    clean_line: &mut Vec<u8>,
) {
    let current = content[*i];
    clean_line.push(current);
    if current == b'\\' && *i + 1 < content.len() {
        clean_line.push(content[*i + 1]);
        *i += 2;
    } else {
        if current == flags.string_char {
            flags.in_string = false;
        }
        *i += 1;
    }
}

fn process_line_end(
    content: &[u8],
    line: &LineData,
    end_offset: ByteOffset,
    res: &mut ScanResult,
    state: &ScannerState,
) {
    let depth = line.leading_whitespace / state.indent_size;
    if depth > res.max_depth {
        res.max_depth = depth;
    }
    if depth > state.depth_threshold {
        res.deep_lines.push((line.num, depth));
    }

    if end_offset > line.start_offset {
        let line_bytes = &content[line.start_offset..end_offset];
        if let Ok(text) = str::from_utf8(line_bytes) {
            let trimmed = text.trim();
            if !trimmed.is_empty() {
                for keyword in state.import_keywords {
                    if trimmed.starts_with(keyword) {
                        res.imports += 1;
                        break;
                    }
                }
            }
        }
    }

    let trimmed = trim_bytes(&line.clean_line);
    if !trimmed.is_empty() {
        let mut line_hasher = DefaultHasher::new();
        trimmed.hash(&mut line_hasher);
        res.hashes.push((line.num, line_hasher.finish()));
    }
}

fn trim_bytes(bytes: &[u8]) -> &[u8] {
    let start = bytes
        .iter()
        .position(|&b| !b.is_ascii_whitespace())
        .unwrap_or(bytes.len());
    let end = bytes
        .iter()
        .rposition(|&b| !b.is_ascii_whitespace())
        .map_or(start, |p| p + 1);
    &bytes[start..end]
}
