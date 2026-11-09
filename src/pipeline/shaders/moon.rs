use super::{FragmentInput, FragmentShader};
use crate::buffer::Color;
use crate::math::Vec3;

const INV_U32_MAX: f32 = 1.0 / 4294967295.0;

/// Shader procedural lunar sin atmósfera con regolito, cuencas de impacto y cráteres afilados.
///
/// Modula el color mediante 4 capas analíticas:
/// - Capa 1: Mares basálticos lunares oscuros (llanuras volcánicas antiguas).
/// - Capa 2: Tierras altas de anortosita en tonos gris plateado claro.
/// - Capa 3: Cráteres de impacto con crestas afiladas y rayos de eyección blanquecinos.
/// - Capa 4: Contraste fotométrico de iluminación solar directa (terminator abrupto sin atmósfera).
pub struct MoonShader {
    pub seed: u32,
    pub light_pos: Vec3,
    pub ambient_intensity: f32,
    pub body_center: Vec3,
    pub rotation_rad: f32,
}

impl MoonShader {
    pub fn new(seed: u32, light_pos: Vec3) -> Self {
        Self {
            seed,
            light_pos,
            ambient_intensity: 0.08,
            body_center: Vec3::zero(),
            rotation_rad: 0.0,
        }
    }

    #[inline]
    fn value_noise(p: Vec3, seed: u32) -> f32 {
        let ix = p.x.floor() as i32;
        let iy = p.y.floor() as i32;
        let iz = p.z.floor() as i32;

        let fx = p.x - ix as f32;
        let fy = p.y - iy as f32;
        let fz = p.z - iz as f32;

        let ux = fx * fx * (3.0 - 2.0 * fx);
        let uy = fy * fy * (3.0 - 2.0 * fy);
        let uz = fz * fz * (3.0 - 2.0 * fz);

        let hx0 = (ix as u32).wrapping_mul(374761393);
        let hx1 = ((ix + 1) as u32).wrapping_mul(374761393);
        let hy0 = (iy as u32).wrapping_mul(668265263);
        let hy1 = ((iy + 1) as u32).wrapping_mul(668265263);
        let hz0 = (iz as u32).wrapping_mul(951214213);
        let hz1 = ((iz + 1) as u32).wrapping_mul(951214213);
        let hs = seed.wrapping_mul(1103515245);

        #[inline(always)]
        fn mix_hash(h_in: u32) -> f32 {
            let h = (h_in ^ (h_in >> 13)).wrapping_mul(1274126177);
            (h as f32) * INV_U32_MAX
        }

        let c000 = mix_hash(hx0 ^ hy0 ^ hz0 ^ hs);
        let c100 = mix_hash(hx1 ^ hy0 ^ hz0 ^ hs);
        let c010 = mix_hash(hx0 ^ hy1 ^ hz0 ^ hs);
        let c110 = mix_hash(hx1 ^ hy1 ^ hz0 ^ hs);
        let c001 = mix_hash(hx0 ^ hy0 ^ hz1 ^ hs);
        let c101 = mix_hash(hx1 ^ hy0 ^ hz1 ^ hs);
        let c011 = mix_hash(hx0 ^ hy1 ^ hz1 ^ hs);
        let c111 = mix_hash(hx1 ^ hy1 ^ hz1 ^ hs);

        let x00 = c000 + ux * (c100 - c000);
        let x10 = c010 + ux * (c110 - c010);
        let x01 = c001 + ux * (c101 - c001);
        let x11 = c011 + ux * (c111 - c011);

        let y0 = x00 + uy * (x10 - x00);
        let y1 = x01 + uy * (x11 - x01);

        y0 + uz * (y1 - y0)
    }
}

impl FragmentShader for MoonShader {
    fn shade(&self, input: &FragmentInput) -> Color {
        let offset = input.world_pos - self.body_center;
        let len_sq = offset.length_squared();
        let n = if len_sq > 1e-4 {
            offset * (1.0 / len_sq.sqrt())
        } else {
            input.world_normal.normalize()
        };

        let cos_r = self.rotation_rad.cos();
        let sin_r = self.rotation_rad.sin();
        let local_n = Vec3::new(cos_r * n.x - sin_r * n.z, n.y, sin_r * n.x + cos_r * n.z);

        // 1. Mares basálticos oscuros
        let maria = Self::value_noise(local_n * 3.2, self.seed);

        // 2. Tierras altas y colinas
        let highlands = Self::value_noise(local_n * 6.5, self.seed.wrapping_add(33));

        // 3. Cráteres afilados (ridged noise)
        let crater_noise = Self::value_noise(local_n * 11.0, self.seed.wrapping_add(77));
        let crater_rim = (1.0 - (crater_noise - 0.5).abs() * 2.0).clamp(0.0, 1.0).powi(2);

        let raw_val = maria * 0.45 + highlands * 0.35 + crater_rim * 0.20;

        let tone = if raw_val < 0.42 {
            // Mares basálticos oscuros
            0.14 + 0.16 * (raw_val / 0.42)
        } else if raw_val < 0.75 {
            // Anortosita gris estándar
            0.30 + 0.28 * ((raw_val - 0.42) / 0.33)
        } else {
            // Rayos de eyección brillantes
            0.58 + 0.32 * ((raw_val - 0.75) / 0.25)
        };

        // 4. Iluminación solar pura con sombras nítidas (vacío espacial)
        let l = (self.light_pos - input.world_pos).normalize();
        let n_dot_l = n.dot(l).max(0.0);
        let lighting = (self.ambient_intensity + n_dot_l * (1.0 - self.ambient_intensity)).clamp(0.0, 1.0);

        let final_val = (tone * lighting).clamp(0.0, 1.0);
        Color::from_f32(final_val, final_val * 0.98, final_val * 0.95, 1.0)
    }
}
