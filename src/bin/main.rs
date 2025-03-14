use rene::{
    curves::generate_sine_curve,
    tube::tube_s,
    wireframe::{edge_transformations, vertex_transformations},
};
use std::path::Path;

use three_d::{
    AmbientLight, Axes, Camera, ClearState, ColorMaterial, Context, CpuMaterial, CpuMesh, CpuModel,
    Cull, DirectionalLight, EuclideanSpace, FrameOutput, Gm, Indices, InnerSpace, InstancedMesh,
    Mat4, Mesh, Model, Object, One, OrbitControl, PhysicalMaterial, Point3, Positions, Quaternion,
    Rotation, Rotation3, SquareMatrix, Srgba, Vec3, Vector4, Viewport, Window, WindowSettings,
    degrees, radians, vec3,
};

const WINDOW_WIDTH: u32 = 1280;
const WINDOW_HEIGHT: u32 = 720;

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

    // debug menu
    let mut show_axes = false;
    let mut sphere_position = vec3(0.5, 0.3, 0.35);
    let arrow_direction = Vec3::unit_x();
    let arrow_position = vec3(2., 2., 0.);

    let context = window.gl();
    let mut gui = three_d::GUI::new(&context);

    let axes = Gm::new(Axes::new(&context, 0.01, 10.0), ColorMaterial::default());

    let mut models = Vec::new();

    let ambient = AmbientLight::new(&context, 0.4, Srgba::WHITE);
    let directional = DirectionalLight::new(
        &context,
        10.0,
        Srgba::new_opaque(204, 178, 127),
        vec3(0.0, -1.0, -1.0),
    );

    // test arrow
    let direction_vector = vec3(sphere_position.x, sphere_position.y, sphere_position.z);

    println!("{:?}", direction_vector.normalize());

    let rotation = Mat4::look_at_lh(
        Point3::from_vec(arrow_position),
        Point3::from_vec(direction_vector.normalize()),
        Vec3::unit_y(),
    );

    println!("{:?}", rotation);

    let mut arrow = CpuMesh::arrow(0.9, 0.5, 16);
    let brr = Mat4::from_translation(arrow_position) * rotation * Mat4::from_nonuniform_scale(0.5, 0.01, 0.01);
    // let brr = Mat4::from_scale(0.5);
    arrow.transform(brr).unwrap();

    let mut arrow = Gm::new(
        Mesh::new(&context, &arrow),
        PhysicalMaterial {
            albedo: Srgba::RED,
            ..Default::default()
        },
    );

    // test sphere
    let mut sphere = CpuMesh::sphere(16);
    let mv = Mat4::from_scale(0.1);
    sphere.transform(mv).unwrap();
    let mut sphere = Gm::new(Mesh::new(&context, &sphere), PhysicalMaterial::default());

    // camera part
    let camera_position = vec3(0., 0., 2.);
    let target = vec3(0., 0.0, 0.);
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
                    ui.checkbox(&mut show_axes, "Display axes");
                    ui.add(
                        Slider::new(&mut sphere_position.x, -1.0..=1.0).text("Sphere X position"),
                    );
                    ui.add(
                        Slider::new(&mut sphere_position.y, -1.0..=1.0).text("Sphere Y position"),
                    );
                    ui.add(
                        Slider::new(&mut sphere_position.z, -1.0..=1.0).text("Sphere Z position"),
                    );

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
        sphere.set_transformation(Mat4::from_translation(sphere_position));

        // rotate arrow
        // assume that arrow at the origin (0., 0., 0.)
        // let initial_direction = arrow_direction; // (1., 0., 0.);
        // let direction_vector= sphere_position - arrow_position;
        // let norm_direction_vector = direction_vector.normalize();

        // let rotation_axis = Vec3::cross( initial_direction, norm_direction_vector);

        // let norm_rotation_axis = rotation_axis.normalize();

        // let dot = Vec3::dot(initial_direction, norm_direction_vector);

        // let rotation_angle = dot.acos();

        // let degree = rotation_angle * 180.0 / std::f32::consts::PI;

        // let rotation = if norm_rotation_axis.x.is_nan() || norm_rotation_axis.y.is_nan() || norm_rotation_axis.y.is_nan() {
        //     // why this method ignores me when 3.1415927 180.0° Vector3 [-1.0, 0.0, 0.0] and not flipping x?
        //     Mat4::from_axis_angle(norm_direction_vector, degrees(degree))
        // } else {
        //     Mat4::from_axis_angle(norm_rotation_axis, degrees(degree))
        // };

        // arrow.set_transformation(rotation);

        frame_input
            .screen()
            .clear(ClearState::color_and_depth(1., 1., 1., 1.0, 1.0))
            .render(&camera, &arrow, &[&ambient, &directional])
            .render(&camera, &sphere, &[&ambient, &directional])
            .write(|| {
                if show_axes {
                    axes.render(&camera, &[&ambient, &directional]);
                }

                gui.render()
            })
            .unwrap();

        FrameOutput::default()
    });
}
