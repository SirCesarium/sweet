// @swt-disable max-repetition
use crate::define_language;

define_language!(
    TypeScript,
    "TypeScript",
    extensions: ["ts", "tsx"],
    line_comment: Some("//"),
    block_comment: Some(("/*", "*/")),
    import_keywords: ["import", "require("]
);
