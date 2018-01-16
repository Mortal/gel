extern crate sdl2;
mod geom;
use geom::*;
mod obj;
use obj::*;
use std::env;
use std::f32;
use sdl2::EventPump;
use sdl2::surface::Surface;
use sdl2::pixels::{PixelFormatEnum, PixelFormat};
use sdl2::rect::Rect;
use sdl2::event::{Event, WindowEvent, EventWaitIterator};

fn make_pixel_format(format: PixelFormatEnum) -> PixelFormat {
    Surface::new(1, 1, format).unwrap().pixel_format()
}

struct ZBufferedTarget<'a> {
    yres: u32,
    pixel: &'a mut[u8],
    zbuff: &'a mut[f32],
}

impl<'a> ZBufferedTarget<'a> {
    fn draw<F: FnOnce() -> (u8, u8, u8)>(&mut self, x: isize, y: isize, z: f32, f: F) {
        let idx = y as usize + x as usize * self.yres as usize;
        if self.zbuff[idx] < z {
            self.zbuff[idx] = z;
            let (r, g, b) = f();
            let output = idx * 4;
            self.pixel[output] = b;
            self.pixel[output + 1] = g;
            self.pixel[output + 2] = r;
        }
    }

    fn reset(&mut self) {
        for v in self.pixel.iter_mut() { *v = 0; }
        for v in self.zbuff.iter_mut() { *v = f32::MIN; }
    }
}

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

fn draw(target: &mut ZBufferedTarget, vew: &Triangle, nrm: &Triangle, tex: &Triangle, shader: &TextureShader) {
    let xmin = vew.a.x.min(vew.b.x).min(vew.c.x) as isize;
    let ymin = vew.a.y.min(vew.b.y).min(vew.c.y) as isize;
    let xmax = vew.a.x.max(vew.b.x).max(vew.c.x) as isize + 1;
    let ymax = vew.a.y.max(vew.b.y).max(vew.c.y) as isize + 1;
    for x in xmin..xmax {
        for y in ymin..ymax {
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
        }
    }
}

struct MouseIterator<'a> {
    inner: EventWaitIterator<'a>,
    x: f32,
    y: f32,
    sens: f32,
}

impl<'a> Iterator for MouseIterator<'a> {
    type Item = (f32, f32);

    fn next(&mut self) -> Option<(f32, f32)> {
        loop {
            let event = match self.inner.next() {
                None => return None,
                Some(e) => e,
            };
            match event {
                Event::Quit { timestamp: _ } => return None,
                Event::Window { timestamp: _, window_id: _, win_event: WindowEvent::Exposed } =>
                    break,
                Event::Window { timestamp: _, window_id: _, win_event: WindowEvent::Close } =>
                    return None,
                Event::MouseMotion {
                    timestamp: _, window_id: _, which: _, mousestate: _, x: _, y: _, xrel, yrel,
                } => {
                    self.x -= self.sens * xrel as f32;
                    self.y += self.sens * yrel as f32;
                    break;
                },
                _ => (),
            };
        }
        Some((self.x, self.y))
    }
}

fn mouse_iter(pump: &mut EventPump) -> MouseIterator {
    MouseIterator {
        inner: pump.wait_iter(),
        x: 0.0,
        y: 0.0,
        sens: 0.005,
    }
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

    let sdl = sdl2::init().unwrap();
    let video = sdl.video().unwrap();
    let window = video.window(&model, xres, yres).build().unwrap();
    let mut canvas = window.into_canvas().present_vsync().build().unwrap();
    let renderer = canvas.texture_creator();
    // Notice the flip between xres and yres - the renderer is on its side to maximize cache effeciency.
    let mut texture = renderer.create_texture_streaming(PixelFormatEnum::ARGB8888, yres, xres).unwrap();

    let mut zbuff = Vec::new();
    zbuff.resize((xres * yres) as usize, 0f32);
    sdl.mouse().set_relative_mouse_mode(false);
    for (xt, yt) in mouse_iter(&mut sdl.event_pump().unwrap()) {
        texture.with_lock(None, |pixel, _pitch| {
            dif.with_lock(|difpixels| {
                let mut target = ZBufferedTarget {
                    yres: yres,
                    pixel: pixel,
                    zbuff: &mut zbuff,
                };
                let shader = TextureShader {
                    pixels: difpixels,
                    width: dif.width(),
                    height: dif.height(),
                };
                target.reset();
                let eye = Vertex { x: xt.sin(), y: yt.sin(), z: xt.cos() };
                let z = (eye.clone() - Vertex::center()).unit();
                let x = (Vertex::upward().cross(z.clone())).unit();
                let y = z.clone().cross(x.clone());
                for ((nrm, tex), tri) in normals.iter().zip(textures.iter()).zip(vertices.iter()) {
                    let nrm = nrm.clone().view_normal(&x, &y, &z).unit();
                    let tri = tri.clone().view_triangle(&x, &y, &z, &eye);
                    let per = tri.perspective();
                    let vew = per.viewport(xres, yres);
                    draw(&mut target, &vew, &nrm, &tex, &shader);
                }
            });
        }).unwrap();
        let dst = Rect::new((xres as i32 - yres as i32) / 2,
                            (yres as i32 - xres as i32) / 2,
                            yres, xres);
        canvas.copy_ex(&texture, None, dst, -90f64, None, false, false).unwrap();
        canvas.present();
    }
}
