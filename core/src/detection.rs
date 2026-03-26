use std::fs::File;
use std::io::Read;
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
            let xml = std::fs::read_to_string(path.as_ref())
                .map_err(|_| DetectErr::CantOpenFile("Provided xml could not be loaded."))?;

            detect_xccdf_str(&xml).ok_or(DetectErr::NotStig("Provided xml could not be loaded."))
        }
        Some("zip") => detect_xccdf_in_zip(path.as_ref())
            .ok_or(DetectErr::NotStig("Provided zip could not be loaded.")),
        Some("ckl") => {
            let xml = std::fs::read_to_string(path.as_ref())
                .map_err(|_| DetectErr::CantOpenFile("Provided ckl could not be loaded."))?;
            Ok(Format::CKL(xml))
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

/// Detect the XCCDF version from a raw XML string.
/// For XccdfV1_1/V1_2 the string is moved into the variant so the caller does not
/// need to re-read (or re-unzip) the file.
fn detect_xccdf_str(xml: &str) -> Option<Format> {
    let xml_tree = roxmltree::Document::parse(xml).ok()?;

    let str = xml_tree
        .descendants()
        .find(|node| node.tag_name().name() == "Benchmark")?
        .tag_name()
        .namespace()
        .unwrap_or("");

    if str.contains("checklists.nist.gov/xccdf/1.2") {
        Some(Format::XccdfV1_2)
    } else if str.contains("checklists.nist.gov/xccdf/1.1") {
        Some(Format::XccdfV1_1(xml.to_owned()))
    } else {
        None
    }
}

/// See if the input zip contains an XCCDF STIG.
fn detect_xccdf_in_zip(path: &Path) -> Option<Format> {
    let mut archive = ZipArchive::new(File::open(path).ok()?).ok()?;

    let xml_names: Vec<String> = archive
        .file_names()
        .filter(|name| name.ends_with(".xml"))
        .map(|name| name.to_owned())
        .collect();

    for name in &xml_names {
        let mut entry = match archive.by_name(name) {
            Ok(e) => e,
            Err(_) => continue,
        };

        let mut xml = String::new();
        if entry.read_to_string(&mut xml).is_err() {
            continue;
        }

        if let Some(format) = detect_xccdf_str(&xml) {
            return Some(format);
        }
    }

    None
}

#[test]
fn test_xccdfv1_1_detection() {
    let format = detect_stig_format("../test_assets/U_RHEL_8_V2R6_STIG.zip");
    assert!(matches!(format, Ok(Format::XccdfV1_1(_))));
}

#[test]
fn test_xccdfv1_2_detection() {
    let format =
        detect_stig_format("../test_assets/U_MS_Windows_10_V3R7_STIG_SCAP_1-3_Benchmark.zip");
    eprintln!("{:?}", &format);
    assert!(matches!(format, Ok(Format::XccdfV1_2)));
}

#[test]
fn test_xylok_detection() {
    let format = detect_stig_format("../test_assets/packed.toml");
    assert!(matches!(format, Ok(Format::Xylok(_))));
}
