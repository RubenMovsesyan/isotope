use std::sync::Arc;

use anyhow::Result;
use camera::PhotonCamera;
use lights::{Lights, light::Light};
use texture::{PhotonDepthTexture, View};
use wgpu::{
    Color, CommandEncoder, LoadOp, Operations, PolygonMode, RenderPass, RenderPassColorAttachment,
    RenderPassDepthStencilAttachment, RenderPassDescriptor, RenderPipeline, StoreOp, Surface,
    TextureViewDescriptor, include_wgsl, wgt::CommandEncoderDescriptor,
};
use winit::dpi::PhysicalSize;

use crate::{
    GpuController, construct_debug_render_pipeline,
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
    pub(crate) debugging: bool,

    // Rendering requirements
    depth_texture: PhotonDepthTexture,
    pub(crate) lights: Lights,
}

impl PhotonRenderer {
    pub fn new(gpu_controller: Arc<GpuController>) -> Self {
        // Create the depth texture
        let depth_texture = PhotonDepthTexture::new_depth_texture(&gpu_controller);

        // Initialize with no lights
        let lights = Lights::new_with_lights(&gpu_controller, &[]);

        Self {
            gpu_controller,
            debugging: false,
            depth_texture,
            lights,
        }
    }

    // Function to modify the lights in the scene
    pub fn update_lights(&mut self, lights: &[Light]) {
        self.lights
            .update(&self.gpu_controller, &self.gpu_controller.layouts, lights);
    }

    // Change the render configuration and camera and all other necessary items to resize the render
    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
        self.depth_texture =
            PhotonDepthTexture::new_depth_texture_from_size(&self.gpu_controller, new_size);
        // camera.set_aspect(new_size.width as f32 / new_size.height as f32);
    }

    // Runs the render pass with the callbacks
    pub fn render<U, R, D>(
        &mut self,
        surface: &Surface<'static>,
        camera: &mut PhotonCamera,
        update_callback: U,
        render_callback: R,
        debug_callback: D,
    ) -> Result<()>
    where
        U: FnOnce(&mut CommandEncoder),
        R: FnOnce(&mut RenderPass),
        D: FnOnce(&mut RenderPass),
    {
        // Write to the camera buffer if needed
        camera.write_buffer();

        let output = surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&TextureViewDescriptor::default());

        // Create a command encoder for sending commands to the gpu
        let mut encoder =
            self.gpu_controller
                .device
                .create_command_encoder(&CommandEncoderDescriptor {
                    label: Some("Render Encoder"),
                });

        // Run any GPU update functions
        update_callback(&mut encoder);

        // Scene Render Pass
        {
            // Create the render pass with a descriptor
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

            // Every render pipeline is required to have CAMERA_BIND_GROUP at 0 and LIGHTS_BIND_GROUP at 1
            render_pass.set_bind_group(CAMERA_BIND_GROUP, &camera.bind_group, &[]);
            render_pass.set_bind_group(LIGHTS_BIND_GROUP, &self.lights.bind_group, &[]);

            render_callback(&mut render_pass);
        }

        // Debugging
        if self.debugging {
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
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            // Every debug render pipeline is required to have CAMERA_BIND_GROUP at 0 and LIGHTS_BIND_GROUP at 1
            render_pass.set_bind_group(CAMERA_BIND_GROUP, &camera.bind_group, &[]);
            render_pass.set_bind_group(LIGHTS_BIND_GROUP, &self.lights.bind_group, &[]);

            debug_callback(&mut render_pass);
        }

        // Once all the callbacks have been called submit the queue to the encoder
        // and present the output texture
        self.gpu_controller.queue.submit(Some(encoder.finish()));
        output.present();

        Ok(())
    }
}
