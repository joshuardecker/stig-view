use rfd::AsyncFileDialog;
use std::sync::Arc;
use stig_view_core::db::{DB, Data};
use stig_view_core::stig_dep::Stig;

#[derive(Debug, Clone)]
pub enum FileError {
    HomeDir(&'static str),
    UserExitedSelect,
    NotAStig(&'static str),
    ReadDir(&'static str),
    DBCacheErr(&'static str),
}

/// Attempts to open a single file selected by the user using their system file picker.
/// Returns the id of the STIG loaded, or an error if it could not load.
pub async fn open_file(db: DB) -> Result<String, FileError> {
    let home_dir = dirs::home_dir().ok_or(FileError::HomeDir(
        "Program could not detect home directory.",
    ))?;

    let file_handle = AsyncFileDialog::new()
        .add_filter("text", &["txt"])
        .set_directory(home_dir)
        .set_title("Stig View - Select File")
        .pick_file()
        .await
        .ok_or(FileError::UserExitedSelect)?;

    let stig = Stig::from_xylok_txt(file_handle.path()).ok_or(FileError::NotAStig(
        "Selected file could not be loaded as a STIG.",
    ))?;

    let name = stig.version.clone();

    db.insert(name.clone(), Data::new(Arc::new(stig)))
        .await
        .map_err(|_| FileError::DBCacheErr("Error inserting STIG into the DB cache."))?;

    Ok(name)
}

/// Open a folder selected by the user with their system file picker.
/// Scans this dir and any sub dirs for valid STIGs.
/// Returns the first found STIGs id AND if an error occured.
/// Does not report mutiple errors, only one.
pub async fn open_folder(db: DB) -> (Option<String>, Option<FileError>) {
    let home_dir = dirs::home_dir();

    if home_dir.is_none() {
        return (
            None,
            Some(FileError::HomeDir(
                "Program could not detect home directory.",
            )),
        );
    }

    let folder_handle = AsyncFileDialog::new()
        .set_directory(home_dir.unwrap()) // Safe unwrap call.
        .set_title("Stig View - Select Folder")
        .pick_folder()
        .await;

    if folder_handle.is_none() {
        return (None, Some(FileError::UserExitedSelect));
    }

    // Initialize variables that will be modified in the loop:
    // Directories found that need to be scanned through.
    let mut dirs_to_load = vec![folder_handle.unwrap().path().to_path_buf()]; // Safe unwrap call.
    // All text files found.
    let mut txt_files = Vec::new();
    // If an error occurs, dont quit the loop, just save it here.
    let mut error = None;

    // While there is still a dir to look through.
    while !dirs_to_load.is_empty() {
        // Remove and read this dir from the list at the same time.
        let path = dirs_to_load.swap_remove(0);

        let mut read_dir = match tokio::fs::read_dir(&path).await {
            Ok(rd) => rd,
            Err(_) => {
                error = Some(FileError::ReadDir("Error when reading a directory."));
                continue;
            }
        };

        // Go through every entry in this dir.
        loop {
            match read_dir.next_entry().await {
                Ok(Some(entry)) => {
                    let is_dir = entry
                        .file_type()
                        .await
                        .map(|ft| ft.is_dir())
                        .unwrap_or(false);

                    if is_dir {
                        dirs_to_load.push(entry.path());
                        continue;
                    }

                    let entry_path = entry.path();

                    // If its a txt file.
                    if entry_path.extension().unwrap_or_default() == "txt" {
                        txt_files.push(entry_path);
                    }
                }
                Ok(None) => break,
                Err(_) => {
                    error = Some(FileError::ReadDir("Error when reading a directory."));
                    break;
                }
            }
        }
    }

    // Id could be None if no valid STIGs are loaded.
    let mut id = None;

    for path in txt_files {
        let stig = Stig::from_xylok_txt(path);

        if let Some(stig) = stig {
            // If this is the first stig to be inserted into the db,
            // save its id so that the program can automatically display it.
            if id.is_none() {
                id = Some(stig.version.clone());
            }

            let insert_err = db
                .insert(stig.version.clone(), Data::new(Arc::new(stig)))
                .await
                .map_err(|_| FileError::DBCacheErr("Error inserting STIG into the DB cache."));

            if let Err(insert_err) = insert_err {
                error = Some(insert_err);
            }
        }
    }

    (id, error)
}
