pub(crate) struct HistoryRes<History: 'static> {
    undos: Vec<History>,
    redos: Vec<History>,
}

impl<History: 'static> HistoryRes<History> {
    pub const MAX_HISTORY: usize = 100;

    pub fn new() -> Self {
        Self {
            undos: Vec::new(),
            redos: Vec::new(),
        }
    }

    pub fn add_history(&mut self, history: Option<History>) {
        if let Some(history) = history {
            self.undos.push(history);
            if self.undos.len() >= Self::MAX_HISTORY {
                self.undos.remove(0);
            }

            self.redos.clear();
        }
    }

    /// redo -> undo
    pub fn add_undo(&mut self, history: History) {
        self.undos.push(history);
    }

    pub fn pop_undo(&mut self) -> Option<History> {
        self.undos.pop()
    }

    /// undo -> redo
    pub fn add_redo(&mut self, history: History) {
        self.redos.push(history);
    }

    pub fn pop_redo(&mut self) -> Option<History> {
        self.redos.pop()
    }
}
