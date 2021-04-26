use voxrs_math::Rect2;
use voxrs_render::blueprint;

use crate::WidgetEvent;

use super::{id::WidgetId, Widget, WidgetInput, WidgetRepository};

pub struct WidgetNode {
    pub(crate) id: WidgetId,
    pub(crate) parent: Option<WidgetId>,
    pub(crate) children: Vec<WidgetId>,
    pub(crate) widget: Widget,
}

impl WidgetNode {
    pub fn render(
        &self,
        parent_region: Rect2,
        repository: &WidgetRepository,
        bp: &mut blueprint::Blueprint,
    ) {
        self.widget.render(parent_region, bp);

        let self_region = self.widget.intersect_region(parent_region);

        // render children
        for child_id in &self.children {
            let child_widget = repository.nodes.get(child_id).unwrap();
            child_widget.render(self_region, repository, bp);
        }
    }

    pub fn process(
        &self,
        input: &WidgetInput,
        parent_region: Rect2,
        repository: &WidgetRepository,
        events: &mut Vec<WidgetEvent>,
    ) -> bool {
        let self_region = self.widget.intersect_region(parent_region);
        match input {
            WidgetInput::MouseClick { pos } => {
                if self_region.has_ivec2(*pos) {
                    if self.widget.process(input, events) {
                        return true;
                    }
                }
            }
        }

        // pass to child
        for child_id in &self.children {
            let child_widget = repository.nodes.get(child_id).unwrap();
            if child_widget.process(input, self_region, repository, events) {
                return true;
            }
        }

        false
    }
}
