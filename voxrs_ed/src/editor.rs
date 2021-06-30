use anyhow::Result;
use legion::*;
use voxrs_asset::{AssetManager, AssetPath, FontAsset};
use voxrs_core::res::{CameraRes, ElapsedTimeRes, KeyInputRes, MouseInputRes, WorldBlockRes};
use voxrs_math::*;
use voxrs_render::blueprint::Blueprint;
use voxrs_types::{io::FileSystem, Clock};
use voxrs_ui::{
    AnchorHorizon, AnchorVertical, EditableTextInfo, PanelInfo, TerminalInfo, WidgetBuilder, WidgetPlacementInfo,
};
use winit::event::{ElementState, KeyboardInput, ModifiersState, MouseButton};

use crate::{command::Command, res::EditorAssetRes, system::shortcut::Shortcut, WidgetMessage};

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
    pub fn new<F: FileSystem>(width: u32, height: u32, mut asset_manager: AssetManager<F>) -> Self {
        let mut world = World::default();
        let mut resources = Resources::default();

        let world_block_res = WorldBlockRes::new(&AssetPath::from("assets/world_01.wb"), &mut asset_manager);
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

        voxrs_ui::init_resources::<WidgetMessage>(&mut resources, width, height);
        let mut terminal_id = None;
        let console_font = asset_manager.get::<FontAsset>(&AssetPath::from("assets/fonts/NanumBarunGothic.ttf"));
        let mut builder = WidgetBuilder::<WidgetMessage>::new(&mut world, &mut resources);
        builder
            .panel(PanelInfo {
                placement: WidgetPlacementInfo {
                    pos: (10, 10).into(),
                    v_anchor: Some(AnchorVertical::Top),
                    h_anchor: Some(AnchorHorizon::Left),
                    size: (200, 100).into(),
                },
                color: (1.0, 0.0, 0.0, 1.0).into(),
            })
            .child(|b| {
                b.panel(PanelInfo {
                    placement: WidgetPlacementInfo {
                        pos: (15, 15).into(),
                        v_anchor: Some(AnchorVertical::Top),
                        h_anchor: Some(AnchorHorizon::Left),
                        size: (80, 50).into(),
                    },
                    color: (0.0, 1.0, 1.0, 1.0).into(),
                })
                .child(|b| {
                    b.editable_text(EditableTextInfo {
                        placement: WidgetPlacementInfo {
                            pos: (10, 10).into(),
                            v_anchor: Some(AnchorVertical::Top),
                            h_anchor: Some(AnchorHorizon::Left),
                            size: (100, 50).into(),
                        },
                        font: console_font.clone(),
                        font_size: 24,
                        contents: "text".to_string(),
                    });
                });
            })
            .panel(PanelInfo {
                placement: WidgetPlacementInfo {
                    pos: (30, 30).into(),
                    v_anchor: Some(AnchorVertical::Top),
                    h_anchor: Some(AnchorHorizon::Left),
                    size: (200, 100).into(),
                },
                color: (0.0, 0.0, 1.0, 0.5).into(),
            })
            .terminal(TerminalInfo {
                placement: WidgetPlacementInfo {
                    pos: (0, 0).into(),
                    v_anchor: Some(AnchorVertical::Bottom),
                    h_anchor: Some(AnchorHorizon::Fill),
                    size: (0, 300).into(),
                },
                color: (0.0, 0.0, 0.0, 0.7).into(),
                font: console_font.clone(),
                font_size: 20,
                contents: vec!["hello, world".to_string(), "this is terminal".to_string()],
            })
            .handle_event(|_, interaction| match interaction {
                voxrs_ui::Interaction::TerminalInput(input) => {
                    let command = input.parse::<Command>();
                    if let Ok(command) = command {
                        Some(WidgetMessage::ConsoleCommand(command))
                    } else {
                        None
                    }
                }
                _ => None,
            })
            .query_id(&mut terminal_id);

        let terminal_id = terminal_id.unwrap();

        let key_input = KeyInputRes::new();
        resources.insert(key_input);

        let mouse_input = MouseInputRes::new();
        resources.insert(mouse_input);

        let editor_asset = EditorAssetRes::new(&mut asset_manager);
        resources.insert(editor_asset);

        resources.insert(asset_manager);

        let shortcut = Shortcut::new(terminal_id, true);

        let tick_schedule = Schedule::builder()
            .add_system(system::shortcut::process_shortcut_system(shortcut))
            .add_system(voxrs_ui::system::process_inputs_system::<WidgetMessage>())
            .add_system(system::disable_input::disable_input_system())
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
            .add_system(system::process_widget_message::process_widget_message_system::<F>())
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
        if let Some(key_code) = input.virtual_keycode {
            if input.state == ElementState::Pressed {
                // set key input for editor
                {
                    let mut key_input = self.res.get_mut_or_default::<KeyInputRes>();
                    key_input.on_key_pressed(key_code);
                }

                // send key input to ui
                {
                    let mut input_queue = self.res.get_mut_or_default::<voxrs_ui::InputQueue>();
                    input_queue.add(voxrs_ui::input::WidgetInput::KeyboardInput(
                        voxrs_ui::input::KeyboardInput::new(key_code),
                    ));
                }
            } else {
                let mut key_input = self.res.get_mut_or_default::<KeyInputRes>();
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

            // send to ui event
            let pos = IVec2::new(mouse_input.get_position().0 as i32, mouse_input.get_position().1 as i32);
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
        if let Some(mut camera_res) = self.res.get_mut::<CameraRes>() {
            camera_res.resize(width, height);
        }

        if let Some(mut screen_res) = self.res.get_mut::<voxrs_ui::ScreenResolution>() {
            screen_res.width = width;
            screen_res.height = height;
        }
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
        self.end_frame_schedule.execute(&mut self.world, &mut self.res);
    }

    pub fn save<F: FileSystem>(&self, path: &AssetPath) -> Result<()> {
        let world_block = self.res.get::<WorldBlockRes>().unwrap();
        let raw_asset = world_block.make_raw_asset();
        let raw_asset_json = serde_json::to_string(&raw_asset)?;

        F::write_text(&path.to_path_buf(), &raw_asset_json)
    }
}
