use ::shaders::buffer::{Color, DepthBuffer, Framebuffer};
use ::shaders::camera::Camera;
use ::shaders::math::{Mat4, Vec3};
use ::shaders::mesh::Mesh;
use ::shaders::pipeline::shaders::{
    EarthPlanetShader, GasGiantShader, MoonShader, RockyPlanetShader, SunShader,
};
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

    // Cámara orbital panorámica que abarca al Sol y a todos los planetas
    let aspect = WINDOW_WIDTH as f32 / WINDOW_HEIGHT as f32;
    let mut camera = Camera::new(
        Vec3::zero(),
        18.5,
        0.3,
        0.38,
        45.0,
        aspect,
        0.1,
        250.0,
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
    let mut earth_shader = EarthPlanetShader::new(107, Vec3::zero());
    let mut moon_shader = MoonShader::new(999, Vec3::zero());
    let mut gas_giant_shader = GasGiantShader::new(314, Vec3::zero());

    let mut sim_time: f32 = 0.0;

    println!("============================================================");
    println!("[Sistema Solar: Sol, Planeta Rocoso, Tierra, Gigante Gaseoso y Lunas]");
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
        // 1. SOL (Estrella central)
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
        // 2. PLANETA ROCOSO (Órbita interior)
        // =====================================================================
        let rocky_orbit_radius = 3.4;
        let rocky_orbit_speed = 0.65;
        let rocky_angle = sim_time * rocky_orbit_speed;
        let rocky_pos = Vec3::new(
            rocky_orbit_radius * rocky_angle.cos(),
            0.0,
            rocky_orbit_radius * rocky_angle.sin(),
        );
        let rocky_rotation = sim_time * 1.8;
        let rocky_scale = 0.38;

        let rocky_model = Mat4::translation(rocky_pos.x, rocky_pos.y, rocky_pos.z)
            * Mat4::rotation_y(rocky_rotation)
            * Mat4::scaling(rocky_scale, rocky_scale, rocky_scale);
        let rocky_rot = Mat4::rotation_y(rocky_rotation);

        rocky_shader.body_center = rocky_pos;
        rocky_shader.rotation_rad = rocky_rotation;
        rocky_shader.light_pos = Vec3::zero();

        let rocky_ctx = RenderContext::new(&rocky_model, &rocky_rot, &vp_matrix, &rocky_shader)
            .with_blend_mode(BlendMode::Opaque);

        renderer.render_mesh(&sphere_mesh, &rocky_ctx, &mut target);

        // =====================================================================
        // 3. PLANETA TIERRA (Órbita habitable)
        // =====================================================================
        let earth_orbit_radius = 6.2;
        let earth_orbit_speed = 0.40;
        let earth_angle = sim_time * earth_orbit_speed + 1.2;
        let earth_pos = Vec3::new(
            earth_orbit_radius * earth_angle.cos(),
            0.0,
            earth_orbit_radius * earth_angle.sin(),
        );
        let earth_rotation = sim_time * 1.4;
        let earth_scale = 0.52;

        let earth_model = Mat4::translation(earth_pos.x, earth_pos.y, earth_pos.z)
            * Mat4::rotation_y(earth_rotation)
            * Mat4::scaling(earth_scale, earth_scale, earth_scale);
        let earth_rot = Mat4::rotation_y(earth_rotation);

        earth_shader.body_center = earth_pos;
        earth_shader.rotation_rad = earth_rotation;
        earth_shader.time = sim_time;
        earth_shader.camera_pos = camera.eye;
        earth_shader.light_pos = Vec3::zero();

        let earth_ctx = RenderContext::new(&earth_model, &earth_rot, &vp_matrix, &earth_shader)
            .with_blend_mode(BlendMode::Opaque);

        renderer.render_mesh(&sphere_mesh, &earth_ctx, &mut target);

        // =====================================================================
        // 4. LUNA DE LA TIERRA
        // =====================================================================
        let earth_moon_radius = 1.15;
        let earth_moon_speed = 1.8;
        let earth_moon_angle = sim_time * earth_moon_speed;
        let earth_moon_pos = earth_pos
            + Vec3::new(
                earth_moon_radius * earth_moon_angle.cos(),
                0.22 * earth_moon_angle.sin(),
                earth_moon_radius * earth_moon_angle.sin(),
            );
        let earth_moon_rot = sim_time * 0.9;
        let earth_moon_scale = 0.14;

        let earth_moon_model = Mat4::translation(earth_moon_pos.x, earth_moon_pos.y, earth_moon_pos.z)
            * Mat4::rotation_y(earth_moon_rot)
            * Mat4::scaling(earth_moon_scale, earth_moon_scale, earth_moon_scale);
        let earth_moon_rot_m = Mat4::rotation_y(earth_moon_rot);

        moon_shader.body_center = earth_moon_pos;
        moon_shader.rotation_rad = earth_moon_rot;
        moon_shader.light_pos = Vec3::zero();

        let earth_moon_ctx = RenderContext::new(
            &earth_moon_model,
            &earth_moon_rot_m,
            &vp_matrix,
            &moon_shader,
        )
        .with_blend_mode(BlendMode::Opaque);

        renderer.render_mesh(&sphere_mesh, &earth_moon_ctx, &mut target);

        // =====================================================================
        // 5. GIGANTE GASEOSO (Órbita exterior)
        // =====================================================================
        let gas_orbit_radius = 9.8;
        let gas_orbit_speed = 0.22;
        let gas_angle = sim_time * gas_orbit_speed + 2.5;
        let gas_pos = Vec3::new(
            gas_orbit_radius * gas_angle.cos(),
            0.0,
            gas_orbit_radius * gas_angle.sin(),
        );
        let gas_rotation = sim_time * 1.6;
        let gas_scale = 0.78;

        let gas_model = Mat4::translation(gas_pos.x, gas_pos.y, gas_pos.z)
            * Mat4::rotation_y(gas_rotation)
            * Mat4::scaling(gas_scale, gas_scale, gas_scale);
        let gas_rot = Mat4::rotation_y(gas_rotation);

        gas_giant_shader.body_center = gas_pos;
        gas_giant_shader.rotation_rad = gas_rotation;
        gas_giant_shader.time = sim_time;
        gas_giant_shader.camera_pos = camera.eye;
        gas_giant_shader.light_pos = Vec3::zero();

        let gas_ctx = RenderContext::new(&gas_model, &gas_rot, &vp_matrix, &gas_giant_shader)
            .with_blend_mode(BlendMode::Opaque);

        renderer.render_mesh(&sphere_mesh, &gas_ctx, &mut target);

        // =====================================================================
        // 6. LUNAS DEL GIGANTE GASEOSO (Luna 1 interior y Luna 2 exterior inclinada)
        // =====================================================================
        // Luna 1 (Titán)
        let gas_moon1_radius = 1.55;
        let gas_moon1_speed = 1.4;
        let gas_moon1_angle = sim_time * gas_moon1_speed;
        let gas_moon1_pos = gas_pos
            + Vec3::new(
                gas_moon1_radius * gas_moon1_angle.cos(),
                0.15 * gas_moon1_angle.sin(),
                gas_moon1_radius * gas_moon1_angle.sin(),
            );
        let gas_moon1_scale = 0.12;
        let gas_moon1_model = Mat4::translation(gas_moon1_pos.x, gas_moon1_pos.y, gas_moon1_pos.z)
            * Mat4::scaling(gas_moon1_scale, gas_moon1_scale, gas_moon1_scale);
        let gas_moon1_rot = Mat4::IDENTITY;

        moon_shader.body_center = gas_moon1_pos;
        let gas_moon1_ctx = RenderContext::new(
            &gas_moon1_model,
            &gas_moon1_rot,
            &vp_matrix,
            &moon_shader,
        )
        .with_blend_mode(BlendMode::Opaque);
        renderer.render_mesh(&sphere_mesh, &gas_moon1_ctx, &mut target);

        // Luna 2 (Encélado / exterior inclinada)
        let gas_moon2_radius = 2.15;
        let gas_moon2_speed = 0.85;
        let gas_moon2_angle = sim_time * gas_moon2_speed + 1.8;
        let gas_moon2_pos = gas_pos
            + Vec3::new(
                gas_moon2_radius * gas_moon2_angle.cos(),
                0.45 * gas_moon2_angle.sin(),
                gas_moon2_radius * gas_moon2_angle.sin(),
            );
        let gas_moon2_scale = 0.09;
        let gas_moon2_model = Mat4::translation(gas_moon2_pos.x, gas_moon2_pos.y, gas_moon2_pos.z)
            * Mat4::scaling(gas_moon2_scale, gas_moon2_scale, gas_moon2_scale);
        let gas_moon2_rot = Mat4::IDENTITY;

        moon_shader.body_center = gas_moon2_pos;
        let gas_moon2_ctx = RenderContext::new(
            &gas_moon2_model,
            &gas_moon2_rot,
            &vp_matrix,
            &moon_shader,
        )
        .with_blend_mode(BlendMode::Opaque);
        renderer.render_mesh(&sphere_mesh, &gas_moon2_ctx, &mut target);

        // Actualización y presentación en Raylib
        screen_texture
            .update_texture(framebuffer.as_bytes())
            .expect("Error al transferir framebuffer a la textura de Raylib");

        let mut d = rl.begin_drawing(&thread);
        d.clear_background(RColor::BLACK);
        d.draw_texture(&screen_texture, 0, 0, RColor::WHITE);

        // Overlay informativo
        d.draw_text(
            "Sistema Solar: Sol, Rocoso, Tierra, Gigante Gaseoso y Sistema de Lunas",
            20,
            20,
            20,
            RColor::GOLD,
        );
        d.draw_text(
            "Flechas: Orbitar camara | Traslaciones heliocéntricas y múltiples órbitas lunares",
            20,
            46,
            16,
            RColor::RAYWHITE,
        );
    }
}
