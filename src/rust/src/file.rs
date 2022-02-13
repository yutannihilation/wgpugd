// Since there's no direct equivalent to sprintf() in Rust (i.e., we cannot
// simply pass the user-supplied string to format!()), we need to parse the
// filename and construct the filename template.

use once_cell::sync::Lazy;
use std::path::{Path, PathBuf};

use extendr_api::prelude::*;

#[derive(Debug)]
pub(crate) struct FilenameTemplate {
    parent: Option<PathBuf>,
    prefix: String,
    zero_padded: bool,
    digit_width: Option<usize>,
    suffix: String,
}

static FILENAME_PATTERN: Lazy<regex::Regex> = Lazy::new(|| {
    regex::Regex::new(
        r"^(?P<prefix>[^%]+)(?:%(?P<zero>0)?(?P<digit_width>[1-9]\d*)d)?(?P<suffix>.*)$",
    )
    .unwrap()
});

impl FilenameTemplate {
    pub(crate) fn new(filename: &str) -> Result<Self> {
        let p = Path::new(filename);

        let parent = match p.parent() {
            // It doesn't have any parent directory.
            Some(m) if m.to_string_lossy() == "" => None,
            None => None,
            // If it has a parent directory but it deosn't exist yet, create it
            Some(m) if !m.exists() => {
                reprintln!("Creating the parent directory: {m:?}");
                if let Err(e) = std::fs::create_dir_all(m) {
                    return Err(Error::Other(e.to_string()));
                }
                Some(m.to_path_buf())
            }
            // It has a parent directory but it's not a directory in actual.
            Some(m) if !m.is_dir() => {
                return Err(Error::Other(
                    "{m:?} is not a directory. Something is wrong...!".to_string(),
                ));
            }
            // If it has a parent directory and it exists, do nothing
            Some(m) => Some(m.to_path_buf()),
        };

        let basename = match p.file_name() {
            Some(basename) => basename.to_string_lossy(),
            None => {
                return Err(Error::Other(
                    "The specified filename doesn't contain a filename in actual.".to_string(),
                ))
            }
        };

        match FILENAME_PATTERN.captures(&basename) {
            Some(cap) => {
                let prefix = cap.name("prefix").unwrap().as_str().to_string();
                let zero_padded = cap.name("zero").is_some();
                let digit_width = cap
                    .name("digit_width")
                    .map(|m| m.as_str().parse::<usize>().unwrap());
                let suffix = cap
                    .name("suffix")
                    .map(|m| m.as_str().to_string())
                    .unwrap_or_default();

                Ok(Self {
                    parent,
                    prefix,
                    zero_padded,
                    digit_width,
                    suffix,
                })
            }
            None => return Err(Error::Other(format!("Invalid filename: {basename}"))),
        }
    }

    pub(crate) fn filename(&self, page_num: u32) -> PathBuf {
        let prefix = &self.prefix;
        let suffix = &self.suffix;

        let filename = match self.digit_width {
            Some(digit_width) => {
                if self.zero_padded {
                    format!("{prefix}{page_num:0digit_width$}{suffix}")
                } else {
                    format!("{prefix}{page_num:digit_width$}{suffix}")
                }
            }
            None => format!("{prefix}{suffix}"),
        };

        if let Some(parent) = &self.parent {
            parent.join(Path::new(&filename))
        } else {
            Path::new(&filename).to_path_buf()
        }
    }
}

#[test]
fn test_file_pattern() {
    let cap = FILENAME_PATTERN.captures("Rplot%03d.png").unwrap();
    assert_eq!(cap.name("prefix").unwrap().as_str(), "Rplot");
    assert!(cap.name("zero").is_some());
    assert_eq!(cap.name("digit_width").unwrap().as_str(), "3");
    assert_eq!(cap.name("suffix").unwrap().as_str(), ".png");

    let cap = FILENAME_PATTERN.captures("Rplot%3d.png").unwrap();
    assert_eq!(cap.name("prefix").unwrap().as_str(), "Rplot");
    assert!(cap.name("zero").is_none());
    assert_eq!(cap.name("digit_width").unwrap().as_str(), "3");
    assert_eq!(cap.name("suffix").unwrap().as_str(), ".png");

    let cap = FILENAME_PATTERN.captures("Rplot.png").unwrap();
    assert_eq!(cap.name("prefix").unwrap().as_str(), "Rplot.png");
}

#[test]
fn test_filename() -> Result<()> {
    // No placeholder
    assert_eq!(
        FilenameTemplate::new("Rplot.png")?
            .filename(10)
            .to_string_lossy(),
        "Rplot.png"
    );

    // Padded with space
    assert_eq!(
        FilenameTemplate::new("Rplot%3d.png")?
            .filename(10)
            .to_string_lossy(),
        "Rplot 10.png"
    );

    // Padded with zero
    assert_eq!(
        FilenameTemplate::new("Rplot%03d.png")?
            .filename(10)
            .to_string_lossy(),
        "Rplot010.png"
    );

    Ok(())
}
