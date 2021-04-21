#![allow(unused_variables, dead_code)]

use std::{collections::HashMap, ops::AddAssign};

use voxrs_asset::{AssetHandle, FontAsset};
use voxrs_math::{IVec2, Vec2, Vec4};
use voxrs_render::blueprint::{self, TextSection};

/*
- what widget do
- spawn widget (and child)
- receive input (click)
- react
*/
pub struct WidgetRepository {
    nodes: HashMap<WidgetNodeId, WidgetNode>,
    root_nodes: Vec<WidgetNodeId>,
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

pub struct WidgetNode {
    id: WidgetNodeId,
    parent: Option<WidgetNodeId>,
    children: Vec<WidgetNodeId>,
    widget: Widget,
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

pub enum Widget {
    Panel(PanelWidget),
    Text(TextWidget),
    Button(ButtonWidget),
}

impl Widget {
    pub fn render(&self, bp: &mut blueprint::Blueprint) {
        match self {
            Widget::Panel(panel) => panel.render(bp),
            Widget::Text(text) => text.render(bp),
            Widget::Button(button) => button.render(bp),
        }
    }
}

pub struct PanelWidget {
    pos: Vec2,
    size: Vec2,
    color: Vec4,
}

impl PanelWidget {
    pub fn new() -> Self {
        PanelWidget {
            pos: Vec2::new(0.0, 0.0),
            size: Vec2::new(100.0, 100.0),
            color: Vec4::new(1.0, 1.0, 1.0, 1.0),
        }
    }

    pub fn render(&self, bp: &mut blueprint::Blueprint) {
        let bp_panel = blueprint::Panel::new(self.pos, self.size, self.color);
        bp.uis.push(blueprint::Ui::Panel(bp_panel));
    }
}

pub struct TextWidget {
    pos: Vec2,
    size: Vec2,
    font: AssetHandle<FontAsset>,
    font_size: u32,
    contents: String,
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

pub struct ButtonWidget {
    pos: Vec2,
    size: Vec2,
}

impl ButtonWidget {
    pub fn new() -> Self {
        ButtonWidget {
            pos: Vec2::new(0.0, 0.0),
            size: Vec2::new(100.0, 100.0),
        }
    }
    pub fn render(&self, bp: &mut blueprint::Blueprint) {
        let bp_panel = blueprint::Panel {
            pos: self.pos,
            size: self.size,
            color: Vec4::new(1.0, 1.0, 1.0, 1.0),
        };

        bp.uis.push(blueprint::Ui::Panel(bp_panel));
    }
}

/// input for widgets
pub enum WidgetInput {
    MouseClick { pos: IVec2 },
}

/// reaction from widget when processing widget input
pub enum WidgetEvent {
    ButtonClicked(WidgetNodeId),
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
