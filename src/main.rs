use std::time::Instant;

use voxrs::{
    asset::AssetManager,
    blueprint::CHUNK_TOTAL_CUBE_COUNT,
    ecs::{game::Game, resources::KeyInput},
    io::GeneralFileSystem,
    math::Vector3,
    render,
    safecloner::SafeCloner,
};
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

fn main() {
    //env_logger::init();
    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .init();

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    let aspect = window.inner_size().width as f32 / window.inner_size().height as f32;
    let mut game = Game::new(aspect);
    let mut key_input: Option<KeyInput> = None;

    let asset_manager = AssetManager::<GeneralFileSystem>::new();
    let (sender, receiver) = crossbeam_channel::bounded(1);

    render::create_rendering_thread(receiver, &window, asset_manager.clone());

    let mut prev_tick: Option<Instant> = None;
    let mut resized: Option<(u32, u32)> = None;

    event_loop.run(move |event, _, control_flow| match event {
        Event::WindowEvent {
            ref event,
            window_id,
        } if window_id == window.id() => match event {
            WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
            WindowEvent::Resized(physical_size) => {
                resized = Some((physical_size.width, physical_size.height));
                let _ = sender.send(render::Command::Resize(*physical_size));
            }
            WindowEvent::KeyboardInput { input, .. } => {
                key_input = Some((*input).into());
            }
            _ => {}
        },
        Event::RedrawRequested(_) => {}
        Event::MainEventsCleared => {
            if let Some(resize) = resized {
                game.resize(resize.0, resize.1);
                resized = None;
            }

            game.set_input(key_input);
            key_input = None;

            // calc tick
            let elapsed_time;
            if let Some(prev) = prev_tick {
                let now = Instant::now();
                let interval = now - prev;
                elapsed_time = interval.as_secs_f32();

                prev_tick = Some(now);
            } else {
                elapsed_time = 0.0;
                prev_tick = Some(Instant::now());
            }

            game.tick(elapsed_time);

            let mut bp = game.render();

            //let mut bp = Blueprint::new(camera.clone());

            let cubes = (0..CHUNK_TOTAL_CUBE_COUNT).map(|v| (v % 3) as u8).collect();

            let chunk = voxrs::blueprint::Chunk::new(Vector3::new(0.0, 0.0, 0.0), cubes);
            let chunk = SafeCloner::new(chunk);

            bp.add_chunk(chunk.clone_read());

            if let Err(_) = sender.send(render::Command::Render(bp)) {
                *control_flow = ControlFlow::Exit;
            }

            //window.request_redraw();
        }
        _ => {}
    });
}
