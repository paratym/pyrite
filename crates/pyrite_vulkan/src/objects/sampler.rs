use std::{ops::Deref, sync::Arc};

use ash::vk;
use pyrite_util::Dependable;

use crate::{Vulkan, VulkanDep};

pub type SamplerDep = Arc<SamplerInner>;
pub struct Sampler {
    inner: Arc<SamplerInner>,
}

impl Sampler {
    pub fn new(vulkan: &Vulkan, info: &SamplerInfo) -> Self {
        Self {
            inner: Arc::new(SamplerInner::new(vulkan, info)),
        }
    }

    pub fn create_dep(&self) -> SamplerDep {
        self.inner.clone()
    }
}

impl Deref for Sampler {
    type Target = SamplerInner;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

pub struct SamplerInner {
    vulkan_dep: VulkanDep,
    sampler: vk::Sampler,
}

impl Drop for SamplerInner {
    fn drop(&mut self) {
        unsafe {
            self.vulkan_dep.device().destroy_sampler(self.sampler, None);
        }
    }
}

pub struct SamplerInfo {
    pub mag_filter: vk::Filter,
    pub min_filter: vk::Filter,
    pub mipmap_mode: vk::SamplerMipmapMode,
    pub address_mode_u: vk::SamplerAddressMode,
    pub address_mode_v: vk::SamplerAddressMode,
    pub address_mode_w: vk::SamplerAddressMode,
    pub mip_lod_bias: f32,
    pub anisotropy_enable: bool,
    pub max_anisotropy: f32,
    pub compare_enable: bool,
    pub compare_op: vk::CompareOp,
    pub min_lod: f32,
    pub max_lod: f32,
    pub border_color: vk::BorderColor,
    pub unnormalized_coordinates: bool,
}

impl SamplerInfo {
    pub fn builder() -> SamplerInfoBuilder {
        SamplerInfoBuilder::default()
    }
}

pub struct SamplerInfoBuilder {
    mag_filter: vk::Filter,
    min_filter: vk::Filter,
    mipmap_mode: vk::SamplerMipmapMode,
    address_mode_u: vk::SamplerAddressMode,
    address_mode_v: vk::SamplerAddressMode,
    address_mode_w: vk::SamplerAddressMode,
    mip_lod_bias: f32,
    anisotropy_enable: bool,
    max_anisotropy: f32,
    compare_enable: bool,
    compare_op: vk::CompareOp,
    min_lod: f32,
    max_lod: f32,
    border_color: vk::BorderColor,
    unnormalized_coordinates: bool,
}

impl Default for SamplerInfoBuilder {
    fn default() -> Self {
        Self {
            mag_filter: vk::Filter::LINEAR,
            min_filter: vk::Filter::LINEAR,
            mipmap_mode: vk::SamplerMipmapMode::LINEAR,
            address_mode_u: vk::SamplerAddressMode::REPEAT,
            address_mode_v: vk::SamplerAddressMode::REPEAT,
            address_mode_w: vk::SamplerAddressMode::REPEAT,
            mip_lod_bias: 0.0,
            anisotropy_enable: false,
            max_anisotropy: 0.0,
            compare_enable: false,
            compare_op: vk::CompareOp::NEVER,
            min_lod: 0.0,
            max_lod: 0.0,
            border_color: vk::BorderColor::FLOAT_OPAQUE_WHITE,
            unnormalized_coordinates: false,
        }
    }
}

impl SamplerInfoBuilder {
    pub fn build(&self) -> SamplerInfo {
        SamplerInfo {
            mag_filter: self.mag_filter,
            min_filter: self.min_filter,
            mipmap_mode: self.mipmap_mode,
            address_mode_u: self.address_mode_u,
            address_mode_v: self.address_mode_v,
            address_mode_w: self.address_mode_w,
            mip_lod_bias: self.mip_lod_bias,
            anisotropy_enable: self.anisotropy_enable,
            max_anisotropy: self.max_anisotropy,
            compare_enable: self.compare_enable,
            compare_op: self.compare_op,
            min_lod: self.min_lod,
            max_lod: self.max_lod,
            border_color: self.border_color,
            unnormalized_coordinates: self.unnormalized_coordinates,
        }
    }
}

impl SamplerInner {
    pub fn new(vulkan: &Vulkan, info: &SamplerInfo) -> Self {
        let sampler = unsafe {
            vulkan.device().create_sampler(
                &vk::SamplerCreateInfo::builder()
                    .mag_filter(info.mag_filter)
                    .min_filter(info.min_filter)
                    .mipmap_mode(info.mipmap_mode)
                    .address_mode_u(info.address_mode_u)
                    .address_mode_v(info.address_mode_v)
                    .address_mode_w(info.address_mode_w)
                    .mip_lod_bias(info.mip_lod_bias)
                    .anisotropy_enable(info.anisotropy_enable)
                    .max_anisotropy(info.max_anisotropy)
                    .compare_enable(info.compare_enable)
                    .compare_op(info.compare_op)
                    .min_lod(info.min_lod)
                    .max_lod(info.max_lod)
                    .border_color(info.border_color)
                    .unnormalized_coordinates(info.unnormalized_coordinates)
                    .build(),
                None,
            )
        }
        .expect("Failed to create sampler");

        Self {
            vulkan_dep: vulkan.create_dep(),
            sampler,
        }
    }

    pub fn sampler(&self) -> vk::Sampler {
        self.sampler
    }
}
