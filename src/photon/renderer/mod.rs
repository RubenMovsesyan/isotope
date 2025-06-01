use std::sync::Arc;

use anyhow::Result;
use camera::PhotonCamera;
use cgmath::{Point3, Vector3};
use lights::{Lights, light::Light};
use log::debug;
use photon_layouts::PhotonLayoutsManager;
use texture::{PhotonDepthTexture, View};
use wgpu::{
    Color, LoadOp, Operations, PolygonMode, RenderPass, RenderPassColorAttachment,
    RenderPassDepthStencilAttachment, RenderPassDescriptor, RenderPipeline, StoreOp, Surface,
    SurfaceConfiguration, TextureViewDescriptor, include_wgsl, wgt::CommandEncoderDescriptor,
};
use winit::dpi::PhysicalSize;

use crate::{
    GpuController, ModelInstance, construct_debug_render_pipeline, construct_render_pipeline,
    element::{buffered::Buffered, model_vertex::ModelVertex},
};

pub mod camera;
pub mod lights;
pub mod photon_layouts;
pub mod texture;

mod render_macros;

pub const CAMERA_BIND_GROUP: u32 = 0;
pub const LIGHTS_BIND_GROUP: u32 = 1;

#[derive(Debug)]
pub struct PhotonRenderer {
    gpu_controller: Arc<GpuController>,
    pub layouts: Arc<PhotonLayoutsManager>,
    render_pipeline: RenderPipeline,
    debug_render_pipeline: Option<RenderPipeline>,

    // Rendering requirements
    depth_texture: PhotonDepthTexture,
    pub(crate) camera: PhotonCamera,
    pub(crate) lights: Lights,
}

impl PhotonRenderer {
    pub fn new(
        gpu_controller: Arc<GpuController>,
        surface_configuration: &SurfaceConfiguration,
    ) -> Self {
        // Load in the shaders
        let vertex_shader = gpu_controller
            .device
            .create_shader_module(include_wgsl!("shaders/vert.wgsl"));
        let fragment_shader = gpu_controller
            .device
            .create_shader_module(include_wgsl!("shaders/frag.wgsl"));

        // Initialie the layout manager
        let layouts = PhotonLayoutsManager::new(&gpu_controller);

        // Create the render pipeline
        let render_pipeline = construct_render_pipeline!(
            &gpu_controller.device,
            surface_configuration,
            vertex_shader,
            fragment_shader,
            String::from("Photon"),
            PolygonMode::Fill,
            &[ModelVertex::desc(), ModelInstance::desc()],
            &layouts.camera_layout,
            &layouts.lights_layout,
            &layouts.texture_layout,
            &layouts.model_layout,
            &layouts.material_layout
        );

        // Create the depth texture
        let depth_texture =
            PhotonDepthTexture::new_depth_texture(&gpu_controller, surface_configuration);

        // Initialie the camera
        // TODO: ADD INITIALIZATION OPTION
        // TODO: ADD CAMERA 2D
        let camera = PhotonCamera::create_new_camera_3d(
            gpu_controller.clone(),
            &layouts,
            Point3 {
                x: 2.0,
                y: 2.0,
                z: 2.0,
            },
            Vector3 {
                x: -1.0,
                y: -1.0,
                z: -1.0,
            },
            Vector3 {
                x: 0.0,
                y: 1.0,
                z: 0.0,
            },
            surface_configuration.width as f32 / surface_configuration.height as f32,
            90.0,
            0.1,
            100.0,
        );

        // Initialize with no lights
        let lights = Lights::new_with_lights(&gpu_controller, &layouts, &[]);

        Self {
            gpu_controller,
            layouts: Arc::new(layouts),
            render_pipeline,
            debug_render_pipeline: None,
            depth_texture,
            camera,
            lights,
        }
    }

    pub fn add_debug_render_pipeline(&mut self, surface_configuration: &SurfaceConfiguration) {
        let vertex_shader = self
            .gpu_controller
            .device
            .create_shader_module(include_wgsl!("shaders/debug_vert.wgsl"));

        let fragment_shader = self
            .gpu_controller
            .device
            .create_shader_module(include_wgsl!("shaders/debug_frag.wgsl"));

        let debug_render_pipeline = construct_debug_render_pipeline!(
            &self.gpu_controller.device,
            surface_configuration,
            vertex_shader,
            fragment_shader,
            String::from("Photon Debug"),
            PolygonMode::Line,
            &[ModelVertex::desc()],
            &self.layouts.camera_layout,
            &self.layouts.collider_layout
        );

        self.debug_render_pipeline = Some(debug_render_pipeline);
    }

    // Function to modify the lights in the scene
    pub fn update_lights(&mut self, lights: &[Light]) {
        self.lights
            .update(&self.gpu_controller, &self.layouts, lights);
    }

    // Change the render configuration and camera and all other necessary items to resize the render
    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
        self.depth_texture =
            PhotonDepthTexture::new_depth_texture_from_size(&self.gpu_controller, new_size);
        self.camera
            .set_aspect(new_size.width as f32 / new_size.height as f32);
    }

    // Renders all elements in the engine
    // pub fn render(&self, surface: &Surface<'static>, elements: &[Arc<dyn Element>]) -> Result<()> {
    pub fn render<F, D>(
        &mut self,
        surface: &Surface<'static>,
        callback: F,
        debug_callback: D,
    ) -> Result<()>
    where
        F: FnOnce(&mut RenderPass),
        D: FnOnce(&mut RenderPass),
    {
        // Write to the camera buffer if needed
        self.camera.write_buffer(); // only writing when rendering has a huge performance improvement

        let output = surface.get_current_texture()?;

        let view = output
            .texture
            .create_view(&TextureViewDescriptor::default());

        let mut encoder =
            self.gpu_controller
                .device
                .create_command_encoder(&CommandEncoderDescriptor {
                    label: Some("Render Encoder"),
                });

        // Scene Render Pass
        {
            let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("Scene Render Pass"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Clear(Color::BLACK),
                        store: StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(RenderPassDepthStencilAttachment {
                    view: self.depth_texture.view(),
                    depth_ops: Some(Operations {
                        load: LoadOp::Clear(1.0),
                        store: StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            // render_pass.set_pipeline(&self.render_pipeline);

            // Camera
            render_pass.set_bind_group(CAMERA_BIND_GROUP, &self.camera.bind_group, &[]);
            render_pass.set_bind_group(LIGHTS_BIND_GROUP, &self.lights.bind_group, &[]);

            callback(&mut render_pass);
        }

        // Debug render pass
        if let Some(debug_pipeline) = self.debug_render_pipeline.as_mut() {
            let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("Debug Render Pass"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Load,
                        store: StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(RenderPassDepthStencilAttachment {
                    view: self.depth_texture.view(),
                    depth_ops: Some(Operations {
                        load: LoadOp::Load,
                        store: StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                // depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            render_pass.set_pipeline(&debug_pipeline);

            // Camera
            render_pass.set_bind_group(CAMERA_BIND_GROUP, &self.camera.bind_group, &[]);

            debug_callback(&mut render_pass);
        }

        self.gpu_controller.queue.submit(Some(encoder.finish()));
        output.present();

        Ok(())
    }
}
