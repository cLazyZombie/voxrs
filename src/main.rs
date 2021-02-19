use voxrs::{asset::AssetManager, blueprint::{Blueprint, CHUNK_TOTAL_CUBE_COUNT}, camera::Camera, io::GeneralFileSystem, math::Vector3, readwrite::ReadWrite, render};
use winit::{
    event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

fn main() {
    //env_logger::init();
    env_logger::builder().filter_level(log::LevelFilter::Info).init();

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    let mut camera = Camera::new(
        Vector3::new(3.5, 3.5, -10.0),
        Vector3::new(0.5, 0.5, 10.0),
        Vector3::new(0.0, 1.0, 0.0),
        window.inner_size().width as f32 / window.inner_size().height as f32,
        45.0,
        0.1,
        100.0,
    );


    let asset_manager = AssetManager::<GeneralFileSystem>::new();
    let (sender, receiver) = crossbeam_channel::bounded(1);
    
    render::create_rendering_thread(receiver, &window, asset_manager.clone());

    event_loop.run(move |event, _, control_flow| match event {
        Event::WindowEvent {
            ref event,
            window_id,
        } if window_id == window.id() => match event {
            WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
            WindowEvent::Resized(physical_size) => {
                camera.resize(physical_size.width, physical_size.height);
                let _ = sender.send(render::Command::Resize(*physical_size));
            }
            WindowEvent::KeyboardInput {
                input:
                    KeyboardInput {
                        state,
                        virtual_keycode: Some(keycode),
                        ..
                    },
                ..
            } => {
                let is_pressed = *state == ElementState::Pressed;
                match *keycode {
                    VirtualKeyCode::W | VirtualKeyCode::Up if is_pressed => {
                        camera.move_camera(Vector3::new(0.0, 0.0, 0.1));
                    }
                    VirtualKeyCode::S | VirtualKeyCode::Down if is_pressed => {
                        camera.move_camera(Vector3::new(0.0, 0.0, -0.1));
                    }
                    VirtualKeyCode::A | VirtualKeyCode::Left if is_pressed => {
                        camera.move_camera(Vector3::new(-0.1, 0.0, 0.0));
                    }
                    VirtualKeyCode::D | VirtualKeyCode::Right if is_pressed => {
                        camera.move_camera(Vector3::new(0.1, 0.0, 0.0));
                    }

                    _ => {}
                }
            }
            _ => {}
        },
        Event::RedrawRequested(_) => {
            // renderer.update_camera();

            // match renderer.render() {
            //     Ok(_) => {}
            //     Err(wgpu::SwapChainError::Lost) => renderer.resize_self(),
            //     Err(wgpu::SwapChainError::OutOfMemory) => *control_flow = ControlFlow::Exit,
            //     Err(e) => eprintln!("{:?}", e),
            // };
        }
        Event::MainEventsCleared => {
            let mut bp = Blueprint::new(camera.clone());

            let cubes = (0..CHUNK_TOTAL_CUBE_COUNT).map(|v| (v % 3) as u8).collect();

            let chunk = voxrs::blueprint::Chunk::new(
                Vector3::new(0.0, 0.0, 0.0),
                cubes,
            );
            let chunk = ReadWrite::new(chunk);

            bp.add_chunk(chunk.clone_read());

            if let Err(_) = sender.send(render::Command::Render(bp)) {
                *control_flow = ControlFlow::Exit;
            }
            
            //window.request_redraw();
        }
        _ => {}
    });
}
