use voxrs_render::blueprint;

use super::{id::WidgetNodeId, Widget, WidgetRepository};

pub struct WidgetNode {
    pub(crate) id: WidgetNodeId,
    pub(crate) parent: Option<WidgetNodeId>,
    pub(crate) children: Vec<WidgetNodeId>,
    pub(crate) widget: Widget,
}

impl WidgetNode {
    pub fn render(&self, repository: &WidgetRepository, bp: &mut blueprint::Blueprint) {
        self.widget.render(bp);

        // render children
        for child_id in &self.children {
            let child_widget = repository.nodes.get(child_id).unwrap();
            child_widget.render(repository, bp);
        }
    }
}
