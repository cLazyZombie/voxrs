use std::ops::AddAssign;

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub struct WidgetNodeId(u64);

impl AddAssign<u64> for WidgetNodeId {
    fn add_assign(&mut self, rhs: u64) {
        self.0 = rhs + self.0;
    }
}

impl WidgetNodeId {
    pub fn new(id: u64) -> Self {
        WidgetNodeId(id)
    }
}

impl Default for WidgetNodeId {
    fn default() -> Self {
        Self(0)
    }
}
