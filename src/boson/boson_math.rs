use cgmath::{MetricSpace, Vector3, Zero};

use crate::{Model, element::mesh::Mesh};

use log::*;

const TRIANGLE_SIZE: usize = 3;

// TODO: IMPLEMENT VOLUMETRIC CENTER OF MASS RATHER THAN AREA CENTER IF MASS

/// Calculates the center of mass of a model from all the points in all of its meshes
pub fn calculate_center_of_mass(model: &Model) -> Vector3<f32> {
    // Calcuate the center of mass of a mesh
    fn calculate_mesh_com(mesh: &Mesh) -> Vector3<f32> {
        // The indices should be divisible by 3 otherwise the mesh is not triangulated
        let (indices, vertices) = match mesh {
            Mesh::Unbuffered {
                indices, vertices, ..
            }
            | Mesh::Buffered {
                indices, vertices, ..
            } => (indices, vertices),
        };

        if indices.len() % 3 != 0 {
            error!(
                "Mesh labeled {} indices of length {} is not triangulated",
                mesh.label(),
                indices.len()
            );

            // panic for now because I don't want to deal with error handling yet
            panic!();
        }

        // Helper functions ====================================================

        // closure that gets the points from the mesh given the index
        let point = |index: u32| -> Vector3<f32> {
            let vertex = vertices[index as usize];

            Vector3::from(vertex.position)
        };

        #[inline]
        fn area(a: Vector3<f32>, b: Vector3<f32>, c: Vector3<f32>) -> f32 {
            let side_1 = a.distance(b);
            let side_2 = b.distance(c);
            let side_3 = c.distance(a);

            let half_perimeter = (side_1 + side_2 + side_3) / 2.0;

            f32::sqrt(
                half_perimeter
                    * (half_perimeter - side_1)
                    * (half_perimeter - side_2)
                    * (half_perimeter - side_3),
            )
        }

        // Helper functions ====================================================

        // Iterate through all the indices chunked
        let triangle_info = indices
            .chunks(TRIANGLE_SIZE)
            .map(|triangle| {
                // points of each of the vertices
                let a = point(triangle[0]);
                let b = point(triangle[1]);
                let c = point(triangle[2]);

                let center = (a + b + c) / TRIANGLE_SIZE as f32;
                let area = area(a, b, c);

                (center, area)
            })
            .collect::<Vec<(Vector3<f32>, f32)>>();

        let mut center: Vector3<f32> = Vector3::zero();
        let mut area_sum: f32 = 0.0;

        for (tri_center, tri_area) in triangle_info.iter() {
            center += tri_center * *tri_area;
            area_sum += tri_area;
        }

        center / area_sum
    }

    let mut absolute_center = Vector3::zero();
    model
        .meshes
        .iter()
        .for_each(|mesh| absolute_center += calculate_mesh_com(mesh));

    absolute_center / model.meshes.len() as f32
}
