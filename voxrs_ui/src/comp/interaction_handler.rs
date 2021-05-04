use crate::{output::WidgetOutput, res, Interaction};

type InteractionFn = dyn Fn(Interaction) -> Option<Box<dyn WidgetOutput>> + Send + Sync + 'static;

pub struct InteractionHandler {
    handler: Box<InteractionFn>,
}

impl InteractionHandler {
    pub fn new<F>(handler: F) -> Self
    where
        F: Fn(Interaction) -> Option<Box<dyn WidgetOutput>> + Send + Sync + 'static,
    {
        let handler = Box::new(handler) as Box<InteractionFn>;
        Self { handler }
    }

    pub fn process(&self, msg: Interaction, output_queue: &mut res::OutputQueue) {
        if let Some(output) = self.handler.as_ref()(msg) {
            output_queue.add(output)
        }
    }
}
