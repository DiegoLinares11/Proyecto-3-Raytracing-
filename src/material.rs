use crate::color::Color;

#[derive(Debug, Clone, Copy)]
pub struct Material {
    pub diffuse: Color,
    pub specular: f32,
    pub albedo: [f32; 4],
    pub refractive_index: f32,
}

impl Material {
    pub fn new(
        diffuse: Color,
        specular: f32,
        albedo: [f32; 4],
        refractive_index: f32,
    ) -> Self {
        Self {
            diffuse,
            specular,
            albedo,
            refractive_index,
        }
    }

    pub fn black() -> Self {
        Self {
            diffuse: Color::black(),
            specular: 0.0,
            albedo: [0.0; 4],
            refractive_index: 0.0,
        }
    }
}
