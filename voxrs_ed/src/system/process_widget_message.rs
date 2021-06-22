use std::path::Path;

use legion::*;
use voxrs_asset::{AssetManager, AssetPath};
use voxrs_core::res::WorldBlockRes;
use voxrs_types::io::FileSystem;

use crate::{command::Command, widget_message::WidgetMessage};
use voxrs_ui::OutputQueue;

#[system]
pub fn process_widget_message<F: FileSystem + 'static>(
    #[resource] asset_manager: &mut AssetManager<F>,
    #[resource] output_queue: &mut OutputQueue<WidgetMessage>,
    #[resource] world_block: &mut WorldBlockRes,
) {
    //todo. into iterator로 처리
    for m in output_queue.iter() {
        match m {
            WidgetMessage::ConsoleCommand(command) => match command {
                Command::Save(path) => {
                    let raw_asset = world_block.make_raw_asset();
                    let raw_asset_json = serde_json::to_string(&raw_asset);
                    if let Ok(json) = raw_asset_json {
                        let result = F::write_text(Path::new(path), &json);
                        if let Err(err) = result {
                            eprintln!("error on save: {:?}", err);
                        }
                    }
                }
                Command::Load(path) => {
                    let asset_path = path.to_str();
                    if let Some(asset_path) = asset_path {
                        let world_block_res = WorldBlockRes::new(&AssetPath::from(asset_path), asset_manager);
                        *world_block = world_block_res;
                    } else {
                        eprintln!("can not convert {:?} as &str", path);
                    }
                }
            },
            WidgetMessage::Other => {}
        }
    }

    output_queue.clear();
}
