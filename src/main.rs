use voxrs::renderer::Renderer;
use winit::{event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent}, event_loop::{ControlFlow, EventLoop}, window::WindowBuilder};

extern crate nalgebra_glm as glm;

fn main() {
    env_logger::init();
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    let mut renderer = futures::executor::block_on(Renderer::new(&window));

    event_loop.run(move |event, _, control_flow| match event {
        Event::WindowEvent {
            ref event,
            window_id,
        } if window_id == window.id() => match event {
            WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
            WindowEvent::Resized(physical_size) => {
                renderer.resize(*physical_size);
            },
            WindowEvent::KeyboardInput {
                input: KeyboardInput {
                    state,
                    virtual_keycode: Some(keycode),
                    ..
                },
                ..
            } => {
                let is_pressed = *state == ElementState::Pressed;
                match *keycode {
                    VirtualKeyCode::W | VirtualKeyCode::Up if is_pressed => {
                        renderer.camera.move_camera(glm::vec3(0.0, 0.0, 0.1));
                    }
                    VirtualKeyCode::S | VirtualKeyCode::Down if is_pressed => {
                        renderer.camera.move_camera(glm::vec3(0.0, 0.0, -0.1));
                    }
                    _ => {}
                }
            },
            _ => {}
        },
        Event::RedrawRequested(_) => {
            renderer.update_camera();
            
            match renderer.render() {
                Ok(_) => {}
                Err(wgpu::SwapChainError::Lost) => renderer.resize_self(),
                Err(wgpu::SwapChainError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                Err(e) => eprintln!("{:?}", e),
            };
        },
        Event::MainEventsCleared => {
            window.request_redraw();
        },
        _ => {}
    });
}
