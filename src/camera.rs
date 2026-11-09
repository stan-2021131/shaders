use crate::math::{Mat4, Vec3, deg_to_rad};

/// Cámara orbital para el visor del sistema solar.
#[derive(Debug, Clone, Copy)]
pub struct Camera {
    pub eye: Vec3,
    pub target: Vec3,
    pub up: Vec3,
    pub fov_y_rad: f32,
    pub aspect: f32,
    pub z_near: f32,
    pub z_far: f32,
    pub yaw: f32,
    pub pitch: f32,
    pub distance: f32,
}

impl Camera {
    pub fn new(
        target: Vec3,
        distance: f32,
        yaw: f32,
        pitch: f32,
        fov_deg: f32,
        aspect: f32,
        z_near: f32,
        z_far: f32,
    ) -> Self {
        let mut cam = Self {
            eye: Vec3::zero(),
            target,
            up: Vec3::up(),
            fov_y_rad: deg_to_rad(fov_deg),
            aspect,
            z_near,
            z_far,
            yaw,
            pitch,
            distance,
        };
        cam.update_orbit();
        cam
    }

    /// Actualiza la posición esférica del ojo (`eye`) respecto al `target`.
    pub fn update_orbit(&mut self) {
        let pitch_clamped = self.pitch.clamp(-1.45, 1.45);
        let x = self.distance * pitch_clamped.cos() * self.yaw.sin();
        let y = self.distance * pitch_clamped.sin();
        let z = self.distance * pitch_clamped.cos() * self.yaw.cos();
        self.eye = self.target + Vec3::new(x, y, z);
    }

    /// Modifica los ángulos orbitales y actualiza el ojo.
    pub fn rotate_orbit(&mut self, d_yaw: f32, d_pitch: f32) {
        self.yaw += d_yaw;
        self.pitch += d_pitch;
        self.update_orbit();
    }

    /// Ajusta la distancia orbital (zoom).
    pub fn zoom(&mut self, delta: f32) {
        self.distance = (self.distance + delta).clamp(1.5, 300.0);
        self.update_orbit();
    }

    /// Fija un nuevo objetivo hacia el cual orbita la cámara.
    pub fn set_target(&mut self, target: Vec3) {
        self.target = target;
        self.update_orbit();
    }

    /// Matriz de vista (View Matrix) mediante Look-At.
    pub fn view_matrix(&self) -> Mat4 {
        Mat4::look_at(self.eye, self.target, self.up)
    }

    /// Matriz de proyección en perspectiva estándar.
    pub fn projection_matrix(&self) -> Mat4 {
        Mat4::perspective(self.fov_y_rad, self.aspect, self.z_near, self.z_far)
    }

    /// Matriz combinada de Vista-Proyección: $VP = P \cdot V$.
    pub fn view_projection_matrix(&self) -> Mat4 {
        self.projection_matrix() * self.view_matrix()
    }
}

