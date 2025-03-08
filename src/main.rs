use three_d::{
    AmbientLight, Angle, Axes, Camera, ClearState, ColorMaterial, CpuMaterial, CpuMesh, Cull,
    DirectionalLight, FrameOutput, Geometry, Gm, Mat4, Mesh, Model, PhysicalMaterial, Srgba,
    Viewport, Window, WindowSettings, degrees, radians, vec3,
};

const WINDOW_WIDTH: u32 = 1280;
const WINDOW_HEIGHT: u32 = 720;

fn main() {
    let application_title = String::from("Example");

    let window = Window::new(WindowSettings {
        title: application_title,
        min_size: (WINDOW_WIDTH, WINDOW_HEIGHT),
        max_size: Some((WINDOW_WIDTH, WINDOW_HEIGHT)),
        ..Default::default()
    })
    .unwrap();

    let context = window.gl();
    let mut gui = three_d::GUI::new(&context);

    let axes = Gm::new(Axes::new(&context, 0.01, 0.5), ColorMaterial::default());

    let ambient = AmbientLight::new(&context, 0.4, Srgba::WHITE);
    let directional = DirectionalLight::new(&context, 2.0, Srgba::WHITE, vec3(-1.0, -1.0, -1.0));

    let mut load_teapot = three_d_asset::io::load(&["meshes/teapot.obj"]).unwrap();
    let model = load_teapot.deserialize("teapot.obj").unwrap();
    let mut teapot: Gm<_, _> = Model::<PhysicalMaterial>::new(&context, &model)
        .unwrap()
        .remove(0)
        .into();
    teapot.material.render_states.cull = Cull::Back;
    teapot.set_transformation(Mat4::from_translation(vec3(2.0, -2.0, 0.0)));

    let mut model_scale = 0.3;
    let mut time_scale = 0.005;

    let mut cpu_plane = CpuMesh::square();

    cpu_plane
        .transform(
            Mat4::from_translation(vec3(0.0, -1.0, 0.0))
                * Mat4::from_scale(10.0)
                * Mat4::from_angle_x(degrees(-90.0)),
        )
        .unwrap();

    let mut plane = Gm::new(
        Mesh::new(&context, &cpu_plane),
        PhysicalMaterial::new_opaque(
            &context,
            &CpuMaterial {
                albedo: Srgba::new_opaque(128, 200, 70),
                ..Default::default()
            },
        ),
    );

    teapot.set_animation(move |time| {
        Mat4::new(
            1.,
            0.,
            0.,
            0.,
            0.,
            radians(time).cos(),
            -radians(time).sin(),
            0.,
            0.,
            radians(time).sin(),
            radians(time).cos(),
            0.,
            0.,
            0.,
            0.,
            1.,
        ) * Mat4::from_angle_y(radians(time))
            * Mat4::from_translation(vec3((time).sin(), 0., 0.))
    });

    let mut camera_position = vec3(0., 0., 2.);
    let target = vec3(0., 0., 0.);
    let up_v = vec3(0., 1., 0.);

    let mut camera = Camera::new_perspective(
        window.viewport(),
        camera_position,
        target,
        up_v,
        degrees(45.0),
        0.1,
        10.0,
    );

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
                    ui.add(
                        Slider::new(&mut camera_position.x, -5.0..=5.0).text("Camera x position"),
                    );
                    ui.add(
                        Slider::new(&mut camera_position.y, -5.0..=5.0).text("Camera y position"),
                    );
                    ui.add(
                        Slider::new(&mut camera_position.z, -5.0..=5.0).text("Camera z position"),
                    );
                    ui.add(Slider::new(&mut model_scale, 0.3..=0.1).text("Model scale"));
                    ui.add(Slider::new(&mut time_scale, 0.005..=0.00005).text("Time scale"));
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
        camera.set_view(camera_position, target, up_v);

        teapot.set_transformation(Mat4::from_scale(model_scale));
        teapot.animate(frame_input.accumulated_time as f32 * time_scale);

        frame_input
            .screen()
            // Clear the color and depth of the screen render target
            .clear(ClearState::color_and_depth(0., 0., 0., 1.0, 1.0))
            // Render the triangle with the color material which uses the per vertex colors defined at construction
            .render(
                &camera,
                teapot.into_iter().chain(&plane).chain(&axes),
                &[&ambient, &directional],
            )
            .write(|| gui.render())
            .unwrap();

        FrameOutput::default()
    });
}
