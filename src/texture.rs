use image::{DynamicImage, GenericImageView};

#[derive(Debug, Clone)]
pub struct Texture {
    data: Vec<u8>,
    width: u32,
    height: u32,
}

impl Texture {
    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }
    pub fn new(file_path: &str) -> Self {
        let img = image::open(file_path).expect("Failed to load texture");
        let (width, height) = img.dimensions();
        let data = img.into_bytes();

        Texture { data, width, height }
    }

    pub fn get_color(&self, x: u32, y: u32) -> (u8, u8, u8) {
        let idx = ((y * self.width + x) * 3) as usize;
        (
            self.data[idx],
            self.data[idx + 1],
            self.data[idx + 2],
        )
    }
}
