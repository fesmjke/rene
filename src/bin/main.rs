use rene::{
    tube::tube_s,
    wireframe::{edge_transformations, vertex_transformations},
};
use std::path::Path;

use three_d::{
    AmbientLight, Axes, Camera, ClearState, ColorMaterial, Context, CpuMesh, CpuModel,
    DirectionalLight, EuclideanSpace, FrameOutput, Gm, InnerSpace, Mat4, Mesh, Model, Object,
    OrbitControl, PhysicalMaterial, Point3, Srgba, Vector3, Viewport, Window, WindowSettings,
    degrees, rotation_matrix_from_dir_to_dir, vec3,
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

pub fn arrow_to_dir_pos(pos: Point3<f32>, dir: Vector3<f32>) -> three_d::Matrix4<f32> {
    // for sure
    let dir = dir.normalize();

    let rotation = rotation_matrix_from_dir_to_dir(
        // потому что стрелка направлена вдоль x
        Point3::new(1.0, 0., 0.).to_vec(),
        dir.normalize(),
    );

    let brr = Mat4::from_translation(pos.to_vec())
        * rotation
        * Mat4::from_nonuniform_scale(0.6, 0.01, 0.01);
    brr
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
    let mut show_debug_sphere = false;
    let mut show_debug_arrow = false;

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

    // debug arrow
    let mut arrow = CpuMesh::arrow(0.9, 0.5, 16);

    let mut arrow = Gm::new(
        Mesh::new(&context, &arrow),
        PhysicalMaterial {
            albedo: Srgba::RED,
            ..Default::default()
        },
    );

    // debug sphere
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
                    ui.checkbox(&mut show_debug_arrow, "Display debug arrow");
                    ui.checkbox(&mut show_debug_sphere, "Display debug sphere");

                    if show_debug_sphere {
                        ui.add(
                            Slider::new(&mut sphere_position.x, -1.0..=1.0)
                                .text("Sphere X position"),
                        );
                        ui.add(
                            Slider::new(&mut sphere_position.y, -1.0..=1.0)
                                .text("Sphere Y position"),
                        );
                        ui.add(
                            Slider::new(&mut sphere_position.z, -1.0..=1.0)
                                .text("Sphere Z position"),
                        );
                    }

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

        if show_debug_sphere {
            sphere.set_transformation(Mat4::from_translation(sphere_position));
        }

        if show_debug_arrow && show_debug_sphere {
            let arrow_origin = Point3::new(0., 0., 0.);
            let dir = Point3::from_vec(sphere_position) - arrow_origin;

            let brr = arrow_to_dir_pos(arrow_origin, dir);

            arrow.set_transformation(brr);
        }

        frame_input
            .screen()
            .clear(ClearState::color_and_depth(1., 1., 1., 1.0, 1.0))
            .write(|| {
                if show_axes {
                    axes.render(&camera, &[&ambient, &directional]);
                }

                if show_debug_arrow {
                    arrow.render(&camera, &[&ambient, &directional]);
                }

                if show_debug_sphere {
                    sphere.render(&camera, &[&ambient, &directional]);
                }

                gui.render()
            })
            .unwrap();

        FrameOutput::default()
    });
}
