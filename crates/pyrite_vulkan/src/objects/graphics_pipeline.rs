use crate::{DescriptorSetLayout, Image, ImageDep, Shader, Vulkan, VulkanDep};
use ash::{
    vk,
    vk::{AttachmentDescription, Handle},
};
use pyrite_util::Dependable;
use std::{collections::HashMap, ops::Deref, sync::Arc};

pub struct GraphicsPipeline {
    vulkan_dep: VulkanDep,
    render_pass: RenderPass,
    pipeline_layout: vk::PipelineLayout,
    pipeline: vk::Pipeline,
}

pub struct GraphicsPipelineInfo {
    vertex_shader: Shader,
    fragment_shader: Shader,
    vertex_input_state: vk::PipelineVertexInputStateCreateInfo,
    input_assembly_state: vk::PipelineInputAssemblyStateCreateInfo,
    viewport_state: vk::PipelineViewportStateCreateInfo,
    rasterization_state: vk::PipelineRasterizationStateCreateInfo,
    multisample_state: vk::PipelineMultisampleStateCreateInfo,
    depth_stencil_state: vk::PipelineDepthStencilStateCreateInfo,
    color_blend_state: vk::PipelineColorBlendStateCreateInfo,
    dynamic_state: vk::PipelineDynamicStateCreateInfo,
    render_pass: RenderPass,
    descriptor_set_layouts: Vec<DescriptorSetLayout>,
}

impl GraphicsPipelineInfo {
    pub fn builder() -> GraphicsPipelineInfoBuilder {
        GraphicsPipelineInfoBuilder::default()
    }
}

pub struct GraphicsPipelineInfoBuilder {
    vertex_shader: Option<Shader>,
    fragment_shader: Option<Shader>,
    vertex_input_state: vk::PipelineVertexInputStateCreateInfo,
    input_assembly_state: vk::PipelineInputAssemblyStateCreateInfo,
    viewport_state: vk::PipelineViewportStateCreateInfo,
    rasterization_state: vk::PipelineRasterizationStateCreateInfo,
    multisample_state: vk::PipelineMultisampleStateCreateInfo,
    depth_stencil_state: vk::PipelineDepthStencilStateCreateInfo,
    color_blend_state: vk::PipelineColorBlendStateCreateInfo,
    dynamic_state: vk::PipelineDynamicStateCreateInfo,
    render_pass: Option<RenderPass>,
    descriptor_set_layouts: Vec<DescriptorSetLayout>,
}

impl Default for GraphicsPipelineInfoBuilder {
    fn default() -> Self {
        Self {
            vertex_shader: None,
            fragment_shader: None,
            vertex_input_state: vk::PipelineVertexInputStateCreateInfo::builder()
                .vertex_attribute_descriptions(&[])
                .vertex_binding_descriptions(&[])
                .build(),
            input_assembly_state: vk::PipelineInputAssemblyStateCreateInfo::builder()
                .topology(vk::PrimitiveTopology::TRIANGLE_LIST)
                .build(),
            viewport_state: vk::PipelineViewportStateCreateInfo::default(),
            rasterization_state: vk::PipelineRasterizationStateCreateInfo::builder()
                .cull_mode(vk::CullModeFlags::BACK)
                .line_width(1.0)
                .polygon_mode(vk::PolygonMode::FILL)
                .depth_clamp_enable(false)
                .rasterizer_discard_enable(false)
                .front_face(vk::FrontFace::COUNTER_CLOCKWISE)
                .build(),
            multisample_state: vk::PipelineMultisampleStateCreateInfo::builder()
                .rasterization_samples(vk::SampleCountFlags::TYPE_1)
                .build(),
            depth_stencil_state: vk::PipelineDepthStencilStateCreateInfo::default(),
            color_blend_state: vk::PipelineColorBlendStateCreateInfo::default(),
            dynamic_state: vk::PipelineDynamicStateCreateInfo::default(),
            render_pass: None,
            descriptor_set_layouts: Vec::new(),
        }
    }
}

impl GraphicsPipelineInfoBuilder {
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
        vertex_input_state: vk::PipelineVertexInputStateCreateInfo,
    ) -> Self {
        self.vertex_input_state = vertex_input_state;
        self
    }

    pub fn input_assembly_state(
        mut self,
        input_assembly_state: vk::PipelineInputAssemblyStateCreateInfo,
    ) -> Self {
        self.input_assembly_state = input_assembly_state;
        self
    }

    pub fn viewport_state(mut self, viewport_state: vk::PipelineViewportStateCreateInfo) -> Self {
        self.viewport_state = viewport_state;
        self
    }

    pub fn rasterization_state(
        mut self,
        rasterization_state: vk::PipelineRasterizationStateCreateInfo,
    ) -> Self {
        self.rasterization_state = rasterization_state;
        self
    }

    pub fn multisample_state(
        mut self,
        multisample_state: vk::PipelineMultisampleStateCreateInfo,
    ) -> Self {
        self.multisample_state = multisample_state;
        self
    }

    pub fn depth_stencil_state(
        mut self,
        depth_stencil_state: vk::PipelineDepthStencilStateCreateInfo,
    ) -> Self {
        self.depth_stencil_state = depth_stencil_state;
        self
    }

    pub fn color_blend_state(
        mut self,
        color_blend_state: vk::PipelineColorBlendStateCreateInfo,
    ) -> Self {
        self.color_blend_state = color_blend_state;
        self
    }

    pub fn dynamic_state(mut self, dynamic_state: vk::PipelineDynamicStateCreateInfo) -> Self {
        self.dynamic_state = dynamic_state;
        self
    }

    pub fn render_pass(mut self, render_pass: RenderPass) -> Self {
        self.render_pass = Some(render_pass);
        self
    }

    pub fn descriptor_set_layout(mut self, descriptor_set_layout: DescriptorSetLayout) -> Self {
        self.descriptor_set_layouts.push(descriptor_set_layout);
        self
    }

    pub fn descriptor_set_layouts(
        mut self,
        descriptor_set_layouts: Vec<DescriptorSetLayout>,
    ) -> Self {
        self.descriptor_set_layouts = descriptor_set_layouts;
        self
    }

    pub fn build(self) -> GraphicsPipelineInfo {
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
        }
    }
}

impl Drop for GraphicsPipeline {
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

impl GraphicsPipeline {
    pub fn new(vulkan: &Vulkan, info: GraphicsPipelineInfo) -> Self {
        let shader_main_c_str = std::ffi::CString::new("main").unwrap();
        let shader_stages = [
            vk::PipelineShaderStageCreateInfo::builder()
                .stage(vk::ShaderStageFlags::VERTEX)
                .module(info.vertex_shader.module())
                .name(shader_main_c_str.as_c_str())
                .build(),
            vk::PipelineShaderStageCreateInfo::builder()
                .stage(vk::ShaderStageFlags::FRAGMENT)
                .module(info.fragment_shader.module())
                .name(shader_main_c_str.as_c_str())
                .build(),
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
        let pipeline_layout_create_info = vk::PipelineLayoutCreateInfo::builder()
            .set_layouts(&descriptor_set_layouts)
            .build();

        // Safety: The pipeline layout is dropped when the internal pipeline is dropped
        let pipeline_layout = unsafe {
            vulkan
                .device()
                .create_pipeline_layout(&pipeline_layout_create_info, None)
                .unwrap()
        };

        let graphics_pipeline_create_info = vk::GraphicsPipelineCreateInfo::builder()
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
                    &[graphics_pipeline_create_info.build()],
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
        let attachments = subpasses
            .iter()
            .flat_map(|subpass| {
                // Map from unique images to attachments
                let mut attachments: HashMap<vk::Image, Attachment> = HashMap::new();

                attachments.extend(
                    subpass
                        .color_attachments
                        .iter()
                        .chain(&subpass.depth_attachment)
                        .chain(&subpass.input_attachments)
                        .map(|attachment_reference| {
                            let attachment = attachment_reference.attachment.clone();
                            let image = attachment.image_dep.image();
                            (image, attachment)
                        }),
                );

                attachments
            })
            .collect::<HashMap<vk::Image, Attachment>>();

        let attachment_indices = attachments
            .iter()
            .enumerate()
            .map(|(index, (image, _))| (*image, index as u32))
            .collect::<HashMap<vk::Image, u32>>();

        let subpass_attachments_references = subpasses
            .iter()
            .enumerate()
            .map(|(i, subpass)| {
                let color_attachments = subpass
                    .color_attachments
                    .iter()
                    .map(|attachment_reference| {
                        vk::AttachmentReference::builder()
                            .attachment(
                                attachment_indices
                                    [&attachment_reference.attachment.image_dep.image()],
                            )
                            .layout(attachment_reference.layout)
                            .build()
                    })
                    .collect::<Vec<_>>();

                (i, color_attachments)
            })
            .collect::<Vec<_>>();

        let subpass_descriptions = subpass_attachments_references
            .iter()
            .map(|(i, color_attachments)| {
                vk::SubpassDescription::builder()
                    .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
                    .color_attachments(color_attachments)
                    .build()
            })
            .collect::<Vec<_>>();

        let attachment_descriptions = attachments
            .iter()
            .map(|(_, attachment)| {
                vk::AttachmentDescription::builder()
                    .format(attachment.image_dep.image_format())
                    .samples(attachment.info.samples)
                    .load_op(attachment.info.load_op)
                    .store_op(attachment.info.store_op)
                    .stencil_load_op(attachment.info.load_op)
                    .stencil_store_op(attachment.info.store_op)
                    .initial_layout(attachment.info.initial_layout)
                    .final_layout(attachment.info.final_layout)
                    .build()
            })
            .collect::<Vec<_>>();

        let render_pass_create_info = vk::RenderPassCreateInfo::builder()
            .attachments(&attachment_descriptions)
            .subpasses(&subpass_descriptions);

        // Safety: The render pass is dropped when the internal render pass is dropped
        let render_pass = unsafe {
            vulkan
                .device()
                .create_render_pass(&render_pass_create_info, None)
                .unwrap()
        };

        let attachment_image_views = attachments
            .iter()
            .map(|(_, attachment)| attachment.image_dep.image_view())
            .collect::<Vec<_>>();

        let (width, height) = attachments
            .iter()
            .map(|(_, attachment)| {
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

        let framebuffer_create_info = vk::FramebufferCreateInfo::builder()
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
    pub depth_attachment: Option<AttachmentReference>,
    pub input_attachments: Vec<AttachmentReference>,
}

impl Subpass {
    pub fn new() -> Self {
        Self {
            color_attachments: Vec::new(),
            depth_attachment: None,
            input_attachments: Vec::new(),
        }
    }

    pub fn color_attachment(&mut self, attachment: &Attachment) {
        self.color_attachments
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
}

impl Default for AttachmentInfo {
    fn default() -> Self {
        Self {
            samples: vk::SampleCountFlags::TYPE_1,
            load_op: vk::AttachmentLoadOp::CLEAR,
            store_op: vk::AttachmentStoreOp::STORE,
            initial_layout: vk::ImageLayout::UNDEFINED,
            final_layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
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
