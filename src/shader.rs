use super::{Triangle, Vertex, Viewport};

pub trait Pixels {
    fn get_pixel(&self, xx: usize, yy: usize) -> (u8, u8, u8);
}

pub struct BgraPixels<'a> {
    pixels: &'a [u8],
    width: u32,
}

impl<'a> BgraPixels<'a> {
    pub fn new(pixels: &'a [u8], width: u32) -> Self {
        BgraPixels { pixels, width }
    }
}

impl<'a> Pixels for BgraPixels<'a> {
    fn get_pixel(&self, xx: usize, yy: usize) -> (u8, u8, u8) {
        // Image is upwards contrary to sideways renderer.
        let offs = (xx + yy * self.width as usize) * 4;
        let b = self.pixels[offs];
        let g = self.pixels[offs + 1];
        let r = self.pixels[offs + 2];
        (r, g, b)
    }
}

pub struct TextureShader<P: Pixels> {
    pub pixels: P,
    pub width: u32,
    pub height: u32,
}

impl<P: Pixels> TextureShader<P> {
    fn shade(&self, x: f32, y: f32, intensity: f32) -> (u8, u8, u8) {
        let xx = ((self.width - 1) as f32 * x) as usize;
        let yy = ((self.height - 1) as f32 * y) as usize;
        let shading = (255.0 * intensity.max(0.0)) as u8;
        let (r, g, b) = self.pixels.get_pixel(xx, yy);
        let r = (r as usize * shading as usize) / 256;
        let g = (g as usize * shading as usize) / 256;
        let b = (b as usize * shading as usize) / 256;
        (r as u8, g as u8, b as u8)
    }
}

pub fn draw_shaded_triangle<P: Pixels>(
    target: &mut Viewport,
    vew: &Triangle,
    nrm: &Triangle,
    tex: &Triangle,
    shader: &TextureShader<P>,
) {
    target.draw_triangle(vew, |target, x, y| {
        let bc = vew.clone().barycenter(x, y);
        if bc.x >= 0.0 && bc.y >= 0.0 && bc.z >= 0.0 {
            // Barycenter above is upwards. Everything below rotated 90 degrees to accomodate sideways renderer.
            let z = bc.x * vew.b.z + bc.y * vew.c.z + bc.z * vew.a.z;
            target.draw(x, y, z, || {
                let light = Vertex {
                    x: 0.0,
                    y: 0.0,
                    z: 1.0,
                };
                let varying = Vertex {
                    x: light.dot(&nrm.b),
                    y: light.dot(&nrm.c),
                    z: light.dot(&nrm.a),
                };
                let xx = 0.0 + (bc.x * tex.b.x + bc.y * tex.c.x + bc.z * tex.a.x);
                let yy = 1.0 - (bc.x * tex.b.y + bc.y * tex.c.y + bc.z * tex.a.y);
                let intensity = bc.dot(&varying);
                shader.shade(xx, yy, intensity)
            });
        }
    });
}
