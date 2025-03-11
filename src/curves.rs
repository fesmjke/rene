use three_d::{InnerSpace, Vec3};

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