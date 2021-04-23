use std::collections::HashMap;

use voxrs_render::blueprint;

use super::{
    button::ButtonWidget, id::WidgetNodeId, node::WidgetNode, panel::PanelWidget, text::TextWidget,
    Widget,
};

pub struct WidgetRepository {
    pub(crate) nodes: HashMap<WidgetNodeId, WidgetNode>,
    pub(crate) root_nodes: Vec<WidgetNodeId>,
    next_node_id: WidgetNodeId,
}

impl WidgetRepository {
    pub fn new() -> Self {
        WidgetRepository {
            nodes: HashMap::new(),
            root_nodes: Vec::new(),
            next_node_id: WidgetNodeId::new(1),
        }
    }

    pub fn build(&mut self) -> WidgetBuilder<'_> {
        WidgetBuilder::new(self)
    }

    pub fn get_next_node_id(&mut self) -> WidgetNodeId {
        let next = self.next_node_id;
        self.next_node_id += 1;
        next
    }

    pub fn render(&self, bp: &mut blueprint::Blueprint) {
        for root_id in &self.root_nodes {
            let root_widget = self.nodes.get(root_id).unwrap();
            root_widget.render(self, bp);
        }
    }
}

pub struct WidgetBuilder<'a> {
    repository: &'a mut WidgetRepository,
    widgets: Vec<WidgetNode>,
    parent: Option<WidgetNodeId>,
}

impl<'a> WidgetBuilder<'a> {
    pub fn new(repository: &'a mut WidgetRepository) -> Self {
        Self {
            repository,
            widgets: Vec::new(),
            parent: None,
        }
    }

    fn add_widget(&mut self, widget: Widget) {
        let widget_id = self.repository.get_next_node_id();
        let widget_node = WidgetNode {
            id: widget_id,
            parent: self.parent,
            children: Vec::new(),
            widget,
        };
        self.widgets.push(widget_node);
    }

    pub fn panel(mut self, panel_widget: PanelWidget) -> Self {
        let widget = Widget::Panel(panel_widget);
        self.add_widget(widget);
        self
    }

    pub fn button(mut self, button_widget: ButtonWidget) -> Self {
        let widget = Widget::Button(button_widget);
        self.add_widget(widget);
        self
    }

    pub fn text(mut self, text_widget: TextWidget) -> Self {
        let widget = Widget::Text(text_widget);
        self.add_widget(widget);
        self
    }

    pub fn query_id(self, id: &mut WidgetNodeId) -> Self {
        *id = self.widgets.last().unwrap().id;
        self
    }

    // pub fn position(mut self, pos: Vec2) -> Self {
    //     let widget_node = self.widgets.last_mut().unwrap();
    //     match &mut widget_node.widget {
    //         Widget::Panel(panel) => panel.pos = pos,
    //         Widget::Text(text) => text.pos = pos,
    //         Widget::Button(button) => button.pos = pos,
    //     }
    //     self
    // }

    // pub fn size(mut self, size: Vec2) -> Self {
    //     let widget_node = self.widgets.last_mut().unwrap();
    //     match &mut widget_node.widget {
    //         Widget::Panel(panel) => panel.size = size,
    //         Widget::Text(text) => text.size = size,
    //         Widget::Button(button) => button.size = size,
    //     }
    //     self
    // }

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
    use voxrs_render::blueprint;

    use super::*;

    #[test]
    fn test_create_button() {
        let mut repository = WidgetRepository::new();
        let mut button_id_1 = WidgetNodeId::default();
        let mut button_id_2 = WidgetNodeId::default();

        repository
            .build()
            .panel(PanelWidget {
                pos: (0.0, 0.0).into(),
                size: (100.0, 100.0).into(),
                color: (1.0, 1.0, 1.0, 1.0).into(),
            })
            .child(|builder| {
                builder
                    .button(ButtonWidget {
                        pos: (10.0, 10.0).into(),
                        size: (100.0, 50.0).into(),
                    })
                    .query_id(&mut button_id_1)
            })
            .child(|builder| {
                builder
                    .button(ButtonWidget {
                        pos: (30.0, 10.0).into(),
                        size: (100.0, 50.0).into(),
                    })
                    .query_id(&mut button_id_2)
            })
            .finish();

        assert_eq!(button_id_1, WidgetNodeId::new(2));
        assert_eq!(button_id_2, WidgetNodeId::new(3));

        assert_eq!(repository.nodes.len(), 3);
        assert_eq!(repository.root_nodes.len(), 1);
        assert_eq!(
            repository.nodes.get(&WidgetNodeId::new(1)).unwrap().id,
            WidgetNodeId::new(1)
        );
        assert_eq!(
            repository.nodes.get(&WidgetNodeId::new(2)).unwrap().id,
            WidgetNodeId::new(2)
        );
        assert_eq!(
            repository.nodes.get(&WidgetNodeId::new(3)).unwrap().id,
            WidgetNodeId::new(3)
        );

        let mut bp = blueprint::Blueprint::new();
        repository.render(&mut bp);

        assert_eq!(bp.uis.len(), 3);
    }
}
