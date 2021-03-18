use voxrs_asset::AssetManager;
use voxrs_ed::{res, Editor};
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

    let aspect = window.inner_size().width as f32 / window.inner_size().height as f32;
    let mut key_input: Option<res::KeyInput> = None;

    let mut asset_manager = AssetManager::<GeneralFileSystem>::new();

    let mut renderer =
        futures::executor::block_on(render::Renderer::new(&window, &mut asset_manager));

    let mut editor = Editor::new(aspect, &mut asset_manager);

    event_loop.run(move |event, _, control_flow| match event {
        Event::WindowEvent {
            ref event,
            window_id,
        } if window_id == window.id() => match event {
            WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
            WindowEvent::Resized(physical_size) => {
                editor.resize(physical_size.width, physical_size.height);
                renderer.resize(*physical_size);
            }
            WindowEvent::KeyboardInput { input, .. } => {
                key_input = Some((*input).into());
            }
            _ => {}
        },
        Event::RedrawRequested(_) => {}
        Event::MainEventsCleared => {
            editor.set_input(key_input);
            editor.tick();
            let bp = editor.render();
            renderer.render(bp).unwrap();
        }
        _ => {}
    });
}
