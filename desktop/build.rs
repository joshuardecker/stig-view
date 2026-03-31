fn main() {
    if std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default() == "windows" {
        let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
        let icon_path = std::path::Path::new(&manifest_dir)
            .parent()
            .unwrap()
            .join("assets/icon.ico");

        let rc_content = format!("1 ICON \"{}\"\n", icon_path.display());
        let out_dir = std::env::var("OUT_DIR").unwrap();
        let rc_path = std::path::Path::new(&out_dir).join("resources.rc");
        std::fs::write(&rc_path, rc_content).unwrap();

        embed_resource::compile(&rc_path, embed_resource::NONE)
            .manifest_required()
            .expect("Failed to compile Windows resource file");
    }
}
