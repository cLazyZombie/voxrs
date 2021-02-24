use legion::{Entity, Resources, Schedule, World};

use crate::math::Vector3;

use super::{components::Position, resources::ElapsedTimeRes, systems::camera::{self, CameraComp}};


pub struct Game {
    world: World,
    res: Resources,
    schedule: Schedule,
    _camera: Entity,
}

impl Game {
    pub fn new() -> Self {
        let mut world = World::default();
        let mut res = Resources::default();

        // init camera
        let camera = world.push((
            CameraComp,
            Position::from(Vector3::new(3.5, 3.5, -10.0)),
        ));

        res.insert(ElapsedTimeRes::from(0.0));

        let schedule = Schedule::builder()
            .add_system(camera::camera_move_system())
            .build();

        Self {
            world,
            res,
            schedule,
            _camera: camera,
        }
    }

    pub fn tick(&mut self, elapsed_time: f32) {
        // change res
        {
            let mut elapsed = self.res.get_mut::<ElapsedTimeRes>().unwrap();
            *elapsed = elapsed_time.into();
        }

        self.schedule.execute(&mut self.world, &mut self.res);
    }
}