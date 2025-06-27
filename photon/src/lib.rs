use std::sync::Arc;

use anyhow::Result;
use gpu_controller::GpuController;
use texture::PhotonTexture;
use wgpu::{
    Color, CommandEncoder, Extent3d, LoadOp, Operations, RenderPass, RenderPassColorAttachment,
    RenderPassDepthStencilAttachment, RenderPassDescriptor, StoreOp, Surface,
    wgt::TextureViewDescriptor,
};

pub use render_descriptor::PhotonRenderDescriptor;

mod render_descriptor;
mod texture;

pub trait Render {
    fn render(&self, render_pass: &mut RenderPass);

    #[allow(unused_variables)]
    fn debug_render(&self, debug_render_pass: &mut RenderPass) {}
}

#[derive(Debug)]
pub struct PhotonRenderer {
    gpu_controller: Arc<GpuController>,

    depth_texture: PhotonTexture,
}

impl PhotonRenderer {
    pub fn new(gpu_controller: Arc<GpuController>) -> Result<Self> {
        let depth_texture = PhotonTexture::new_depth_texture(&gpu_controller)?;

        Ok(Self {
            gpu_controller,
            depth_texture,
        })
    }

    pub fn render<R>(
        &mut self,
        encoder: Option<CommandEncoder>,
        surface: &Surface<'static>,
        render_callback: R,
    ) -> Result<()>
    where
        R: FnOnce(&mut RenderPass),
    {
        // Create the view to render to
        let output = surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&TextureViewDescriptor::default());

        let mut encoder = if let Some(encoder) = encoder {
            encoder
        } else {
            self.gpu_controller.create_command_encoder("Render Encoder")
        };

        // Run the render pass
        {
            let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Clear(Color::BLACK),
                        store: StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(RenderPassDepthStencilAttachment {
                    view: &self.depth_texture.view,
                    depth_ops: Some(Operations {
                        load: LoadOp::Clear(1.0),
                        store: StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            render_callback(&mut render_pass);
        }

        // TODO: add debug render pass

        self.gpu_controller.submit(encoder);
        output.present();

        Ok(())
    }
}

/// ============== Photon Tests ==============
#[cfg(test)]
mod test {
    use super::*;
    use smol::block_on;

    #[test]
    fn test_create_photon_renderer() {
        if let Ok(gpu_controller) = block_on(GpuController::new(None, None, None)) {
            assert!(PhotonRenderer::new(gpu_controller).is_ok());
        } else {
            assert!(false, "Failed to Create GpuController");
        }
    }
}
