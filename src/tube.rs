use three_d::{InnerSpace, Vec3};

use crate::curves::{Curve, FrenetFrame};

pub struct VPair {
    pub point: Vec3,
    pub direction: Vec3,
}

pub struct Tube {
    pub vertices: Vec<Vec3>,
    pub indices: Vec<u32>,
    pub center_points: Vec<Vec3>,
    pub normals: Vec<Vec3>,
    pub normals_frame: Vec<Vec3>,
    pub binormals_frame: Vec<Vec3>,
    pub tangents_frame: Vec<Vec3>,
}

impl Tube {
    pub fn new(
        path: &dyn Curve,
        tubular_segments: usize,
        closed: bool,
        radius: f32,
        radial_segments: usize,
    ) -> Self {
        let frame = path.compute_frenet_frames(tubular_segments, closed);

        let mut vertices = vec![];
        let mut normals = vec![];
        let mut indices = vec![];
        let mut center_points = vec![];

        Self::generate_buffer(
            path,
            &mut indices,
            &mut vertices,
            &mut normals,
            &mut center_points,
            tubular_segments,
            radius,
            radial_segments,
            &frame,
        );

        Self {
            vertices,
            indices,
            normals,
            center_points,
            normals_frame: frame.normals,
            binormals_frame: frame.binormals,
            tangents_frame: frame.tangents,
        }
    }

    fn generate_buffer(
        curve: &dyn Curve,
        indices: &mut Vec<u32>,
        vertices: &mut Vec<Vec3>,
        normals: &mut Vec<Vec3>,
        points: &mut Vec<Vec3>,
        tubular_segments: usize,
        radius: f32,
        radial_segments: usize,
        frames: &FrenetFrame,
    ) {
        // tubular segments -> len of path

        for (i, point) in curve.get_points(tubular_segments).iter().enumerate() {
            points.push(*point);

            Self::generate_segment(
                vertices,
                normals,
                &point,
                radial_segments,
                radius,
                &frames.normals[i],
                &frames.binormals[i],
            );
        }

        Self::generate_indices(indices, tubular_segments, radial_segments);
    }

    fn generate_segment(
        vertices: &mut Vec<Vec3>,
        normals: &mut Vec<Vec3>,
        point: &Vec3,
        radial_segments: usize,
        radius: f32,
        frame_N: &Vec3,
        frame_B: &Vec3,
    ) {
        // // generate normals and vertices for the current segment

        for j in 0..radial_segments {
            // std::f32::consts::PI * 2.
            // (3.0 * std::f32::consts::PI / 2.0)
            let v = j as f32 / radial_segments as f32 * (3.0 * std::f32::consts::PI / 2.0);

            let sin = v.sin();
            let cos = v.cos();

            let mut normal = Vec3::new(0., 0., 0.);

            normal.x = cos * frame_N.x + sin * frame_B.x;
            normal.y = cos * frame_N.y + sin * frame_B.y;
            normal.z = cos * frame_N.z + sin * frame_B.z;
            normal.normalize();
            normals.push(normal);

            // vertex

            let mut vertex = Vec3::new(0., 0., 0.);

            vertex.x = point.x + radius * normal.x;
            vertex.y = point.y + radius * normal.y;
            vertex.z = point.z + radius * normal.z;

            vertices.push(vertex);
        }
    }

    fn generate_indices(indices: &mut Vec<u32>, tubular_segments: usize, radial_segments: usize) {
        for i in 0..tubular_segments {
            for j in 0..radial_segments - 1 {
                let next_j = (j + 1) % radial_segments;
                let current = (i * radial_segments + j) as u32;
                let next = ((i + 1) * radial_segments + j) as u32;
                let current_next = (i * radial_segments + next_j) as u32;
                let next_next = ((i + 1) * radial_segments + next_j) as u32;

                indices.push(current);
                indices.push(next);
                indices.push(current_next);

                indices.push(current_next);
                indices.push(next);
                indices.push(next_next);
            }
        }
    }
}
