use ash::vk;

#[derive(Clone, PartialEq, Eq)]
pub struct PushConstantRange {
    pub stage_flags: vk::ShaderStageFlags,
    pub offset: u32,
    pub size: u32,
}

impl Into<vk::PushConstantRange> for PushConstantRange {
    fn into(self) -> vk::PushConstantRange {
        vk::PushConstantRange {
            stage_flags: self.stage_flags,
            offset: self.offset,
            size: self.size,
        }
    }
}
