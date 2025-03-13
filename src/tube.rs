use three_d::{InnerSpace, Vec3, Vector3};

use crate::curves::{Frame, compute_frenet_frames};

pub struct Tube {
    pub vertices: Vec<Vec3>,
    pub indices: Vec<u32>,
    pub normals: Vec<Vec3>,
    pub binormals: Vec<Vec3>,
    pub tangents: Vec<Vec3>,
}

impl Tube {
    pub fn new(
        path: &[Vec3],
        tubular_segments: usize,
        radius: f32,
        radial_segments: usize,
    ) -> Self {
        let frame = compute_frenet_frames(tubular_segments);

        let mut vertices = vec![];
        let mut normals = vec![];
        let mut indices = vec![];

        // buffer

        Self::generate_buffer(
            path,
            &mut indices,
            &mut vertices,
            &mut normals,
            tubular_segments,
            radius,
            radial_segments,
            &frame,
        );

        Self {
            vertices,
            indices,
            normals,
            binormals: frame.binormals,
            tangents: frame.tangents,
        }
    }

    fn generate_buffer(
        path: &[Vec3],
        indices: &mut Vec<u32>,
        vertices: &mut Vec<Vec3>,
        normals: &mut Vec<Vec3>,
        tubular_segments: usize,
        radius: f32,
        radial_segments: usize,
        frames: &Frame,
    ) {
        // tubular segments -> len of path
        for i in 0..tubular_segments {
            Self::generate_segment(
                vertices,
                normals,
                &path[i],
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
        point: &Vector3<f32>,
        radial_segments: usize,
        radius: f32,
        frame_N: &Vec3,
        frame_B: &Vec3,
    ) {
        // // generate normals and vertices for the current segment

        for j in 0..radial_segments {
            let v = j as f32 / radial_segments as f32 * std::f32::consts::PI * 2.;

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
        for i in 0..radial_segments - 1 {
            for j in 0..tubular_segments {
                let next_j = (j + 1) % tubular_segments;
                let current = (i * tubular_segments + j) as u32;
                let next = ((i + 1) * tubular_segments + j) as u32;
                let current_next = (i * tubular_segments + next_j) as u32;
                let next_next = ((i + 1) * tubular_segments + next_j) as u32;

                indices.push(current);
                indices.push(next);
                indices.push(current_next);

                indices.push(current_next);
                indices.push(next);
                indices.push(next_next);
            }
        }
        println!("{:?}", indices);
    }
}

pub struct VPair {
    pub point: Vec3,
    pub direction: Vec3,
}

pub struct TubeS {
    pub vertices: Vec<Vec3>,
    pub indices: Vec<u32>,
    pub normals: Vec<VPair>,
    pub binormals: Vec<VPair>,
    pub tangents: Vec<VPair>,
}

pub fn tube_s(curve: &[Vec3], radius: f32, segments: usize) -> TubeS {
    let mut vertices = Vec::new();
    let mut indices = Vec::new();

    let mut normals = vec![];
    let mut binormals = vec![];
    let mut tangents = vec![];

    for i in 0..curve.len() {
        let p = curve[i];
        let tangent = if i < curve.len() - 1 {
            (curve[i + 1] - p).normalize()
        } else {
            (p - curve[i - 1]).normalize()
        };

        tangents.push(VPair {
            point: p,
            direction: tangent,
        });

        let normal = if tangent.z.abs() < tangent.x.abs() {
            Vector3::new(-tangent.y, tangent.x, 0.0).normalize()
        } else {
            Vector3::new(0.0, -tangent.z, tangent.y).normalize()
        };

        normals.push(VPair {
            point: p,
            direction: normal,
        });

        let binormal = tangent.cross(normal).normalize();

        binormals.push(VPair {
            point: p,
            direction: binormal,
        });

        for j in 0..segments {
            let angle = (j as f32) / (segments as f32) * std::f32::consts::TAU;
            let offset = normal * angle.cos() + binormal * angle.sin();
            vertices.push(p + offset * radius);
        }
    }

    let rings = curve.len();
    for i in 0..rings - 1 {
        for j in 0..segments {
            let next_j = (j + 1) % segments;
            let current = (i * segments + j) as u32;
            let next = ((i + 1) * segments + j) as u32;
            let current_next = (i * segments + next_j) as u32;
            let next_next = ((i + 1) * segments + next_j) as u32;

            indices.push(current);
            indices.push(next);
            indices.push(current_next);

            indices.push(current_next);
            indices.push(next);
            indices.push(next_next);
        }
    }

    TubeS {
        vertices,
        indices,
        normals,
        binormals,
        tangents,
    }
}
