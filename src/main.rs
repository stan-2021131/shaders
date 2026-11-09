use ::shaders::buffer::{Color, DepthBuffer, Framebuffer};
use ::shaders::camera::Camera;
use ::shaders::math::{Mat4, Vec3};
use ::shaders::mesh::Mesh;
use ::shaders::pipeline::shaders::FlatColorShader;
use ::shaders::pipeline::{BlendMode, RenderContext, RenderTarget, SoftwareRenderer};
use raylib::ffi;
use raylib::prelude::{Color as RColor, Image, KeyboardKey, RaylibDraw, RaylibTexture2D};
use std::path::Path;

const WINDOW_WIDTH: usize = 800;
const WINDOW_HEIGHT: usize = 600;

fn main() {
    let (mut rl, thread) = raylib::init()
        .size(WINDOW_WIDTH as i32, WINDOW_HEIGHT as i32)
        .title("Lab Shaders - Sistema Solar (Fase 2)")
        .build();

    rl.set_target_fps(60);

    // Carga de la geometría base de la esfera
    let model_path = Path::new("assets/models/sphere.obj");
    let sphere_mesh = Mesh::load_obj(model_path).unwrap_or_else(|err| {
        panic!(
            "No se pudo cargar la malla de la esfera desde {:?}: {}",
            model_path, err
        );
    });

    println!(
        "[RECURSOS] Malla cargada exitosamente: {} vertices, {} triangulos",
        sphere_mesh.vertices.len(),
        sphere_mesh.indices.len() / 3
    );

    // Inicialización del pipeline de renderizado y buffers en CPU
    let mut renderer = SoftwareRenderer::new();
    let mut framebuffer = Framebuffer::new(WINDOW_WIDTH, WINDOW_HEIGHT);
    let mut depth_buffer = DepthBuffer::new(WINDOW_WIDTH, WINDOW_HEIGHT);

    // Cámara orbital centrada en el origen (0, 0, 0)
    let aspect = WINDOW_WIDTH as f32 / WINDOW_HEIGHT as f32;
    let mut camera = Camera::new(
        Vec3::zero(),
        4.5,
        0.0,
        0.2,
        45.0,
        aspect,
        0.1,
        100.0,
    );

    // Textura intermedia de Raylib para presentar el framebuffer de CPU
    let raw_image = unsafe {
        Image::from_raw(ffi::GenImageColor(
            WINDOW_WIDTH as i32,
            WINDOW_HEIGHT as i32,
            ffi::Color {
                r: 0,
                g: 0,
                b: 0,
                a: 255,
            },
        ))
    };
    let mut screen_texture = rl
        .load_texture_from_image(&thread, &raw_image)
        .expect("Error al inicializar la textura en GPU para Raylib");

    let shader = FlatColorShader::new(Color::new(245, 175, 40, 255));
    let mut sphere_rotation: f32 = 0.0;

    println!("============================================================");
    println!("[Fase 2: Visualizacion de la esfera]");
    println!("  [Flecha Izq / Der] : Orbitar horizontalmente alrededor de la esfera");
    println!("  [Flecha Arr / Aba] : Orbitar verticalmente (pitch)");
    println!("  [Escape]           : Cerrar la ventana");
    println!("============================================================");

    while !rl.window_should_close() {
        let dt = rl.get_frame_time().min(0.05);

        // Control orbital de la cámara con las flechas
        let orbit_speed = 1.6;
        if rl.is_key_down(KeyboardKey::KEY_LEFT) {
            camera.rotate_orbit(-orbit_speed * dt, 0.0);
        }
        if rl.is_key_down(KeyboardKey::KEY_RIGHT) {
            camera.rotate_orbit(orbit_speed * dt, 0.0);
        }
        if rl.is_key_down(KeyboardKey::KEY_UP) {
            camera.rotate_orbit(0.0, orbit_speed * dt);
        }
        if rl.is_key_down(KeyboardKey::KEY_DOWN) {
            camera.rotate_orbit(0.0, -orbit_speed * dt);
        }

        // Rotación continua de la esfera sobre su propio eje Y
        sphere_rotation += 0.8 * dt;

        // Limpieza de buffers (espacio profundo oscuro)
        framebuffer.clear(Color::new(12, 12, 20, 255));
        depth_buffer.clear();

        // Construcción de matrices de transformación
        let model_matrix = Mat4::rotation_y(sphere_rotation);
        let rotation_matrix = Mat4::rotation_y(sphere_rotation);
        let vp_matrix = camera.view_projection_matrix();

        let ctx = RenderContext::new(
            &model_matrix,
            &rotation_matrix,
            &vp_matrix,
            &shader,
        ).with_blend_mode(BlendMode::Opaque);

        let mut target = RenderTarget {
            framebuffer: &mut framebuffer,
            depth_buffer: &mut depth_buffer,
        };

        // Renderizado 3D por software en CPU
        renderer.render_mesh(&sphere_mesh, &ctx, &mut target);

        // Actualización y presentación en Raylib
        screen_texture
            .update_texture(framebuffer.as_bytes())
            .expect("Error al transferir framebuffer a la textura de Raylib");

        let mut d = rl.begin_drawing(&thread);
        d.clear_background(RColor::BLACK);
        d.draw_texture(&screen_texture, 0, 0, RColor::WHITE);

        // Overlay informativo
        d.draw_text(
            "Fase 2: Esfera base (Sol) en el origen",
            20,
            20,
            20,
            RColor::GOLD,
        );
        d.draw_text(
            "Flechas: Orbitar camara | Rotacion axial continua activa",
            20,
            46,
            16,
            RColor::RAYWHITE,
        );
    }
}
