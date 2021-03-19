use legion::*;
use voxrs_asset::{AssetManager, AssetPath};
use voxrs_core::res::{CameraRes, ElapsedTimeRes, KeyInputRes, MouseInputRes, WorldBlockRes};
use voxrs_math::Vector3;
use voxrs_render::blueprint::Blueprint;
use voxrs_types::{io::FileSystem, Clock};
use winit::event::{ElementState, KeyboardInput, MouseButton};

use super::system;

pub struct Editor {
    world: World,
    res: Resources,
    tick_schedule: Schedule,
    render_schedule: Schedule,
    clock: Clock,
}

impl Editor {
    pub fn new<F: FileSystem>(aspect: f32, asset_manager: &mut AssetManager<F>) -> Self {
        let world = World::default();
        let mut res = Resources::default();

        let world_block_res =
            WorldBlockRes::new(&AssetPath::from("assets/world_01.wb"), asset_manager);
        res.insert(world_block_res);

        let camera = CameraRes::new(
            Vector3::new(0.0, 50.0, -50.0),
            0.0,
            -std::f32::consts::FRAC_PI_4,
            aspect,
            45.0,
            0.1,
            100.0,
        );
        res.insert(camera);

        let key_input = KeyInputRes::new();
        res.insert(key_input);

        let mouse_input = MouseInputRes::new();
        res.insert(mouse_input);

        let tick_schedule = Schedule::builder()
            .add_system(system::camera::control_system())
            .build();

        let render_schedule = Schedule::builder()
            .add_system(system::camera::render_system())
            .add_system(system::world_block_render::render_system())
            .build();

        let clock = Clock::new();

        Self {
            world,
            res,
            tick_schedule,
            render_schedule,
            clock,
        }
    }

    pub fn on_key_input(&mut self, input: &KeyboardInput) {
        let mut key_input = self.res.get_mut_or_default::<KeyInputRes>();

        if let Some(key_code) = input.virtual_keycode {
            if input.state == ElementState::Pressed {
                key_input.on_key_pressed(key_code);
            } else {
                key_input.on_key_released(key_code);
            }
        }
    }

    pub fn on_mouse_motion(&mut self, delta: (f64, f64)) {
        let mut mouse_input = self.res.get_mut_or_default::<MouseInputRes>();
        mouse_input.on_mouse_motion(delta);
    }

    pub fn on_mouse_input(&mut self, button: MouseButton, state: ElementState) {
        let mut mouse_input = self.res.get_mut_or_default::<MouseInputRes>();
        if state == ElementState::Pressed {
            mouse_input.on_mouse_pressed(button);
        } else {
            mouse_input.on_mouse_released(button);
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        let mut camera_res = self.res.get_mut::<CameraRes>().unwrap();
        camera_res.resize(width, height);
    }

    pub fn tick(&mut self) {
        let interval = self.clock.tick().as_secs_f32();

        // change time
        {
            let mut elapsed = self.res.get_mut_or_default::<ElapsedTimeRes>();
            *elapsed = interval.into();
        }

        self.tick_schedule.execute(&mut self.world, &mut self.res);
    }

    pub fn render(&mut self) -> Blueprint {
        self.res.insert(Blueprint::new());

        self.render_schedule.execute(&mut self.world, &mut self.res);

        self.res.remove::<Blueprint>().unwrap()
    }
}
