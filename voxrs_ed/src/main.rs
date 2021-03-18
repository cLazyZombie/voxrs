use voxrs_asset::{AssetManager, WorldMaterialAsset};
use voxrs_render::blueprint::Blueprint;
use voxrs_render::render;
use voxrs_types::io::GeneralFileSystem;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

fn main() {
    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .init();

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    let _aspect = window.inner_size().width as f32 / window.inner_size().height as f32;
    let mut asset_manager = AssetManager::<GeneralFileSystem>::new();

    let mut renderer =
        futures::executor::block_on(render::Renderer::new(&window, &mut asset_manager));
    
    let world_block_mat = asset_manager.get::<WorldMaterialAsset>(&"assets/world_mat.wmt".into());

    event_loop.run(move |event, _, control_flow| match event {
        Event::WindowEvent {
            ref event,
            window_id,
        } if window_id == window.id() => match event {
            WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
            WindowEvent::Resized(physical_size) => {
                renderer.resize(*physical_size);
            }
            WindowEvent::KeyboardInput { input: _, .. } => {}
            _ => {}
        },
        Event::RedrawRequested(_) => {}
        Event::MainEventsCleared => {
            let mut bp = Blueprint::new();
            bp.world_block_mat_handle = Some(world_block_mat.clone());
            renderer.render(bp).unwrap();
        }
        _ => {}
    });
}
