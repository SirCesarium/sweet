// @swt-disable max-repetition
use crate::define_language;

define_language!(
    JavaScript,
    "JavaScript",
    extensions: ["js", "mjs", "cjs", "jsx"],
    line_comment: Some("//"),
    block_comment: Some(("/*", "*/")),
    import_keywords: ["import", "require("]
);
