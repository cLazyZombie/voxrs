use voxrs_asset::{AssetHandle, FontAsset};
use voxrs_math::Vec4;

use crate::WidgetPlacementInfo;

pub struct TerminalInfo {
    pub placement: WidgetPlacementInfo,
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

    history: Vec<String>,
    cursor: Option<usize>,
}

impl TerminalWidget {
    pub fn new(info: &TerminalInfo) -> Self {
        Self {
            font: info.font.clone(),
            font_size: info.font_size,
            contents: info.contents.clone(),
            input: String::new(),
            history: Vec::new(),
            cursor: None,
        }
    }

    pub fn enter(&mut self) -> String {
        let mut input = String::new();
        std::mem::swap(&mut input, &mut self.input);
        self.contents.push(input.clone());
        self.history.push(input.clone());

        self.cursor = None;

        input
    }

    pub fn prev(&mut self) {
        match self.cursor {
            Some(idx) => {
                let prev_idx = idx.checked_sub(1);
                if let Some(prev_idx) = prev_idx {
                    if prev_idx < self.history.len() {
                        self.cursor = Some(prev_idx);
                        self.input = self.history[prev_idx].clone();
                    }
                }
            }
            None => {
                if !self.history.is_empty() {
                    self.cursor = Some(self.history.len() - 1);
                    self.input = self.history.last().unwrap().clone();
                }
            }
        }
    }

    pub fn next(&mut self) {
        #[allow(clippy::single_match)]
        match self.cursor {
            Some(idx) => {
                let next_idx = idx + 1;
                if next_idx < self.history.len() {
                    self.cursor = Some(next_idx);
                    self.input = self.history[next_idx].clone();
                }
            }
            None => {}
        }
    }
}

// impl TerminalWidget {
//     pub fn process_input_char<Message: 'static>(c: char) -> Option<Message> {
//         None
//     }
// }
