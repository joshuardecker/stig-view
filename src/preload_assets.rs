#[derive(Debug, Clone)]
pub struct Assets {
    pub file_svg: Vec<u8>,
    pub folder_svg: Vec<u8>,
    pub lightbulb_svg: Vec<u8>,
    pub lightbulb_filled_svg: Vec<u8>,
    pub right_tick_svg: Vec<u8>,
    pub terminal_svg: Vec<u8>,
}

impl Assets {
    pub fn new() -> Self {
        Self {
            file_svg: include_bytes!("../assets/images/file.svg").to_vec(),
            folder_svg: include_bytes!("../assets/images/folder.svg").to_vec(),
            lightbulb_svg: include_bytes!("../assets/images/lightbulb.svg").to_vec(),
            lightbulb_filled_svg: include_bytes!("../assets/images/lightbulb-filled.svg").to_vec(),
            right_tick_svg: include_bytes!("../assets/images/right-tick.svg").to_vec(),
            terminal_svg: include_bytes!("../assets/images/terminal.svg").to_vec(),
        }
    }
}
