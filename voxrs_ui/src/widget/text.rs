use blueprint::TextSection;
use voxrs_asset::{AssetHandle, FontAsset};
use voxrs_math::Vec2;
use voxrs_render::blueprint;

pub struct TextWidget {
    pub pos: Vec2,
    pub size: Vec2,
    pub font: AssetHandle<FontAsset>,
    pub font_size: u32,
    pub contents: String,
}

impl TextWidget {
    pub fn render(&self, bp: &mut blueprint::Blueprint) {
        let section = TextSection {
            font: self.font.clone(),
            font_size: self.font_size,
            text: self.contents.clone(),
        };

        let bp_text = blueprint::Text {
            pos: self.pos,
            size: self.size,
            sections: vec![section],
        };

        bp.uis.push(blueprint::Ui::Text(bp_text));
    }
}
