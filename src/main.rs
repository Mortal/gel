extern crate sdl2;
use std::f32;
use std::ops::{Sub, Mul};
use std::path::Path;
use std::io::{BufRead, BufReader};
use std::fs::File;
use sdl2::surface::Surface;
use sdl2::pixels::{PixelFormatEnum, PixelFormat};
use sdl2::rect::Rect;

#[derive(Debug, Clone)]
struct Vertex {
    x: f32,
    y: f32,
    z: f32,
}

impl Vertex {
    fn center() -> Self {
        Vertex { x: 0.0, y: 0.0, z: 0.0 }
    }

    fn upward() -> Self {
        Vertex { x: 0.0, y: 1.0, z: 0.0 }
    }

    fn len(&self) -> f32 {
        (self.x*self.x + self.y*self.y + self.z*self.z).sqrt()
    }

    fn unit(self) -> Self {
        let l = self.len();
        self * (1f32 / l)
    }

    fn dot(&self, other: &Vertex) -> f32 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    fn cross(self, other: Vertex) -> Self {
        Vertex {
            x: self.y * other.z - self.z * other.y,
            y: self.z * other.x - self.x * other.z,
            z: self.x * other.y - self.y * other.x,
        }
    }

    fn view_normal(&self, x: &Vertex, y: &Vertex, z: &Vertex) -> Self {
        Vertex {
            x: self.dot(x),
            y: self.dot(y),
            z: self.dot(z),
        }
    }

    fn view_triangle(&self, x: &Vertex, y: &Vertex, z: &Vertex, eye: &Vertex) -> Self {
        Vertex {
            x: self.dot(x) - x.dot(eye),
            y: self.dot(y) - y.dot(eye),
            z: self.dot(z) - z.dot(eye),
        }
    }
}

impl Mul<f32> for Vertex {
    type Output = Vertex;
    fn mul(mut self, f: f32) -> Vertex {
        self.x *= f;
        self.y *= f;
        self.z *= f;
        self
    }
}

impl Sub for Vertex {
    type Output = Vertex;
    fn sub(mut self, other: Vertex) -> Vertex {
        self.x -= other.x;
        self.y -= other.y;
        self.z -= other.z;
        self
    }
}

struct Face {
    va: usize,
    vb: usize,
    vc: usize,
    ta: usize,
    tb: usize,
    tc: usize,
    na: usize,
    nb: usize,
    nc: usize,
}

#[derive(Debug, Clone)]
struct Triangle {
    a: Vertex,
    b: Vertex,
    c: Vertex,
}

impl Triangle {
    fn new(a: Vertex, b: Vertex, c: Vertex) -> Triangle {
        Triangle {
            a: a,
            b: b,
            c: c,
        }
    }

    fn unit(self) -> Triangle {
        Triangle {
            a: self.a.unit(),
            b: self.b.unit(),
            c: self.c.unit(),
        }
    }

    fn view_normal(self, x: &Vertex, y: &Vertex, z: &Vertex) -> Self {
        Triangle {
            a: self.a.view_normal(x, y, z),
            b: self.b.view_normal(x, y, z),
            c: self.c.view_normal(x, y, z),
        }
    }

    fn view_triangle(self, x: &Vertex, y: &Vertex, z: &Vertex, eye: &Vertex) -> Self {
        Triangle {
            a: self.a.view_triangle(x, y, z, eye),
            b: self.b.view_triangle(x, y, z, eye),
            c: self.c.view_triangle(x, y, z, eye),
        }
    }

    fn perspective(mut self) -> Self {
        let c = 3.0;
        let za = 1.0 - self.a.z / c;
        let zb = 1.0 - self.b.z / c;
        let zc = 1.0 - self.c.z / c;
        self.a = self.a * (1.0 / za);
        self.b = self.b * (1.0 / zb);
        self.c = self.c * (1.0 / zc);
        self
    }

    fn viewport(self, xres: u32, yres: u32) -> Self {
        let w = yres as f32 / 1.5; // Should maybe be xres?
        let h = yres as f32 / 1.5;
        let x = xres as f32 / 2.0;
        let y = yres as f32 / 4.0;
        Triangle {
            a: Vertex { x: w * self.a.x + x, y: h * self.a.y + y, z: (self.a.z + 1.0) / 1.5 },
            b: Vertex { x: w * self.b.x + x, y: h * self.b.y + y, z: (self.b.z + 1.0) / 1.5 },
            c: Vertex { x: w * self.c.x + x, y: h * self.c.y + y, z: (self.c.z + 1.0) / 1.5 },
        }
    }

    fn barycenter(self, x: isize, y: isize) -> Vertex {
        let p = Vertex { x: x as f32, y: y as f32, z: 0.0 };
        let v0 = self.b.clone() - self.a.clone();
        let v1 = self.c.clone() - self.a.clone();
        let v2 = p - self.a.clone();
        let d00 = v0.dot(&v0);
        let d01 = v0.dot(&v1);
        let d11 = v1.dot(&v1);
        let d20 = v2.dot(&v0);
        let d21 = v2.dot(&v1);
        let v = (d11 * d20 - d01 * d21) / (d00 * d11 - d01 * d01);
        let w = (d00 * d21 - d01 * d20) / (d00 * d11 - d01 * d01);
        let u = 1.0 - v - w;
        Vertex { x: v, y: w, z: u }
    }
}

impl Mul<f32> for Triangle {
    type Output = Triangle;
    fn mul(mut self, f: f32) -> Triangle {
        self.a = self.a * f;
        self.b = self.b * f;
        self.c = self.c * f;
        self
    }
}

struct Obj {
    vsv: Vec<Vertex>,
    vsn: Vec<Vertex>,
    vst: Vec<Vertex>,
    fs: Vec<Face>,
}

// scan! from https://stackoverflow.com/a/31048103/1570972
macro_rules! scan {
    ( $string:expr, $sep:expr, $( $x:ty ),+ ) => {(|| {
        let mut iter = $string.split($sep);
        let r = ($(match iter.next() { Some(v) => v.parse::<$x>().unwrap(), None => return None },)*);
        match iter.next() {
            Some(s) => panic!("scan!() got unexpected token {}", s),
            None => (),
        };
        Some(r)
    })()}
}

// fn main() {
//     let output = scan!("2 false fox", char::is_whitespace, u8, bool, String);
//     println!("{:?}", output); // (Some(2), Some(false), Some("fox"))
// }

impl Obj {
    fn load<P: AsRef<Path>>(path: P) -> Self {
        let mut vsv = Vec::new();
        let mut vsn = Vec::new();
        let mut vst = Vec::new();
        let mut fs = Vec::new();
        for line in BufReader::new(File::open(path).unwrap()).lines() {
            let line = line.unwrap();
            if line.len() == 0 || line.starts_with("#") {
                continue;
            }
            if line.starts_with("f ") {
                let (_, sx, sy, sz) = scan!(line, ' ', String, String, String, String).expect(&line);
                let (va, ta, na) = scan!(sx, '/', usize, usize, usize).unwrap();
                let (vb, tb, nb) = scan!(sy, '/', usize, usize, usize).unwrap();
                let (vc, tc, nc) = scan!(sz, '/', usize, usize, usize).unwrap();
                fs.push(Face {
                    va: va-1, vb: vb-1, vc: vc-1,
                    ta: ta-1, tb: tb-1, tc: tc-1,
                    na: na-1, nb: nb-1, nc: nc-1,
                });
            } else if line.starts_with("vn ") {
                let (_, x, y, z) = scan!(line, ' ', String, f32, f32, f32).expect(&line);
                let v = Vertex { x: x, y: y, z: z };
                vsn.push(v);
            } else if line.starts_with("vt ") {
                let (_, x, y) = scan!(line, ' ', String, f32, f32).expect(&line);
                let v = Vertex { x: x, y: y, z: 0f32 };
                vst.push(v);
            } else {
                assert!(line.starts_with("v "));
                let (_, x, y, z) = scan!(line, ' ', String, f32, f32, f32).expect(&line);
                let v = Vertex { x: x, y: y, z: z };
                vsv.push(v);
            }
        }
        Obj {
            vsv: vsv,
            vsn: vsn,
            vst: vst,
            fs: fs,
        }
    }
}

fn make_pixel_format(format: PixelFormatEnum) -> PixelFormat {
    Surface::new(1, 1, format).unwrap().pixel_format()
}

fn tvgen(obj: &Obj) -> Vec<Triangle> {
    let scale = obj.vsv.iter().map(|v| v.len()).fold(0.0, f32::max);
    obj.fs.iter().map(
        |f| Triangle::new(obj.vsv[f.va].clone(),
                          obj.vsv[f.vb].clone(),
                          obj.vsv[f.vc].clone()) * (1f32 / scale)).collect()
}

fn tngen(obj: &Obj) -> Vec<Triangle> {
    obj.fs.iter().map(
        |f| Triangle::new(obj.vsn[f.na].clone(),
                          obj.vsn[f.nb].clone(),
                          obj.vsn[f.nc].clone())).collect()
}

fn ttgen(obj: &Obj) -> Vec<Triangle> {
    obj.fs.iter().map(
        |f| Triangle::new(obj.vst[f.ta].clone(),
                          obj.vst[f.tb].clone(),
                          obj.vst[f.tc].clone())).collect()
}

fn pshade(r: u8, g: u8, b: u8, shading: u8) -> (u8, u8, u8) {
    let r = (r as usize * shading as usize) / 256;
    let g = (g as usize * shading as usize) / 256;
    let b = (b as usize * shading as usize) / 256;
    (r as u8, g as u8, b as u8)
}

fn draw(yres: u32, pixel: &mut[u8], zbuff: &mut[f32], vew: &Triangle, nrm: &Triangle, tex: &Triangle, difpixels: &[u8], difsize: (u32, u32)) {
    let xmin = vew.a.x.min(vew.b.x).min(vew.c.x) as isize;
    let ymin = vew.a.y.min(vew.b.y).min(vew.c.y) as isize;
    let xmax = vew.a.x.max(vew.b.x).max(vew.c.x) as isize + 1;
    let ymax = vew.a.y.max(vew.b.y).max(vew.c.y) as isize + 1;
    let (difwidth, difheight) = difsize;
    for x in xmin..xmax {
        for y in ymin..ymax {
            let bc = vew.clone().barycenter(x, y);
            if bc.x >= 0.0 && bc.y >= 0.0 && bc.z >= 0.0 {
                // Barycenter above is upwards. Everything below rotated 90 degrees to accomodate sideways renderer.
                let z = bc.x * vew.b.z + bc.y * vew.c.z + bc.z * vew.a.z;
                let zb = &mut zbuff[y as usize + x as usize * yres as usize];
                if z > *zb {
                    let light = Vertex { x: 0.0, y: 0.0, z: 1.0 };
                    let varying = Vertex { x: light.dot(&nrm.b), y: light.dot(&nrm.c), z: light.dot(&nrm.a) };
                    let xx = ((difwidth - 1) as f32 * (0.0 + (bc.x * tex.b.x + bc.y * tex.c.x + bc.z * tex.a.x))) as usize;
                    let yy = ((difheight - 1) as f32 * (1.0 - (bc.x * tex.b.y + bc.y * tex.c.y + bc.z * tex.a.y))) as usize;
                    let intensity = bc.dot(&varying);
                    let shading = (255.0 * intensity.max(0.0)) as u8;
                    // Image is upwards contrary to sideways renderer.
                    *zb = z;
                    let offs = (xx + yy * difwidth as usize) * 4;
                    let b = difpixels[offs];
                    let g = difpixels[offs+1];
                    let r = difpixels[offs+2];
                    let (r, g, b) = pshade(r, g, b, shading);
                    let output = (y as usize + x as usize * yres as usize) * 4;
                    pixel[output] = b;
                    pixel[output + 1] = g;
                    pixel[output + 2] = r;
                }
            }
        }
    }
}

fn main() {
    let obj = Obj::load("model/salesman.obj");
    let dif = Surface::load_bmp("model/salesman.bmp").unwrap();
    let dif = dif.convert(&make_pixel_format(PixelFormatEnum::RGB888)).unwrap();
    assert_eq!(dif.pitch(), 4*dif.width());
    let vertices = tvgen(&obj);
    let textures = ttgen(&obj);
    let normals = tngen(&obj);
    let xres: u32 = 800;
    let yres: u32 = 600;

    let sdl = sdl2::init().unwrap();
    let video = sdl.video().unwrap();
    let window = video.window("water", xres, yres).build().unwrap();
    let mut canvas = window.into_canvas().present_vsync().build().unwrap();
    let renderer = canvas.texture_creator();
    // Notice the flip between xres and yres - the renderer is on its side to maximize cache effeciency.
    let mut texture = renderer.create_texture_streaming(PixelFormatEnum::ARGB8888, yres, xres).unwrap();

    let mut zbuff = Vec::new();
    zbuff.resize((xres * yres) as usize, 0f32);
    sdl.mouse().set_relative_mouse_mode(false);
    let mut pump = sdl.event_pump().unwrap();
    let mut xt = 0f32;
    let mut yt = 0f32;
    let sens = 0.005f32;
    loop {
        pump.wait_event();
        let mouse = pump.relative_mouse_state();
        let dx = mouse.x();
        let dy = mouse.y();
        xt -= sens * (dx as f32);
        yt += sens * (dy as f32);
        let difsize = dif.size();
        texture.with_lock(None, |pixel, _pitch| {
            dif.with_lock(|difpixels| {
                for v in pixel.iter_mut() { *v = 0; }
                for v in zbuff.iter_mut() { *v = f32::MIN; }
                let eye = Vertex { x: xt.sin(), y: yt.sin(), z: xt.cos() };
                let z = (eye.clone() - Vertex::center()).unit();
                let x = (Vertex::upward().cross(z.clone())).unit();
                let y = z.clone().cross(x.clone());
                for ((nrm, tex), tri) in normals.iter().zip(textures.iter()).zip(vertices.iter()) {
                    let nrm = nrm.clone().view_normal(&x, &y, &z).unit();
                    let tri = tri.clone().view_triangle(&x, &y, &z, &eye);
                    let per = tri.perspective();
                    let vew = per.viewport(xres, yres);
                    draw(yres, pixel, &mut zbuff, &vew, &nrm, &tex, difpixels, difsize);
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
