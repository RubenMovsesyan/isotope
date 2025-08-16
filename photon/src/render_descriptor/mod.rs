use gpu_controller::Mesh;
use matter_vault::SharedMatter;
use wgpu::RenderPass;

pub struct GeometryDescriptor {
    pub meshes: Vec<SharedMatter<Mesh>>,
}

impl GeometryDescriptor {
    pub(crate) fn geomtry_pass(&self, render_pass: &mut RenderPass) {
        for mesh in &self.meshes {
            mesh.read(|mesh| {
                mesh.render(render_pass);
            });
        }
    }
}
