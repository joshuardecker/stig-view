use regex::Regex;
use std::fs::File;
use std::io::Read;
use std::path::Path;

// Anything commented out is present in a stig xml file,
// but not found in the xylok info.txt file.
#[derive(Debug, Default, PartialEq)]
pub struct Stig {
    //group_id: String,
    //title: String,
    pub version: String,

    //rule_id: String,
    //weight: String,
    //severity: String,
    pub introduction: String,
    pub description: String,
    pub check_text: String,
    pub fix_text: String,
}

#[derive(Debug)]
pub enum StigError {
    FileError,
    RegexError,
}

// Nicer printing for Stig.
impl std::fmt::Display for Stig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Version: {}\n\nIntro: {}\n\nDesc: {}\n\nCheck: {}\n\nFix: {}",
            self.version, self.introduction, self.description, self.check_text, self.fix_text
        )
    }
}

/// Load a stig file from the xylok format given a file path. Found as info.txt throughout xylok.
pub fn load_stig<P: AsRef<Path>>(path: P) -> Result<Stig, StigError> {
    let mut file = File::open(path).map_err(|_| StigError::FileError)?;

    //let mut file = File::open(path).err;
    let mut content = String::new();

    file.read_to_string(&mut content)
        .map_err(|_| StigError::FileError)?;

    // Set up the regex's.
    // Safe to call unwrap because these expressions are not runtime dependent.
    let re_version = Regex::new(r"# Title\n([a-zA-Z0-9-]*):").unwrap();
    let re_intro = Regex::new(r"(?s)# Title\n.*:(.*)#################\n# Similar checks").unwrap();
    let re_desc = Regex::new(r"(?s)# Discussion\n(.*)#################\n# Fix").unwrap();
    let re_check_text = Regex::new(r"(?s)# Content\n(.*)#################\n# Discussion").unwrap();
    let re_fix = Regex::new(r"(?s)# Fix\n(.*)").unwrap();

    // Capture regex content.
    let version_capture = re_version.captures(&content).ok_or(StigError::RegexError)?;
    let intro_capture = re_intro.captures(&content).ok_or(StigError::RegexError)?;
    let desc_capture = re_desc.captures(&content).ok_or(StigError::RegexError)?;
    let check_text_capture = re_check_text
        .captures(&content)
        .ok_or(StigError::RegexError)?;
    let fix_capture = re_fix.captures(&content).ok_or(StigError::RegexError)?;

    let mut stig = Stig::default();

    // Store captured data.
    // Use .trim() to remove extra \n the regex will capture.
    stig.version = version_capture[1].to_string();
    stig.introduction = intro_capture[1].trim().to_string();
    stig.description = desc_capture[1].trim().to_string();
    stig.check_text = check_text_capture[1].trim().to_string();
    stig.fix_text = fix_capture[1].trim().to_string();

    Ok(stig)
}
