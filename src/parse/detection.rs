use std::fs::{File, read_to_string};
use std::io::{Cursor, Read, Seek};
use std::path::Path;
use zip::ZipArchive;

use crate::parse::{Format, ckl::CKLB, xylok::XylokToml};

/// Detect the format of STIG the user provided given a path.
/// Returns all formats found. If no formats are found, the file is unsupported.
pub fn detect_stig_format<P: AsRef<Path>>(path: P) -> Vec<Format> {
    match path.as_ref().extension().and_then(|os_str| os_str.to_str()) {
        // Attempt to deserialize the toml as a Xylok Benchmark.
        Some("toml") => {
            let Ok(toml_str) = read_to_string(path) else {
                return Vec::new();
            };

            let Ok(xylok_toml) = toml::from_str::<XylokToml>(&toml_str) else {
                return Vec::new();
            };

            vec![Format::Xylok(xylok_toml)]
        }

        // Look in the xml for version keywords to detect its version.
        Some("xml") => {
            let Ok(xml) = std::fs::read_to_string(path.as_ref()) else {
                return Vec::new();
            };

            detect_xccdf_str(&xml).into_iter().collect()
        }

        // Unzip the file, and then check for key words in the xml.
        Some("zip") => detect_xccdf_in_zip(path.as_ref()),

        // Just attempt to parse the data, without looking for a keyword.
        Some("ckl") => {
            let Ok(xml) = std::fs::read_to_string(path.as_ref()) else {
                return Vec::new();
            };

            vec![Format::CKL(xml)]
        }

        Some("cklb") => {
            let Ok(json_str) = read_to_string(path) else {
                return Vec::new();
            };

            let Ok(cklb_benchmark) = serde_json::from_str::<CKLB>(&json_str) else {
                return Vec::new();
            };

            vec![Format::CKLB(cklb_benchmark)]
        }

        _ => Vec::new(),
    }
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
/// Recursively scans nested zip files up to a depth of 3.
fn detect_xccdf_in_zip(path: &Path) -> Vec<Format> {
    let Ok(file) = File::open(path) else {
        return Vec::new();
    };

    let Ok(mut archive) = ZipArchive::new(file) else {
        return Vec::new();
    };

    detect_xccdf_in_archive(&mut archive, 3)
}

/// Recursively collect all XCCDF formats from a zip archive.
fn detect_xccdf_in_archive<R: Read + Seek>(
    archive: &mut ZipArchive<R>,
    depth: usize,
) -> Vec<Format> {
    let mut formats = Vec::new();
    let len = archive.len();

    // First pass: scan all xml files.
    for i in 0..len {
        let mut entry = match archive.by_index(i) {
            Ok(e) => e,
            Err(_) => continue,
        };

        let name = entry.name().to_owned();
        if !name.ends_with(".xml") {
            continue;
        }

        let mut xml = String::new();
        if entry.read_to_string(&mut xml).is_err() {
            continue;
        }

        if let Some(format) = detect_xccdf_str(&xml) {
            formats.push(format);
        }
    }

    // Second pass: recurse into nested zip files.
    if depth > 0 {
        for i in 0..len {
            let mut entry = match archive.by_index(i) {
                Ok(e) => e,
                Err(_) => continue,
            };

            let name = entry.name().to_owned();
            if !name.ends_with(".zip") {
                continue;
            }

            let mut buf = Vec::new();
            if entry.read_to_end(&mut buf).is_err() {
                continue;
            }

            let cursor = Cursor::new(buf);
            let Ok(mut nested) = ZipArchive::new(cursor) else {
                continue;
            };

            let nested_formats = detect_xccdf_in_archive(&mut nested, depth - 1);
            formats.extend(nested_formats);
        }
    }

    formats
}

#[test]
fn test_xccdfv1_1_detection() {
    let formats = detect_stig_format("test_assets/U_RHEL_8_V2R6_STIG.zip");
    assert_eq!(formats.len(), 1);
    assert!(matches!(formats[0], Format::XccdfV1_1(_)));
}

#[test]
fn test_xccdfv1_2_detection() {
    let formats =
        detect_stig_format("test_assets/U_MS_Windows_10_V3R7_STIG_SCAP_1-3_Benchmark.zip");
    assert_eq!(formats.len(), 1);
    assert!(matches!(formats[0], Format::XccdfV1_2));
}

#[test]
fn test_xylok_detection() {
    let formats = detect_stig_format("test_assets/packed.toml");
    assert_eq!(formats.len(), 1);
    assert!(matches!(formats[0], Format::Xylok(_)));
}

#[test]
fn test_nested_zip_xccdfv1_1_detection() {
    let formats =
        detect_stig_format("test_assets/U_Cisco_IOS_XE_Router_NDM_RTR_V2R3_STIG_Ansible.zip");
    // The nested zip contains two xccdf v1.1 benchmarks.
    assert_eq!(formats.len(), 2);
    assert!(matches!(formats[0], Format::XccdfV1_1(_)));
    assert!(matches!(formats[1], Format::XccdfV1_1(_)));
}
