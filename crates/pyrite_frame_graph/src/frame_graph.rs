use pyrite_app::resource::Resource;

#[derive(Resource)]
pub struct FrameGraph {
    nodes: Vec<Node>,
    images: Vec<ImageRef>,
}

pub struct FrameGraphExecutor {}
