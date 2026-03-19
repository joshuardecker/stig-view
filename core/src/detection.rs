use quick_xml::events::Event;
use quick_xml::reader::Reader;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;
use zip::ZipArchive;

use crate::{Format, XylokToml};

#[derive(Debug, Clone)]
#[cfg_attr(test, derive(PartialEq, Eq))]
pub enum DetectErr {
    InvalidFileFormat(&'static str),
    CantOpenFile(&'static str),
    NotStig(&'static str),
}

/// Detect the format of STIG the user provided given a path.
/// If its not a STIG, that is still returned as an error.
pub fn detect_stig_format<P: AsRef<Path>>(path: P) -> Result<Format, DetectErr> {
    match path.as_ref().extension().and_then(|os_str| os_str.to_str()) {
        Some("toml") => detect_xylok(path.as_ref())
            .ok_or(DetectErr::NotStig("Provided toml could not be loaded.")),
        Some("xml") => {
            let format = detect_xccdf(
                Reader::from_file(path)
                    .map_err(|_| DetectErr::CantOpenFile("Provided xml could not be loaded."))?,
            );

            return match format {
                Some(format) => Ok(format),
                None => Err(DetectErr::NotStig("Provided xml could not be loaded.")),
            };
        }
        Some("zip") => {
            return match detect_xccdf_in_zip(path.as_ref()) {
                Some(format) => Ok(format),
                None => Err(DetectErr::NotStig("Provided zip could not be loaded.")),
            };
        }
        _ => Err(DetectErr::InvalidFileFormat(
            "Provided file does not have a supported file extension.",
        )),
    }
}

/// See if the input is a Xylok STIG.
fn detect_xylok(path: &Path) -> Option<Format> {
    use std::fs::read_to_string;

    let toml_str = read_to_string(path).ok()?;

    let xylok_toml: XylokToml = toml::from_str(&toml_str).ok()?;

    Some(Format::Xylok(xylok_toml))
}

/// See if the input is an XML STIG.
fn detect_xccdf<R: BufRead>(mut xml: Reader<R>) -> Option<Format> {
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
                        return Some(Format::XccdfV1_2);
                    } else if value.contains("checklists.nist.gov/xccdf/1.1") {
                        return Some(Format::XccdfV1_1);
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
fn detect_xccdf_in_zip(path: &Path) -> Option<Format> {
    let mut archive = ZipArchive::new(File::open(path).ok()?).ok()?;

    let xml_names: Vec<String> = archive
        .file_names()
        .filter(|name| name.ends_with(".xml"))
        .map(|name| name.to_owned())
        .collect();

    for name in &xml_names {
        let entry = archive.by_name(name).ok()?;

        let format = detect_xccdf(Reader::from_reader(BufReader::new(entry)));

        if let Some(format) = format {
            return Some(format);
        }
    }

    None
}

#[test]
fn test_xccdfv1_1_detection() {
    let format = detect_stig_format("../test_assets/U_RHEL_8_V2R6_STIG.zip");
    assert_eq!(format, Ok(Format::XccdfV1_1));
}

#[test]
fn test_xccdfv1_2_detection() {
    let format =
        detect_stig_format("../test_assets/U_MS_Windows_10_V3R7_STIG_SCAP_1-3_Benchmark.zip");
    assert_eq!(format, Ok(Format::XccdfV1_2));
}

#[test]
fn test_xylok_detection() {
    let format = detect_stig_format("../test_assets/packed.toml");
    assert!(matches!(format, Ok(Format::Xylok(_))));
}
