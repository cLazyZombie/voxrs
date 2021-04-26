use std::ops::AddAssign;

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub struct WidgetId(u64);

impl AddAssign<u64> for WidgetId {
    fn add_assign(&mut self, rhs: u64) {
        self.0 = rhs + self.0;
    }
}

impl WidgetId {
    pub fn new(id: u64) -> Self {
        WidgetId(id)
    }
}

impl Default for WidgetId {
    fn default() -> Self {
        Self(0)
    }
}
