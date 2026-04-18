// Ported from openai/codex apply-patch/src/parser.rs
// https://github.com/openai/codex/tree/fe7c959e90d46abb8311e4a0b369e6cb32bf337e
// Licensed under Apache License 2.0. See NOTICE at workspace root.

//! This module is responsible for parsing & validating a patch into a list of "hunks".
//! (It does not attempt to actually check that the patch can be applied to the filesystem.)
//!
//! The official Lark grammar for the apply-patch format is:
//!
//! start: begin_patch hunk+ end_patch
//! begin_patch: "*** Begin Patch" LF
//! end_patch: "*** End Patch" LF?
//!
//! hunk: add_hunk | delete_hunk | update_hunk
//! add_hunk: "*** Add File: " filename LF add_line+
//! delete_hunk: "*** Delete File: " filename LF
//! update_hunk: "*** Update File: " filename LF change_move? change?
//! filename: /(.+)/
//! add_line: "+" /(.+)/ LF -> line
//!
//! change_move: "*** Move to: " filename LF
//! change: (change_context | change_line)+ eof_line?
//! change_context: ("@@" | "@@ " /(.+)/) LF
//! change_line: ("+" | "-" | " ") /(.+)/ LF
//! eof_line: "*** End of File" LF
//!
//! The parser below is a little more lenient than the explicit spec and allows for
//! leading/trailing whitespace around patch markers.
use std::path::PathBuf;

const BEGIN_PATCH_MARKER: &str = "*** Begin Patch";
const END_PATCH_MARKER: &str = "*** End Patch";
const ADD_FILE_MARKER: &str = "*** Add File: ";
const DELETE_FILE_MARKER: &str = "*** Delete File: ";
const UPDATE_FILE_MARKER: &str = "*** Update File: ";
const MOVE_TO_MARKER: &str = "*** Move to: ";
const EOF_MARKER: &str = "*** End of File";
const CHANGE_CONTEXT_MARKER: &str = "@@ ";
const EMPTY_CHANGE_CONTEXT_MARKER: &str = "@@";

#[derive(Debug, PartialEq, Clone)]
pub enum ParseError {
    InvalidPatchError(String),
    InvalidHunkError {
        message: String,
        line_number: usize,
        snippet: Option<String>,
    },
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::InvalidPatchError(msg) => write!(f, "invalid patch: {msg}"),
            ParseError::InvalidHunkError {
                message,
                line_number,
                snippet,
            } => write!(
                f,
                "invalid hunk at line {line_number}, {message}{}",
                snippet.as_deref().unwrap_or("")
            ),
        }
    }
}

impl std::error::Error for ParseError {}

use ParseError::*;

#[derive(Debug, PartialEq, Clone)]
#[allow(clippy::enum_variant_names)]
pub enum Hunk {
    AddFile {
        path: PathBuf,
        contents: String,
    },
    DeleteFile {
        path: PathBuf,
    },
    UpdateFile {
        path: PathBuf,
        move_path: Option<PathBuf>,

        /// Chunks should be in order, i.e. the `change_context` of one chunk
        /// should occur later in the file than the previous chunk.
        chunks: Vec<UpdateFileChunk>,
    },
}

use Hunk::*;

#[derive(Debug, PartialEq, Clone)]
pub struct UpdateFileChunk {
    /// A single line of context used to narrow down the position of the chunk
    /// (this is usually a class, method, or function definition.)
    pub change_context: Option<String>,

    /// A contiguous block of lines that should be replaced with `new_lines`.
    /// `old_lines` must occur strictly after `change_context`.
    pub old_lines: Vec<String>,
    pub new_lines: Vec<String>,

    /// If set to true, `old_lines` must occur at the end of the source file.
    /// (Tolerance around trailing newlines should be encouraged.)
    pub is_end_of_file: bool,
}

pub fn parse_patch(patch: &str) -> Result<Vec<Hunk>, ParseError> {
    parse_patch_text(patch)
}

fn parse_patch_text(patch: &str) -> Result<Vec<Hunk>, ParseError> {
    let lines: Vec<&str> = patch.trim().lines().collect();
    let (_patch_lines, hunk_lines) = check_patch_boundaries_strict(&lines)?;

    let mut hunks: Vec<Hunk> = Vec::new();
    let mut remaining_lines = hunk_lines;
    let mut line_number = 2;
    while !remaining_lines.is_empty() {
        let (hunk, hunk_lines) =
            parse_one_hunk(remaining_lines, line_number).map_err(|e| annotate(e, &lines))?;
        hunks.push(hunk);
        line_number += hunk_lines;
        remaining_lines = &remaining_lines[hunk_lines..]
    }
    Ok(hunks)
}

/// Attach a snippet of the patch body around the failing line to a hunk
/// parse error, so callers can see what they wrote without counting lines.
fn annotate(err: ParseError, all_lines: &[&str]) -> ParseError {
    match err {
        ParseError::InvalidHunkError {
            message,
            line_number,
            snippet: None,
        } => {
            let snippet = snippet_for(all_lines, line_number);
            ParseError::InvalidHunkError {
                message,
                line_number,
                snippet,
            }
        }
        other => other,
    }
}

fn snippet_for(lines: &[&str], line_number: usize) -> Option<String> {
    if line_number == 0 || line_number > lines.len() {
        return None;
    }
    let idx = line_number - 1;
    let start = idx.saturating_sub(1);
    let end = (idx + 2).min(lines.len());
    let width = end.to_string().len();
    let mut out = String::from("\npatch near error:\n");
    for (offset, line) in lines[start..end].iter().enumerate() {
        let num = start + offset + 1;
        let marker = if start + offset == idx {
            ">>> "
        } else {
            "    "
        };
        out.push_str(&format!("{marker}{num:>width$}: {}\n", line, width = width));
    }
    Some(out)
}

/// Checks the start and end lines of the patch text for `apply_patch`,
/// returning an error if they do not match the expected markers.
fn check_patch_boundaries_strict<'a>(
    lines: &'a [&'a str],
) -> Result<(&'a [&'a str], &'a [&'a str]), ParseError> {
    let (first_line, last_line) = match lines {
        [] => (None, None),
        [first] => (Some(first), Some(first)),
        [first, .., last] => (Some(first), Some(last)),
    };
    check_start_and_end_lines_strict(first_line, last_line)?;
    Ok((lines, &lines[1..lines.len() - 1]))
}

fn check_start_and_end_lines_strict(
    first_line: Option<&&str>,
    last_line: Option<&&str>,
) -> Result<(), ParseError> {
    let first_trimmed = first_line.map(|line| line.trim());
    let last_trimmed = last_line.map(|line| line.trim());

    match (first_trimmed, last_trimmed) {
        (Some(first), Some(last)) if first == BEGIN_PATCH_MARKER && last == END_PATCH_MARKER => {
            Ok(())
        }
        (Some(first), _) if first != BEGIN_PATCH_MARKER => Err(InvalidPatchError(
            boundary_error_message("first", BEGIN_PATCH_MARKER, first_line.copied()),
        )),
        _ => Err(InvalidPatchError(boundary_error_message(
            "last",
            END_PATCH_MARKER,
            last_line.copied(),
        ))),
    }
}

fn boundary_error_message(which: &str, expected: &str, observed: Option<&str>) -> String {
    let Some(line) = observed else {
        return format!("The {which} line of the patch must be '{expected}' (patch was empty)");
    };
    let trimmed = line.trim();
    let hint = marker_prefix_hint(trimmed, expected);
    let base = format!("The {which} line of the patch must be '{expected}', got: {line:?}");
    match hint {
        Some(h) => format!("{base}. {h}"),
        None => base,
    }
}

fn marker_prefix_hint(trimmed: &str, expected: &str) -> Option<&'static str> {
    for prefix in ["+", "-", " "] {
        if let Some(rest) = trimmed.strip_prefix(prefix)
            && rest.trim_start() == expected
        {
            return Some(
                "It looks like the envelope marker was written as a hunk line. \
                 Drop the leading '+', '-', or ' ' prefix so the marker terminates the envelope.",
            );
        }
    }
    None
}

/// Attempts to parse a single hunk from the start of lines.
/// Returns the parsed hunk and the number of lines parsed (or a ParseError).
fn parse_one_hunk(lines: &[&str], line_number: usize) -> Result<(Hunk, usize), ParseError> {
    // Be tolerant of case mismatches and extra padding around marker strings.
    let first_line = lines[0].trim();
    if let Some(path) = first_line.strip_prefix(ADD_FILE_MARKER) {
        // Add File
        let mut contents = String::new();
        let mut parsed_lines = 1;
        for add_line in &lines[1..] {
            if let Some(line_to_add) = add_line.strip_prefix('+') {
                contents.push_str(line_to_add);
                contents.push('\n');
                parsed_lines += 1;
            } else {
                break;
            }
        }
        return Ok((
            AddFile {
                path: PathBuf::from(path),
                contents,
            },
            parsed_lines,
        ));
    } else if let Some(path) = first_line.strip_prefix(DELETE_FILE_MARKER) {
        // Delete File
        return Ok((
            DeleteFile {
                path: PathBuf::from(path),
            },
            1,
        ));
    } else if let Some(path) = first_line.strip_prefix(UPDATE_FILE_MARKER) {
        // Update File
        let mut remaining_lines = &lines[1..];
        let mut parsed_lines = 1;

        // Optional: move file line
        let move_path = remaining_lines
            .first()
            .and_then(|x| x.strip_prefix(MOVE_TO_MARKER));

        if move_path.is_some() {
            remaining_lines = &remaining_lines[1..];
            parsed_lines += 1;
        }

        let mut chunks = Vec::new();
        // NOTE: we need to know to stop once we reach the next special marker header.
        while !remaining_lines.is_empty() {
            // Skip over any completely blank lines that may separate chunks.
            if remaining_lines[0].trim().is_empty() {
                parsed_lines += 1;
                remaining_lines = &remaining_lines[1..];
                continue;
            }

            if remaining_lines[0].starts_with('*') {
                break;
            }

            let (chunk, chunk_lines) = parse_update_file_chunk(
                remaining_lines,
                line_number + parsed_lines,
                chunks.is_empty(),
            )?;
            chunks.push(chunk);
            parsed_lines += chunk_lines;
            remaining_lines = &remaining_lines[chunk_lines..]
        }

        if chunks.is_empty() && move_path.is_none() {
            return Err(InvalidHunkError {
                message: format!("Update file hunk for path '{path}' is empty"),
                line_number,
                snippet: None,
            });
        }

        return Ok((
            UpdateFile {
                path: PathBuf::from(path),
                move_path: move_path.map(PathBuf::from),
                chunks,
            },
            parsed_lines,
        ));
    }

    Err(InvalidHunkError {
        message: format!(
            "'{first_line}' is not a valid hunk header. Valid hunk headers: '*** Add File: {{path}}', '*** Delete File: {{path}}', '*** Update File: {{path}}'"
        ),
        line_number,
        snippet: None,
    })
}

fn parse_update_file_chunk(
    lines: &[&str],
    line_number: usize,
    allow_missing_context: bool,
) -> Result<(UpdateFileChunk, usize), ParseError> {
    if lines.is_empty() {
        return Err(InvalidHunkError {
            message: "Update hunk does not contain any lines".to_string(),
            line_number,
            snippet: None,
        });
    }
    // If we see an explicit context marker @@ or @@ <context>, consume it; otherwise, optionally
    // allow treating the chunk as starting directly with diff lines.
    let (change_context, start_index) = if lines[0] == EMPTY_CHANGE_CONTEXT_MARKER {
        (None, 1)
    } else if let Some(context) = lines[0].strip_prefix(CHANGE_CONTEXT_MARKER) {
        (Some(context.to_string()), 1)
    } else {
        if !allow_missing_context {
            return Err(InvalidHunkError {
                message: format!(
                    "Expected update hunk to start with a @@ context marker, got: '{}'",
                    lines[0]
                ),
                line_number,
                snippet: None,
            });
        }
        (None, 0)
    };
    if start_index >= lines.len() {
        return Err(InvalidHunkError {
            message: "Update hunk does not contain any lines".to_string(),
            line_number: line_number + 1,
            snippet: None,
        });
    }
    let mut chunk = UpdateFileChunk {
        change_context,
        old_lines: Vec::new(),
        new_lines: Vec::new(),
        is_end_of_file: false,
    };
    let mut parsed_lines = 0;
    for line in &lines[start_index..] {
        match *line {
            EOF_MARKER => {
                if parsed_lines == 0 {
                    return Err(InvalidHunkError {
                        message: "Update hunk does not contain any lines".to_string(),
                        line_number: line_number + 1,
                        snippet: None,
                    });
                }
                chunk.is_end_of_file = true;
                parsed_lines += 1;
                break;
            }
            line_contents => {
                match line_contents.chars().next() {
                    None => {
                        // Interpret this as an empty line.
                        chunk.old_lines.push(String::new());
                        chunk.new_lines.push(String::new());
                    }
                    Some(' ') => {
                        chunk.old_lines.push(line_contents[1..].to_string());
                        chunk.new_lines.push(line_contents[1..].to_string());
                    }
                    Some('+') => {
                        chunk.new_lines.push(line_contents[1..].to_string());
                    }
                    Some('-') => {
                        chunk.old_lines.push(line_contents[1..].to_string());
                    }
                    _ => {
                        if parsed_lines == 0 {
                            return Err(InvalidHunkError {
                                message: format!(
                                    "Unexpected line found in update hunk: '{line_contents}'. Every line should start with ' ' (context line), '+' (added line), or '-' (removed line)"
                                ),
                                line_number: line_number + 1,
                                snippet: None,
                            });
                        }
                        // Assume this is the start of the next hunk.
                        break;
                    }
                }
                parsed_lines += 1;
            }
        }
    }

    Ok((chunk, parsed_lines + start_index))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_patch() {
        assert_eq!(
            parse_patch_text("bad"),
            Err(InvalidPatchError(
                "The first line of the patch must be '*** Begin Patch', got: \"bad\"".to_string()
            ))
        );
        assert_eq!(
            parse_patch_text("*** Begin Patch\nbad"),
            Err(InvalidPatchError(
                "The last line of the patch must be '*** End Patch', got: \"bad\"".to_string()
            ))
        );

        assert_eq!(
            parse_patch_text(concat!(
                "*** Begin Patch",
                " ",
                "\n*** Add File: foo\n+hi\n",
                " ",
                "*** End Patch"
            ))
            .unwrap(),
            vec![AddFile {
                path: PathBuf::from("foo"),
                contents: "hi\n".to_string()
            }]
        );
        match parse_patch_text(
            "*** Begin Patch\n\
             *** Update File: test.py\n\
             *** End Patch",
        ) {
            Err(ParseError::InvalidHunkError {
                ref message,
                line_number: 2,
                snippet: Some(_),
            }) => {
                assert_eq!(message, "Update file hunk for path 'test.py' is empty");
            }
            other => panic!("expected annotated InvalidHunkError, got {other:?}"),
        }
        assert_eq!(
            parse_patch_text(
                "*** Begin Patch\n\
                 *** End Patch",
            )
            .unwrap(),
            Vec::<Hunk>::new()
        );

        let err = parse_patch_text(
            "*** Begin Patch\n\
             *** Add File: foo\n\
             +hi\n\
             +*** End Patch",
        )
        .unwrap_err();
        let ParseError::InvalidPatchError(msg) = err else {
            panic!("expected InvalidPatchError, got {err:?}");
        };
        assert!(msg.contains("got: \"+*** End Patch\""), "msg = {msg}");
        assert!(
            msg.contains("envelope marker was written as a hunk line"),
            "msg = {msg}"
        );
        assert_eq!(
            parse_patch_text(
                "*** Begin Patch\n\
                 *** Add File: path/add.py\n\
                 +abc\n\
                 +def\n\
                 *** Delete File: path/delete.py\n\
                 *** Update File: path/update.py\n\
                 *** Move to: path/update2.py\n\
                 @@ def f():\n\
                 -    pass\n\
                 +    return 123\n\
                 *** End Patch",
            )
            .unwrap(),
            vec![
                AddFile {
                    path: PathBuf::from("path/add.py"),
                    contents: "abc\ndef\n".to_string()
                },
                DeleteFile {
                    path: PathBuf::from("path/delete.py")
                },
                UpdateFile {
                    path: PathBuf::from("path/update.py"),
                    move_path: Some(PathBuf::from("path/update2.py")),
                    chunks: vec![UpdateFileChunk {
                        change_context: Some("def f():".to_string()),
                        old_lines: vec!["    pass".to_string()],
                        new_lines: vec!["    return 123".to_string()],
                        is_end_of_file: false
                    }]
                }
            ]
        );
        // Update hunk followed by another hunk (Add File).
        assert_eq!(
            parse_patch_text(
                "*** Begin Patch\n\
                 *** Update File: file.py\n\
                 @@\n\
                 +line\n\
                 *** Add File: other.py\n\
                 +content\n\
                 *** End Patch",
            )
            .unwrap(),
            vec![
                UpdateFile {
                    path: PathBuf::from("file.py"),
                    move_path: None,
                    chunks: vec![UpdateFileChunk {
                        change_context: None,
                        old_lines: vec![],
                        new_lines: vec!["line".to_string()],
                        is_end_of_file: false
                    }],
                },
                AddFile {
                    path: PathBuf::from("other.py"),
                    contents: "content\n".to_string()
                }
            ]
        );

        // Update hunk without an explicit @@ header for the first chunk should parse.
        // Use a raw string to preserve the leading space diff marker on the context line.
        assert_eq!(
            parse_patch_text(
                r#"*** Begin Patch
*** Update File: file2.py
 import foo
+bar
*** End Patch"#,
            )
            .unwrap(),
            vec![UpdateFile {
                path: PathBuf::from("file2.py"),
                move_path: None,
                chunks: vec![UpdateFileChunk {
                    change_context: None,
                    old_lines: vec!["import foo".to_string()],
                    new_lines: vec!["import foo".to_string(), "bar".to_string()],
                    is_end_of_file: false,
                }],
            }]
        );
    }

    #[test]
    fn test_parse_patch_accepts_relative_and_absolute_hunk_paths() {
        let dir = tempfile::tempdir().unwrap();
        let absolute_delete = dir.path().join("absolute-delete.py");
        let absolute_update = dir.path().join("absolute-update.py");
        let patch_text = format!(
            r#"*** Begin Patch
*** Add File: relative-add.py
+content
*** Delete File: {}
*** Update File: {}
@@
-old
+new
*** End Patch"#,
            absolute_delete.display(),
            absolute_update.display()
        );

        assert_eq!(
            parse_patch_text(&patch_text).unwrap(),
            vec![
                AddFile {
                    path: PathBuf::from("relative-add.py"),
                    contents: "content\n".to_string()
                },
                DeleteFile {
                    path: absolute_delete.clone()
                },
                UpdateFile {
                    path: absolute_update.clone(),
                    move_path: None,
                    chunks: vec![UpdateFileChunk {
                        change_context: None,
                        old_lines: vec!["old".to_string()],
                        new_lines: vec!["new".to_string()],
                        is_end_of_file: false
                    }]
                },
            ]
        );
    }

    #[test]
    fn test_parse_one_hunk() {
        assert_eq!(
            parse_one_hunk(&["bad"], /*line_number*/ 234),
            Err(InvalidHunkError {
                message: "'bad' is not a valid hunk header. \
            Valid hunk headers: '*** Add File: {path}', '*** Delete File: {path}', '*** Update File: {path}'".to_string(),
                line_number: 234,
                snippet: None,
            })
        );
        // Other edge cases are already covered by tests above/below.
    }

    #[test]
    fn test_update_file_chunk() {
        assert_eq!(
            parse_update_file_chunk(
                &["bad"],
                /*line_number*/ 123,
                /*allow_missing_context*/ false
            ),
            Err(InvalidHunkError {
                message: "Expected update hunk to start with a @@ context marker, got: 'bad'"
                    .to_string(),
                line_number: 123,
                snippet: None,
            })
        );
        assert_eq!(
            parse_update_file_chunk(
                &["@@"],
                /*line_number*/ 123,
                /*allow_missing_context*/ false
            ),
            Err(InvalidHunkError {
                message: "Update hunk does not contain any lines".to_string(),
                line_number: 124,
                snippet: None,
            })
        );
        assert_eq!(
            parse_update_file_chunk(&["@@", "bad"], /*line_number*/ 123, /*allow_missing_context*/ false),
            Err(InvalidHunkError {
                message:  "Unexpected line found in update hunk: 'bad'. \
                       Every line should start with ' ' (context line), '+' (added line), or '-' (removed line)".to_string(),
                line_number: 124,
                snippet: None,
            })
        );
        assert_eq!(
            parse_update_file_chunk(
                &["@@", "*** End of File"],
                /*line_number*/ 123,
                /*allow_missing_context*/ false
            ),
            Err(InvalidHunkError {
                message: "Update hunk does not contain any lines".to_string(),
                line_number: 124,
                snippet: None,
            })
        );
        assert_eq!(
            parse_update_file_chunk(
                &[
                    "@@ change_context",
                    "",
                    " context",
                    "-remove",
                    "+add",
                    " context2",
                    "*** End Patch",
                ],
                /*line_number*/ 123,
                /*allow_missing_context*/ false
            ),
            Ok((
                (UpdateFileChunk {
                    change_context: Some("change_context".to_string()),
                    old_lines: vec![
                        "".to_string(),
                        "context".to_string(),
                        "remove".to_string(),
                        "context2".to_string()
                    ],
                    new_lines: vec![
                        "".to_string(),
                        "context".to_string(),
                        "add".to_string(),
                        "context2".to_string()
                    ],
                    is_end_of_file: false
                }),
                6
            ))
        );
        assert_eq!(
            parse_update_file_chunk(
                &["@@", "+line", "*** End of File"],
                /*line_number*/ 123,
                /*allow_missing_context*/ false
            ),
            Ok((
                (UpdateFileChunk {
                    change_context: None,
                    old_lines: vec![],
                    new_lines: vec!["line".to_string()],
                    is_end_of_file: true
                }),
                3
            ))
        );
    }
}
