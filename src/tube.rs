use three_d::{CpuMesh, Indices, InnerSpace, Positions, Vec3};

type Path = Vec<Vec3>;

struct Plane {
    pub normal: Vec3,
    pub d: f32,
}

impl Plane {
    pub fn new(point: Vec3, normal: Vec3) -> Self {
        let d = -(normal.x * point.x + normal.y * point.y + normal.z * point.z);

        Self { normal, d }
    }
}

struct Line {
    pub direction: Vec3,
    pub point: Vec3,
}

impl Line {
    pub fn new(direction: Vec3, point: Vec3) -> Self {
        Self { direction, point }
    }
}

fn plane_vector_intersect(line: Line, plane: Plane) -> Vec3 {
    let dot1 = plane.normal.dot(line.direction);
    let dot2 = plane.normal.dot(line.point);

    println!("{:?}", dot1);

    if dot1 == 0.0 {
        return Vec3::new(0., 0., 0.);
    }

    let t = -(dot2 + plane.d) / dot1;

    return line.point + (t * line.direction);
}

//  extruding a contour around path
pub fn tube(path: &Path, angle_subdivisions: i32) -> (Vec<Vec3>, CpuMesh) {
    // for now

    let mut positions = vec![];
    let mut indices = vec![];

    const CHUNK_SIZE: usize = 3;

    for q in path.chunks(CHUNK_SIZE) {
        let mut p1 = vec![];
        let mut p2 = vec![];
        let mut p3 = vec![];

        let v1 = q[1] - q[0]; // direction
        let v2 = q[2] - q[1]; // direction
        let normal = v1 + v2; // plane normal p2 to p3

        println!("{:?}", v1);

        let v1_norm = v1.normalize();

        let arbitrary = if v1_norm.x.abs() < 0.9 {
            Vec3::unit_x()
        } else {
            Vec3::unit_y()
        };

        let u = v1_norm.cross(arbitrary).normalize();
        let v = v1_norm.cross(u).normalize();

        for j in 0..angle_subdivisions {
            let angle = 2.0 * std::f32::consts::PI * j as f32 / angle_subdivisions as f32;

            let point = q[0] + (angle.cos() * u + angle.sin() * v);
            p1.push(point); // p1
        }

        println!("{:?}", p1);

        for p1_point in p1.iter() {
            let plane = Plane::new(q[1], normal);
            let line = Line::new(v1, *p1_point);

            let intersection = plane_vector_intersect(line, plane);

            p2.push(intersection);
        }

        for p2_point in p2.iter() {
            let plane = Plane::new(q[2], normal);
            let line = Line::new(v2, *p2_point);

            let intersection = plane_vector_intersect(line, plane);

            p3.push(intersection);
        }

        positions.extend(p1);
        positions.extend(p2);
        positions.extend(p3);
    }

    for i in 0..(path.len() - 1) as i32 {
        for j in 0..angle_subdivisions {
            indices.push((i * angle_subdivisions + j) as u16);
            indices.push((i * angle_subdivisions + (j + 1) % angle_subdivisions) as u16);
            indices.push(((i + 1) * angle_subdivisions + (j + 1) % angle_subdivisions) as u16);

            indices.push((i * angle_subdivisions + j) as u16);
            indices.push(((i + 1) * angle_subdivisions + (j + 1) % angle_subdivisions) as u16);
            indices.push(((i + 1) * angle_subdivisions + j) as u16);
        }
    }

    println!("{:?}", positions);
    println!("{:?}", indices);

    let mut mesh: CpuMesh = CpuMesh {
        positions: Positions::F32(positions.clone()),
        indices: Indices::U16(indices),
        ..Default::default()
    };

    mesh.compute_normals();

    (positions, mesh)
}
