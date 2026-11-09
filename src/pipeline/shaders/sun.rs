use super::{FragmentInput, FragmentShader};
use crate::buffer::Color;
use crate::math::Vec3;

/// Shader procedural animado de estrella solar evaluado íntegramente en CPU.
///
/// Modula el color superficial mediante 4 capas analíticas:
/// - Cromosfera base con gradiente térmico de radiación incandescente.
/// - Macro-ondas de convección y llamaradas solares continuas en función del tiempo.
/// - Micro-granulación fotosférica (células térmicas de alta frecuencia).
/// - Manchas solares magnéticas y resplandor de corona / efecto limb (Fresnel).
pub struct SunShader {
    pub time: f32,
    pub speed: f32,
    pub scale: f32,
    pub sun_center: Vec3,
    pub camera_pos: Vec3,
}

impl SunShader {
    pub fn new(sun_center: Vec3) -> Self {
        Self {
            time: 0.0,
            speed: 1.2,
            scale: 1.0,
            sun_center,
            camera_pos: Vec3::new(0.0, 0.0, 5.0),
        }
    }

    pub fn set_time(&mut self, time: f32) {
        self.time = time;
    }

    pub fn set_camera_pos(&mut self, cam_pos: Vec3) {
        self.camera_pos = cam_pos;
    }
}

impl Default for SunShader {
    fn default() -> Self {
        Self::new(Vec3::zero())
    }
}

impl FragmentShader for SunShader {
    fn shade(&self, input: &FragmentInput) -> Color {
        // Dirección esférica normalizada desde el centro para eliminar facetas poligonales
        let offset = input.world_pos - self.sun_center;
        let n = if offset.length_squared() > 1e-4 {
            offset.normalize()
        } else {
            input.world_normal.normalize()
        };

        let t = self.time * self.speed;

        // 1. Macro-ondas de convección (llamaradas en movimiento)
        let v1 = (n.x * 3.5 * self.scale + t * 1.5).sin();
        let v2 = (n.y * 3.5 * self.scale + t * 1.2).sin();
        let v3 = (n.z * 3.5 * self.scale + (n.x + n.y) * 2.0 + t * 1.8).sin();
        let macro_convection = (v1 + v2 + v3 + 3.0) / 6.0;

        // 2. Micro-granulación fotosférica (células de ebullición térmica)
        let micro_granulation = ((n.x * 14.0 - t * 2.2).sin()
            * (n.y * 14.0 + t * 1.9).sin()
            * (n.z * 14.0 - t * 1.6).sin())
            * 0.5
            + 0.5;

        let plasma_energy = (macro_convection * 0.72 + micro_granulation * 0.28).clamp(0.0, 1.0);

        // 3. Gradiente de radiación térmica incandescente
        let (mut r, mut g, mut b) = if plasma_energy < 0.45 {
            let f = plasma_energy / 0.45;
            (0.70 + 0.30 * f, 0.08 + 0.35 * f, 0.0)
        } else if plasma_energy < 0.82 {
            let f = (plasma_energy - 0.45) / 0.37;
            (1.0, 0.43 + 0.47 * f, 0.0 + 0.15 * f)
        } else {
            let f = (plasma_energy - 0.82) / 0.18;
            (1.0, 0.90 + 0.10 * f, 0.15 + 0.70 * f)
        };

        // 4. Manchas solares magnéticas oscuras y resplandor de corona
        let spot_noise = ((n.x * 6.0 + 1.2).sin() * (n.y * 6.0 - 0.8).cos() * (n.z * 6.0 + t * 0.3).sin()).abs();
        if spot_noise > 0.65 {
            let spot_factor = ((spot_noise - 0.65) / 0.35).clamp(0.0, 1.0);
            let dark_mult = 1.0 - spot_factor * 0.78;
            r *= dark_mult;
            g *= dark_mult * 0.6;
            b *= dark_mult * 0.4;
        }

        // Resplandor de corona en la silueta (Fresnel / limb glow)
        let view_dir = (self.camera_pos - input.world_pos).normalize();
        let ndotv = n.dot(view_dir).max(0.0);
        let rim_glow = (1.0 - ndotv).powf(2.8) * 0.65;

        r = (r + rim_glow * 1.0).min(1.0);
        g = (g + rim_glow * 0.75).min(1.0);
        b = (b + rim_glow * 0.25).min(1.0);

        Color::from_f32(r, g, b, 1.0)
    }
}

