use legion::*;
use voxrs_asset::{AssetManager, AssetPath};
use voxrs_core::res::{CameraRes, ElapsedTimeRes, KeyInputRes, WorldBlockRes};
use voxrs_math::Vector3;
use voxrs_render::blueprint::Blueprint;
use voxrs_types::{io::FileSystem, Clock};

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
            Vector3::get_normalized(&Vector3::new(0.0, -1.0, 1.0)),
            Vector3::get_normalized(&Vector3::new(0.0, 1.0, 1.0)),
            aspect,
            45.0,
            0.1,
            100.0,
        );
        res.insert(camera);

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

    pub fn set_input(&mut self, key_input: Option<KeyInputRes>) {
        let mut key_res = self.res.get_mut_or_default::<Option<KeyInputRes>>();
        *key_res = key_input;
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