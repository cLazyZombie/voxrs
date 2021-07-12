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
                        self.cursor = self.input.chars().count();
                    }
                }
            }
            None => {
                if !self.history.is_empty() {
                    self.history_cursor = Some(self.history.len() - 1);
                    self.input = self.history.last().unwrap().clone();
                    self.cursor = self.input.chars().count();
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
                self.cursor = self.input.chars().count();
            }
        }
    }

    pub fn add_input(&mut self, c: char) {
        let left = self.input.chars().take(self.cursor).collect::<String>();
        let right = self.input.chars().skip(self.cursor).collect::<String>();
        self.input = format!("{}{}{}", left, c, right);
        self.cursor += 1;
    }

    pub fn remove_left(&mut self) {
        let right = self.input.chars().skip(self.cursor).collect::<String>();

        let cursor = self.cursor.saturating_sub(1);
        let left = self.input.chars().take(cursor).collect::<String>();

        self.input = format!("{}{}", left, right);
        self.cursor = cursor;
    }

    pub fn remove_right(&mut self) {
        let left = self.input.chars().take(self.cursor).collect::<String>();

        let cursor = usize::min(self.cursor + 1, self.input.chars().count());
        let right = self.input.chars().skip(cursor).collect::<String>();

        self.input = format!("{}{}", left, right);
    }

    pub fn left(&mut self) {
        self.cursor = self.cursor.saturating_sub(1);
    }

    pub fn right(&mut self) {
        self.cursor = usize::min(self.cursor + 1, self.input.chars().count());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use voxrs_types::io::tests::MockFileSystem;

    #[test]
    fn test_create() {
        let mut manager = voxrs_asset::AssetManager::<MockFileSystem>::new();
        let font_asset = manager.get::<FontAsset>(&"font.ttf".into());

        let info = TerminalInfo{
            placement: WidgetPlacementInfo {
                pos: (0, 0).into(),
                h_anchor: None,
                v_anchor: None,
                size: (100, 100).into(),
            },
            color: Vec4::new(1.0,1.0, 1.0, 1.0),
            font: font_asset,
            font_size: 10,
            contents: Vec::new(),
        };

        let mut term = TerminalWidget::new(&info);

        // test add input
        for c in "hello, world".chars() {
            term.add_input(c);
        }
        term.enter();

        assert_eq!(term.contents[0], "hello, world");

        // backspace
        "hello, world".chars().for_each(|c| term.add_input(c));
        term.remove_left();
        assert_eq!(term.input, "hello, worl");

        // remove comma using left key and backspace
        (0..5).for_each(|_| term.left());
        term.remove_left();
        assert_eq!(term.input, "hello worl");
        assert_eq!(term.cursor, 5);

        // remove space using remove right
        term.remove_right();
        assert_eq!(term.input, "helloworl");
        assert_eq!(term.cursor, 5);

        // test prev
        term.enter();

        term.prev();
        assert_eq!(term.input, "helloworl");
        assert_eq!(term.cursor, 9);

        term.prev();
        assert_eq!(term.input, "hello, world");
        assert_eq!(term.cursor, 12);

        // test next
        term.next();
        assert_eq!(term.input,"helloworl");
        assert_eq!(term.cursor, 9);
    }
}