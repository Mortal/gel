use super::geom::*;
use super::mouse::*;
use super::asciidigits;

use std::f32;
use std::time::{Instant, Duration};

use super::sdl2;
use super::sdl2::surface::Surface;
use super::sdl2::pixels::{PixelFormatEnum, PixelFormat};
use super::sdl2::rect::Rect;

pub fn make_pixel_format(format: PixelFormatEnum) -> PixelFormat {
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

    fn xres(&self) -> usize {
        self.zbuff.len() / self.yres as usize
    }
}

struct SumWindow {
    history: Vec<Duration>,
    idx: usize,
    sum: Duration,
}

impl SumWindow {
    fn new(window_size: usize) -> Self {
        let mut history = Vec::new();
        history.resize(window_size, Duration::new(0, 0));
        SumWindow {
            history: history,
            idx: 0,
            sum: Duration::new(0, 0),
        }
    }

    fn tick(&mut self, d: Duration) -> f64 {
        self.sum += d;
        self.sum -= self.history[self.idx];
        self.history[self.idx] = d;
        self.idx = if self.idx + 1 == self.history.len() { 0 } else { self.idx + 1 };
        let elapsed = self.sum.as_secs() as f64 + self.sum.subsec_nanos() as f64 * 1e-9;
        self.history.len() as f64 / elapsed
    }
}

pub struct Viewport<'a> {
    target: ZBufferedTarget<'a>,
    x: Vertex,
    y: Vertex,
    z: Vertex,
    eye: Vertex,
}

impl<'a> Viewport<'a> {
    pub fn x(&self) -> Vertex { self.x.clone() }
    pub fn y(&self) -> Vertex { self.y.clone() }
    pub fn z(&self) -> Vertex { self.z.clone() }
    pub fn eye(&self) -> Vertex { self.eye.clone() }

    pub fn draw<F: FnOnce() -> (u8, u8, u8)>(&mut self, x: isize, y: isize, z: f32, f: F) {
        self.target.draw(x, y, z, f)
    }

    pub fn xres(&self) -> usize { self.target.xres() }
    pub fn yres(&self) -> u32 { self.target.yres }

    pub fn draw_triangle<F: FnMut(&mut Viewport, isize, isize)>(&mut self, vew: &Triangle, mut f: F) {
        let xmin = vew.a.x.min(vew.b.x).min(vew.c.x).max(0.0) as isize;
        let ymin = vew.a.y.min(vew.b.y).min(vew.c.y).max(0.0) as isize;
        let xmax = (vew.a.x.max(vew.b.x).max(vew.c.x) as isize + 1).min(self.xres() as isize);
        let ymax = (vew.a.y.max(vew.b.y).max(vew.c.y) as isize + 1).min(self.yres() as isize);
        if xmin >= xmax || ymin >= ymax {
            return;
        }
        for x in xmin..xmax {
            for y in ymin..ymax {
                f(self, x, y);
            }
        }
    }
}

pub fn render_loop<R, F>(title: &str, xres: u32, yres: u32, mut render_frame: F) -> Option<R>
    where F: FnMut(&mut Viewport) -> Option<R>
{
    let sdl = sdl2::init().unwrap();
    let video = sdl.video().unwrap();
    let window = video.window(title, xres, yres).build().unwrap();
    let mut canvas = window.into_canvas().present_vsync().build().unwrap();
    let renderer = canvas.texture_creator();
    // Notice the flip between xres and yres - the renderer is on its side to maximize cache effeciency.
    let mut texture = renderer.create_texture_streaming(PixelFormatEnum::ARGB8888, yres, xres).unwrap();

    let mut zbuff = Vec::new();
    zbuff.resize((xres * yres) as usize, 0f32);
    sdl.mouse().set_relative_mouse_mode(false);
    let mut fps_counter = SumWindow::new(30);
    let mut result = None;
    for (eye, eye_dir) in mouse_camera(&mut sdl.event_pump().unwrap()) {
        texture.with_lock(None, |pixel, _pitch| {
            let t = Instant::now();
            let mut target = ZBufferedTarget {
                yres: yres,
                pixel: pixel,
                zbuff: &mut zbuff,
            };
            target.reset();
            let z = (eye_dir.clone() - Vertex::center()).unit();
            let x = (Vertex::upward().cross(z.clone())).unit();
            let y = z.clone().cross(x.clone());

            let mut viewport = Viewport {
                target: target,
                x: x,
                y: y,
                z: z,
                eye: eye,
            };

            result = render_frame(&mut viewport);

            let fps = fps_counter.tick(t.elapsed());
            for (i, d) in format!("{}", fps as usize).chars().enumerate() {
                asciidigits::draw(d as usize - '0' as usize, |y, x| {
                    let o = (x+i*6)*yres as usize + (yres as usize - 1 - y);
                    viewport.target.pixel[4*o] = 255;
                    viewport.target.pixel[4*o+1] = 255;
                    viewport.target.pixel[4*o+2] = 255;
                });
            }
        }).unwrap();
        match result {
            Some(r) => return Some(r),
            None => (),
        };
        let dst = Rect::new((xres as i32 - yres as i32) / 2,
                            (yres as i32 - xres as i32) / 2,
                            yres, xres);
        canvas.copy_ex(&texture, None, dst, -90f64, None, false, false).unwrap();
        canvas.present();
    }
    None
}
