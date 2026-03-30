use glam::{Mat4, Vec3};

#[derive(Debug, Clone)]
pub struct Camera {
    pub eye: Vec3,
    pub target: Vec3,
    pub up: Vec3,
    pub aspect: f32,
    pub fovy_radians: f32,
    pub znear: f32,
    pub zfar: f32,
}

impl Camera {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            eye: Vec3::new(0.0, 2.2, 8.0),
            target: Vec3::new(0.0, 0.0, 0.5),
            up: Vec3::Y,
            aspect: (width as f32 / height.max(1) as f32).max(0.01),
            fovy_radians: 50.0f32.to_radians(),
            znear: 0.1,
            zfar: 100.0,
        }
    }

    pub fn view_proj(&self) -> Mat4 {
        let view = Mat4::look_at_rh(self.eye, self.target, self.up);
        let proj = Mat4::perspective_rh(self.fovy_radians, self.aspect, self.znear, self.zfar);
        proj * view
    }
}
