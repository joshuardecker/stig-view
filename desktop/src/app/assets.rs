/// All asset files are bundled into the binary as a vector of bytes.
#[derive(Debug, Clone)]
pub struct Assets {
    pub right_tick_svg: Vec<u8>,
    pub down_tick_svg: Vec<u8>,
    pub cross_svg: Vec<u8>,
    pub square_svg: Vec<u8>,
    pub bookmark_svg: Vec<u8>,
    pub bookmark_filled_svg: Vec<u8>,
    pub settings_svg: Vec<u8>,

    pub app_icon: Vec<u8>,
}

impl Assets {
    pub fn new() -> Self {
        Self {
            right_tick_svg: include_bytes!("../../../assets/images/right-tick.svg").to_vec(),
            down_tick_svg: include_bytes!("../../../assets/images/down-tick.svg").to_vec(),
            cross_svg: include_bytes!("../../../assets/images/cross.svg").to_vec(),
            square_svg: include_bytes!("../../../assets/images/square.svg").to_vec(),
            bookmark_svg: include_bytes!("../../../assets/images/bookmark.svg").to_vec(),
            bookmark_filled_svg: include_bytes!("../../../assets/images/bookmark-filled.svg")
                .to_vec(),
            settings_svg: include_bytes!("../../../assets/images/settings.svg").to_vec(),

            app_icon: include_bytes!("../../../assets/io.github.joshuardecker.stig-view.png")
                .to_vec(),
        }
    }
}
