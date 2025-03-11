use core::f32;
use std::path::Path;
use three_d::{
    AmbientLight, Axes, Camera, ClearState, ColorMaterial, Context, CpuMaterial, CpuMesh, CpuModel,
    Cull, DirectionalLight, FrameOutput, Gm, Indices, InnerSpace, Mat4, Matrix4, Mesh, Model,
    OrbitControl, PhysicalMaterial, Positions, Srgba, Vec3, Viewport, Window, WindowSettings,
    degrees, vec3,
};
use three_d_asset::TriMesh;

const WINDOW_WIDTH: u32 = 1280;
const WINDOW_HEIGHT: u32 = 720;

type TubePath = Vec<Vec3>;

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
fn tube(path: &TubePath, angle_subdivisions: i32) -> (Vec<Vec3>, TriMesh) {
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

        let arbitrary = if v1_norm.x.abs() < 0.9 { Vec3::unit_x() } else { Vec3::unit_y() };

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

    let mut mesh = TriMesh {
        positions: Positions::F32(positions.clone()),
        indices: Indices::U16(indices),
        ..Default::default()
    };

    mesh.compute_normals();

    (positions, mesh)
}

fn tube_path(path: &TubePath, angle_subdivisions: i32, scale: f32) -> TriMesh {
    let mut positions: Vec<Vec3> = Vec::new();
    let mut indices = Vec::new();

    for point in path.iter() {
        for j in 0..angle_subdivisions {
            let angle = 2.0 * std::f32::consts::PI * j as f32 / angle_subdivisions as f32;

            positions.push(Vec3::new(
                point.x * scale,
                (point.y + angle.cos()) * scale,
                (point.z + angle.sin()) * scale,
            ));
        }
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

    let mut mesh = TriMesh {
        positions: Positions::F32(positions),
        indices: Indices::U16(indices),
        ..Default::default()
    };

    mesh.compute_normals();

    mesh
}

// sync model creation
fn create_model(path: &Path, context: &Context) -> Gm<Mesh, PhysicalMaterial> {
    let model_name = path.file_name().expect("No file name");

    let mut load = three_d_asset::io::load(&[path]).expect(&format!("Unable to load {:?}", path));

    let cpu_model: CpuModel = load
        .deserialize(model_name)
        .expect(&format!("Unable to deserialize {:?}", model_name));

    let model = Model::<PhysicalMaterial>::new(&context, &cpu_model)
        .unwrap()
        .remove(0)
        .into();

    model
}

fn generate_sine_curve(
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
        vec3(1., 0., 0.)
    } else {
        vec3(0., 1., 0.)
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

fn main() {
    let application_title = String::from("Example");
    let current_dir = std::env::current_dir().unwrap();

    let window = Window::new(WindowSettings {
        title: application_title,
        min_size: (WINDOW_WIDTH, WINDOW_HEIGHT),
        max_size: Some((WINDOW_WIDTH, WINDOW_HEIGHT)),
        ..Default::default()
    })
    .unwrap();

    let context = window.gl();
    let mut gui = three_d::GUI::new(&context);

    let axes = Gm::new(Axes::new(&context, 0.01, 10.0), ColorMaterial::default());

    let mut suzanne = create_model(&Path::new("meshes/suzanne.obj"), &context);
    suzanne.material.render_states.cull = Cull::Back;
    suzanne
        .set_transformation(Mat4::from_scale(0.2) * Matrix4::from_translation(vec3(0., 0.3, 0.)));

    let mut models = Vec::new();

    let ambient = AmbientLight::new(&context, 0.4, Srgba::WHITE);
    let mut directional = DirectionalLight::new(
        &context,
        10.0,
        Srgba::new_opaque(204, 178, 127),
        vec3(0.0, -1.0, -1.0),
    );

    let mut model_scale = 0.3;
    let mut cpu_plane = CpuMesh::square();

    cpu_plane
        .transform(
            Mat4::from_translation(vec3(0.0, 0.0, 0.0))
                * Mat4::from_scale(10.0)
                * Mat4::from_angle_x(degrees(-90.0)),
        )
        .unwrap();

    let plane = Gm::new(
        Mesh::new(&context, &cpu_plane),
        PhysicalMaterial::new_opaque(
            &context,
            &CpuMaterial {
                albedo: Srgba::new_opaque(128, 0, 70),
                ..Default::default()
            },
        ),
    );

    let mut cylinder = Gm::new(
        Mesh::new(&context, &CpuMesh::cylinder(3)),
        PhysicalMaterial::new_opaque(
            &context,
            &CpuMaterial {
                albedo: Srgba {
                    r: 0,
                    g: 255,
                    b: 0,
                    a: 200,
                },
                ..Default::default()
            },
        ),
    );

    cylinder.set_transformation(
        Mat4::from_angle_z(degrees(90.0)) * Mat4::from_translation(vec3(0., 0., 0.)),
    );

    let sphere_sub = 16;

    let start = Vec3::new(0.0, 0.0, 0.0); // Start position
    let direction = Vec3::new(10.0, 0.0, 0.0); // Along X-axis
    let amplitude = 1.0;
    let period = 2.0 * f32::consts::PI / 1.0; // One full wave every 5 units
    let length = 24.0; // Total length of the sine wave
    let points_count = 16; // Number of points to generate

    let sin_path = generate_sine_curve(start, direction, amplitude, period, length, points_count);

    // scale applied to whole tube
    // whole tube can be splitted in separate cylinders -> Vec<TriMesh>
    let sin_tri_tube = tube_path(&sin_path, 16, 1.);
    let sin_tube = Gm::new(
        Mesh::new(&context, &sin_tri_tube),
        PhysicalMaterial::default(),
    );

    // models.push(sin_tube);

    let tube_v2_path = vec![vec3(0., 0., 0.), vec3(0., 2., 0.), vec3(2., 2., 0.)];

    let tube_v2 = tube_path_v2(&tube_v2_path, 4);

    let tube_v2_model = Gm::new(Mesh::new(&context, &tube_v2.1), PhysicalMaterial::default());

    for point in tube_v2.0.iter() {
        let mut debug_sphere = Gm::new(
            Mesh::new(&context, &CpuMesh::sphere(sphere_sub)),
            PhysicalMaterial {
                albedo: Srgba::RED,
                ..Default::default()
            },
        );

        debug_sphere.set_transformation(Mat4::from_translation(*point) * Mat4::from_scale(0.05));

        models.push(debug_sphere);
    }

    models.push(tube_v2_model);

    // let arc_tube = tube_path(
    //     &vec![
    //         vec3(-1., 0., 0.),
    //         vec3(0., 2., 0.),
    //         vec3(1., 2., 0.),
    //         vec3(2., 2., 0.),
    //         vec3(2., 0., 0.),
    //     ],
    //     16,
    //     1.0,
    // );
    // let arc_tube = Gm::new(Mesh::new(&context, &arc_tube), PhysicalMaterial::default());

    // models.push(arc_tube);

    let camera_position = vec3(0., 0., 2.);
    let target = vec3(0., 0.5, 0.);
    let up_v = vec3(0., 1., 0.);

    let mut camera = Camera::new_perspective(
        window.viewport(),
        camera_position,
        target,
        up_v,
        degrees(45.0),
        0.1,
        100.0,
    );

    let mut control = OrbitControl::new(camera.target(), 1.0, 100.0);
    let mut metalic = 0.0;

    window.render_loop(move |mut frame_input| {
        let mut panel_width = 0.0;

        gui.update(
            &mut frame_input.events,
            frame_input.accumulated_time,
            frame_input.viewport,
            frame_input.device_pixel_ratio,
            |gui_context| {
                use three_d::egui::*;

                SidePanel::left("side_panel").show(gui_context, |ui| {
                    ui.heading("Debug Panel");

                    ui.add(Slider::new(&mut metalic, 0.0..=1.0).text("Model metalic"));
                    ui.add(Slider::new(&mut model_scale, 0.3..=0.1).text("Model scale"));

                    if ui.button("Select file").clicked() {
                        // block draw thread
                        let response = rfd::FileDialog::new()
                            .set_directory(&current_dir)
                            .pick_file();

                        match response {
                            Some(buf) => {
                                let new_model = create_model(buf.as_path(), &context);
                                models.push(new_model);
                            }
                            None => {}
                        }
                    }
                });

                panel_width = gui_context.used_rect().width();
            },
        );

        let viewport = Viewport {
            x: (panel_width * frame_input.device_pixel_ratio) as i32,
            y: 0,
            width: frame_input.viewport.width
                - (panel_width * frame_input.device_pixel_ratio) as u32,
            height: frame_input.viewport.height,
        };

        camera.set_viewport(viewport);
        control.handle_events(&mut camera, &mut frame_input.events);

        suzanne.set_transformation(
            Mat4::from_translation(vec3(0., 0.5, 0.)) * Mat4::from_scale(model_scale),
        );
        suzanne.material.metallic = metalic;

        directional.generate_shadow_map(1024, &suzanne);

        frame_input
            .screen()
            // Clear the color and depth of the screen render target
            .clear(ClearState::color_and_depth(0.8, 0.8, 0.7, 1.0, 1.0))
            // Render the triangle with the color material which uses the per vertex colors defined at construction
            .render(&camera, &suzanne, &[&ambient, &directional])
            .render(&camera, &models, &[&ambient])
            .render(
                &camera,
                plane.into_iter().chain(&axes),
                &[&ambient, &directional],
            )
            .write(|| gui.render())
            .unwrap();

        FrameOutput::default()
    });
}
