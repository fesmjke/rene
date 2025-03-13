use std::f32::consts::PI;

use three_d::{radians, vec3, InnerSpace, Mat4, Transform, Vec3};

pub fn generate_sine_curve(
    start: Vec3,
    direction: Vec3,
    amplitude: f32,
    period: f32,
    length: f32,
    points_count: usize,
) -> Vec<Vec3> {
    let mut points = Vec::new();

    let direction = direction.normalize();

    let arbitrary = if direction.x.abs() < 0.9 {
        Vec3::new(1., 0., 0.)
    } else {
        Vec3::new(0., 1., 0.)
    };

    let up = direction.cross(arbitrary).normalize();

    for i in 0..points_count {
        let t = i as f32 / (points_count - 1) as f32 * length;
        let wave_offset = amplitude * (period * t).sin();
        let point = start + t * direction + wave_offset * up;
        points.push(point);
    }

    points
}

pub struct Frame {
    pub tangents: Vec<Vec3>,
    pub normals: Vec<Vec3>,
    pub binormals: Vec<Vec3>,
}

fn get_tangent_at(u: f32) -> Vec3 {
    let t = u * 2.0 * PI;
    Vec3::new(-t.sin(), t.cos(), 0.0).normalize()
}

// not sure that is the correct way of compute frenet frames
pub fn compute_frenet_frames(segments: usize) -> Frame {
    let mut tangents = Vec::with_capacity(segments + 1);
    let mut normals = Vec::with_capacity(segments + 1);
    let mut binormals = Vec::with_capacity(segments + 1);

    // compute the tangent vectors for each segment on the curve
    for i in 0..=segments {
        let u = i as f32 / segments as f32;
        tangents.push(get_tangent_at(u));
    }

    // select an initial normal vector perpendicular to the first tangent vector,
    // and in the direction of the minimum tangent xyz component
    let mut normal = Vec3::new(0.0, 0.0, 0.0);
    let mut min = f32::MAX;
    let tx = tangents[0].x.abs();
    let ty = tangents[0].y.abs();
    let tz = tangents[0].z.abs();

    if tx <= min {
        min = tx;
        normal = vec3(1.0, 0.0, 0.0);
    }

    if ty <= min {
        min = ty;
        normal = vec3(0.0, 1.0, 0.0);
    }

    if tz <= min {
        normal = vec3(0.0, 0.0, 1.0);
    }

    let vec = tangents[0].cross(normal).normalize();
    normals.push(tangents[0].cross(vec).normalize());
    binormals.push(tangents[0].cross(normals[0]).normalize());

    // compute the slowly-varying normal and binormal vectors for each segment on the curve
    for i in 1..=segments {
        normals.push(normals[i - 1]);
        binormals.push(binormals[i - 1]);

        let vec = tangents[i - 1].cross(tangents[i]);

        if vec.magnitude() > f32::EPSILON {
            let vec = vec.normalize();
            let theta = (tangents[i - 1].dot(tangents[i]).clamp(-1.0, 1.0)).acos();
            let rot_mat = Mat4::from_axis_angle(vec, radians(theta));
            normals[i] = rot_mat.transform_vector(normals[i]);
        }

        binormals[i] = tangents[i].cross(normals[i]).normalize();
    }

    // todo closed

    Frame {
        tangents,
        normals,
        binormals,
    }
}
