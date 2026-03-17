use regex::Regex;
use std::fs::File;
use std::io::Read;
use std::path::Path;

/// A data structure containing the useful values of a DISA stig.
#[derive(Debug, PartialEq, Clone)]
pub struct Stig {
    pub version: String,
    pub intro: String,
    pub similar_checks: String,
    pub check_text: String,
    pub desc: String,
    pub fix_text: String,
}

impl Stig {
    /// Create a stig from a given xylok generated txt file of a stig.
    /// Returns none if a random txt is given.
    pub fn from_xylok_txt<P: AsRef<Path>>(path: P) -> Option<Stig> {
        let mut file = File::open(path).ok()?;
        let mut buf = String::new();

        file.read_to_string(&mut buf).ok()?;

        let re_stig = Regex::new(
            r"(?s).*# Title\n([\w-]+):(.*)#################\n# Similar checks(.*)#################\n# Content(.*)#################\n# Discussion(.*)#################\n# Fix(.*)",
        )
        .unwrap();

        let captures = re_stig.captures(&buf)?;

        Some(Stig {
            version: captures.get(1).unwrap().as_str().trim().to_string(),
            intro: captures.get(2).unwrap().as_str().trim().to_string(),
            similar_checks: captures.get(3).unwrap().as_str().trim().to_string(),
            check_text: captures.get(4).unwrap().as_str().trim().to_string(),
            desc: captures.get(5).unwrap().as_str().trim().to_string(),
            fix_text: captures.get(6).unwrap().as_str().trim().to_string(),
        })
    }
}
