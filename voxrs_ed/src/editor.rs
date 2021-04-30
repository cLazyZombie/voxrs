use anyhow::Result;
use legion::*;
use voxrs_asset::{AssetManager, AssetPath, FontAsset};
use voxrs_core::res::{CameraRes, ElapsedTimeRes, KeyInputRes, MouseInputRes, WorldBlockRes};
use voxrs_math::*;
use voxrs_render::blueprint::Blueprint;
use voxrs_types::{io::FileSystem, Clock};
use voxrs_ui::{PanelInfo, TextInfo, WidgetRepository};
use winit::event::{ElementState, KeyboardInput, ModifiersState, MouseButton, VirtualKeyCode};

use crate::res::EditorAssetRes;

use super::system;

pub struct Editor {
    world: World,
    res: Resources,
    tick_schedule: Schedule,
    render_schedule: Schedule,
    end_frame_schedule: Schedule,
    clock: Clock,
}

impl Editor {
    pub fn new<F: FileSystem>(
        width: u32,
        height: u32,
        asset_manager: &mut AssetManager<F>,
    ) -> Self {
        let mut world = World::default();
        let mut resources = Resources::default();

        let world_block_res =
            WorldBlockRes::new(&AssetPath::from("assets/world_01.wb"), asset_manager);
        resources.insert(world_block_res);

        let camera = CameraRes::new(
            Vec3::new(0.0, 30.0, -30.0),
            Angle::from_degrees(0.0),
            Angle::from_degrees(-45.0),
            width,
            height,
            45.0,
            0.1,
            100.0,
        );
        resources.insert(camera);

        let widget_repository = WidgetRepository::new(&mut resources);
        // temp
        let panel1 = widget_repository.add_panel(
            PanelInfo {
                pos: (10.0, 10.0).into(),
                size: (200.0, 100.0).into(),
                color: (1.0, 0.0, 0.0, 1.0).into(),
            },
            None,
            &mut world,
            &mut resources,
        );

        let panel2 = widget_repository.add_panel(
            PanelInfo {
                pos: (15.0, 15.0).into(),
                size: (80.0, 50.0).into(),
                color: (0.0, 1.0, 1.0, 1.0).into(),
            },
            Some(panel1),
            &mut world,
            &mut resources,
        );

        let console_font =
            asset_manager.get::<FontAsset>(&AssetPath::from("assets/fonts/NanumBarunGothic.ttf"));
        let _text = widget_repository.add_text(
            TextInfo {
                pos: (10.0, 10.0).into(),
                size: (100.0, 50.0).into(),
                font: console_font,
                font_size: 24,
                contents: "text".to_string(),
            },
            Some(panel2),
            &mut world,
            &mut resources,
        );
        //let console_font =
        //    asset_manager.get::<FontAsset>(&AssetPath::from("assets/fonts/NanumBarunGothic.ttf"));
        // widget_repository
        //     .build()
        //     .panel(PanelWidgetInfo {
        //         pos: (10.0, 10.0).into(),
        //         size: (200.0, 100.0).into(),
        //         color: (1.0, 0.0, 0.0, 1.0).into(),
        //     })
        //     .child(|builder| {
        //         builder.panel(PanelWidgetInfo {
        //             pos: (5.0, 5.0).into(),
        //             size: (50.0, 50.0).into(),
        //             color: (0.0, 1.0, 0.0, 1.0).into(),
        //         })
        //     })
        //     .console(ConsoleWidgetInfo {
        //         pos: (0.0, 500.0).into(),
        //         size: (300.0, 100.0).into(),
        //         font: console_font,
        //     })
        //     .finish();

        resources.insert(widget_repository);

        let key_input = KeyInputRes::new();
        resources.insert(key_input);

        let mouse_input = MouseInputRes::new();
        resources.insert(mouse_input);

        let editor_asset = EditorAssetRes::new(asset_manager);
        resources.insert(editor_asset);

        let tick_schedule = Schedule::builder()
            .add_system(voxrs_ui::system::process_inputs_system())
            .add_system(system::camera::control_system())
            .add_system(system::world_block_modify::modify_system())
            .build();

        let render_schedule = Schedule::builder()
            .add_system(system::camera::render_system())
            .add_system(system::world_block_render::render_system())
            .add_system(system::world_block_modify::indicator_render_system())
            .add_system(voxrs_ui::system::render_system())
            .build();

        let end_frame_schedule = Schedule::builder()
            .add_system(system::end_frame::end_frame_system())
            .add_system(voxrs_ui::system::clear_inputs_system())
            .build();

        let clock = Clock::new();

        Self {
            world,
            res: resources,
            tick_schedule,
            render_schedule,
            end_frame_schedule,
            clock,
        }
    }

    pub fn on_key_input<F: FileSystem>(&mut self, input: &KeyboardInput) {
        let mut key_input = self.res.get_mut_or_default::<KeyInputRes>();

        if let Some(key_code) = input.virtual_keycode {
            if input.state == ElementState::Pressed {
                key_input.on_key_pressed(key_code);
            } else {
                key_input.on_key_released(key_code);

                // temporary
                if key_code == VirtualKeyCode::S && key_input.is_ctrl_pressed() {
                    drop(key_input);

                    let result = self.save::<F>(&"assets/world_temp.wb".into());
                    if let Err(error) = result {
                        log::error!("save error. {:?}", error);
                    }
                }
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

            // send to ui event
            let pos = IVec2::new(mouse_input.position.0 as i32, mouse_input.position.1 as i32);
            drop(mouse_input);

            let mut input_queue = self.res.get_mut_or_default::<voxrs_ui::InputQueue>();
            input_queue.add(voxrs_ui::input::WidgetInput::MouseClick { pos });
        }
    }

    pub fn on_cursor_moved(&mut self, pos: (f32, f32)) {
        let mut mouse_input = self.res.get_mut_or_default::<MouseInputRes>();
        mouse_input.on_mouse_pos(pos);
    }

    pub fn on_modifier_changed(&mut self, modifier: &ModifiersState) {
        let mut key_input = self.res.get_mut_or_default::<KeyInputRes>();
        key_input.on_modifier_changed(modifier);
    }

    pub fn on_receive_character(&mut self, c: char) {
        let mut input_queue = self.res.get_mut_or_default::<voxrs_ui::InputQueue>();
        input_queue.add(voxrs_ui::input::WidgetInput::Character(c));
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        let mut camera_res = self.res.get_mut::<CameraRes>().unwrap();
        camera_res.resize(width, height);
    }

    #[profiling::function]
    pub fn tick(&mut self) {
        let interval = self.clock.tick().as_secs_f32();

        // change time
        {
            let mut elapsed = self.res.get_mut_or_default::<ElapsedTimeRes>();
            *elapsed = interval.into();
        }

        self.tick_schedule.execute(&mut self.world, &mut self.res);
    }

    #[profiling::function]
    pub fn render(&mut self) -> Blueprint {
        self.res.insert(Blueprint::new());

        self.render_schedule.execute(&mut self.world, &mut self.res);

        self.res.remove::<Blueprint>().unwrap()
    }

    #[profiling::function]
    pub fn end_frame(&mut self) {
        self.end_frame_schedule
            .execute(&mut self.world, &mut self.res);
    }

    pub fn save<F: FileSystem>(&self, path: &AssetPath) -> Result<()> {
        let world_block = self.res.get::<WorldBlockRes>().unwrap();
        let raw_asset = world_block.make_raw_asset();
        let raw_asset_json = serde_json::to_string(&raw_asset)?;

        F::write_text(&path.to_path_buf(), &raw_asset_json)
    }
}
