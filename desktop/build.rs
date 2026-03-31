fn main() {
    if std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default() == "windows" {
        let mut res = winresource::WindowsResource::new();
        res.set_icon("../../assets/io.github.joshuardecker.stig-view.ico");
        res.compile().unwrap();
    }
}
