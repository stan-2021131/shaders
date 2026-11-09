use super::{FragmentInput, FragmentShader};
use crate::buffer::Color;
use crate::math::Vec3;

const INV_U32_MAX: f32 = 1.0 / 4294967295.0;

/// Shader procedural para planeta tipo Tierra con biosfera, océanos, casquetes y atmósfera.
///
/// Modula el color mediante 4 capas analíticas:
/// - Capa 1: Océanos profundos y plataformas costeras turquesas.
/// - Capa 2: Masas continentales con gradientes de vegetación, praderas y mesetas áridas.
/// - Capa 3: Casquetes polares glaciares en latitudes árticas y antárticas.
/// - Capa 4: Sistema dinámico de nubes procedurales y dispersión atmosférica Rayleigh (Fresnel).
pub struct EarthPlanetShader {
    pub seed: u32,
    pub light_pos: Vec3,
    pub ambient_intensity: f32,
    pub body_center: Vec3,
    pub rotation_rad: f32,
    pub time: f32,
    pub camera_pos: Vec3,
}

impl EarthPlanetShader {
    pub fn new(seed: u32, light_pos: Vec3) -> Self {
        Self {
            seed,
            light_pos,
            ambient_intensity: 0.12,
            body_center: Vec3::zero(),
            rotation_rad: 0.0,
            time: 0.0,
            camera_pos: Vec3::new(0.0, 0.0, 10.0),
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

    /// Ruido fractal (FBM) sumando dos octavas continuas
    #[inline]
    fn fbm2(p: Vec3, seed: u32) -> f32 {
        let o1 = Self::value_noise(p, seed);
        let o2 = Self::value_noise(p * 2.1, seed.wrapping_add(101));
        o1 * 0.70 + o2 * 0.30
    }
}

impl FragmentShader for EarthPlanetShader {
    fn shade(&self, input: &FragmentInput) -> Color {
        let offset = input.world_pos - self.body_center;
        let len_sq = offset.length_squared();
        let n = if len_sq > 1e-4 {
            offset * (1.0 / len_sq.sqrt())
        } else {
            input.world_normal.normalize()
        };

        // Rotación de coordenadas solidaria con el giro axial de la superficie
        let cos_r = self.rotation_rad.cos();
        let sin_r = self.rotation_rad.sin();
        let local_n = Vec3::new(cos_r * n.x - sin_r * n.z, n.y, sin_r * n.x + cos_r * n.z);

        // =========================================================================
        // 1 & 2. OCÉANOS Y CONTINENTES (Máscara continental por ruido fractal)
        // =========================================================================
        let continent_val = Self::fbm2(local_n * 2.6, self.seed);
        let sea_level = 0.51;

        let (mut r, mut g, mut b) = if continent_val < sea_level {
            // Capa 1: Océano (azul profundo en abismos a turquesa en costas)
            let depth = continent_val / sea_level;
            let ocean_r = 0.04 + 0.06 * depth;
            let ocean_g = 0.15 + 0.30 * depth;
            let ocean_b = 0.45 + 0.35 * depth;
            (ocean_r, ocean_g, ocean_b)
        } else {
            // Capa 2: Continentes (playa/llanura verde a mesetas áridas/montañas)
            let land_elev = (continent_val - sea_level) / (1.0 - sea_level);
            if land_elev < 0.55 {
                let f = land_elev / 0.55;
                // Costas y praderas de vegetación exuberante
                (0.12 + 0.15 * f, 0.48 - 0.05 * f, 0.14 - 0.04 * f)
            } else {
                let f = (land_elev - 0.55) / 0.45;
                // Mesetas montañosas áridas y roca marrón
                (0.27 + 0.28 * f, 0.43 - 0.15 * f, 0.10 + 0.08 * f)
            }
        };

        // =========================================================================
        // 3. CASQUETES POLARES (Nieve y glaciares en latitudes extremas)
        // =========================================================================
        let latitude = local_n.y.abs();
        let polar_noise = Self::value_noise(local_n * 5.0, self.seed.wrapping_add(50)) * 0.08;
        let ice_threshold = 0.70 + polar_noise;
        if latitude > ice_threshold {
            let ice_factor = ((latitude - ice_threshold) / 0.18).clamp(0.0, 1.0);
            let ice_r = 0.90;
            let ice_g = 0.94;
            let ice_b = 0.98;
            r = r * (1.0 - ice_factor) + ice_r * ice_factor;
            g = g * (1.0 - ice_factor) + ice_g * ice_factor;
            b = b * (1.0 - ice_factor) + ice_b * ice_factor;
        }

        // =========================================================================
        // 4. CAPA DE NUBES Y DISPERSIÓN ATMOSFÉRICA (Fresnel)
        // =========================================================================
        // 4.1. Nubes que giran con una velocidad angular diferencial
        let cloud_rot = self.rotation_rad * 1.15 + self.time * 0.05;
        let cos_c = cloud_rot.cos();
        let sin_c = cloud_rot.sin();
        let cloud_n = Vec3::new(cos_c * n.x - sin_c * n.z, n.y, sin_c * n.x + cos_c * n.z);
        let cloud_raw = Self::fbm2(cloud_n * 3.4, self.seed.wrapping_add(88));

        let cloud_threshold = 0.55;
        if cloud_raw > cloud_threshold {
            let cloud_alpha = ((cloud_raw - cloud_threshold) / 0.25).clamp(0.0, 0.85);
            let cloud_color = 0.95;
            r = r * (1.0 - cloud_alpha) + cloud_color * cloud_alpha;
            g = g * (1.0 - cloud_alpha) + cloud_color * cloud_alpha;
            b = b * (1.0 - cloud_alpha) + cloud_color * cloud_alpha;
        }

        // 4.2. Iluminación solar (difusa desde el Sol central)
        let l = (self.light_pos - input.world_pos).normalize();
        let n_dot_l = n.dot(l).max(0.0);
        let lighting = (self.ambient_intensity + n_dot_l * (1.0 - self.ambient_intensity)).clamp(0.0, 1.0);

        r *= lighting;
        g *= lighting;
        b *= lighting;

        // 4.3. Dispersión atmosférica Rayleigh en el borde de la silueta (Fresnel azul)
        let view_dir = (self.camera_pos - input.world_pos).normalize();
        let ndotv = n.dot(view_dir).max(0.0);
        let atmo_glow = (1.0 - ndotv).powf(3.0) * 0.70;

        r = (r + atmo_glow * 0.25).min(1.0);
        g = (g + atmo_glow * 0.55).min(1.0);
        b = (b + atmo_glow * 1.00).min(1.0);

        Color::from_f32(r, g, b, 1.0)
    }
}
