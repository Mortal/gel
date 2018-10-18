extern crate gel;
extern crate sdl2;

use gel::obj::Obj;
use gel::*;
use sdl2::pixels::{PixelFormat, PixelFormatEnum};
use sdl2::surface::Surface;
use std::env;
mod render;
use render::render_loop;
mod mouse;

fn make_pixel_format(format: PixelFormatEnum) -> PixelFormat {
    Surface::new(1, 1, format).unwrap().pixel_format()
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
        Err(e) => {
            println!("Could not read {}: {}", obj_filename, e);
            return;
        }
    };
    let bmp_filename = format!("model/{}.bmp", model);
    let dif = match Surface::load_bmp(&bmp_filename) {
        Ok(bmp) => bmp,
        Err(e) => {
            println!("Could not read {}: {}", bmp_filename, e);
            return;
        }
    };
    let dif = dif.convert(&make_pixel_format(PixelFormatEnum::RGB888))
        .unwrap();
    assert_eq!(dif.pitch(), 4 * dif.width());

    let xres = 800;
    let yres = 600;

    render_loop(&model, xres, yres, |viewport: &mut Viewport| {
        dif.with_lock(|difpixels| {
            let shader = TextureShader {
                pixels: BgraPixels::new(difpixels, dif.width()),
                width: dif.width(),
                height: dif.height(),
            };
            obj.draw_shaded(viewport, &shader);
            None as Option<()>
        })
    });
}
