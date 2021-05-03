use legion::*;

pub struct WidgetOutput {
    pub entity: Entity,
    pub typ: WidgetOutputType,
}

pub enum WidgetOutputType {
    /// EditableText receive enter (contents)
    TextEdited(String),
}
