use super::{FragmentInput, FragmentShader};
use crate::buffer::Color;
use crate::math::Vec3;

const INV_U32_MAX: f32 = 1.0 / 4294967295.0;

/// Shader procedural para gigante gaseoso tipo Saturno con bandas zonales, turbulencia de vientos y halo traslúcido.
///
/// Modula el color mediante 4 capas analíticas:
/// - Capa 1: Bandas zonales latitudinales con gradientes de gas crema, caramelo, ocre y ámbar.
/// - Capa 2: Mareas y turbulencia de fluidos por vientos contrarrotantes modulados con ruido y tiempo.
/// - Capa 3: Vórtices ciclónicos y gran mancha ovalada de tormenta persistente.
/// - Capa 4: Sutil traslucidez gaseosa y halo difuso en el limbo atmosférico (Fresnel).
pub struct GasGiantShader {
    pub seed: u32,
    pub light_pos: Vec3,
    pub ambient_intensity: f32,
    pub body_center: Vec3,
    pub rotation_rad: f32,
    pub time: f32,
    pub camera_pos: Vec3,
}

impl GasGiantShader {
    pub fn new(seed: u32, light_pos: Vec3) -> Self {
        Self {
            seed,
            light_pos,
            ambient_intensity: 0.14,
            body_center: Vec3::zero(),
            rotation_rad: 0.0,
            time: 0.0,
            camera_pos: Vec3::new(0.0, 0.0, 15.0),
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

impl FragmentShader for GasGiantShader {
    fn shade(&self, input: &FragmentInput) -> Color {
        let offset = input.world_pos - self.body_center;
        let len_sq = offset.length_squared();
        let n = if len_sq > 1e-4 {
            offset * (1.0 / len_sq.sqrt())
        } else {
            input.world_normal.normalize()
        };

        // Rotación de coordenadas solidaria con el giro del planeta
        let cos_r = self.rotation_rad.cos();
        let sin_r = self.rotation_rad.sin();
        let local_n = Vec3::new(cos_r * n.x - sin_r * n.z, n.y, sin_r * n.x + cos_r * n.z);

        let t = self.time * 0.4;

        // =========================================================================
        // CAPA 2: Mareas y turbulencia de gas (Vientos contrarrotantes por latitud)
        // =========================================================================
        // Corrientes en chorro con cizalladura zonal este-oeste dependiente de la latitud
        let wind_shear = (local_n.y * 7.0).sin() * 0.85;
        let flow_coord = Vec3::new(
            local_n.x * 2.2 + wind_shear + t * 0.6,
            local_n.y * 3.8,
            local_n.z * 2.2 + t * 0.4,
        );
        let turbulence = Self::value_noise(flow_coord, self.seed);
        let micro_tide = Self::value_noise(flow_coord * 2.8, self.seed.wrapping_add(45)) * 0.25;
        let total_turb = turbulence * 0.75 + micro_tide;

        // =========================================================================
        // CAPA 1: Bandas zonales de latitud (Gradientes térmicos de gas)
        // =========================================================================
        // Coordenada latitudinal perturbada por las mareas de gas
        let perturbed_lat = local_n.y * 9.0 + (total_turb - 0.5) * 1.8;
        let band_factor = (perturbed_lat.sin() * 0.5 + 0.5).clamp(0.0, 1.0);

        // Paleta de Saturno (tonos cálidos de amoníaco e hidrocarburos):
        // [0.0 .. 0.5]: Caramelo dorado / ocre ámbar
        // [0.5 .. 1.0]: Crema cálida / beige pajizo / arena suave
        let (mut r, mut g, mut b) = if band_factor < 0.5 {
            let f = band_factor * 2.0;
            (
                0.78 + 0.12 * f,
                0.58 + 0.14 * f,
                0.36 + 0.16 * f,
            )
        } else {
            let f = (band_factor - 0.5) * 2.0;
            (
                0.90 + 0.08 * f,
                0.72 + 0.16 * f,
                0.52 + 0.25 * f,
            )
        };

        // Modulación adicional por bandas ecuatoriales y polares
        let polar_dimming = 1.0 - (local_n.y.abs().powf(2.2) * 0.35);
        r *= polar_dimming;
        g *= polar_dimming;
        b *= polar_dimming * 0.95;

        // =========================================================================
        // CAPA 3: Vórtices ciclónicos y gran mancha de tormenta persistente
        // =========================================================================
        // Tormenta ovalada situada en latitud sur (~ -0.32)
        let spot_y = (local_n.y + 0.32) * 4.5;
        let spot_x = (local_n.x - 0.25) * 2.5;
        let dist_to_storm = spot_x * spot_x + spot_y * spot_y;
        if dist_to_storm < 0.65 {
            let storm_strength = (1.0 - dist_to_storm / 0.65).clamp(0.0, 1.0).powf(1.5);
            // Matiz rojizo / caoba intenso en el ojo de la tormenta
            r = r * (1.0 - storm_strength * 0.45) + 0.88 * storm_strength * 0.45;
            g = g * (1.0 - storm_strength * 0.65) + 0.32 * storm_strength * 0.65;
            b = b * (1.0 - storm_strength * 0.70) + 0.18 * storm_strength * 0.70;
        }

        // =========================================================================
        // CAPA 4: Traslucidez sutil y halo difuso en el limbo atmosférico (Fresnel)
        // =========================================================================
        let view_dir = (self.camera_pos - input.world_pos).normalize();
        let ndotv = n.dot(view_dir).max(0.0);

        // Suave traslucidez volumétrica: el centro tiene un matiz ligeramente etéreo
        let center_translucency = (1.0 - ndotv) * 0.18;
        r += center_translucency * 0.10;
        g += center_translucency * 0.08;
        b += center_translucency * 0.14;

        // Halo difuso gaseoso en el borde exterior
        let limb_halo = (1.0 - ndotv).powf(2.4) * 0.45;
        r = (r + limb_halo * 0.65).min(1.0);
        g = (g + limb_halo * 0.55).min(1.0);
        b = (b + limb_halo * 0.40).min(1.0);

        // Iluminación solar fotométrica (difusa suave con atenuación crepuscular)
        let l = (self.light_pos - input.world_pos).normalize();
        let n_dot_l = n.dot(l).max(0.0);
        // Atenuación crepuscular de gas (difumina suavemente el paso día/noche)
        let gas_terminator = (n_dot_l + 0.08) / 1.08;
        let lighting = (self.ambient_intensity + gas_terminator * (1.0 - self.ambient_intensity))
            .clamp(0.0, 1.0);

        Color::from_f32(
            (r * lighting).min(1.0),
            (g * lighting).min(1.0),
            (b * lighting).min(1.0),
            1.0,
        )
    }
}
