pub(crate) struct HistoryRes<History: 'static> {
    history: Vec<History>,
    undo_count: Option<usize>,
}

impl<History: 'static> HistoryRes<History> {
    pub const MAX_HISTORY: usize = 100;

    pub fn new() -> Self {
        Self {
            history: Vec::new(),
            undo_count: None,
        }
    }

    pub fn add_history(&mut self, history: History) {
        if let Some(cursor) = self.undo_count {
            if cursor != self.history.len() {
                self.history.truncate(cursor);
            }
        }

        self.history.push(history);

        if self.history.len() <= Self::MAX_HISTORY {
            let undo_count = self.undo_count.unwrap_or(0) + 1;
            self.undo_count = Some(undo_count);
        } else {
            self.history.remove(0);
        }
    }

    pub fn undo(&mut self) -> Option<&History> {
        let undo_idx = self.undo_count.unwrap_or(0);
        if undo_idx == 0 {
            return None;
        }

        let undo_idx = usize::saturating_sub(undo_idx, 1);
        self.undo_count = Some(undo_idx);

        if let Some(history) = self.history.get(undo_idx) {
            Some(history)
        } else {
            None
        }
    }

    pub fn redo(&mut self) -> Option<&History> {
        if self.undo_count.is_none() {
            return None;
        }

        let redo_idx = self.undo_count.unwrap();
        if let Some(history) = self.history.get(redo_idx) {
            self.undo_count = Some(redo_idx + 1);
            Some(history)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(PartialEq, Eq, Debug)]
    enum History {
        A,
        B,
        C,
        D,
        E,
    }

    #[test]
    fn test_history() {
        let mut history_res = HistoryRes::<History>::new();
        assert_eq!(history_res.undo_count, None);

        history_res.add_history(History::A);
        history_res.add_history(History::B);
        history_res.add_history(History::C);
        history_res.add_history(History::D);
        assert_eq!(history_res.history.len(), 4);
        assert_eq!(history_res.undo_count, Some(4));

        assert_eq!(history_res.undo(), Some(&History::D));
        assert_eq!(history_res.undo(), Some(&History::C));

        assert_eq!(history_res.redo(), Some(&History::C));
        assert_eq!(history_res.redo(), Some(&History::D));
        assert_eq!(history_res.redo(), None);

        assert_eq!(history_res.undo(), Some(&History::D));
        assert_eq!(history_res.undo(), Some(&History::C));
        assert_eq!(history_res.undo(), Some(&History::B));
        assert_eq!(history_res.undo(), Some(&History::A));
        assert_eq!(history_res.undo(), None);

        assert_eq!(history_res.redo(), Some(&History::A));
        assert_eq!(history_res.redo(), Some(&History::B));

        history_res.add_history(History::E);
        assert_eq!(history_res.history.len(), 3);
        assert_eq!(history_res.undo_count, Some(3));

        assert_eq!(history_res.undo(), Some(&History::E));
        assert_eq!(history_res.undo(), Some(&History::B));
        assert_eq!(history_res.undo(), Some(&History::A));
    }

    #[test]
    fn test_max_count() {
        let mut history_res = HistoryRes::<History>::new();
        for _ in 0..HistoryRes::<History>::MAX_HISTORY {
            history_res.add_history(History::A);
        }
        assert_eq!(history_res.history.len(), HistoryRes::<History>::MAX_HISTORY);

        history_res.add_history(History::B);
        assert_eq!(history_res.history.len(), HistoryRes::<History>::MAX_HISTORY);
        assert_eq!(history_res.undo_count, Some(HistoryRes::<History>::MAX_HISTORY));
    }
}
