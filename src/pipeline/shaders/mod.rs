pub mod rocky;
pub mod sun;

use crate::buffer::Color;
use crate::math::{Vec2, Vec3};
pub use rocky::RockyPlanetShader;
pub use sun::SunShader;

/// Datos interpolados que recibe el shader de fragmentos en cada píxel evaluado en CPU.
#[derive(Debug, Clone)]
pub struct FragmentInput {
    pub screen_x: usize,
    pub screen_y: usize,
    pub depth_ndc: f32,
    pub world_pos: Vec3,
    pub world_normal: Vec3,
    pub world_tangent: Vec3,
    pub uv: Vec2,
    pub barycentric: Vec3,
}

/// Trait para shaders de fragmentos evaluados íntegramente en CPU.
pub trait FragmentShader: Sync {
    fn shade(&self, input: &FragmentInput) -> Color;
}

/// Shader básico con iluminación difusa direccional simple para depuración de mallas.
pub struct FlatColorShader {
    pub color: Color,
}

impl FlatColorShader {
    pub fn new(color: Color) -> Self {
        Self { color }
    }
}

impl FragmentShader for FlatColorShader {
    fn shade(&self, input: &FragmentInput) -> Color {
        let light_dir = Vec3::new(0.5, 0.8, 1.0).normalize();
        let diff = input.world_normal.dot(light_dir).max(0.18);
        Color::new(
            (self.color.r as f32 * diff).min(255.0) as u8,
            (self.color.g as f32 * diff).min(255.0) as u8,
            (self.color.b as f32 * diff).min(255.0) as u8,
            self.color.a,
        )
    }
}
