use ::shaders::buffer::{Color, DepthBuffer, Framebuffer};
use ::shaders::camera::Camera;
use ::shaders::math::{Mat4, Vec3};
use ::shaders::mesh::Mesh;
use ::shaders::pipeline::shaders::{RockyPlanetShader, SunShader};
use ::shaders::pipeline::{BlendMode, RenderContext, RenderTarget, SoftwareRenderer};
use raylib::ffi;
use raylib::prelude::{Color as RColor, Image, KeyboardKey, RaylibDraw, RaylibTexture2D};
use std::path::Path;

const WINDOW_WIDTH: usize = 800;
const WINDOW_HEIGHT: usize = 600;

fn main() {
    let (mut rl, thread) = raylib::init()
        .size(WINDOW_WIDTH as i32, WINDOW_HEIGHT as i32)
        .title("Lab Shaders - Sistema Solar")
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

    // Inicialización del pipeline de software en CPU
    let mut renderer = SoftwareRenderer::new();
    let mut framebuffer = Framebuffer::new(WINDOW_WIDTH, WINDOW_HEIGHT);
    let mut depth_buffer = DepthBuffer::new(WINDOW_WIDTH, WINDOW_HEIGHT);

    // Cámara orbital panorámica que abarca al Sol y la órbita del planeta rocoso
    let aspect = WINDOW_WIDTH as f32 / WINDOW_HEIGHT as f32;
    let mut camera = Camera::new(
        Vec3::zero(),
        8.2,
        0.3,
        0.4,
        45.0,
        aspect,
        0.1,
        150.0,
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

    // Shaders procedurales en CPU
    let mut sun_shader = SunShader::new(Vec3::zero());
    let mut rocky_shader = RockyPlanetShader::new(42, Vec3::zero());

    let mut sim_time: f32 = 0.0;

    println!("============================================================");
    println!("[Sistema Solar: Sol y Planeta Rocoso]");
    println!("  [Flecha Izq / Der] : Orbitar cámara horizontalmente");
    println!("  [Flecha Arr / Aba] : Ajustar elevación (pitch)");
    println!("  [Escape]           : Cerrar la ventana");
    println!("============================================================");

    while !rl.window_should_close() {
        let dt = rl.get_frame_time().min(0.05);
        sim_time += dt;

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

        // Limpieza de buffers (espacio profundo oscuro)
        framebuffer.clear(Color::new(10, 10, 18, 255));
        depth_buffer.clear();

        let vp_matrix = camera.view_projection_matrix();

        // =====================================================================
        // 1. RENDERIZADO DEL SOL (Estrella central)
        // =====================================================================
        let sun_rotation = sim_time * 0.4;
        let sun_model = Mat4::scaling(1.15, 1.15, 1.15) * Mat4::rotation_y(sun_rotation);
        let sun_rot = Mat4::rotation_y(sun_rotation);

        sun_shader.set_time(sim_time);
        sun_shader.set_camera_pos(camera.eye);

        let sun_ctx = RenderContext::new(&sun_model, &sun_rot, &vp_matrix, &sun_shader)
            .with_blend_mode(BlendMode::Opaque);

        let mut target = RenderTarget {
            framebuffer: &mut framebuffer,
            depth_buffer: &mut depth_buffer,
        };
        renderer.render_mesh(&sphere_mesh, &sun_ctx, &mut target);

        // =====================================================================
        // 2. RENDERIZADO DEL PLANETA ROCOSO (Traslación orbital y rotación)
        // =====================================================================
        let rocky_orbit_radius = 3.6;
        let rocky_orbit_speed = 0.65;
        let rocky_angle = sim_time * rocky_orbit_speed;
        let rocky_pos = Vec3::new(
            rocky_orbit_radius * rocky_angle.cos(),
            0.0,
            rocky_orbit_radius * rocky_angle.sin(),
        );
        let rocky_rotation = sim_time * 1.8;
        let rocky_scale = 0.42;

        let rocky_model = Mat4::translation(rocky_pos.x, rocky_pos.y, rocky_pos.z)
            * Mat4::rotation_y(rocky_rotation)
            * Mat4::scaling(rocky_scale, rocky_scale, rocky_scale);
        let rocky_rot = Mat4::rotation_y(rocky_rotation);

        rocky_shader.body_center = rocky_pos;
        rocky_shader.rotation_rad = rocky_rotation;
        rocky_shader.light_pos = Vec3::zero(); // Iluminado desde el Sol en el origen

        let rocky_ctx = RenderContext::new(&rocky_model, &rocky_rot, &vp_matrix, &rocky_shader)
            .with_blend_mode(BlendMode::Opaque);

        renderer.render_mesh(&sphere_mesh, &rocky_ctx, &mut target);

        // Actualización y presentación en Raylib
        screen_texture
            .update_texture(framebuffer.as_bytes())
            .expect("Error al transferir framebuffer a la textura de Raylib");

        let mut d = rl.begin_drawing(&thread);
        d.clear_background(RColor::BLACK);
        d.draw_texture(&screen_texture, 0, 0, RColor::WHITE);

        // Overlay informativo
        d.draw_text(
            "Sistema Solar: Sol (Estrella) y Planeta Rocoso (Orbita activa)",
            20,
            20,
            20,
            RColor::GOLD,
        );
        d.draw_text(
            "Flechas: Orbitar camara | Traslacion y rotacion analiticas por tiempo",
            20,
            46,
            16,
            RColor::RAYWHITE,
        );
    }
}
