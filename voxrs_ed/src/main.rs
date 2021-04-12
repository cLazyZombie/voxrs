use voxrs_asset::AssetManager;
use voxrs_ed::Editor;
use voxrs_render::render;
use voxrs_types::io::GeneralFileSystem;
use voxrs_ui::iced_winit::{conversion, futures, program, winit, Clipboard, Debug, Size};
use voxrs_ui::{iced_wgpu::Viewport, iced_winit};
use winit::{
    dpi::PhysicalPosition,
    event::{DeviceEvent, Event, ModifiersState, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

fn main() {
    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .init();

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    let mut asset_manager = AssetManager::<GeneralFileSystem>::new();

    let mut renderer =
        futures::executor::block_on(render::Renderer::new(&window, &mut asset_manager));

    // temp
    let physical_size = window.inner_size();
    let mut viewport = Viewport::with_physical_size(
        Size::new(physical_size.width, physical_size.height),
        window.scale_factor(),
    );
    let mut cursor_position = PhysicalPosition::new(-1.0, -1.0);
    let mut clipboard = Clipboard::connect(&window);
    let mut modifiers = ModifiersState::default();
    let mut local_pool = futures::executor::LocalPool::new();

    let controls = voxrs_ed::Controls::new();
    let mut debug = Debug::new();
    let mut state = program::State::new(
        controls,
        viewport.logical_size(),
        conversion::cursor_position(cursor_position, viewport.scale_factor()),
        renderer.get_iced_renderer(),
        &mut debug,
    );

    let mut editor = Editor::new(
        window.inner_size().width,
        window.inner_size().height,
        &mut asset_manager,
    );

    #[allow(clippy::single_match, clippy::clippy::collapsible_match)]
    event_loop.run(move |event, _, control_flow| match event {
        Event::WindowEvent {
            ref event,
            window_id,
        } if window_id == window.id() => {
            match event {
                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                WindowEvent::Resized(physical_size) => {
                    viewport = Viewport::with_physical_size(
                        Size::new(physical_size.width, physical_size.height),
                        window.scale_factor(),
                    );

                    editor.resize(physical_size.width, physical_size.height);
                    renderer.resize(*physical_size);
                }
                WindowEvent::KeyboardInput { input, .. } => {
                    editor.on_key_input(input);
                }
                WindowEvent::MouseInput { state, button, .. } => {
                    editor.on_mouse_input(*button, *state);
                }
                WindowEvent::CursorMoved { position, .. } => {
                    cursor_position = *position;
                    editor.on_cursor_moved((position.x as f32, position.y as f32));
                }
                WindowEvent::ModifiersChanged(modifier) => {
                    modifiers = *modifier;
                    editor.on_modifier_changed(modifier);
                }
                _ => {}
            }

            if let Some(event) =
                iced_winit::conversion::window_event(&event, window.scale_factor(), modifiers)
            {
                state.queue_event(event);
            }
        }
        Event::DeviceEvent { ref event, .. } => match event {
            DeviceEvent::MouseMotion { delta } => {
                editor.on_mouse_motion(*delta);
            }
            _ => {}
        },
        Event::RedrawRequested(_) => {}
        Event::MainEventsCleared => {
            // tick iced
            if !state.is_queue_empty() {
                let _ = state.update(
                    viewport.logical_size(),
                    conversion::cursor_position(cursor_position, viewport.scale_factor()),
                    renderer.get_iced_renderer(),
                    &mut clipboard,
                    &mut debug,
                );
            }

            // tick editor
            editor.tick();
            let mut bp = editor.render();
            bp.iced_primitive = Some(state.primitive().clone());
            renderer.render(bp, &mut local_pool).unwrap();
            editor.end_frame();
        }
        _ => {}
    });
}
