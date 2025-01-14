// This code is auto-generated by the shader processor.

#[allow(unused_imports)]
use rafx_api::RafxResult;

#[allow(unused_imports)]
use crate::{
    DescriptorSetAllocator, DescriptorSetArc, DescriptorSetBindings, DescriptorSetInitializer,
    DescriptorSetWriter, DescriptorSetWriterContext, DynDescriptorSet, ImageViewResource,
    ResourceArc,
};

#[derive(Copy, Clone, Debug)]
#[repr(C)]
pub struct ConfigStd140 {
    pub src_uv_min: [f32; 2], // +0 (size: 8)
    pub src_uv_max: [f32; 2], // +8 (size: 8)
} // 16 bytes

impl Default for ConfigStd140 {
    fn default() -> Self {
        ConfigStd140 {
            src_uv_min: <[f32; 2]>::default(),
            src_uv_max: <[f32; 2]>::default(),
        }
    }
}

pub type ConfigUniform = ConfigStd140;

pub const SMP_DESCRIPTOR_SET_INDEX: usize = 0;
pub const SMP_DESCRIPTOR_BINDING_INDEX: usize = 0;
pub const SRC_TEX_DESCRIPTOR_SET_INDEX: usize = 0;
pub const SRC_TEX_DESCRIPTOR_BINDING_INDEX: usize = 1;
pub const CONFIG_DESCRIPTOR_SET_INDEX: usize = 0;
pub const CONFIG_DESCRIPTOR_BINDING_INDEX: usize = 2;

pub struct DescriptorSet0Args<'a> {
    pub src_tex: &'a ResourceArc<ImageViewResource>,
    pub config: &'a ConfigUniform,
}

impl<'a> DescriptorSetInitializer<'a> for DescriptorSet0Args<'a> {
    type Output = DescriptorSet0;

    fn create_dyn_descriptor_set(
        descriptor_set: DynDescriptorSet,
        args: Self,
    ) -> Self::Output {
        let mut descriptor = DescriptorSet0(descriptor_set);
        descriptor.set_args(args);
        descriptor
    }

    fn create_descriptor_set(
        descriptor_set_allocator: &mut DescriptorSetAllocator,
        descriptor_set: DynDescriptorSet,
        args: Self,
    ) -> RafxResult<DescriptorSetArc> {
        let mut descriptor = Self::create_dyn_descriptor_set(descriptor_set, args);
        descriptor.0.flush(descriptor_set_allocator)?;
        Ok(descriptor.0.descriptor_set().clone())
    }
}

impl<'a> DescriptorSetWriter<'a> for DescriptorSet0Args<'a> {
    fn write_to(
        descriptor_set: &mut DescriptorSetWriterContext,
        args: Self,
    ) {
        descriptor_set.set_image(SRC_TEX_DESCRIPTOR_BINDING_INDEX as u32, args.src_tex);
        descriptor_set.set_buffer_data(CONFIG_DESCRIPTOR_BINDING_INDEX as u32, args.config);
    }
}

pub struct DescriptorSet0(pub DynDescriptorSet);

impl DescriptorSet0 {
    pub fn set_args_static(
        descriptor_set: &mut DynDescriptorSet,
        args: DescriptorSet0Args,
    ) {
        descriptor_set.set_image(SRC_TEX_DESCRIPTOR_BINDING_INDEX as u32, args.src_tex);
        descriptor_set.set_buffer_data(CONFIG_DESCRIPTOR_BINDING_INDEX as u32, args.config);
    }

    pub fn set_args(
        &mut self,
        args: DescriptorSet0Args,
    ) {
        self.set_src_tex(args.src_tex);
        self.set_config(args.config);
    }

    pub fn set_src_tex(
        &mut self,
        src_tex: &ResourceArc<ImageViewResource>,
    ) {
        self.0
            .set_image(SRC_TEX_DESCRIPTOR_BINDING_INDEX as u32, src_tex);
    }

    pub fn set_config(
        &mut self,
        config: &ConfigUniform,
    ) {
        self.0
            .set_buffer_data(CONFIG_DESCRIPTOR_BINDING_INDEX as u32, config);
    }

    pub fn flush(
        &mut self,
        descriptor_set_allocator: &mut DescriptorSetAllocator,
    ) -> RafxResult<()> {
        self.0.flush(descriptor_set_allocator)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_struct_config_std140() {
        assert_eq!(std::mem::size_of::<ConfigStd140>(), 16);
        assert_eq!(std::mem::size_of::<[f32; 2]>(), 8);
        assert_eq!(std::mem::align_of::<[f32; 2]>(), 4);
        assert_eq!(memoffset::offset_of!(ConfigStd140, src_uv_min), 0);
        assert_eq!(std::mem::size_of::<[f32; 2]>(), 8);
        assert_eq!(std::mem::align_of::<[f32; 2]>(), 4);
        assert_eq!(memoffset::offset_of!(ConfigStd140, src_uv_max), 8);
    }
}
