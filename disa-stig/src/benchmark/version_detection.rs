//! A small module that supports detecting what file format a provided file path is in relation to DISA STIGs.

use std::{fs::File, io::Cursor, io::Read, io::Seek, path::Path};

use zip::ZipArchive;

use crate::{
    Benchmark, BenchmarkError,
    benchmark::{
        ckl::Checklist, cklb::CKLB, xccdf_v1_1, xccdf_v1_2::DataStreamCollection, xylok::XylokToml,
    },
};

/// Maximum depth to recurse into nested zip archives.
const MAX_NESTED_DEPTH: usize = 3;

/// Contains every format a DISA STIG can be.
/// Format is detected by serializing the data into memory successfully,
/// so to save time and energy, this also wraps successfully serialized data.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FormatWithData {
    XccdfV1_1(xccdf_v1_1::Benchmark),
    XccdfV1_2(DataStreamCollection),

    Xylok(XylokToml),

    CKL(Checklist),
    CKLB(CKLB),

    InHouse(Benchmark),
}

/// Scan a zip archive for benchmark files, recursing into nested zips up to
/// `MAX_NESTED_DEPTH`.
fn scan_zip_archive<R: Read + Seek>(
    archive: &mut ZipArchive<R>,
    formats: &mut Vec<FormatWithData>,
    depth: usize,
) -> Result<(), BenchmarkError> {
    if depth > MAX_NESTED_DEPTH {
        return Ok(());
    }

    for index in 0..archive.len() {
        let mut entry = archive.by_index(index)?;

        if entry.name().ends_with(".xml") {
            let mut entry_string = String::new();

            entry.read_to_string(&mut entry_string)?;

            // Try XCCDF v1.2 first (SCAP data stream).
            if let Ok(data_stream_collection) =
                quick_xml::de::from_str::<DataStreamCollection>(&entry_string)
            {
                let has_benchmark = data_stream_collection
                    .components
                    .iter()
                    .any(|component| !component.benchmarks.is_empty());

                if has_benchmark {
                    formats.push(FormatWithData::XccdfV1_2(data_stream_collection));

                    continue;
                }
            }

            // Try XCCDF v1.1 (standalone benchmark).
            if let Ok(benchmark) = quick_xml::de::from_str::<xccdf_v1_1::Benchmark>(&entry_string) {
                if !benchmark.groups.is_empty() {
                    formats.push(FormatWithData::XccdfV1_1(benchmark));

                    continue;
                }
            }

            // Try CKL checklist.
            if let Ok(ckl) = quick_xml::de::from_str::<Checklist>(&entry_string) {
                formats.push(FormatWithData::CKL(ckl));

                continue;
            }
        }

        if entry.name().ends_with(".cklb") {
            let mut entry_string = String::new();

            entry.read_to_string(&mut entry_string)?;

            if let Ok(cklb) = serde_json::from_str::<CKLB>(&entry_string) {
                formats.push(FormatWithData::CKLB(cklb));

                continue;
            }
        }

        if entry.name().ends_with(".zip") {
            let mut buffer = Vec::new();

            entry.read_to_end(&mut buffer)?;

            let cursor = Cursor::new(buffer);
            let mut nested_archive = ZipArchive::new(cursor)?;

            scan_zip_archive(&mut nested_archive, formats, depth + 1)?;
        }
    }

    Ok(())
}

/// Attempt to detect the file format of the given path.
pub fn detect_format_with_path(
    path: impl AsRef<Path>,
) -> Result<Vec<FormatWithData>, BenchmarkError> {
    let path_extension = path
        .as_ref()
        .extension()
        .ok_or(BenchmarkError::UnsupportedFileType)?
        .to_str()
        .ok_or(BenchmarkError::UnsupportedFileType)?;

    let mut file = File::open(path.as_ref())?;

    match path_extension {
        "zip" => {
            let mut zip_archive = ZipArchive::new(file)?;
            let mut formats = Vec::new();

            scan_zip_archive(&mut zip_archive, &mut formats, 0)?;

            if formats.is_empty() {
                return Err(BenchmarkError::UnsupportedFileType);
            }

            Ok(formats)
        }

        "xml" => {
            let mut file_string = String::new();

            file.read_to_string(&mut file_string)?;

            // Try XCCDF v1.2 first (SCAP data stream).
            if let Ok(data_stream_collection) =
                quick_xml::de::from_str::<DataStreamCollection>(&file_string)
            {
                let has_benchmark = data_stream_collection
                    .components
                    .iter()
                    .any(|component| !component.benchmarks.is_empty());

                if has_benchmark {
                    return Ok(vec![FormatWithData::XccdfV1_2(data_stream_collection)]);
                }
            }

            // Try XCCDF v1.1 (standalone benchmark).
            if let Ok(benchmark) = quick_xml::de::from_str::<xccdf_v1_1::Benchmark>(&file_string) {
                if !benchmark.groups.is_empty() {
                    return Ok(vec![FormatWithData::XccdfV1_1(benchmark)]);
                }
            }

            // Try CKL checklist.
            if let Ok(ckl) = quick_xml::de::from_str::<Checklist>(&file_string) {
                return Ok(vec![FormatWithData::CKL(ckl)]);
            }

            Err(BenchmarkError::CorruptFile)
        }

        "ckl" => {
            let mut file_string = String::new();

            file.read_to_string(&mut file_string)?;

            if let Ok(ckl) = quick_xml::de::from_str::<Checklist>(&file_string) {
                return Ok(vec![FormatWithData::CKL(ckl)]);
            } else {
                return Err(BenchmarkError::CorruptFile);
            }
        }

        "cklb" => {
            let mut file_string = String::new();

            file.read_to_string(&mut file_string)?;

            if let Ok(cklb) = serde_json::from_str::<CKLB>(&file_string) {
                return Ok(vec![FormatWithData::CKLB(cklb)]);
            } else {
                return Err(BenchmarkError::CorruptFile);
            }
        }

        "toml" => {
            let mut file_string = String::new();

            file.read_to_string(&mut file_string)?;

            if let Ok(xylok_toml) = toml::from_str::<XylokToml>(&file_string) {
                return Ok(vec![FormatWithData::Xylok(xylok_toml)]);
            } else {
                return Err(BenchmarkError::CorruptFile);
            }
        }

        "zstd" => {
            let mut buffer = Vec::new();

            file.read_to_end(&mut buffer)?;

            let decoded = zstd::decode_all(buffer.as_slice())?;

            if let Ok(benchmark) = serde_json::from_slice::<Benchmark>(&decoded) {
                return Ok(vec![FormatWithData::InHouse(benchmark)]);
            } else {
                return Err(BenchmarkError::CorruptFile);
            }
        }

        _ => Err(BenchmarkError::UnsupportedFileType),
    }
}
