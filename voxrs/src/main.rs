use voxrs::{
    ecs::{game::Game, resources::KeyInputRes},
    render,
};
use voxrs_asset::AssetManager;
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
    let mut key_input: Option<KeyInputRes> = None;

    let mut asset_manager = AssetManager::<GeneralFileSystem>::new();
    let (sender, receiver) = crossbeam_channel::bounded(1);

    render::create_rendering_thread(receiver, &window, asset_manager.clone());

    let mut game = Game::new(aspect, &mut asset_manager);

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
                key_input = Some((*input).into());
            }
            _ => {}
        },
        Event::RedrawRequested(_) => {}
        Event::MainEventsCleared => {
            game.set_input(key_input);
            //key_input = None;

            game.tick();

            let bp = game.render();

            if let Err(_) = sender.send(render::Command::Render(bp)) {
                *control_flow = ControlFlow::Exit;
            }
            //window.request_redraw();
        }
        _ => {}
    });
}
