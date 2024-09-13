use nalgebra_glm::{Vec3, dot};
use crate::ray_intersect::{RayIntersect, Intersect};
use crate::material::Material;

pub struct Plane {
    pub point: Vec3,
    pub normal: Vec3,
    pub material: Material,
}

impl Plane {
    pub fn new(point: Vec3, normal: Vec3, material: Material) -> Self {
        Plane { point, normal, material }
    }
}

impl RayIntersect for Plane {
    fn ray_intersect(&self, ray_origin: &Vec3, ray_direction: &Vec3) -> Intersect {
        let denom = dot(&self.normal, &ray_direction);
        
        // Verificar si el rayo es paralelo al plano
        if denom.abs() > 1e-6 {
            let t = dot(&(self.point - ray_origin), &self.normal) / denom;
            
            // Verificar si la intersección está en frente del origen del rayo
            if t > 1e-6 {
                let intersection_point = ray_origin + t * ray_direction;
                return Intersect {
                    is_intersecting: true,
                    distance: t,
                    point: intersection_point,
                    normal: self.normal,
                    material: self.material.clone(),
                };
            }
        }
        
        Intersect::empty()
    }
}
