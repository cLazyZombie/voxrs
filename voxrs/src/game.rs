use legion::*;
use voxrs_asset::{AssetManager, AssetPath};
use voxrs_core::res::{CameraRes, ElapsedTimeRes, KeyInputRes, WorldBlockRes};
use voxrs_render::blueprint::Blueprint;
use voxrs_types::{io::FileSystem, Clock};

use voxrs_math::*;
use winit::event::{ElementState, KeyboardInput};

use super::system::{camera, world_block_render};

pub struct Game {
    world: World,
    res: Resources,
    tick_schedule: Schedule,
    render_schedule: Schedule,

    clock: Clock,
}

impl Game {
    pub fn new<F: FileSystem>(aspect: f32, asset_manager: &mut AssetManager<F>) -> Self {
        let world = World::default();
        let mut res = Resources::default();

        let world_block_res =
            WorldBlockRes::new(&AssetPath::from("assets/world_01.wb"), asset_manager);
        res.insert(world_block_res);

        let camera = CameraRes::new(
            Vector3::new(3.5, 3.5, -10.0),
            0.0,
            0.0,
            aspect,
            45.0,
            0.1,
            100.0,
        );
        res.insert(camera);

        let key_input = KeyInputRes::new();
        res.insert(key_input);

        let tick_schedule = Schedule::builder()
            .add_system(camera::camera_move_system())
            .build();

        let render_schedule = Schedule::builder()
            .add_system(camera::camera_render_system())
            .add_system(world_block_render::world_block_render_system())
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

    pub fn resize(&mut self, width: u32, height: u32) {
        let mut camera_res = self.res.get_mut::<CameraRes>().unwrap();
        camera_res.resize(width, height);
    }

    pub fn tick(&mut self) {
        let interval = self.clock.tick().as_secs_f32();

        // change res
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
