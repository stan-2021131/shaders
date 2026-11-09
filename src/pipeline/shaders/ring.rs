use super::{FragmentInput, FragmentShader};
use crate::buffer::Color;
use crate::math::{Mat4, Vec3};

/// Shader procedural para el sistema de anillos de Saturno con bandas concéntricas y transparencia.
///
/// Modula el color y la opacidad mediante 4 capas analíticas:
/// - Capa 1: Bandas concéntricas y divisiones de densidad (División de Cassini y anillos A, B, C).
/// - Capa 2: Gradiente espectral de partículas (hielo de agua reflectante y polvo de silicatos).
/// - Capa 3: Micro-variación granular angular en rotación.
/// - Capa 4: Densidad óptica y transparencia alfa variable (Alpha Blending en CPU).
pub struct RingShader {
    pub body_center: Vec3,
    pub inv_rotation: Mat4,
    pub light_pos: Vec3,
    pub ambient_intensity: f32,
    pub inner_radius: f32,
    pub outer_radius: f32,
}

impl RingShader {
    pub fn new(body_center: Vec3, inner_radius: f32, outer_radius: f32) -> Self {
        Self {
            body_center,
            inv_rotation: Mat4::IDENTITY,
            light_pos: Vec3::zero(),
            ambient_intensity: 0.28,
            inner_radius,
            outer_radius,
        }
    }

    pub fn set_rotation(&mut self, rot_matrix: Mat4) {
        self.inv_rotation = rot_matrix.transpose();
    }
}

impl FragmentShader for RingShader {
    fn shade(&self, input: &FragmentInput) -> Color {
        // Transformar la posición al espacio local alineado con el disco del anillo
        let world_offset = input.world_pos - self.body_center;
        let local_pos = self.inv_rotation.transform_direction(world_offset);

        let radius = (local_pos.x * local_pos.x + local_pos.z * local_pos.z).sqrt();

        // 1. Bandas concéntricas y divisiones
        let ring_span = (self.outer_radius - self.inner_radius).max(0.01);
        let t = ((radius - self.inner_radius) / ring_span).clamp(0.0, 1.0);

        let harmonic1 = (t * 46.0).sin() * 0.5 + 0.5;
        let harmonic2 = (t * 92.0 + 1.4).sin() * 0.5 + 0.5;
        let harmonic3 = (t * 20.0).cos() * 0.5 + 0.5;

        let mut density = harmonic1 * 0.52 + harmonic2 * 0.28 + harmonic3 * 0.20;

        // División de Cassini: brecha vacía cerca de t ~ 0.65 .. 0.74
        if t > 0.64 && t < 0.74 {
            let gap_center = 0.69;
            let gap_dist = (t - gap_center).abs() / 0.05;
            density *= gap_dist.clamp(0.06, 1.0);
        }

        // Anillo C interior más tenue (t < 0.22)
        if t < 0.22 {
            density *= 0.40 + 0.60 * (t / 0.22);
        }

        // Desvanecimiento suave en bordes extremos
        let edge_fade = ((t / 0.05).min(1.0)) * (((1.0 - t) / 0.05).min(1.0));
        density *= edge_fade;

        // 2. Gradiente espectral de partículas de hielo y silicatos
        let (mut r, mut g, mut b) = if t < 0.40 {
            // Anillo interior: tonos ocres y ámbar apagado
            let f = t / 0.40;
            (0.72 + 0.12 * f, 0.58 + 0.14 * f, 0.40 + 0.16 * f)
        } else if t < 0.75 {
            // Anillo principal B (hielo brillante reflectante)
            let f = (t - 0.40) / 0.35;
            (0.86 + 0.10 * f, 0.80 + 0.12 * f, 0.68 + 0.18 * f)
        } else {
            // Anillo exterior A: beige suave
            let f = (t - 0.75) / 0.25;
            (0.80 - 0.10 * f, 0.72 - 0.10 * f, 0.60 - 0.12 * f)
        };

        // 3. Micro-variación angular
        let angle = local_pos.z.atan2(local_pos.x);
        let angle_var = (angle * 7.0).sin() * 0.05;
        density = (density + angle_var).clamp(0.0, 1.0);

        // 4. Iluminación solar e índice de transparencia alfa
        let l = (self.light_pos - input.world_pos).normalize();
        let ndotl = input.world_normal.dot(l).abs().max(0.15);
        let lighting = (self.ambient_intensity + ndotl * (1.0 - self.ambient_intensity)).clamp(0.0, 1.0);

        r *= lighting;
        g *= lighting;
        b *= lighting;

        // Opacidad translúcida: oscila entre 15% y 75% según la densidad del anillo
        let alpha = (density * 0.72).clamp(0.06, 0.76);

        Color::from_f32(r, g, b, alpha)
    }
}
