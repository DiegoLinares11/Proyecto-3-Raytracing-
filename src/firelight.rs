// fire.rs
use crate::light::Light;
use nalgebra_glm::{Vec3, dot};
use crate::ray_intersect::{RayIntersect, Intersect};
use crate::material::Material;
use crate::color::Color;

pub struct FireLight {
    position: Vec3,
    color: Color,
    intensity: f32,
    flicker: f32, // Para la variación en la intensidad
}

impl FireLight {
    pub fn new(position: Vec3, color: Color, intensity: f32, flicker: f32) -> Self {
        FireLight { position, color, intensity, flicker }
    }

    fn get_light(&self, time: f32) -> Color {
        let intensity_variation = 1.0 + (self.flicker * (time.sin() * 0.5 + 0.5));
        self.color * self.intensity * intensity_variation
    }
}
