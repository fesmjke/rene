use three_d::{InnerSpace, MetricSpace, Vec3, Vector3};

pub struct FrenetFrame {
    pub tangents: Vec<Vec3>,
    pub normals: Vec<Vec3>,
    pub binormals: Vec<Vec3>,
}

/// Computes a perpendicular vector to `v` that is as close as possible to `preferred_direction`.
fn perpendicular_vector(v: Vector3<f32>, preferred_direction: Vector3<f32>) -> Vector3<f32> {
    let dot_product = v.dot(preferred_direction);
    let v_norm_sq = v.magnitude2();

    // Project preferred_direction onto v
    let projection = (dot_product / v_norm_sq) * v;

    // Subtract projection to get a perpendicular component
    preferred_direction - projection
}

pub trait Curve {
    /// Returns the number of divisions for arc length calculations.
    fn arc_length_divisions(&self) -> usize {
        200 // magic constant?
    }

    /// Returns a vector in 2D or 3D space for the given interpolation factor.
    /// Must be implemented by the concrete curve type.
    fn get_point(&self, t: f32) -> Option<Vec3>;

    /// Returns a vector in 2D or 3D space for the given interpolation factor,
    /// honoring the length of the curve for equidistant samples.
    fn get_point_at(&self, u: f32) -> Option<Vec3> {
        let t = self.get_u_to_t_mapping(u, None);
        self.get_point(t)
    }

    /// Samples the curve and returns a vector of points representing the curve shape.
    fn get_points(&self, divisions: usize) -> Vec<Vec3> {
        let mut points = Vec::new();
        let divisions = divisions as f32;
        for d in 0..=divisions as usize {
            let t = d as f32 / divisions;
            if let Some(point) = self.get_point(t) {
                points.push(point);
            }
        }
        points
    }

    /// Returns the total arc length of the curve.
    fn get_length(&self) -> f32 {
        let lengths = self.get_lengths(None);
        lengths[lengths.len() - 1]
    }

    /// Returns an array of cumulative segment lengths of the curve.
    fn get_lengths(&self, divisions: Option<usize>) -> Vec<f32> {
        let divisions = divisions.unwrap_or(self.arc_length_divisions());
        let mut cache = Vec::with_capacity(divisions + 1);
        let mut sum = 0.0;
        let mut last = self.get_point(0.0).expect("Failed to get initial point");

        cache.push(0.0);

        for p in 1..=divisions {
            let t = p as f32 / divisions as f32;
            let current = self.get_point(t).expect("Failed to get point");
            sum += current.distance(last);
            cache.push(sum);
            last = current;
        }

        cache
    }

    /// Maps a u value to a t value for equidistant sampling.
    fn get_u_to_t_mapping(&self, u: f32, distance: Option<f32>) -> f32 {
        let arc_lengths = self.get_lengths(None);
        let il = arc_lengths.len();
        let target_arc_length = distance.unwrap_or(u * arc_lengths[il - 1]);

        let mut low = 0;
        let mut high = il - 1;
        let mut i;

        while low <= high {
            i = low + (high - low) / 2;
            let comparison = arc_lengths[i] - target_arc_length;

            if comparison < 0.0 {
                low = i + 1;
            } else if comparison > 0.0 {
                high = i - 1;
            } else {
                high = i;
                break;
            }
        }

        i = high;

        if arc_lengths[i] == target_arc_length {
            return i as f32 / (il - 1) as f32;
        }

        let length_before = arc_lengths[i];
        let length_after = arc_lengths[i + 1];
        let segment_length = length_after - length_before;
        let segment_fraction = (target_arc_length - length_before) / segment_length;

        (i as f32 + segment_fraction) / (il - 1) as f32
    }

    /// Returns a unit vector tangent for the given interpolation factor.
    fn get_tangent(&self, t: f32) -> Vec3 {
        let delta = 0.0001;
        let t1 = (t - delta).max(0.0);
        let t2 = (t + delta).min(1.0);

        let pt1 = self.get_point(t1).expect("Failed to get point");
        let pt2 = self.get_point(t2).expect("Failed to get point");

        (pt2 - pt1).normalize()
    }

    /// Returns a unit vector tangent for the given interpolation factor with equidistant samples.
    fn get_tangent_at(&self, u: f32) -> Vec3 {
        let t = self.get_u_to_t_mapping(u, None);
        self.get_tangent(t)
    }

    /// Generates the Frenet Frames for the curve in 3D space.
    fn compute_frenet_frames(&self, segments: usize, closed: bool) -> FrenetFrame {
        let mut tangents = Vec::with_capacity(segments + 1);
        let mut normals = Vec::with_capacity(segments + 1);
        let mut binormals = Vec::with_capacity(segments + 1);

        for i in 0..=segments {
            let u = i as f32 / segments as f32;
            tangents.push(self.get_tangent_at(u));
        }

        let p = self.get_point(0.).unwrap();

        let camera = Vector3::new(0., 0., 2.);

        let direction = camera - p;

        let normal = perpendicular_vector(tangents[0], direction);

        normals.push(normal.normalize());
        binormals.push(tangents[0].cross(normals[0]));

        // Compute subsequent normals and binormals
        for i in 1..=segments {
            let u = i as f32 / segments as f32;

            let p = self.get_point(u).unwrap();

            let camera = Vector3::new(0., 0., 2.);

            let direction = camera - p;

            let normal = perpendicular_vector(tangents[0], direction);
            normals.push(normal.normalize());
            binormals.push(tangents[i].cross(normals[i]));
        }

        //  later if needed for closed curves
        // if closed {
        //     let mut theta = normals[0].dot(normals[segments]).clamp(-1.0, 1.0) / segments as f32;
        //     if tangents[0].dot(normals[0].cross(normals[segments])) > 0.0 {
        //         theta = -theta;
        //     }
        //     for i in 1..=segments {
        //         let rot_mat = Mat4::from_axis_angle(tangents[i], radians(theta * i as f32));
        //         normals[i] = rot_mat.transform_vector(normals[i]);
        //         binormals[i] = tangents[i].cross(normals[i]);
        //     }
        // }

        FrenetFrame {
            tangents,
            normals,
            binormals,
        }
    }
}

pub struct SineCurve;

impl Curve for SineCurve {
    fn get_point(&self, t: f32) -> Option<Vec3> {
        let tx = t * 3.0 - 1.5;
        let ty = (2. * std::f32::consts::PI * t).sin();
        let tz = (2. * std::f32::consts::PI * t).cos();

        Some(Vec3::new(tx, ty, tz))
    }
}
