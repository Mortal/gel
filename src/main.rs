extern crate sdl2;
mod geom;
use geom::*;
mod obj;
use obj::*;
mod asciidigits;
mod mouse;
mod viewport;
use viewport::*;
use std::env;
use std::f32;
use sdl2::surface::Surface;
use sdl2::pixels::{PixelFormatEnum};

struct TextureShader<'a> {
    pixels: &'a [u8],
    width: u32,
    height: u32,
}

impl<'a> TextureShader<'a> {
    fn shade(&self, x: f32, y: f32, intensity: f32) -> (u8, u8, u8) {
        let xx = ((self.width - 1) as f32 * x) as usize;
        let yy = ((self.height - 1) as f32 * y) as usize;
        let shading = (255.0 * intensity.max(0.0)) as u8;
        // Image is upwards contrary to sideways renderer.
        let offs = (xx + yy * self.width as usize) * 4;
        let b = self.pixels[offs];
        let g = self.pixels[offs+1];
        let r = self.pixels[offs+2];
        let r = (r as usize * shading as usize) / 256;
        let g = (g as usize * shading as usize) / 256;
        let b = (b as usize * shading as usize) / 256;
        (r as u8, g as u8, b as u8)
    }
}

fn draw_shaded_triangle(target: &mut Viewport, vew: &Triangle, nrm: &Triangle, tex: &Triangle, shader: &TextureShader) {
    target.draw_triangle(vew, |target, x, y| {
        let bc = vew.clone().barycenter(x, y);
        if bc.x >= 0.0 && bc.y >= 0.0 && bc.z >= 0.0 {
            // Barycenter above is upwards. Everything below rotated 90 degrees to accomodate sideways renderer.
            let z = bc.x * vew.b.z + bc.y * vew.c.z + bc.z * vew.a.z;
            target.draw(x, y, z, || {
                let light = Vertex { x: 0.0, y: 0.0, z: 1.0 };
                let varying = Vertex { x: light.dot(&nrm.b), y: light.dot(&nrm.c), z: light.dot(&nrm.a) };
                let xx = 0.0 + (bc.x * tex.b.x + bc.y * tex.c.x + bc.z * tex.a.x);
                let yy = 1.0 - (bc.x * tex.b.y + bc.y * tex.c.y + bc.z * tex.a.y);
                let intensity = bc.dot(&varying);
                shader.shade(xx, yy, intensity)
            });
        }
    });
}

fn main() {
    let model = {
        let mut args = env::args();
        let program_name = args.next().unwrap_or("gel".to_owned());
        if let Some(arg) = args.next() {
            if arg.starts_with("-") {
                println!("Usage: {} [model_name]", program_name);
                return;
            }
            arg
        } else {
            "salesman".to_owned()
        }
    };
    let obj_filename = format!("model/{}.obj", model);
    let obj = match Obj::load(&obj_filename) {
        Ok(obj) => obj,
        Err(e) => {println!("Could not read {}: {}", obj_filename, e); return;},
    };
    let bmp_filename = format!("model/{}.bmp", model);
    let dif = match Surface::load_bmp(&bmp_filename) {
        Ok(bmp) => bmp,
        Err(e) => {println!("Could not read {}: {}", bmp_filename, e); return;},
    };
    let dif = dif.convert(&make_pixel_format(PixelFormatEnum::RGB888)).unwrap();
    assert_eq!(dif.pitch(), 4*dif.width());
    let vertices = obj.tvgen();
    let textures = obj.ttgen();
    let normals = obj.tngen();

    let xres: u32 = 800;
    let yres: u32 = 600;

    render_loop(&model, xres, yres, |viewport: &mut Viewport| {
        dif.with_lock(|difpixels| {
            let shader = TextureShader {
                pixels: difpixels,
                width: dif.width(),
                height: dif.height(),
            };
            for ((nrm, tex), tri) in normals.iter().zip(textures.iter()).zip(vertices.iter()) {
                let nrm = nrm.clone().view_normal(&viewport.x(), &viewport.y(), &viewport.z()).unit();
                let tri = tri.clone().view_triangle(&viewport.x(), &viewport.y(), &viewport.z(), &viewport.eye());
                let per = tri.perspective();
                let vew = per.viewport(xres, yres);
                draw_shaded_triangle(viewport, &vew, &nrm, &tex, &shader);
            }
            None as Option<()>
        })
    });
}
