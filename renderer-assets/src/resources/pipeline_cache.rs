use crate::{
    ResourceLookupSet, RenderPassResource, ResourceArc, GraphicsPipelineResource,
    MaterialPassResource,
};
use renderer_nodes::{RenderPhase, RenderPhaseIndex, MAX_RENDER_PHASE_COUNT, RenderRegistry};
use crate::resources::resource_arc::{WeakResourceArc, ResourceId};
use fnv::FnvHashMap;
use std::hash::Hash;
use ash::prelude::VkResult;

#[derive(PartialEq, Eq, Hash)]
struct CachedGraphicsPipelineKey {
    material_pass: ResourceId,
    renderpass: ResourceId,
}

#[derive(PartialEq, Eq, Hash)]
struct CachedGraphicsPipeline {
    material_pass_resource: WeakResourceArc<MaterialPassResource>,
    renderpass_resource: WeakResourceArc<RenderPassResource>,
    graphics_pipeline: ResourceArc<GraphicsPipelineResource>,
}

#[derive(Debug)]
struct RegisteredRenderpass {
    keep_until_frame: u64,
    renderpass: WeakResourceArc<RenderPassResource>,
}

pub struct GraphicsPipelineCache {
    render_registry: RenderRegistry,

    // index by renderphase index
    renderpass_assignments: Vec<FnvHashMap<ResourceId, RegisteredRenderpass>>,
    material_pass_assignments: Vec<FnvHashMap<ResourceId, WeakResourceArc<MaterialPassResource>>>,

    cached_pipelines: FnvHashMap<CachedGraphicsPipelineKey, CachedGraphicsPipeline>,

    current_frame_index: u64,
    frames_to_persist: u64,
}

impl GraphicsPipelineCache {
    pub fn new(render_registry: &RenderRegistry) -> Self {
        const DEFAULT_FRAMES_TO_PERSIST: u64 = 1;

        let mut renderpass_assignments = Vec::with_capacity(MAX_RENDER_PHASE_COUNT as usize);
        renderpass_assignments.resize_with(MAX_RENDER_PHASE_COUNT as usize, || Default::default());

        let mut material_pass_assignments = Vec::with_capacity(MAX_RENDER_PHASE_COUNT as usize);
        material_pass_assignments
            .resize_with(MAX_RENDER_PHASE_COUNT as usize, || Default::default());

        GraphicsPipelineCache {
            render_registry: render_registry.clone(),
            renderpass_assignments,
            material_pass_assignments,
            cached_pipelines: Default::default(),
            current_frame_index: 0,
            frames_to_persist: DEFAULT_FRAMES_TO_PERSIST,
        }
    }

    pub fn on_frame_complete(&mut self) {
        self.current_frame_index += 1;
        self.drop_unused_pipelines();
    }

    pub fn get_renderphase_by_name(
        &self,
        name: &str,
    ) -> Option<RenderPhaseIndex> {
        self.render_registry.render_phase_index_from_name(name)
    }

    // Register a renderpass as being part of a particular phase. This will a pipeline is created
    // for all appropriate renderpass/material pass pairs.
    pub fn register_renderpass_to_phase_per_frame<T: RenderPhase>(
        &mut self,
        renderpass: &ResourceArc<RenderPassResource>,
    ) {
        self.register_renderpass_to_phase_index_per_frame(renderpass, T::render_phase_index())
    }

    pub fn register_renderpass_to_phase_index_per_frame(
        &mut self,
        renderpass: &ResourceArc<RenderPassResource>,
        render_phase_index: RenderPhaseIndex,
    ) {
        assert!(render_phase_index < MAX_RENDER_PHASE_COUNT);
        if let Some(existing) =
            self.renderpass_assignments[render_phase_index as usize].get_mut(&renderpass.get_hash())
        {
            if existing.renderpass.upgrade().is_some() {
                existing.keep_until_frame = self.current_frame_index + self.frames_to_persist;
                // Nothing to do here, the previous ref is still valid
                return;
            }
        }

        self.renderpass_assignments[render_phase_index as usize].insert(
            renderpass.get_hash(),
            RegisteredRenderpass {
                renderpass: renderpass.downgrade(),
                keep_until_frame: self.current_frame_index + self.frames_to_persist,
            },
        );

        //TODO: Do we need to mark this as a dirty renderpass that may need rebuilding materials?
    }

    pub fn register_material_to_phase_index(
        &mut self,
        material_pass: &ResourceArc<MaterialPassResource>,
        render_phase_index: RenderPhaseIndex,
    ) {
        assert!(render_phase_index < MAX_RENDER_PHASE_COUNT);
        if let Some(existing) = self.material_pass_assignments[render_phase_index as usize]
            .get(&material_pass.get_hash())
        {
            if existing.upgrade().is_some() {
                // Nothing to do here, the previous ref is still valid
                return;
            }
        }

        self.material_pass_assignments[render_phase_index as usize]
            .insert(material_pass.get_hash(), material_pass.downgrade());
        //TODO: Do we need to mark this as a dirty material that may need rebuilding?
    }

    pub fn find_graphics_pipeline(
        &self,
        material: &ResourceArc<MaterialPassResource>,
        renderpass: &ResourceArc<RenderPassResource>,
    ) -> Option<ResourceArc<GraphicsPipelineResource>> {
        let key = CachedGraphicsPipelineKey {
            material_pass: material.get_hash(),
            renderpass: renderpass.get_hash(),
        };

        // Find the swapchain index for the given renderpass
        self.cached_pipelines.get(&key).map(|x| {
            debug_assert!(x.material_pass_resource.upgrade().is_some());
            debug_assert!(x.renderpass_resource.upgrade().is_some());
            x.graphics_pipeline.clone()
        })
    }

    pub fn cache_all_pipelines(
        &mut self,
        resources: &mut ResourceLookupSet,
    ) -> VkResult<()> {
        //TODO: Avoid iterating everything all the time
        for render_phase_index in 0..MAX_RENDER_PHASE_COUNT {
            for (renderpass_hash, renderpass) in
                &self.renderpass_assignments[render_phase_index as usize]
            {
                for (material_pass_hash, material_pass) in
                    &self.material_pass_assignments[render_phase_index as usize]
                {
                    let key = CachedGraphicsPipelineKey {
                        renderpass: *renderpass_hash,
                        material_pass: *material_pass_hash,
                    };

                    if !self.cached_pipelines.contains_key(&key) {
                        if let Some(renderpass) = renderpass.renderpass.upgrade() {
                            if let Some(material_pass) = material_pass.upgrade() {
                                let pipeline = resources
                                    .get_or_create_graphics_pipeline(&material_pass, &renderpass)?;
                                self.cached_pipelines.insert(
                                    key,
                                    CachedGraphicsPipeline {
                                        graphics_pipeline: pipeline,
                                        renderpass_resource: renderpass.downgrade(),
                                        material_pass_resource: material_pass.downgrade(),
                                    },
                                );
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }

    pub fn drop_unused_pipelines(&mut self) {
        let current_frame_index = self.current_frame_index;
        for phase in &mut self.renderpass_assignments {
            phase.retain(|_k, v| {
                v.renderpass.upgrade().is_some() && v.keep_until_frame > current_frame_index
            });
        }

        for phase in &mut self.material_pass_assignments {
            phase.retain(|_k, v| v.upgrade().is_some());
        }

        self.cached_pipelines.retain(|_k, v| {
            let renderpass_still_exists = v.renderpass_resource.upgrade().is_some();
            let material_pass_still_exists = v.material_pass_resource.upgrade().is_some();

            if !renderpass_still_exists || !material_pass_still_exists {
                log::trace!("Dropping pipeline, renderpass_still_exists: {}, material_pass_still_exists: {}", renderpass_still_exists, material_pass_still_exists);
            }

            renderpass_still_exists && material_pass_still_exists
        })
    }

    pub fn clear_pipelines(&mut self) {
        self.cached_pipelines.clear();
    }
}
