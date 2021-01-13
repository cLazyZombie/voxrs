use std::path::PathBuf;

use voxrs::{asset::{AssetPath, TextureAsset}, blueprint::Blueprint, camera::Camera, math::Vector3, renderer::Renderer};
use winit::{
    event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

fn main() {
    env_logger::init();
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

    let mut renderer = futures::executor::block_on(Renderer::new(&window));

    let mut asset_manager = voxrs::asset::AssetManager::<voxrs::io::GeneralFileSystem>::new();
    let texture_handle = asset_manager.get::<TextureAsset>(&AssetPath::new(PathBuf::from("assets/texture.png"))).unwrap();

    event_loop.run(move |event, _, control_flow| match event {
        Event::WindowEvent {
            ref event,
            window_id,
        } if window_id == window.id() => match event {
            WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
            WindowEvent::Resized(physical_size) => {
                camera.resize(physical_size.width, physical_size.height);
                renderer.resize(*physical_size);
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
            bp.add_cube(voxrs::blueprint::Cube::new(
                Vector3::new(0.0, 0.0, 0.0),
                texture_handle.clone(),
            ));
            bp.add_cube(voxrs::blueprint::Cube::new(
                Vector3::new(0.0, 1.0, 0.0),
                texture_handle.clone(),
            ));
            bp.add_cube(voxrs::blueprint::Cube::new(
                Vector3::new(0.0, 2.0, 0.0),
                texture_handle.clone(),
            ));

            match renderer.render(bp, &mut asset_manager) {
                Ok(_) => {}
                Err(wgpu::SwapChainError::Lost) => renderer.resize_self(),
                Err(wgpu::SwapChainError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                Err(e) => eprintln!("{:?}", e),
            }
            
            //window.request_redraw();
        }
        _ => {}
    });
}
