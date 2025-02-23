    mod ray_intersect;
    mod color;
    mod camera;
    mod light;
    mod material;
    mod framebuffer;
    mod block; // Asegúrate de que este módulo esté incluido


    use minifb::{Window, WindowOptions, Key};
    use nalgebra_glm::{Vec3, normalize};
    use std::time::Duration;
    use std::f32::consts::PI;

    use crate::color::Color;
    use crate::ray_intersect::{Intersect, RayIntersect};
    use crate::framebuffer::Framebuffer;
    use crate::camera::Camera;
    use crate::light::Light;
    use crate::material::Material;
    use crate::block::Block; // Importa la clase Block
   

    const ORIGIN_BIAS: f32 = 1e-4;
    const SKYBOX_COLOR: Color = Color::new(68, 142, 228);

    fn offset_origin(intersect: &Intersect, direction: &Vec3) -> Vec3 {
        let offset = intersect.normal * ORIGIN_BIAS;
        if direction.dot(&intersect.normal) < 0.0 {
            intersect.point - offset
        } else {
            intersect.point + offset
        }
    }

    fn reflect(incident: &Vec3, normal: &Vec3) -> Vec3 {
        incident - 2.0 * incident.dot(normal) * normal
    }

    fn refract(incident: &Vec3, normal: &Vec3, eta_t: f32) -> Vec3 {
        let cosi = -incident.dot(normal).max(-1.0).min(1.0);

        let (n_cosi, eta, n_normal);

        if cosi < 0.0 {
            n_cosi = -cosi;
            eta = 1.0 / eta_t;
            n_normal = -normal;
        } else {
            n_cosi = cosi;
            eta = eta_t;
            n_normal = *normal;
        }

        let k = 1.0 - eta * eta * (1.0 - n_cosi * n_cosi);

        if k < 0.0 {
            reflect(incident, &n_normal)
        } else {
            eta * incident + (eta * n_cosi - k.sqrt()) * n_normal
        }
    }

    fn cast_shadow(
        intersect: &Intersect,
        light: &Light,
        objects: &[Box<dyn RayIntersect>], // Use Box<dyn RayIntersect> for polymorphism
    ) -> f32 {
        let light_dir = (light.position - intersect.point).normalize();
        let light_distance = (light.position - intersect.point).magnitude();

        let shadow_ray_origin = offset_origin(intersect, &light_dir);
        let mut shadow_intensity = 0.0;

        for object in objects.iter() {
            let shadow_intersect = object.ray_intersect(&shadow_ray_origin, &light_dir);
            if shadow_intersect.is_intersecting && shadow_intersect.distance < light_distance {
                let distance_ratio = shadow_intersect.distance / light_distance;
                shadow_intensity = 1.0 - distance_ratio.powf(2.0).min(1.0);
                break;
            }
        }

        shadow_intensity
    }



    pub fn cast_ray(
        ray_origin: &Vec3,
        ray_direction: &Vec3,
        objects: &[Box<dyn RayIntersect>],
        lights: &[Light],
        depth: u32,

    ) -> Color {
        if depth > 3 {
            return SKYBOX_COLOR;
        }

        let mut intersect = Intersect::empty();
        let mut zbuffer = f32::INFINITY;

        for object in objects.iter() {
            let i = object.ray_intersect(ray_origin, ray_direction);
            if i.is_intersecting && i.distance < zbuffer {
                zbuffer = i.distance;
                intersect = i;
            }
        }

        if !intersect.is_intersecting {
            return SKYBOX_COLOR;
        }

        let mut color = Color::black();

        for light in lights.iter() {
            let light_dir = (light.position - intersect.point).normalize();
            let view_dir = (ray_origin - intersect.point).normalize();
            let reflect_dir = reflect(&-light_dir, &intersect.normal).normalize();

            let shadow_intensity = cast_shadow(&intersect, light, objects);
            let light_intensity = light.intensity * (1.0 - shadow_intensity);

            let diffuse_intensity = intersect.normal.dot(&light_dir).max(0.0).min(1.0);
            let diffuse = intersect.material.diffuse * intersect.material.albedo[0] * diffuse_intensity * light_intensity;

            let specular_intensity = view_dir.dot(&reflect_dir).max(0.0).powf(intersect.material.specular);
            let specular = light.color.scale(intersect.material.albedo[1]) * specular_intensity * light_intensity;



            let reflect_color = if intersect.material.albedo[2] > 0.0 {
                let reflect_dir = reflect(&ray_direction, &intersect.normal).normalize();
                let reflect_origin = offset_origin(&intersect, &reflect_dir);
                cast_ray(&reflect_origin, &reflect_dir, objects, lights, depth + 1)
            } else {
                Color::black()
            };

            let refract_color = if intersect.material.albedo[3] > 0.0 {
                let refract_dir = refract(&ray_direction, &intersect.normal, intersect.material.refractive_index);
                let refract_origin = offset_origin(&intersect, &refract_dir);
                cast_ray(&refract_origin, &refract_dir, objects, lights, depth + 1)
            } else {
                Color::black()
            };
            // Combina los colores con los factores de Fresnel
            color = color.add(&(diffuse + specular).scale(1.0 - intersect.material.albedo[2] - intersect.material.albedo[3]))
            .add(&reflect_color.scale(intersect.material.albedo[2]))
            .add(&refract_color.scale(intersect.material.albedo[3]));
        }

        color
    }

    

    pub fn render(framebuffer: &mut Framebuffer, objects: &[Box<dyn RayIntersect>], camera: &Camera, lights: &[Light]) {
        let width = framebuffer.width as f32;
        let height = framebuffer.height as f32;
        let aspect_ratio = width / height;
        let fov = PI / 3.0;
        let perspective_scale = (fov * 0.5).tan();

        for y in 0..framebuffer.height {
            for x in 0..framebuffer.width {
                let screen_x = (2.0 * x as f32) / width - 1.0;
                let screen_y = -(2.0 * y as f32) / height + 1.0;

                let screen_x = screen_x * aspect_ratio * perspective_scale;
                let screen_y = screen_y * perspective_scale;

                let ray_direction = normalize(&Vec3::new(screen_x, screen_y, -1.0));
                let rotated_direction = camera.base_change(&ray_direction);

                let pixel_color = cast_ray(&camera.eye, &rotated_direction, objects, lights, 0);

                framebuffer.set_current_color(pixel_color.to_hex());
                framebuffer.point(x, y);
            }
        }
    }


    fn main() {
        let window_width = 400;
        let window_height = 250;
        let framebuffer_width = 400;
        let framebuffer_height =250;
        let frame_delay = Duration::from_millis(16);
        let rotation_speed = 0.05; // Ajusta este valor según lo necesario

        let mut framebuffer = Framebuffer::new(framebuffer_width, framebuffer_height);

        let mut window = Window::new(
            "Refractor",
            window_width,
            window_height,
            WindowOptions::default(),
        ).unwrap();

        let rubber = Material::new(
            Color::new(80, 0, 0),
            1.0,
            [0.9, 0.1, 0.0, 0.0],
            0.0,

        );

        let ivory = Material::new(
            Color::new(100, 100, 80),
            0.0,
            [0.6, 0.3, 0.6, 0.0],
            0.0,

        );


        let block_material = Material::new(
            Color::new(150, 75, 0), // Color similar a la madera
            0.0,
            [0.8, 0.2, 0.0, 0.0],
            0.0,

        );

        let water_material = Material::new(
            Color::new(0, 0, 255),
            0.0,
            [0.0, 0.0, 0.0, 1.0], 
            0.0,

        );
        
            let mirror = Material::new(
            Color::new(255, 255, 255), // El color no importa mucho aquí
            1000.0,                    // Alto valor especular
            [0.0, 1.0, 0.0, 0.0],      // Totalmente reflectivo
            1.5,                        // Índice de refracción alto, como el vidrio
 
        );

        let lava_material = Material::new(
            Color::new(255, 100, 0), // Color rojo para la lava
            1.0,
            [1.0, 1.0, 0.0, 0.0], // No reflexión, pero brillo
            2.0, // Alta reflectividad para simular lava brillante

        );


        let mut objects: Vec<Box<dyn RayIntersect>> = vec![];

        let wall_height = 2.5; // Aumentar la altura de las paredes
        let wall_thickness = 0.1; // Grosor de las paredes
        let room_width = 1.5; // Aumentar el ancho de la habitación
        let room_depth = 1.5; // Aumentar la profundidad de la habitación
        
        // Pared trasera
        objects.push(Box::new(Block { min: Vec3::new(-room_width, -1.0, -room_depth - wall_thickness), max: Vec3::new(room_width, wall_height - 1.0, -room_depth), material: block_material }));
        
        // Pared izquierda
        objects.push(Box::new(Block { min: Vec3::new(-room_width - wall_thickness, -1.0, -room_depth), max: Vec3::new(-room_width, wall_height - 1.0, room_depth), material: block_material }));
        
        
        // Suelo
        objects.push(Box::new(Block { min: Vec3::new(-room_width, -1.0, -room_depth), max: Vec3::new(room_width, -1.0 + wall_thickness, room_depth), material: block_material }));
        
        // Techo a la mitad
        objects.push(Box::new(Block { min: Vec3::new(-room_width, wall_height / 2.0, -room_depth), max: Vec3::new(room_width/2.0, (wall_height / 2.0) + wall_thickness, room_depth), material: block_material }));
        
        //Bloque de lava
        objects.push(Box::new(Block { min: Vec3::new(-room_width - wall_thickness, -0.9, -room_depth-wall_thickness), max: Vec3::new(-1.0, -0.5, -1.0), material: lava_material }));

        //-1.6 -0.9 -1.6       -1.0 -0.5  -1.0 
         
        //Empezare agregando bloques raros en medio esperando que pueda agregarles texturas.
        //Bloque de el medio medio
        objects.push(Box::new(Block { min: Vec3::new(-0.1 , -0.9, -0.3), max: Vec3::new(0.1, -0.7, -0.1), material: ivory }));
        
        //Bloque a la izquierda del de enmedio
        //Otro bloque a la par: 
        objects.push(Box::new(Block { min: Vec3::new(-0.3 , -0.9, -0.3), max: Vec3::new(-0.1, -0.7, -0.1), material: lava_material }));

        //Otro bloque a la par: 
        objects.push(Box::new(Block { min: Vec3::new(-0.5 , -0.9, -0.3), max: Vec3::new(-0.3, -0.7, -0.1), material: ivory }));

        //Empezar con los bloques a la derecha del medio.
        objects.push(Box::new(Block { min: Vec3::new(0.1 , -0.9, -0.3), max: Vec3::new(0.3, -0.7, -0.1), material: lava_material }));

        //Empezar con los bloques a la derecha del medio.
        objects.push(Box::new(Block { min: Vec3::new(0.3 , -0.9, -0.3), max: Vec3::new(0.5, -0.7, -0.1), material: ivory }));


        //Aqui ira el suelo de arriba: 
        objects.push(Box::new(Block { min: Vec3::new(-0.1 , -0.1, -0.3), max: Vec3::new(0.1, 0.1, -0.5), material: ivory }));
        objects.push(Box::new(Block { min: Vec3::new(-0.3 , -0.1, -0.3), max: Vec3::new(-0.1, 0.1, -0.5), material: lava_material }));
        objects.push(Box::new(Block { min: Vec3::new(-0.5 , -0.1, -0.3), max: Vec3::new(-0.3, 0.1, -0.5), material: ivory }));
        objects.push(Box::new(Block { min: Vec3::new(0.1 , -0.1, -0.3), max: Vec3::new(0.3, 0.1, -0.5), material: lava_material }));
        objects.push(Box::new(Block { min: Vec3::new(0.3 , -0.1, -0.3), max: Vec3::new(0.5, 0.1, -0.5), material: ivory }));  

        //Aqui ira el suelo de arriba: 
        objects.push(Box::new(Block { min: Vec3::new(-0.1 , -0.1, -0.3), max: Vec3::new(0.1, 0.1, -0.1), material: ivory }));
        objects.push(Box::new(Block { min: Vec3::new(-0.3 , -0.1, -0.3), max: Vec3::new(-0.1, 0.1, -0.1), material: lava_material }));
        objects.push(Box::new(Block { min: Vec3::new(-0.5 , -0.1, -0.3), max: Vec3::new(-0.3, 0.1, -0.1), material: ivory }));
        objects.push(Box::new(Block { min: Vec3::new(0.1 , -0.1, -0.3), max: Vec3::new(0.3, 0.1, -0.1), material: lava_material }));
        objects.push(Box::new(Block { min: Vec3::new(0.3 , -0.1, -0.3), max: Vec3::new(0.5, 0.1, -0.1), material: ivory }));

         //Aqui ira el suelo de arriba: 
         objects.push(Box::new(Block { min: Vec3::new(-0.1 , -0.1, -0.1), max: Vec3::new(0.1, 0.1, 0.1), material: ivory }));
         objects.push(Box::new(Block { min: Vec3::new(-0.3 , -0.1, -0.1), max: Vec3::new(-0.1, 0.1, 0.1), material: lava_material }));
         objects.push(Box::new(Block { min: Vec3::new(-0.5 , -0.1, -0.1), max: Vec3::new(-0.3, 0.1, 0.1), material: mirror }));
         objects.push(Box::new(Block { min: Vec3::new(0.1 , -0.1, -0.1), max: Vec3::new(0.3, 0.1, 0.1), material: lava_material }));
         objects.push(Box::new(Block { min: Vec3::new(0.3 , -0.1, -0.1), max: Vec3::new(0.5, 0.1, 0.1), material: rubber }));    

        //Aqui ira el suelo de arriba: 
        objects.push(Box::new(Block { min: Vec3::new(-0.1 , -0.1, 0.1), max: Vec3::new(0.1, 0.1, 0.3), material: ivory }));
        objects.push(Box::new(Block { min: Vec3::new(-0.3 , -0.1, 0.1), max: Vec3::new(-0.1, 0.1, 0.3), material: lava_material }));
        objects.push(Box::new(Block { min: Vec3::new(-0.5 , -0.1, 0.1), max: Vec3::new(-0.3, 0.1, 0.3), material: ivory }));
        objects.push(Box::new(Block { min: Vec3::new(0.1 , -0.1, 0.1), max: Vec3::new(0.3, 0.1, 0.3), material: lava_material }));
        objects.push(Box::new(Block { min: Vec3::new(0.3 , -0.1, 0.1), max: Vec3::new(0.5, 0.1, 0.3), material: ivory }));   

        //Aqui ira el suelo de arriba: 
        objects.push(Box::new(Block { min: Vec3::new(-0.1 , -0.1, 0.3), max: Vec3::new(0.1, 0.1, 0.5), material: ivory }));
        objects.push(Box::new(Block { min: Vec3::new(-0.3 , -0.1, 0.3), max: Vec3::new(-0.1, 0.1, 0.5), material: lava_material }));
        objects.push(Box::new(Block { min: Vec3::new(-0.5 , -0.1, 0.3), max: Vec3::new(-0.3, 0.1, 0.5), material: ivory }));
        objects.push(Box::new(Block { min: Vec3::new(0.1 , -0.1, 0.3), max: Vec3::new(0.3, 0.1, 0.5), material: water_material }));
        objects.push(Box::new(Block { min: Vec3::new(0.3 , -0.1, 0.3), max: Vec3::new(0.5, 0.1, 0.5), material: ivory }));   

         
//---------------------------------------------------------------------//
        //Aqui ira el suelo de arriba segunda capa: 
        objects.push(Box::new(Block { min: Vec3::new(-0.1 , 0.1, -0.3), max: Vec3::new(0.1, 0.3, -0.5), material: block_material }));
        objects.push(Box::new(Block { min: Vec3::new(-0.3 , 0.1, -0.3), max: Vec3::new(-0.1, 0.3, -0.5), material: ivory }));
        objects.push(Box::new(Block { min: Vec3::new(-0.5 , 0.1, -0.3), max: Vec3::new(-0.3, 0.3, -0.5), material: lava_material }));
        objects.push(Box::new(Block { min: Vec3::new(0.1 , 0.1, -0.3), max: Vec3::new(0.3, 0.3, -0.5), material: rubber }));
        objects.push(Box::new(Block { min: Vec3::new(0.3 , 0.1, -0.3), max: Vec3::new(0.5, 0.3, -0.5), material: ivory }));  

        //Aqui ira el suelo de arriba segunda capa: 
        objects.push(Box::new(Block { min: Vec3::new(-0.1 , 0.1, -0.3), max: Vec3::new(0.1, 0.3, -0.1), material: lava_material }));
        objects.push(Box::new(Block { min: Vec3::new(-0.3 , 0.1, -0.3), max: Vec3::new(-0.1, 0.3, -0.1), material: block_material }));
        objects.push(Box::new(Block { min: Vec3::new(-0.5 , 0.1, -0.3), max: Vec3::new(-0.3, 0.3, -0.1), material: ivory }));
        objects.push(Box::new(Block { min: Vec3::new(0.1 , 0.1, -0.3), max: Vec3::new(0.3, 0.3, -0.1), material: lava_material }));
        objects.push(Box::new(Block { min: Vec3::new(0.3 , 0.1, -0.3), max: Vec3::new(0.5, 0.3, -0.1), material: block_material }));

         //Aqui ira el suelo de arriba segunda capa: 
         objects.push(Box::new(Block { min: Vec3::new(-0.1 , 0.1, -0.1), max: Vec3::new(0.1, 0.3, 0.1), material: ivory }));
         objects.push(Box::new(Block { min: Vec3::new(-0.3 , 0.1, -0.1), max: Vec3::new(-0.1, 0.3, 0.1), material: lava_material }));
         objects.push(Box::new(Block { min: Vec3::new(-0.5 , 0.1, -0.1), max: Vec3::new(-0.3, 0.3, 0.1), material: block_material }));
         objects.push(Box::new(Block { min: Vec3::new(0.1 , 0.1, -0.1), max: Vec3::new(0.3, 0.3, 0.1), material: ivory }));
         objects.push(Box::new(Block { min: Vec3::new(0.3 , 0.1, -0.1), max: Vec3::new(0.5, 0.3, 0.1), material: water_material }));    

        //Aqui ira el suelo de arriba segunda capa: 
        objects.push(Box::new(Block { min: Vec3::new(-0.1 , 0.1, 0.1), max: Vec3::new(0.1, 0.3, 0.3), material: ivory }));
        objects.push(Box::new(Block { min: Vec3::new(-0.3 , 0.1, 0.1), max: Vec3::new(-0.1, 0.3, 0.3), material: block_material }));
        objects.push(Box::new(Block { min: Vec3::new(-0.5 , 0.1, 0.1), max: Vec3::new(-0.3, 0.3, 0.3), material: lava_material }));
        objects.push(Box::new(Block { min: Vec3::new(0.1 , 0.1, 0.1), max: Vec3::new(0.3, 0.3, 0.3), material: lava_material }));
        objects.push(Box::new(Block { min: Vec3::new(0.3 , 0.1, 0.1), max: Vec3::new(0.5, 0.3, 0.3), material: water_material }));   

        //Aqui ira el suelo de arriba segunda capa: 
        objects.push(Box::new(Block { min: Vec3::new(-0.1 , 0.1, 0.3), max: Vec3::new(0.1, 0.3, 0.5), material: ivory }));
        objects.push(Box::new(Block { min: Vec3::new(-0.3 , 0.1, 0.3), max: Vec3::new(-0.1, 0.3, 0.5), material: lava_material }));
        objects.push(Box::new(Block { min: Vec3::new(-0.5 , 0.1, 0.3), max: Vec3::new(-0.3, 0.3, 0.5), material: block_material }));
        objects.push(Box::new(Block { min: Vec3::new(0.1 , 0.1, 0.3), max: Vec3::new(0.3, 0.3, 0.5), material: lava_material }));
        objects.push(Box::new(Block { min: Vec3::new(0.3 , 0.1, 0.3), max: Vec3::new(0.5, 0.3, 0.5), material: block_material }));  


        // Bloques randoms hasta arriba
        objects.push(Box::new(Block { min: Vec3::new(-0.1 , 0.3, -0.5), max: Vec3::new(0.1, 0.5, -0.3), material: ivory }));
        objects.push(Box::new(Block { min: Vec3::new(-0.3 , 0.3, -0.5), max: Vec3::new(-0.1, 0.5, -0.3), material: lava_material }));
        objects.push(Box::new(Block { min: Vec3::new(-0.5 , 0.3, -0.3), max: Vec3::new(-0.3, 0.5, -0.1), material: block_material }));
        objects.push(Box::new(Block { min: Vec3::new(0.1 , 0.3, -0.5), max: Vec3::new(0.3, 0.5, -0.3), material: lava_material }));
        objects.push(Box::new(Block { min: Vec3::new(0.3 , 0.3, -0.3), max: Vec3::new(0.5, 0.5, -0.1), material: block_material }));  
        objects.push(Box::new(Block { min: Vec3::new(0.3 , 0.3, -0.1), max: Vec3::new(0.5, 0.5, 0.1), material: block_material }));  
        objects.push(Box::new(Block { min: Vec3::new(0.3 , 0.3, 0.3), max: Vec3::new(0.5, 0.5, 0.5), material: block_material }));  
        objects.push(Box::new(Block { min: Vec3::new(-0.5 , 0.3, 0.3), max: Vec3::new(-0.3, 0.5, 0.5), material: block_material }));  

        
        // Bloques randoms hasta arriba mas arriba
        //objects.push(Box::new(Block { min: Vec3::new(-0.1 , 0.5, -0.5), max: Vec3::new(0.1, 0.7, -0.3), material: ivory }));
        objects.push(Box::new(Block { min: Vec3::new(-0.3 , 0.5, -0.5), max: Vec3::new(-0.1, 0.7, -0.3), material: lava_material }));
        objects.push(Box::new(Block { min: Vec3::new(-0.5 , 0.5, -0.5), max: Vec3::new(-0.3, 0.7, -0.3), material: block_material }));
        objects.push(Box::new(Block { min: Vec3::new(0.1 , 0.5, -0.5), max: Vec3::new(0.3, 0.7, -0.3), material: lava_material }));
        objects.push(Box::new(Block { min: Vec3::new(0.3 , 0.5, -0.5), max: Vec3::new(0.5, 0.7, -0.3), material: block_material }));  
        objects.push(Box::new(Block { min: Vec3::new(0.3 , 0.5, -0.3), max: Vec3::new(0.5, 0.7, -0.1), material: block_material }));  
        objects.push(Box::new(Block { min: Vec3::new(0.3 , 0.7, -0.5), max: Vec3::new(0.5, 0.9, -0.3), material: block_material }));  


        //Otro bloque a la par, aqui iran el flujo de los bloques de hasta el fondo
        objects.push(Box::new(Block { min: Vec3::new(-0.3 , -0.9, -0.5), max: Vec3::new(-0.1, -0.7, -0.3), material: ivory }));
        objects.push(Box::new(Block { min: Vec3::new(-0.5 , -0.9, -0.5), max: Vec3::new(-0.3, -0.7, -0.3), material: lava_material }));
        objects.push(Box::new(Block { min: Vec3::new(-0.1 , -0.9, -0.5), max: Vec3::new(0.1, -0.7, -0.3), material: lava_material }));
        objects.push(Box::new(Block { min: Vec3::new(0.1 , -0.9, -0.5), max: Vec3::new(0.3, -0.7, -0.3), material: ivory }));
        objects.push(Box::new(Block { min: Vec3::new(0.3 , -0.9, -0.5), max: Vec3::new(0.5, -0.7, -0.3), material: ivory }));

        //Otro bloque a la par, aqui iran el flujo de los bloques de hasta el fondo pero arriba
        objects.push(Box::new(Block { min: Vec3::new(-0.3 , -0.7, -0.5), max: Vec3::new(-0.1, -0.5, -0.3), material: ivory }));
        objects.push(Box::new(Block { min: Vec3::new(-0.5 , -0.7, -0.5), max: Vec3::new(-0.3, -0.5, -0.3), material: lava_material }));
        objects.push(Box::new(Block { min: Vec3::new(-0.1 , -0.7, -0.5), max: Vec3::new(0.1, -0.5, -0.3), material: lava_material }));
        objects.push(Box::new(Block { min: Vec3::new(0.1 , -0.7, -0.5), max: Vec3::new(0.3, -0.5, -0.3), material: ivory }));
        objects.push(Box::new(Block { min: Vec3::new(0.3 , -0.7, -0.5), max: Vec3::new(0.5, -0.5, -0.3), material: ivory }));

        //Otro bloque a la par, aqui iran el flujo de los bloques de hasta el fondo pero arriba dos capas
        objects.push(Box::new(Block { min: Vec3::new(-0.3 , -0.5, -0.5), max: Vec3::new(-0.1, -0.3, -0.3), material: ivory }));
        objects.push(Box::new(Block { min: Vec3::new(-0.5 , -0.5, -0.5), max: Vec3::new(-0.3, -0.3, -0.3), material: lava_material }));
        objects.push(Box::new(Block { min: Vec3::new(-0.1 , -0.5, -0.5), max: Vec3::new(0.1, -0.3, -0.3), material: lava_material }));
        objects.push(Box::new(Block { min: Vec3::new(0.1 , -0.5, -0.5), max: Vec3::new(0.3, -0.3, -0.3), material: ivory }));
        objects.push(Box::new(Block { min: Vec3::new(0.3 , -0.5, -0.5), max: Vec3::new(0.5, -0.3, -0.3), material: ivory }));

        //Otro bloque aqui iran hacia arriba los de hasta el fondo izquierda
        objects.push(Box::new(Block { min: Vec3::new(-0.5 , -0.3, -0.5), max: Vec3::new(-0.3, -0.1, -0.3), material: lava_material }));
        objects.push(Box::new(Block { min: Vec3::new(-0.5 , -0.1, -0.5), max: Vec3::new(-0.3, 0.1, -0.3), material: lava_material }));
        objects.push(Box::new(Block { min: Vec3::new(-0.5 , 0.1, -0.5), max: Vec3::new(-0.3, 0.3, -0.3), material: lava_material }));
        objects.push(Box::new(Block { min: Vec3::new(-0.5 , 0.3, -0.5), max: Vec3::new(-0.3, 0.5, -0.3), material: lava_material }));

        //Otro bloque aqui iran hacia arriba los de hasta el fondo derecha
        objects.push(Box::new(Block { min: Vec3::new(0.3 , -0.3, -0.5), max: Vec3::new(0.5, -0.1, -0.3), material: lava_material }));
        objects.push(Box::new(Block { min: Vec3::new(0.3 , -0.1, -0.5), max: Vec3::new(0.5, 0.1, -0.3), material: lava_material }));
        objects.push(Box::new(Block { min: Vec3::new(0.3 , 0.1, -0.5), max: Vec3::new(0.5, 0.3, -0.3), material: lava_material }));
        objects.push(Box::new(Block { min: Vec3::new(0.3 , 0.3, -0.5), max: Vec3::new(0.5, 0.5, -0.3), material: lava_material }));

        //iran el flujo de los bloques una capa enfrente de los de enmedio
        objects.push(Box::new(Block { min: Vec3::new(-0.3 , -0.9, -0.1), max: Vec3::new(-0.1, -0.7, 0.1), material: ivory }));
        objects.push(Box::new(Block { min: Vec3::new(-0.5 , -0.9, -0.1), max: Vec3::new(-0.3, -0.7, 0.1), material: lava_material }));
        objects.push(Box::new(Block { min: Vec3::new(-0.1 , -0.9, -0.1), max: Vec3::new(0.1, -0.7, 0.1), material: lava_material }));
        objects.push(Box::new(Block { min: Vec3::new(0.1 , -0.9, -0.1), max: Vec3::new(0.3, -0.7, 0.1), material: ivory }));
        objects.push(Box::new(Block { min: Vec3::new(0.3 , -0.9, -0.1), max: Vec3::new(0.5, -0.7, 0.1), material: ivory }));

        
        //iran el flujo de los bloques dos capas enfrente de los de enmedio
        objects.push(Box::new(Block { min: Vec3::new(-0.3 , -0.9, 0.1), max: Vec3::new(-0.1, -0.7, 0.3), material: ivory }));
        objects.push(Box::new(Block { min: Vec3::new(-0.5 , -0.9, 0.1), max: Vec3::new(-0.3, -0.7, 0.3), material: lava_material }));
        objects.push(Box::new(Block { min: Vec3::new(-0.1 , -0.9, 0.1), max: Vec3::new(0.1, -0.7, 0.3), material: lava_material }));
        objects.push(Box::new(Block { min: Vec3::new(0.1 , -0.9, 0.1), max: Vec3::new(0.3, -0.7, 0.3), material: ivory }));
        objects.push(Box::new(Block { min: Vec3::new(0.3 , -0.9, 0.1), max: Vec3::new(0.5, -0.7, 0.3), material: ivory }));


        let mut lava_light_active = true; // Variable para controlar el estado de la luz

        let mut camera = Camera::new(
            Vec3::new(0.0, 0.0, 5.0),
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(0.0, 1.0, 0.0) // Vector 'up'
        );

        while window.is_open() {
            if window.is_key_down(Key::Left) {
                camera.orbit(rotation_speed, 0.0); 
            }

            if window.is_key_down(Key::Right) {
                camera.orbit(-rotation_speed, 0.0);
            }

            if window.is_key_down(Key::Up) {
                camera.orbit(0.0, -rotation_speed);
            }

            if window.is_key_down(Key::Down) {
                camera.orbit(0.0, rotation_speed);
            }

            // Manejo de la entrada del teclado
            if window.is_key_down(Key::S) {
                camera.zoom(-0.1); // Acercar
            }
            if window.is_key_down(Key::W) {
                camera.zoom(0.1); // Alejar
            }
            if window.is_key_down(Key::L) { //L para la luz de la lava 
            lava_light_active = !lava_light_active; // Alterna el estado
            println!("Luz de lava activa: {}", lava_light_active); // Debug
        }

        let lava_light = Light::new(
            Vec3::new(-1.3, -0.7, -1.3), // Alinea la luz con el centro del bloque de lava // -1.3 -0.7 -1.3
            Color::new(255, 100, 0),    // Color brillante para la lava
            if lava_light_active ==false { 5.0 } else { 0.0 }, // Cambia la intensidad según el estado 
            true,              
        );
        
        
        let sunlight = Light::new(
            Vec3::new(5.0, 5.0, 5.0),
            Color::new(255, 100, 0),
            0.5,
            true,
        );

        let lights = vec![
            lava_light,
            sunlight,
        ];

            render(&mut framebuffer, &objects, &camera, &lights);

            window
                .update_with_buffer(&framebuffer.buffer, framebuffer_width, framebuffer_height)
                .unwrap();

            std::thread::sleep(frame_delay);
        }
    }