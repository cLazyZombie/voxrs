use legion::*;

use crate::{asset::{AssetManager, AssetPath}, blueprint::Blueprint, io::FileSystem, math::Vector3};

use super::{Clock, components::CameraComp, resources::{ElapsedTimeRes, KeyInput, WorldBlockRes}, systems::{camera, world_block_render}};

pub struct Game {
    world: World,
    res: Resources,
    tick_schedule: Schedule,
    render_schedule: Schedule,
    _camera: Entity,

    clock: Clock,
}

impl Game {
    pub fn new<F: FileSystem>(aspect: f32, asset_manager: &mut AssetManager<F>) -> Self {
        let mut world = World::default();
        let mut res = Resources::default();

        let world_block_res = WorldBlockRes::new(&AssetPath::from_str("assets/world_01.wb"), asset_manager);
        res.insert(world_block_res);

        let camera = CameraComp::new(
            Vector3::new(3.5, 3.5, -10.0),
            Vector3::new(0.5, 0.5, 10.0),
            Vector3::new(0.0, 1.0, 0.0),
            aspect,
            45.0,
            0.1,
            100.0,
        );

        let camera = world.push((camera,));

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
            _camera: camera,
            clock,
        }
    }

    pub fn set_input(&mut self, key_input: Option<KeyInput>) {
        let mut key_res = self.res.get_mut_or_default::<Option<KeyInput>>();
        *key_res = key_input;
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        let mut camera_qry = <&mut CameraComp>::query();
        for camera in camera_qry.iter_mut(&mut self.world) {
            camera.resize(width, height);
        }
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