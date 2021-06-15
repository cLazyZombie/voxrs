use std::path::Path;

use legion::*;
use voxrs_core::res::WorldBlockRes;
use voxrs_types::io::FileSystem;

use crate::{command::Command, widget_message::WidgetMessage};
use voxrs_ui::OutputQueue;

#[system]
pub fn process_widget_message<F: FileSystem>(
    #[resource] output_queue: &mut OutputQueue<WidgetMessage>,
    #[resource] world_block: &mut WorldBlockRes,
) {
    for m in output_queue.iter() {
        match m {
            WidgetMessage::ConsoleCommand(command) => {
                match command {
                    Command::Save(path) => {
                        let raw_asset = world_block.make_raw_asset();
                        let raw_asset_json = serde_json::to_string(&raw_asset);
                        if let Ok(json) = raw_asset_json {
                            let result = F::write_text(Path::new(path), &json);
                            if let Err(err) = result {
                                eprintln!("Error on save: {:?}", err);
                            }
                        }
                    }
                    Command::Load(_path) => {}
                }
                println!("command. {:?}", command);
            }
            WidgetMessage::Other => {}
        }
    }

    output_queue.clear();
}
