use legion::*;
use voxrs_render::blueprint::Blueprint;
use voxrs_ui::WidgetRepository;

#[system]
pub fn render(#[resource] widget_repository: &WidgetRepository, #[resource] bp: &mut Blueprint) {
    widget_repository.render(bp);
}
