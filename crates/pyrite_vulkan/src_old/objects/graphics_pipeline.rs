use crate::{
    DescriptorSetLayout, DescriptorSetLayoutDep, Image, ImageDep, Shader, Vulkan, VulkanDep,
};
use ash::vk;
use pyrite_util::Dependable;
use std::{collections::HashMap, ops::Deref, sync::Arc};

pub type GraphicsPipelineDep = Arc<GraphicsPipelineInner>;
pub struct GraphicsPipeline {
    inner: Arc<GraphicsPipelineInner>,
}

impl GraphicsPipeline {
    pub fn new(vulkan: &Vulkan, info: GraphicsPipelineInfo) -> Self {
        Self {
            inner: Arc::new(GraphicsPipelineInner::new(vulkan, info)),
        }
    }

    pub fn create_dep(&self) -> GraphicsPipelineDep {
        self.inner.clone()
    }
}

impl Deref for GraphicsPipeline {
    type Target = GraphicsPipelineInner;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

pub struct GraphicsPipelineInner {
    vulkan_dep: VulkanDep,
    render_pass: RenderPass,
    pipeline_layout: vk::PipelineLayout,
    pipeline: vk::Pipeline,
}

pub struct GraphicsPipelineInfo<'a> {
    vertex_shader: Shader,
    fragment_shader: Shader,
    vertex_input_state: vk::PipelineVertexInputStateCreateInfo<'a>,
    input_assembly_state: vk::PipelineInputAssemblyStateCreateInfo<'a>,
    viewport_state: vk::PipelineViewportStateCreateInfo<'a>,
    rasterization_state: vk::PipelineRasterizationStateCreateInfo<'a>,
    multisample_state: vk::PipelineMultisampleStateCreateInfo<'a>,
    depth_stencil_state: vk::PipelineDepthStencilStateCreateInfo<'a>,
    color_blend_state: vk::PipelineColorBlendStateCreateInfo<'a>,
    dynamic_state: vk::PipelineDynamicStateCreateInfo<'a>,
    render_pass: RenderPass,
    descriptor_set_layouts: Vec<DescriptorSetLayoutDep>,
    push_constant_ranges: Vec<vk::PushConstantRange>,
}

impl<'a> GraphicsPipelineInfo<'a> {
    pub fn builder() -> GraphicsPipelineInfoBuilder<'a> {
        GraphicsPipelineInfoBuilder::default()
    }
}

pub struct GraphicsPipelineInfoBuilder<'a> {
    vertex_shader: Option<Shader>,
    fragment_shader: Option<Shader>,
    vertex_input_state: vk::PipelineVertexInputStateCreateInfo<'a>,
    input_assembly_state: vk::PipelineInputAssemblyStateCreateInfo<'a>,
    viewport_state: vk::PipelineViewportStateCreateInfo<'a>,
    rasterization_state: vk::PipelineRasterizationStateCreateInfo<'a>,
    multisample_state: vk::PipelineMultisampleStateCreateInfo<'a>,
    depth_stencil_state: vk::PipelineDepthStencilStateCreateInfo<'a>,
    color_blend_state: vk::PipelineColorBlendStateCreateInfo<'a>,
    dynamic_state: vk::PipelineDynamicStateCreateInfo<'a>,
    render_pass: Option<RenderPass>,
    descriptor_set_layouts: Vec<DescriptorSetLayoutDep>,
    push_constant_ranges: Vec<vk::PushConstantRange>,
}

impl Default for GraphicsPipelineInfoBuilder<'_> {
    fn default() -> Self {
        Self {
            vertex_shader: None,
            fragment_shader: None,
            vertex_input_state: vk::PipelineVertexInputStateCreateInfo::default()
                .vertex_attribute_descriptions(&[])
                .vertex_binding_descriptions(&[]),
            input_assembly_state: vk::PipelineInputAssemblyStateCreateInfo::default()
                .topology(vk::PrimitiveTopology::TRIANGLE_LIST),
            viewport_state: vk::PipelineViewportStateCreateInfo::default(),
            rasterization_state: vk::PipelineRasterizationStateCreateInfo::default()
                .cull_mode(vk::CullModeFlags::NONE)
                .line_width(1.0)
                .polygon_mode(vk::PolygonMode::FILL)
                .depth_clamp_enable(false)
                .rasterizer_discard_enable(false)
                .front_face(vk::FrontFace::COUNTER_CLOCKWISE),
            multisample_state: vk::PipelineMultisampleStateCreateInfo::default()
                .rasterization_samples(vk::SampleCountFlags::TYPE_1),
            depth_stencil_state: vk::PipelineDepthStencilStateCreateInfo::default(),
            color_blend_state: vk::PipelineColorBlendStateCreateInfo::default(),
            dynamic_state: vk::PipelineDynamicStateCreateInfo::default(),
            render_pass: None,
            descriptor_set_layouts: Vec::new(),
            push_constant_ranges: Vec::new(),
        }
    }
}

impl<'a> GraphicsPipelineInfoBuilder<'a> {
    pub fn vertex_shader(mut self, vertex_shader: Shader) -> Self {
        self.vertex_shader = Some(vertex_shader);
        self
    }

    pub fn fragment_shader(mut self, fragment_shader: Shader) -> Self {
        self.fragment_shader = Some(fragment_shader);
        self
    }

    pub fn vertex_input_state(
        mut self,
        vertex_input_state: vk::PipelineVertexInputStateCreateInfo<'a>,
    ) -> Self {
        self.vertex_input_state = vertex_input_state;
        self
    }

    pub fn input_assembly_state(
        mut self,
        input_assembly_state: vk::PipelineInputAssemblyStateCreateInfo<'a>,
    ) -> Self {
        self.input_assembly_state = input_assembly_state;
        self
    }

    pub fn viewport_state(
        mut self,
        viewport_state: vk::PipelineViewportStateCreateInfo<'a>,
    ) -> Self {
        self.viewport_state = viewport_state;
        self
    }

    pub fn rasterization_state(
        mut self,
        rasterization_state: vk::PipelineRasterizationStateCreateInfo<'a>,
    ) -> Self {
        self.rasterization_state = rasterization_state;
        self
    }

    pub fn multisample_state(
        mut self,
        multisample_state: vk::PipelineMultisampleStateCreateInfo<'a>,
    ) -> Self {
        self.multisample_state = multisample_state;
        self
    }

    pub fn depth_stencil_state(
        mut self,
        depth_stencil_state: vk::PipelineDepthStencilStateCreateInfo<'a>,
    ) -> Self {
        self.depth_stencil_state = depth_stencil_state;
        self
    }

    pub fn color_blend_state(
        mut self,
        color_blend_state: vk::PipelineColorBlendStateCreateInfo<'a>,
    ) -> Self {
        self.color_blend_state = color_blend_state;
        self
    }

    pub fn dynamic_state(mut self, dynamic_state: vk::PipelineDynamicStateCreateInfo<'a>) -> Self {
        self.dynamic_state = dynamic_state;
        self
    }

    pub fn render_pass(mut self, render_pass: RenderPass) -> Self {
        self.render_pass = Some(render_pass);
        self
    }

    pub fn descriptor_set_layout(mut self, descriptor_set_layout: &DescriptorSetLayout) -> Self {
        self.descriptor_set_layouts
            .push(descriptor_set_layout.create_dep());
        self
    }

    pub fn descriptor_set_layouts(
        mut self,
        descriptor_set_layouts: Vec<&DescriptorSetLayout>,
    ) -> Self {
        self.descriptor_set_layouts = descriptor_set_layouts
            .into_iter()
            .map(|layout| layout.create_dep())
            .collect();
        self
    }

    pub fn push_constant_ranges(
        mut self,
        push_constant_ranges: Vec<vk::PushConstantRange>,
    ) -> Self {
        self.push_constant_ranges = push_constant_ranges;
        self
    }

    pub fn build(self) -> GraphicsPipelineInfo<'a> {
        GraphicsPipelineInfo {
            vertex_shader: self.vertex_shader.unwrap(),
            fragment_shader: self.fragment_shader.unwrap(),
            vertex_input_state: self.vertex_input_state,
            input_assembly_state: self.input_assembly_state,
            viewport_state: self.viewport_state,
            rasterization_state: self.rasterization_state,
            multisample_state: self.multisample_state,
            depth_stencil_state: self.depth_stencil_state,
            color_blend_state: self.color_blend_state,
            dynamic_state: self.dynamic_state,
            render_pass: self.render_pass.unwrap(),
            descriptor_set_layouts: self.descriptor_set_layouts,
            push_constant_ranges: self.push_constant_ranges,
        }
    }
}

impl Drop for GraphicsPipelineInner {
    fn drop(&mut self) {
        unsafe {
            self.vulkan_dep
                .device()
                .destroy_pipeline(self.pipeline, None);
            self.vulkan_dep
                .device()
                .destroy_pipeline_layout(self.pipeline_layout, None);
        }
    }
}

impl GraphicsPipelineInner {
    pub fn new(vulkan: &Vulkan, info: GraphicsPipelineInfo) -> Self {
        let shader_main_c_str = std::ffi::CString::new("main").unwrap();
        let shader_stages = [
            vk::PipelineShaderStageCreateInfo::default()
                .stage(vk::ShaderStageFlags::VERTEX)
                .module(info.vertex_shader.module())
                .name(shader_main_c_str.as_c_str()),
            vk::PipelineShaderStageCreateInfo::default()
                .stage(vk::ShaderStageFlags::FRAGMENT)
                .module(info.fragment_shader.module())
                .name(shader_main_c_str.as_c_str()),
        ];

        let vertex_input_state = info.vertex_input_state;
        let input_assembly_state = info.input_assembly_state;
        let viewport_state = info.viewport_state;
        let rasterization_state = info.rasterization_state;
        let multisample_state = info.multisample_state;
        let depth_stencil_state = info.depth_stencil_state;
        let color_blend_state = info.color_blend_state;
        let dynamic_state = info.dynamic_state;
        let render_pass = info.render_pass;

        let descriptor_set_layouts = info
            .descriptor_set_layouts
            .iter()
            .map(|layout| layout.descriptor_set_layout())
            .collect::<Vec<_>>();
        let push_constant_ranges = info.push_constant_ranges;
        let pipeline_layout_create_info = vk::PipelineLayoutCreateInfo::default()
            .set_layouts(&descriptor_set_layouts)
            .push_constant_ranges(&push_constant_ranges);

        // Safety: The pipeline layout is dropped when the internal pipeline is dropped
        let pipeline_layout = unsafe {
            vulkan
                .device()
                .create_pipeline_layout(&pipeline_layout_create_info, None)
                .unwrap()
        };

        let graphics_pipeline_create_info = vk::GraphicsPipelineCreateInfo::default()
            .stages(&shader_stages)
            .vertex_input_state(&vertex_input_state)
            .input_assembly_state(&input_assembly_state)
            .viewport_state(&viewport_state)
            .rasterization_state(&rasterization_state)
            .multisample_state(&multisample_state)
            .depth_stencil_state(&depth_stencil_state)
            .color_blend_state(&color_blend_state)
            .dynamic_state(&dynamic_state)
            .layout(pipeline_layout)
            .render_pass(render_pass.internal.render_pass())
            .subpass(0);

        // Safety: The pipeline is dropped when the internal pipeline is dropped
        let pipeline = unsafe {
            vulkan
                .device()
                .create_graphics_pipelines(
                    vk::PipelineCache::null(),
                    &[graphics_pipeline_create_info],
                    None,
                )
                .unwrap()[0]
        };

        Self {
            vulkan_dep: vulkan.create_dep(),
            render_pass,
            pipeline_layout,
            pipeline,
        }
    }

    pub fn render_pass(&self) -> &RenderPass {
        &self.render_pass
    }

    pub fn pipeline_layout(&self) -> vk::PipelineLayout {
        self.pipeline_layout
    }

    pub fn pipeline(&self) -> vk::Pipeline {
        self.pipeline
    }
}

pub struct RenderPass {
    internal: Arc<InternalRenderPass>,
}

pub struct InternalRenderPass {
    vulkan_dep: VulkanDep,
    render_pass: vk::RenderPass,
    framebuffer: vk::Framebuffer,
}

impl RenderPass {
    pub fn new(vulkan: &Vulkan, subpasses: &[Subpass]) -> Self {
        let mut attachment_index = 0u32;
        let attachments = subpasses
            .iter()
            .flat_map(|subpass| {
                // Map from unique images to attachments
                subpass
                    .color_attachments
                    .iter()
                    .chain(&subpass.resolve_attachments)
                    .chain(&subpass.depth_attachment)
                    .map(|attachment_reference| {
                        let attachment = attachment_reference.attachment.clone();
                        let image = attachment.image_dep.image();
                        let i = attachment_index;
                        attachment_index += 1;
                        (image, (i, attachment))
                    })
                    .collect::<HashMap<_, _>>()
            })
            .collect::<HashMap<vk::Image, (u32, Attachment)>>();

        let attachment_indices = attachments
            .iter()
            .map(|(image, (index, _))| (image.clone(), index.clone()))
            .collect::<HashMap<vk::Image, u32>>();

        let subpass_attachments_references = subpasses
            .iter()
            .map(|subpass| {
                let color_attachments = subpass
                    .color_attachments
                    .iter()
                    .map(|attachment_reference| {
                        vk::AttachmentReference::default()
                            .attachment(
                                attachment_indices
                                    [&attachment_reference.attachment.image_dep.image()],
                            )
                            .layout(attachment_reference.layout)
                    })
                    .collect::<Vec<_>>();
                let resolve_attachments = subpass
                    .resolve_attachments
                    .iter()
                    .map(|attachment_reference| {
                        vk::AttachmentReference::default()
                            .attachment(
                                attachment_indices
                                    [&attachment_reference.attachment.image_dep.image()],
                            )
                            .layout(attachment_reference.layout)
                    })
                    .collect::<Vec<_>>();
                let depth_attachment =
                    subpass
                        .depth_attachment
                        .as_ref()
                        .map(|attachment_reference| {
                            vk::AttachmentReference::default()
                                .attachment(
                                    attachment_indices
                                        [&attachment_reference.attachment.image_dep.image()],
                                )
                                .layout(attachment_reference.layout)
                        });

                (color_attachments, resolve_attachments, depth_attachment)
            })
            .collect::<Vec<_>>();

        let subpass_descriptions = subpass_attachments_references
            .iter()
            .map(
                |(color_attachments, resolve_attachments, depth_attachment)| {
                    let mut subpass_description = vk::SubpassDescription::default()
                        .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
                        .color_attachments(color_attachments)
                        .resolve_attachments(resolve_attachments);

                    if let Some(depth_attachment) = depth_attachment {
                        subpass_description =
                            subpass_description.depth_stencil_attachment(depth_attachment)
                    }

                    subpass_description
                },
            )
            .collect::<Vec<_>>();

        let mut attachment_descriptions = attachments
            .iter()
            .map(|(_, (index, attachment))| {
                (
                    index,
                    vk::AttachmentDescription::default()
                        .format(attachment.image_dep.image_format())
                        .samples(attachment.info.samples)
                        .load_op(attachment.info.load_op)
                        .store_op(attachment.info.store_op)
                        .stencil_load_op(attachment.info.load_op)
                        .stencil_store_op(attachment.info.store_op)
                        .initial_layout(attachment.info.initial_layout)
                        .final_layout(attachment.info.final_layout),
                )
            })
            .collect::<Vec<_>>();

        attachment_descriptions.sort_by(|(a_index, _), (b_index, _)| a_index.cmp(b_index));
        let attachment_descriptions = attachment_descriptions
            .into_iter()
            .map(|(_, attachment_description)| attachment_description)
            .collect::<Vec<_>>();

        let subpass_dependencies = [
            vk::SubpassDependency::default()
                .src_subpass(vk::SUBPASS_EXTERNAL)
                .dst_subpass(0)
                .src_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
                .dst_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
                .src_access_mask(vk::AccessFlags::empty())
                .dst_access_mask(
                    vk::AccessFlags::COLOR_ATTACHMENT_READ
                        | vk::AccessFlags::COLOR_ATTACHMENT_WRITE,
                ),
            vk::SubpassDependency::default()
                .src_subpass(vk::SUBPASS_EXTERNAL)
                .dst_subpass(0)
                .src_stage_mask(vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS)
                .dst_stage_mask(vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS)
                .src_access_mask(vk::AccessFlags::empty())
                .dst_access_mask(
                    vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_READ
                        | vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE,
                ),
        ];

        let render_pass_create_info = vk::RenderPassCreateInfo::default()
            .attachments(&attachment_descriptions)
            .subpasses(&subpass_descriptions)
            .dependencies(&subpass_dependencies);

        // Safety: The render pass is dropped when the internal render pass is dropped
        let render_pass = unsafe {
            vulkan
                .device()
                .create_render_pass(&render_pass_create_info, None)
                .unwrap()
        };

        let mut attachment_image_views = attachments
            .iter()
            .map(|(_, (index, attachment))| (index, attachment.image_dep.image_view()))
            .collect::<Vec<_>>();

        // Sort by index
        attachment_image_views.sort_by(|(a_index, _), (b_index, _)| a_index.cmp(b_index));
        let attachment_image_views = attachment_image_views
            .into_iter()
            .map(|(_, image_view)| image_view)
            .collect::<Vec<_>>();

        let (width, height) = attachments
            .iter()
            .map(|(_, (_, attachment))| {
                (
                    attachment.image_dep.image_extent().width,
                    attachment.image_dep.image_extent().height,
                )
            })
            .fold(
                (0, 0),
                |(width, height), (attachment_width, attachment_height)| {
                    (width.max(attachment_width), height.max(attachment_height))
                },
            );

        let framebuffer_create_info = vk::FramebufferCreateInfo::default()
            .render_pass(render_pass)
            .attachments(&attachment_image_views)
            .width(width)
            .height(height)
            .layers(1);

        // Safety: The framebuffer is dropped when the internal render pass is dropped
        let framebuffer = unsafe {
            vulkan
                .device()
                .create_framebuffer(&framebuffer_create_info, None)
                .unwrap()
        };

        Self {
            internal: Arc::new(InternalRenderPass {
                vulkan_dep: vulkan.create_dep(),
                render_pass,
                framebuffer,
            }),
        }
    }
}

impl InternalRenderPass {
    pub fn render_pass(&self) -> vk::RenderPass {
        self.render_pass
    }
    pub fn framebuffer(&self) -> vk::Framebuffer {
        self.framebuffer
    }
}

impl Deref for RenderPass {
    type Target = InternalRenderPass;

    fn deref(&self) -> &Self::Target {
        &self.internal
    }
}

impl Drop for InternalRenderPass {
    fn drop(&mut self) {
        unsafe {
            self.vulkan_dep
                .device()
                .destroy_framebuffer(self.framebuffer, None);
            self.vulkan_dep
                .device()
                .destroy_render_pass(self.render_pass, None);
        }
    }
}

pub struct Subpass {
    pub color_attachments: Vec<AttachmentReference>,
    pub resolve_attachments: Vec<AttachmentReference>,
    pub depth_attachment: Option<AttachmentReference>,
    pub input_attachments: Vec<AttachmentReference>,
}

impl Subpass {
    pub fn new() -> Self {
        Self {
            color_attachments: Vec::new(),
            resolve_attachments: Vec::new(),
            depth_attachment: None,
            input_attachments: Vec::new(),
        }
    }

    pub fn color_attachment(&mut self, attachment: &Attachment) {
        self.color_attachments
            .push(attachment.reference(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL));
    }

    pub fn resolve_attachment(&mut self, attachment: &Attachment) {
        self.resolve_attachments
            .push(attachment.reference(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL));
    }

    pub fn depth_attachment(&mut self, attachment: &Attachment) {
        self.depth_attachment =
            Some(attachment.reference(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL));
    }

    pub fn input_attachment(&mut self, attachment: &Attachment) {
        self.input_attachments
            .push(attachment.reference(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL));
    }
}

#[derive(Copy, Clone)]
pub struct AttachmentInfo {
    samples: vk::SampleCountFlags,
    load_op: vk::AttachmentLoadOp,
    store_op: vk::AttachmentStoreOp,
    initial_layout: vk::ImageLayout,
    final_layout: vk::ImageLayout,
    is_depth: bool,
}

impl Default for AttachmentInfo {
    fn default() -> Self {
        Self {
            samples: vk::SampleCountFlags::TYPE_1,
            load_op: vk::AttachmentLoadOp::CLEAR,
            store_op: vk::AttachmentStoreOp::STORE,
            initial_layout: vk::ImageLayout::UNDEFINED,
            final_layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
            is_depth: false,
        }
    }
}

impl AttachmentInfo {
    pub fn samples(mut self, samples: vk::SampleCountFlags) -> Self {
        self.samples = samples;
        self
    }

    pub fn load_op(mut self, load_op: vk::AttachmentLoadOp) -> Self {
        self.load_op = load_op;
        self
    }

    pub fn store_op(mut self, store_op: vk::AttachmentStoreOp) -> Self {
        self.store_op = store_op;
        self
    }

    pub fn initial_layout(mut self, initial_layout: vk::ImageLayout) -> Self {
        self.initial_layout = initial_layout;
        self
    }

    pub fn final_layout(mut self, final_layout: vk::ImageLayout) -> Self {
        self.final_layout = final_layout;
        self
    }

    pub fn is_depth(mut self, is_depth: bool) -> Self {
        self.is_depth = is_depth;
        self
    }
}

#[derive(Clone)]
pub struct Attachment {
    image_dep: ImageDep,
    info: AttachmentInfo,
}

impl Attachment {
    pub fn new(image: &Image, info: AttachmentInfo) -> Self {
        Self {
            image_dep: image.create_dep(),
            info,
        }
    }

    pub fn reference(&self, layout: vk::ImageLayout) -> AttachmentReference {
        AttachmentReference {
            attachment: self.clone(),
            layout,
        }
    }
}

pub struct AttachmentReference {
    attachment: Attachment,
    pub layout: vk::ImageLayout,
}
