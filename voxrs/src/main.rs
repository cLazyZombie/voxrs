use voxrs::Game;
use voxrs_asset::AssetManager;
use voxrs_render::render;
use voxrs_types::io::GeneralFileSystem;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

fn main() {
    profiling::register_thread!("Main Thread");

    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .init();

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    let mut asset_manager = AssetManager::<GeneralFileSystem>::new();
    let (sender, receiver) = crossbeam_channel::bounded(1);

    render::create_rendering_thread(receiver, &window, asset_manager.clone());

    let mut game = Game::new(
        window.inner_size().width,
        window.inner_size().height,
        &mut asset_manager,
    );

    event_loop.run(move |event, _, control_flow| match event {
        Event::WindowEvent {
            ref event,
            window_id,
        } if window_id == window.id() => match event {
            WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
            WindowEvent::Resized(physical_size) => {
                game.resize(physical_size.width, physical_size.height);
                let _ = sender.send(render::Command::Resize(*physical_size));
            }
            WindowEvent::KeyboardInput { input, .. } => {
                game.on_key_input(input);
            }
            _ => {}
        },
        Event::RedrawRequested(_) => {}
        Event::MainEventsCleared => {
            game.tick();

            let bp = game.render();

            if sender.send(render::Command::Render(bp)).is_err() {
                *control_flow = ControlFlow::Exit;
            }
            //window.request_redraw();
            profiling::finish_frame!();
        }
        _ => {}
    });
}
