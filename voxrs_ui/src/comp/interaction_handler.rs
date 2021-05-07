use crate::{res, Interaction};

type InteractionFn<Message> = dyn Fn(Interaction) -> Option<Message> + Send + Sync + 'static;

pub struct InteractionHandler<Message: 'static> {
    handler: Box<InteractionFn<Message>>,
}

impl<Message: 'static> InteractionHandler<Message> {
    pub fn new<F>(handler: F) -> Self
    where
        F: Fn(Interaction) -> Option<Message> + Send + Sync + 'static,
    {
        let handler = Box::new(handler) as Box<InteractionFn<Message>>;
        Self { handler }
    }

    pub fn process(&self, msg: Interaction, output_queue: &mut res::OutputQueue<Message>) {
        if let Some(output) = self.handler.as_ref()(msg) {
            output_queue.add(output)
        }
    }
}
