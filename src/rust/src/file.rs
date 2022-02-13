use once_cell::sync::Lazy;
use std::path::{Path, PathBuf};

use extendr_api::prelude::*;

pub(crate) struct FilenameTemplate {
    parent: PathBuf,
    prefix: String,
    digit_width: Option<usize>,
    suffix: String,
}

static FILENAME_PATTERN: Lazy<regex::Regex> = Lazy::new(|| {
    regex::Regex::new(r"^(?P<prefix>[^%]+)(?:%(?P<zero>0)?(?P<page>[1-9]\d*)d)?(?P<suffix>.*)$")
        .unwrap()
});

#[test]
fn test_file_pattern() {
    let cap = FILENAME_PATTERN.captures("Rplot%03d.png").unwrap();
    assert_eq!(cap.name("prefix").unwrap().as_str(), "Rplot");
    assert!(cap.name("zero").is_some());
    assert_eq!(cap.name("page").unwrap().as_str(), "3");
    assert_eq!(cap.name("suffix").unwrap().as_str(), ".png");

    let cap = FILENAME_PATTERN.captures("Rplot%3d.png").unwrap();
    assert_eq!(cap.name("prefix").unwrap().as_str(), "Rplot");
    assert!(cap.name("zero").is_none());
    assert_eq!(cap.name("page").unwrap().as_str(), "3");
    assert_eq!(cap.name("suffix").unwrap().as_str(), ".png");

    let cap = FILENAME_PATTERN.captures("Rplot.png").unwrap();
    assert_eq!(cap.name("prefix").unwrap().as_str(), "Rplot.png");
}

impl FilenameTemplate {
    pub(crate) fn new(filename: &str) -> Result<Self> {
        let p = Path::new(filename);

        let parent = match p.parent() {
            Some(m) => {
                // If the parent doesn't exist yet, create it.
                if !m.exists() {
                    reprintln!("Creating the parent directory: {m:?}");
                    if let Err(e) = std::fs::create_dir_all(m) {
                        return Err(Error::Other(e.to_string()));
                    }
                } else {
                    // This should never happen
                    if !m.is_dir() {
                        return Err(Error::Other(
                            "{m:?} is not a directory. Something is wrong...!".to_string(),
                        ));
                    }
                }

                m.to_path_buf()
            }
            None => PathBuf::new(),
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
            Some(width) => format!("{prefix}{page_num:width$}{suffix}"),
            None => format!("{prefix}{suffix}"),
        };

        self.parent.join(Path::new(&filename))
    }
}
