use voxrs_asset::AssetManager;
use voxrs_ed::Editor;
use voxrs_render::render;
use voxrs_types::io::GeneralFileSystem;
use winit::{
    event::{DeviceEvent, Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

fn main() {
    profiling::register_thread!("Main Thread");

    env_logger::builder().filter_level(log::LevelFilter::Info).init();

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    let mut asset_manager = AssetManager::<GeneralFileSystem>::new();

    let mut renderer = futures::executor::block_on(render::Renderer::new(&window, &mut asset_manager));

    let mut editor = Editor::new(window.inner_size().width, window.inner_size().height, asset_manager);

    #[allow(clippy::single_match, clippy::collapsible_match)]
    event_loop.run(move |event, _, control_flow| match event {
        Event::WindowEvent { ref event, window_id } if window_id == window.id() => match event {
            WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
            WindowEvent::Resized(physical_size) => {
                editor.resize(physical_size.width, physical_size.height);
                renderer.resize(*physical_size);
            }
            WindowEvent::KeyboardInput { input, .. } => {
                editor.on_key_input::<GeneralFileSystem>(input);
            }
            WindowEvent::MouseInput { state, button, .. } => {
                editor.on_mouse_input(*button, *state);
            }
            WindowEvent::CursorMoved { position, .. } => {
                editor.on_cursor_moved((position.x as f32, position.y as f32));
            }
            WindowEvent::ModifiersChanged(modifier) => {
                editor.on_modifier_changed(modifier);
            }
            WindowEvent::ReceivedCharacter(c) => {
                editor.on_receive_character(*c);
            }
            _ => {}
        },
        Event::DeviceEvent { ref event, .. } => match event {
            DeviceEvent::MouseMotion { delta } => {
                editor.on_mouse_motion(*delta);
            }
            _ => {}
        },
        Event::RedrawRequested(_) => {}
        Event::MainEventsCleared => {
            editor.tick();
            let bp = editor.render();
            renderer.render(bp).unwrap();
            editor.end_frame();

            profiling::finish_frame!();
        }
        _ => {}
    });
}
