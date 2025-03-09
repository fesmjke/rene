use three_d::{
    AmbientLight, Axes, Camera, ClearState, ColorMaterial, CpuMaterial, CpuMesh, CpuModel, Cull,
    DirectionalLight, FrameOutput, Gm, Mat4, Matrix4, Mesh, Model, OrbitControl, PhysicalMaterial,
    Srgba, Viewport, Window, WindowSettings, degrees, vec3,
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

    let axes = Gm::new(Axes::new(&context, 0.01, 10.0), ColorMaterial::default());

    let mut load_teapot = three_d_asset::io::load(&["meshes/suzanne.obj"]).unwrap();
    let model: CpuModel = load_teapot.deserialize("suzanne.obj").unwrap();
    let mut suzanne = Model::<PhysicalMaterial>::new(&context, &model).unwrap();

    suzanne.iter_mut().for_each(|m| {
        m.material.render_states.cull = Cull::Back;
        m.set_transformation(Mat4::from_scale(0.2) * Matrix4::from_translation(vec3(0., 0.3, 0.)));
    });

    suzanne.iter_mut().for_each(|m| {
        m.material = PhysicalMaterial::new_opaque(
            &context,
            &CpuMaterial {
                albedo: Srgba::new_opaque(170, 169, 173),
                roughness: 0.7,
                metallic: 0.8,
                ..Default::default()
            },
        )
    });

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
        10.0,
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

        suzanne.iter_mut().for_each(|m| {
            m.set_transformation(
                Mat4::from_translation(vec3(0., 0.5, 0.)) * Mat4::from_scale(model_scale),
            )
        });

        suzanne.iter_mut().for_each(|m| {
            m.material.metallic = metalic;
        });

        directional.generate_shadow_map(1024, &suzanne);

        frame_input
            .screen()
            // Clear the color and depth of the screen render target
            .clear(ClearState::color_and_depth(0.8, 0.8, 0.7, 1.0, 1.0))
            // Render the triangle with the color material which uses the per vertex colors defined at construction
            .render(&camera, suzanne.into_iter(), &[&ambient, &directional])
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
