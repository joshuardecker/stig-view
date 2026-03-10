use rfd::AsyncFileDialog;
use std::path::PathBuf;
use std::sync::Arc;
use stig_view_core::db::{DB, Data};
use stig_view_core::stig::Stig;

pub async fn open_file(db: DB) -> Option<String> {
    let home_dir = std::env::home_dir().unwrap_or(PathBuf::from("/"));

    let file_handle = AsyncFileDialog::new()
        .add_filter("text", &["txt"])
        .set_directory(home_dir)
        .set_title("Stig View - Select File")
        .pick_file()
        .await;

    if let Some(file_handle) = file_handle {
        let stig = Stig::from_xylok_txt(file_handle.path());

        if let Some(stig) = stig {
            let name = stig.version.clone();

            db.insert(name.clone(), Data::new(Arc::new(stig)));

            return Some(name);
        }
    }

    None
}
