use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Language {
    pub id: &'static str,
    pub parseable: bool,
}

const EXTENSIONS: &[(&str, Language)] = &[
    (
        "go",
        Language {
            id: "go",
            parseable: true,
        },
    ),
    (
        "rs",
        Language {
            id: "rust",
            parseable: true,
        },
    ),
    (
        "js",
        Language {
            id: "javascript",
            parseable: true,
        },
    ),
    (
        "ts",
        Language {
            id: "typescript",
            parseable: true,
        },
    ),
    (
        "tsx",
        Language {
            id: "tsx",
            parseable: true,
        },
    ),
    (
        "py",
        Language {
            id: "python",
            parseable: true,
        },
    ),
    (
        "rb",
        Language {
            id: "ruby",
            parseable: true,
        },
    ),
    (
        "java",
        Language {
            id: "java",
            parseable: true,
        },
    ),
    (
        "kt",
        Language {
            id: "kotlin",
            parseable: true,
        },
    ),
    (
        "swift",
        Language {
            id: "swift",
            parseable: true,
        },
    ),
    (
        "c",
        Language {
            id: "c",
            parseable: true,
        },
    ),
    (
        "h",
        Language {
            id: "c",
            parseable: true,
        },
    ),
    (
        "cpp",
        Language {
            id: "cpp",
            parseable: true,
        },
    ),
    (
        "cc",
        Language {
            id: "cpp",
            parseable: true,
        },
    ),
    (
        "hpp",
        Language {
            id: "cpp",
            parseable: true,
        },
    ),
    (
        "cs",
        Language {
            id: "csharp",
            parseable: true,
        },
    ),
    (
        "php",
        Language {
            id: "php",
            parseable: true,
        },
    ),
    (
        "scala",
        Language {
            id: "scala",
            parseable: true,
        },
    ),
    (
        "lua",
        Language {
            id: "lua",
            parseable: true,
        },
    ),
    (
        "sh",
        Language {
            id: "bash",
            parseable: true,
        },
    ),
    (
        "bash",
        Language {
            id: "bash",
            parseable: true,
        },
    ),
    (
        "zsh",
        Language {
            id: "bash",
            parseable: true,
        },
    ),
    (
        "json",
        Language {
            id: "json",
            parseable: false,
        },
    ),
    (
        "md",
        Language {
            id: "markdown",
            parseable: false,
        },
    ),
    (
        "toml",
        Language {
            id: "toml",
            parseable: false,
        },
    ),
    (
        "yaml",
        Language {
            id: "yaml",
            parseable: false,
        },
    ),
    (
        "yml",
        Language {
            id: "yaml",
            parseable: false,
        },
    ),
];

const FILENAMES: &[(&str, Language)] = &[
    (
        "Dockerfile",
        Language {
            id: "dockerfile",
            parseable: false,
        },
    ),
    (
        "Makefile",
        Language {
            id: "make",
            parseable: false,
        },
    ),
];

pub fn language_for_file(path: &Path) -> Option<Language> {
    let file_name = path.file_name()?.to_str()?;
    if let Some((_, lang)) = FILENAMES.iter().find(|(name, _)| *name == file_name) {
        return Some(*lang);
    }

    let extension = path.extension()?.to_str()?.to_ascii_lowercase();
    EXTENSIONS
        .iter()
        .find(|(ext, _)| *ext == extension)
        .map(|(_, lang)| *lang)
}
