use voxrs_asset::{AssetHandle, FontAsset};
use voxrs_math::{Vec2, Vec4};

pub struct TerminalInfo {
    pub pos: Vec2,
    pub size: Vec2,
    pub color: Vec4,
    pub font: AssetHandle<FontAsset>,
    pub font_size: u32,
    pub contents: Vec<String>,
}

pub(crate) struct TerminalWidget {
    pub font: AssetHandle<FontAsset>,
    pub font_size: u32,
    pub contents: Vec<String>,
    pub input: String,
}

// impl TerminalWidget {
//     pub fn process_input_char<Message: 'static>(c: char) -> Option<Message> {
//         None
//     }
// }
