use three_d::{CpuMesh, InnerSpace, Mat4, Quat, Rotation3, SquareMatrix, Vec3, radians};

pub fn two_points(start: Vec3, direction: Vec3, distance: f32) -> (CpuMesh, CpuMesh) {
    let mut start_sphere = CpuMesh::sphere(8);
    start_sphere
        .transform(Mat4::from_translation(start) * Mat4::from_scale(0.01))
        .unwrap();

    let mut end_sphere = CpuMesh::sphere(8);

    let end_position = start + (direction * distance);

    end_sphere
        .transform(Mat4::from_translation(end_position) * Mat4::from_scale(0.01))
        .unwrap();

    (start_sphere, end_sphere)
}

    // // tangent GREEN
    // for tangent in tube.tangents.iter() {
    //     let (mut start, mut end) = two_points(tangent.point, tangent.direction, 0.1);

    //     start.transform(Mat4::from_translation(y_offset)).unwrap();
    //     end.transform(Mat4::from_translation(y_offset)).unwrap();

    //     let sphere_s = Gm::new(
    //         Mesh::new(&context, &start),
    //         PhysicalMaterial {
    //             albedo: Srgba::RED,
    //             ..Default::default()
    //         },
    //     );

    //     let sphere_e = Gm::new(
    //         Mesh::new(&context, &end),
    //         PhysicalMaterial {
    //             albedo: Srgba::GREEN,
    //             ..Default::default()
    //         },
    //     );

    //     vectors.push(sphere_s);
    //     vectors.push(sphere_e);
    // }

    // // normal BLUE
    // for normal in tube.normals.iter() {
    //     let (mut start, mut end) = two_points(normal.point, normal.direction, 0.1);

    //     start.transform(Mat4::from_translation(y_offset)).unwrap();

    //     end.transform(Mat4::from_translation(y_offset)).unwrap();

    //     let sphere_s = Gm::new(
    //         Mesh::new(&context, &start),
    //         PhysicalMaterial {
    //             albedo: Srgba::RED,
    //             ..Default::default()
    //         },
    //     );

    //     let sphere_e = Gm::new(
    //         Mesh::new(&context, &end),
    //         PhysicalMaterial {
    //             albedo: Srgba::BLUE,
    //             ..Default::default()
    //         },
    //     );

    //     vectors.push(sphere_s);
    //     vectors.push(sphere_e);
    // }

    // // binormal WHITE
    // for binormal in tube.binormals.iter() {
    //     let (mut start, mut end) = two_points(binormal.point, binormal.direction, 0.1);

    //     start.transform(Mat4::from_translation(y_offset)).unwrap();

    //     end.transform(Mat4::from_translation(y_offset)).unwrap();

    //     let sphere_s = Gm::new(
    //         Mesh::new(&context, &start),
    //         PhysicalMaterial {
    //             albedo: Srgba::RED,
    //             ..Default::default()
    //         },
    //     );

    //     let sphere_e = Gm::new(
    //         Mesh::new(&context, &end),
    //         PhysicalMaterial {
    //             albedo: Srgba::WHITE,
    //             ..Default::default()
    //         },
    //     );

    //     vectors.push(sphere_s);
    //     vectors.push(sphere_e);
    // }