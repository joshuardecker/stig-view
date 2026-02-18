/// All asset files are bundled into the binary as a vector of bytes.
#[derive(Debug, Clone)]
pub struct Assets {
    pub file_svg: Vec<u8>,
    pub folder_svg: Vec<u8>,
    pub filter_svg: Vec<u8>,
    pub right_tick_svg: Vec<u8>,
    pub down_tick_svg: Vec<u8>,
    pub cross_svg: Vec<u8>,
    pub check_svg: Vec<u8>,
    pub square_svg: Vec<u8>,
    pub bookmark_svg: Vec<u8>,
    pub bookmark_filled_svg: Vec<u8>,
    pub terminal_svg: Vec<u8>,
}

impl Assets {
    pub fn new() -> Self {
        Self {
            file_svg: include_bytes!("../assets/images/file.svg").to_vec(),
            folder_svg: include_bytes!("../assets/images/folder.svg").to_vec(),
            filter_svg: include_bytes!("../assets/images/filter.svg").to_vec(),
            right_tick_svg: include_bytes!("../assets/images/right-tick.svg").to_vec(),
            down_tick_svg: include_bytes!("../assets/images/down-tick.svg").to_vec(),
            cross_svg: include_bytes!("../assets/images/cross.svg").to_vec(),
            check_svg: include_bytes!("../assets/images/check.svg").to_vec(),
            square_svg: include_bytes!("../assets/images/square.svg").to_vec(),
            bookmark_svg: include_bytes!("../assets/images/bookmark.svg").to_vec(),
            bookmark_filled_svg: include_bytes!("../assets/images/bookmark-filled.svg").to_vec(),
            terminal_svg: include_bytes!("../assets/images/terminal.svg").to_vec(),
        }
    }
}
