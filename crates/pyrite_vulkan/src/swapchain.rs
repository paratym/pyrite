use pyrite_app::resource::Resource;

#[derive(Resource)]
pub struct Swapchain {}

impl Swapchain {
    pub fn new() -> Self {
        Self {}
    }
}
