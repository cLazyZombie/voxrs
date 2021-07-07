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
    history_cursor: Option<usize>,

    pub cursor: usize,
}

impl TerminalWidget {
    pub fn new(info: &TerminalInfo) -> Self {
        Self {
            font: info.font.clone(),
            font_size: info.font_size,
            contents: info.contents.clone(),
            input: String::new(),
            history: Vec::new(),
            history_cursor: None,
            cursor: 0,
        }
    }

    pub fn enter(&mut self) -> String {
        let mut input = String::new();
        std::mem::swap(&mut input, &mut self.input);
        self.contents.push(input.clone());
        self.history.push(input.clone());

        self.history_cursor = None;
        self.cursor = 0;

        input
    }

    pub fn prev(&mut self) {
        match self.history_cursor {
            Some(idx) => {
                let prev_idx = idx.checked_sub(1);
                if let Some(prev_idx) = prev_idx {
                    if prev_idx < self.history.len() {
                        self.history_cursor = Some(prev_idx);
                        self.input = self.history[prev_idx].clone();
                    }
                }
            }
            None => {
                if !self.history.is_empty() {
                    self.history_cursor = Some(self.history.len() - 1);
                    self.input = self.history.last().unwrap().clone();
                }
            }
        }
    }

    pub fn next(&mut self) {
        if let Some(idx) = self.history_cursor {
            let next_idx = idx + 1;
            if next_idx < self.history.len() {
                self.history_cursor = Some(next_idx);
                self.input = self.history[next_idx].clone();
            }
        }
    }

    pub fn add_input(&mut self, c: char) {
        let left = self.input.chars().take(self.cursor).collect::<String>();
        let right = self.input.chars().skip(self.cursor).collect::<String>();
        self.input = format!("{}{}{}", left, c, right);
        self.cursor += 1;
    }

    pub fn remove_input(&mut self) {
        let cursor = self.cursor.saturating_sub(1);
        let left = self.input.chars().take(cursor).collect::<String>();
        let right = self.input.chars().skip(cursor).collect::<String>();

        self.input = format!("{}{}", left, right);
        self.cursor = cursor;
    }

    pub fn left(&mut self) {
        self.cursor = self.cursor.saturating_sub(1);
    }

    pub fn right(&mut self) {
        self.cursor = usize::min(self.cursor + 1, self.input.chars().count());
    }
}
