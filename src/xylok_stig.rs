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

impl std::fmt::Display for Stig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Version: {}\n\nIntro: {}\n\nDesc: {}\n\nCheck: {}\n\nFix: {}",
            self.version, self.introduction, self.description, self.check_text, self.fix_text
        )
    }
}

pub fn load_stig<p: AsRef<Path>>(path: p) -> Result<Stig, StigError> {
    let mut file = File::open(path).map_err(|_| StigError::FileError)?;

    //let mut file = File::open(path).err;
    let mut content = String::new();

    file.read_to_string(&mut content)
        .map_err(|_| StigError::FileError)?;

    let re_version = Regex::new(r"# Title\n([a-zA-Z0-9-]*):").unwrap();
    let re_intro = Regex::new(r"(?s)# Title\n.*:(.*)#################\n# Similar checks").unwrap();
    let re_desc = Regex::new(r"(?s)# Discussion\n(.*)#################\n# Fix").unwrap();
    let re_check_text = Regex::new(r"(?s)# Content\n(.*)#################\n# Discussion").unwrap();
    let re_fix = Regex::new(r"(?s)# Fix\n(.*)").unwrap();

    let version_capture = re_version.captures(&content).unwrap();
    let intro_capture = re_intro.captures(&content).unwrap();
    let desc_capture = re_desc.captures(&content).unwrap();
    let check_text_capture = re_check_text.captures(&content).unwrap();
    let fix_capture = re_fix.captures(&content).unwrap();

    let mut stig = Stig::default();

    stig.version = version_capture[1].to_string();
    stig.introduction = intro_capture[1].trim().to_string();
    stig.description = desc_capture[1].trim().to_string();
    stig.check_text = check_text_capture[1].trim().to_string();
    stig.fix_text = fix_capture[1].trim().to_string();

    println!("{}", &stig);

    Ok(stig)
}

#[test]
fn hello_world() {
    let re_version = Regex::new(r"^(\w+(?:-\w*)*)").unwrap();

    let haystack = String::from(r"Hel-lo there!");

    let Some(captures) = re_version.captures(&haystack) else {
        panic!("Uh oh!")
    };

    assert_eq!(&captures[1], "Hel-lo");
}

#[test]
fn test_load_version() {
    let stig = load_stig("info.txt").unwrap();

    let mut test_stig = Stig::default();
    test_stig.version = String::from("CASA-FW-000260");

    assert_eq!(stig, test_stig);
}
