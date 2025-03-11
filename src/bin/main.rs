use rene::tube;
use std::path::Path;

use three_d::{
    AmbientLight, Axes, Camera, ClearState, ColorMaterial, Context, CpuMaterial, CpuMesh, CpuModel,
    DirectionalLight, FrameOutput, Gm, Mat4, Mesh, Model, OrbitControl, PhysicalMaterial, Srgba,
    Viewport, Window, WindowSettings, degrees, vec3,
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

    let sphere_sub = 16;

    let path = vec![vec3(0., 0., 0.), vec3(0., 2., 0.), vec3(2., 2., 0.)];
    let (tube_vertex_positions, cpu_tube) = tube(&path, 4);
    let gm_tube = Gm::new(Mesh::new(&context, &cpu_tube), PhysicalMaterial::default());

    for point in tube_vertex_positions.iter() {
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

    models.push(gm_tube);

    // camera part
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

        frame_input
            .screen()
            // Clear the color and depth of the screen render target
            .clear(ClearState::color_and_depth(0.8, 0.8, 0.7, 1.0, 1.0))
            // Render the triangle with the color material which uses the per vertex colors defined at construction
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
