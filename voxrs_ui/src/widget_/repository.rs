use std::collections::HashMap;

use voxrs_math::{Rect2, Vec2};
use voxrs_render::blueprint;

use crate::{ConsoleWidget, ConsoleWidgetInfo};

use super::{
    button::ButtonWidget, id::WidgetId, node::WidgetNode, panel::PanelWidget, text::TextWidget,
    ButtonWidgetInfo, PanelWidgetInfo, TextWidgetInfo, Widget, WidgetEvent, WidgetInput,
};

pub struct WidgetRepository {
    pub(crate) nodes: HashMap<WidgetId, WidgetNode>,
    pub(crate) root_nodes: Vec<WidgetId>,
    next_id: WidgetId,
}

impl WidgetRepository {
    pub fn new() -> Self {
        WidgetRepository {
            nodes: HashMap::new(),
            root_nodes: Vec::new(),
            next_id: WidgetId::new(1),
        }
    }

    pub fn build(&mut self) -> WidgetBuilder<'_> {
        WidgetBuilder::new(self)
    }

    pub fn get_next_id(&mut self) -> WidgetId {
        let next = self.next_id;
        self.next_id += 1;
        next
    }

    pub fn render(&self, bp: &mut blueprint::Blueprint) {
        let parent_region = Rect2::from_min_max(Vec2::ZERO, Vec2::new(f32::MAX, f32::MAX));

        for root_id in &self.root_nodes {
            let root_widget = self.nodes.get(root_id).unwrap();
            root_widget.render(parent_region, self, bp);
        }
    }

    pub fn process(&self, input: &WidgetInput) -> Vec<WidgetEvent> {
        let root_region = Rect2::from_min_max(Vec2::ZERO, Vec2::new(f32::MAX, f32::MAX));
        let mut events = Vec::new();

        for root_id in &self.root_nodes {
            let root_widget = self.nodes.get(root_id).unwrap();
            root_widget.process(input, root_region, self, &mut events);
        }

        events
    }
}

pub struct WidgetBuilder<'a> {
    repository: &'a mut WidgetRepository,
    widgets: Vec<WidgetNode>,
    parent: Option<WidgetId>,
}

impl<'a> WidgetBuilder<'a> {
    pub fn new(repository: &'a mut WidgetRepository) -> Self {
        Self {
            repository,
            widgets: Vec::new(),
            parent: None,
        }
    }

    fn add_widget(&mut self, widget_id: WidgetId, widget: Widget) {
        let widget_node = WidgetNode {
            id: widget_id,
            parent: self.parent,
            children: Vec::new(),
            widget,
        };
        self.widgets.push(widget_node);
    }

    pub fn panel(mut self, info: PanelWidgetInfo) -> Self {
        let widget_id = self.repository.get_next_id();
        let panel_widget = PanelWidget::new(widget_id, info);
        let widget = Widget::Panel(panel_widget);
        self.add_widget(widget_id, widget);
        self
    }

    pub fn button(mut self, info: ButtonWidgetInfo) -> Self {
        let widget_id = self.repository.get_next_id();
        let button_widget = ButtonWidget::new(widget_id, info);
        let widget = Widget::Button(button_widget);
        self.add_widget(widget_id, widget);
        self
    }

    pub fn text(mut self, info: TextWidgetInfo) -> Self {
        let widget_id = self.repository.get_next_id();
        let text_widget = TextWidget::new(widget_id, info);
        let widget = Widget::Text(text_widget);
        self.add_widget(widget_id, widget);
        self
    }

    pub fn console(mut self, info: ConsoleWidgetInfo) -> Self {
        let widget_id = self.repository.get_next_id();
        let console_widget = ConsoleWidget::new(widget_id, info);
        let widget = Widget::Console(console_widget);
        self.add_widget(widget_id, widget);
        self
    }

    pub fn query_id(self, id: &mut WidgetId) -> Self {
        *id = self.widgets.last().unwrap().id;
        self
    }

    pub fn child<F>(mut self, mut f: F) -> Self
    where
        F: FnMut(WidgetBuilder<'_>) -> WidgetBuilder<'_>, // for<'b> F: FnMut(WidgetBuilder<'b>) -> WidgetBuilder<'b>
    {
        let mut parent = self.widgets.pop().unwrap();
        let child_widget_builder = WidgetBuilder {
            repository: self.repository,
            widgets: Vec::new(),
            parent: Some(parent.id),
        };
        let mut child_widget_builder = f(child_widget_builder);

        parent.children.append(
            &mut child_widget_builder
                .widgets
                .iter()
                .map(|widget| widget.id)
                .collect::<Vec<_>>(),
        );

        self.widgets.append(&mut child_widget_builder.widgets);
        self.widgets.push(parent);
        self
    }

    pub fn finish(self) {
        for widget in self.widgets {
            if widget.parent.is_none() {
                self.repository.root_nodes.push(widget.id);
            }
            self.repository.nodes.insert(widget.id, widget);
        }
    }
}

#[cfg(test)]
mod tests {
    use voxrs_math::IVec2;
    use voxrs_render::blueprint;

    use super::*;

    #[test]
    fn test_create_button() {
        let mut repository = WidgetRepository::new();
        let mut button_id_1 = WidgetId::default();
        let mut button_id_2 = WidgetId::default();

        repository
            .build()
            .panel(PanelWidgetInfo {
                pos: (0.0, 0.0).into(),
                size: (100.0, 100.0).into(),
                color: (1.0, 1.0, 1.0, 1.0).into(),
            })
            .child(|builder| {
                builder
                    .button(ButtonWidgetInfo {
                        pos: (10.0, 10.0).into(),
                        size: (100.0, 50.0).into(),
                    })
                    .query_id(&mut button_id_1)
            })
            .child(|builder| {
                builder
                    .button(ButtonWidgetInfo {
                        pos: (30.0, 10.0).into(),
                        size: (100.0, 50.0).into(),
                    })
                    .query_id(&mut button_id_2)
            })
            .finish();

        assert_eq!(button_id_1, WidgetId::new(2));
        assert_eq!(button_id_2, WidgetId::new(3));

        assert_eq!(repository.nodes.len(), 3);
        assert_eq!(repository.root_nodes.len(), 1);
        assert_eq!(
            repository.nodes.get(&WidgetId::new(1)).unwrap().id,
            WidgetId::new(1)
        );
        assert_eq!(
            repository.nodes.get(&WidgetId::new(2)).unwrap().id,
            WidgetId::new(2)
        );
        assert_eq!(
            repository.nodes.get(&WidgetId::new(3)).unwrap().id,
            WidgetId::new(3)
        );

        // test render
        let mut bp = blueprint::Blueprint::new();
        repository.render(&mut bp);

        assert_eq!(bp.uis.len(), 3);

        // mouse click event to button1
        let input = WidgetInput::MouseClick {
            pos: IVec2::new(30, 30),
        };

        let events = repository.process(&input);
        assert_eq!(events.len(), 1);
        assert!(matches!(events[0], WidgetEvent::ButtonClicked(id) if id == button_id_1));
    }
}
