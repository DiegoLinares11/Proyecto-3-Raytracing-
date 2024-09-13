use nalgebra_glm::{Vec3, dot};
use crate::ray_intersect::{RayIntersect, Intersect};
use crate::material::Material;
pub struct Block {
    pub min: Vec3,
    pub max: Vec3,
    pub material: Material,
}

impl RayIntersect for Block {
    fn ray_intersect(&self, ray_origin: &Vec3, ray_direction: &Vec3) -> Intersect {
        let inv_dir = Vec3::new(
            1.0 / ray_direction.x,
            1.0 / ray_direction.y,
            1.0 / ray_direction.z
        );

        // Calculate t1 and t2 for each component
        let t1 = (self.min - ray_origin).component_mul(&inv_dir);
        let t2 = (self.max - ray_origin).component_mul(&inv_dir);

        // Find the min and max t values across all dimensions
        let t_min = Vec3::new(
            t1.x.min(t2.x),
            t1.y.min(t2.y),
            t1.z.min(t2.z)
        );

        let t_max = Vec3::new(
            t1.x.max(t2.x),
            t1.y.max(t2.y),
            t1.z.max(t2.z)
        );

        // Compute the entry and exit points
        let t1 = t_min.x.max(t_min.y).max(t_min.z);
        let t2 = t_max.x.min(t_max.y).min(t_max.z);

        if t1 <= t2 && t2 > 0.0 {
            let point = ray_origin + ray_direction * t1;
            let normal = if point.x == self.min.x { Vec3::new(-1.0, 0.0, 0.0) }
                        else if point.x == self.max.x { Vec3::new(1.0, 0.0, 0.0) }
                        else if point.y == self.min.y { Vec3::new(0.0, -1.0, 0.0) }
                        else if point.y == self.max.y { Vec3::new(0.0, 1.0, 0.0) }
                        else if point.z == self.min.z { Vec3::new(0.0, 0.0, -1.0) }
                        else { Vec3::new(0.0, 0.0, 1.0) };

            Intersect::new(point, normal, t1, self.material)
        } else {
            Intersect::empty()
        }
    }
}
