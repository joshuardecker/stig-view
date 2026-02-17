use regex::Regex;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use uuid::Uuid;

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

#[test]
fn test_from_xylok_txt() {
    let mut stig = Stig {
        version: String::from("SOME-NUMBER2077"),
        intro: String::from("Hello!"),
        similar_checks: String::from("This is a similar check: Similar check."),
        check_text: String::from("Content!"),
        desc: String::from("Discussion!"),
        fix_text: String::from("Fix!"),
    };

    let loaded_stig = Stig::from_xylok_txt("test_stig.txt");

    match loaded_stig {
        Some(loaded_stig) => {
            // Ensure uuid's are the same for this test.
            stig.uuid = loaded_stig.uuid.clone();

            assert_eq!(loaded_stig, stig);
        }
        None => panic!("Was not able to load test_sig.txt!"),
    }

    let not_real = Stig::from_xylok_txt("not-a-real-path.txt");

    match not_real {
        None => println!("Good!"),
        Some(_) => panic!("How did we get here?"),
    }
}
