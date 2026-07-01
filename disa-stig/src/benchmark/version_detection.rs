//! A small module that supports detecting what file format a provided file path is in relation to DISA STIGs.

use std::{fs::File, io::Read, path::Path};

use zip::ZipArchive;

use crate::{
    Benchmark, BenchmarkError,
    benchmark::{
        ckl::Checklist, cklb::CKLB, xccdf_v1_1, xccdf_v1_2::DataStreamCollection, xylok::XylokToml,
    },
};

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

            for index in 0..zip_archive.len() {
                let mut sub_file = zip_archive.by_index(index)?;

                let mut sub_file_str = String::new();

                sub_file.read_to_string(&mut sub_file_str)?;

                if sub_file.name().ends_with(".xml") {
                    // Check first for xccdf_v1_1, then check for v1_2, then ckl.
                    if let Ok(xccdfv1_1) =
                        quick_xml::de::from_str::<xccdf_v1_1::Benchmark>(&sub_file_str)
                    {
                        formats.push(FormatWithData::XccdfV1_1(xccdfv1_1));
                        continue;
                    }

                    if let Ok(xccdfv1_2) =
                        quick_xml::de::from_str::<DataStreamCollection>(&sub_file_str)
                    {
                        formats.push(FormatWithData::XccdfV1_2(xccdfv1_2));
                        continue;
                    }

                    if let Ok(ckl) = quick_xml::de::from_str::<Checklist>(&sub_file_str) {
                        formats.push(FormatWithData::CKL(ckl));
                        continue;
                    }
                }

                if sub_file.name().ends_with(".cklb") {
                    if let Ok(cklb) = serde_json::from_str::<CKLB>(&sub_file_str) {
                        formats.push(FormatWithData::CKLB(cklb));
                        continue;
                    }
                }
            }

            // If the zip file contained no benchmark files.
            if formats.is_empty() {
                return Err(BenchmarkError::UnsupportedFileType);
            }

            Ok(formats)
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
