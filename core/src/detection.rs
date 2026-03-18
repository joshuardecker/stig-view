use quick_xml::events::Event;
use quick_xml::reader::Reader;
use regex::Regex;
use std::fs::File;
use std::io::{BufRead, BufReader, Read};
use std::path::Path;
use zip::ZipArchive;

use crate::Version;

#[derive(Debug, Clone)]
#[cfg_attr(test, derive(PartialEq, Eq))]
pub enum DetectErr {
    InvalidFileFormat(&'static str),
    CantOpenFile(&'static str),
    NotStig(&'static str),
}

/// Detect the version of STIG the user provided given a path.
/// If its not a STIG, that is still returned as an error.
pub fn detect_stig_version<P: AsRef<Path>>(path: P) -> Result<Version, DetectErr> {
    match path.as_ref().extension().and_then(|os_str| os_str.to_str()) {
        Some("toml") => {
            if detect_xylok(path.as_ref()).is_some() {
                return Ok(Version::Xylok);
            } else {
                return Err(DetectErr::NotStig("Provided toml could not be loaded."));
            }
        }
        Some("xml") => {
            let version = detect_xccdf(
                Reader::from_file(path)
                    .map_err(|_| DetectErr::CantOpenFile("Provided xml could not be loaded."))?,
            );

            return match version {
                Some(version) => Ok(version),
                None => Err(DetectErr::NotStig("Provided xml could not be loaded.")),
            };
        }
        Some("zip") => {
            return match detect_xccdf_in_zip(path.as_ref()) {
                Some(version) => Ok(version),
                None => Err(DetectErr::NotStig("Provided zip could not be loaded.")),
            };
        }
        _ => Err(DetectErr::InvalidFileFormat(
            "Provided file does not have a supported file extension.",
        )),
    }
}

/// See if the input is a Xylok STIG.
fn detect_xylok(path: &Path) -> Option<Version> {
    let mut file = File::open(path).ok()?;
    let mut buf = String::new();

    file.read_to_string(&mut buf).ok()?;

    todo!();
}

/// See if the input is an XML STIG.
fn detect_xccdf<R: BufRead>(mut xml: Reader<R>) -> Option<Version> {
    let mut buf = Vec::new();

    loop {
        match xml.read_event_into(&mut buf).ok()? {
            Event::Start(start) => {
                for attribute in start.attributes().flatten() {
                    let key = attribute.key.as_ref();
                    let value = str::from_utf8(attribute.value.as_ref()).unwrap_or("");

                    if key != b"xmlns" && key != b"xmlns:xccdf" {
                        continue;
                    }

                    if value.contains("checklists.nist.gov/xccdf/1.2") {
                        return Some(Version::XccdfV1_2);
                    } else if value.contains("checklists.nist.gov/xccdf/1.1") {
                        return Some(Version::XccdfV1_1);
                    }
                }
            }
            Event::Eof => break,
            _ => (),
        }

        buf.clear();
    }

    None
}

/// See if the input zip is a STIG.
fn detect_xccdf_in_zip(path: &Path) -> Option<Version> {
    let mut archive = ZipArchive::new(File::open(path).ok()?).ok()?;

    let xml_names: Vec<String> = archive
        .file_names()
        .filter(|name| name.ends_with(".xml"))
        .map(|name| name.to_owned())
        .collect();

    for name in &xml_names {
        let entry = archive.by_name(name).ok()?;

        let version = detect_xccdf(Reader::from_reader(BufReader::new(entry)));

        if let Some(version) = version {
            return Some(version);
        }
    }

    None
}

#[test]
fn test_detection() {
    let version = detect_stig_version("../test_assets/U_RHEL_8_V2R6_STIG.zip");
    assert_eq!(version, Ok(Version::XccdfV1_1));

    let version =
        detect_stig_version("../test_assets/U_MS_Windows_10_V3R7_STIG_SCAP_1-3_Benchmark.zip");
    assert_eq!(version, Ok(Version::XccdfV1_2));

    let version = detect_stig_version("../test_assets/stig.txt");
    assert_eq!(version, Ok(Version::Xylok));
}
