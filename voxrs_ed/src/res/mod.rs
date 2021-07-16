mod editor_asset;
pub use editor_asset::EditorAssetRes;

pub struct EditorRes {
    pub block_mat_id: u8,
}

impl EditorRes {
    pub fn new() -> Self {
        Self { block_mat_id: 1 }
    }
}

impl Default for EditorRes {
    fn default() -> Self {
        Self::new()
    }
}

mod history;
pub(crate) use history::HistoryRes;
