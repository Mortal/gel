use geom::*;

use std::f32;

pub struct ZBufferedTarget<'a> {
    pub xres: usize,
    pub yres: usize,
    pub xstride: usize,
    pub ystride: usize,
    pub pixel: &'a mut [u8],
    pub zbuff: &'a mut [f32],
}

impl<'a> ZBufferedTarget<'a> {
    pub fn new_column_major(xres: usize, yres: usize, pixel: &'a mut [u8], zbuff: &'a mut [f32]) -> Self {
        Self {
            xres,
            yres,
            xstride: yres,
            ystride: 1,
            pixel,
            zbuff
        }
    }

    pub fn new_row_major(xres: usize, yres: usize, pixel: &'a mut [u8], zbuff: &'a mut [f32]) -> Self {
        Self {
            xres,
            yres,
            xstride: 1,
            ystride: xres,
            pixel,
            zbuff
        }
    }

    fn draw<F: FnOnce() -> (u8, u8, u8)>(&mut self, x: isize, y: isize, z: f32, f: F) {
        let idx = y as usize * self.ystride + x as usize * self.xstride;
        if self.zbuff[idx] < z {
            self.zbuff[idx] = z;
            let (r, g, b) = f();
            let output = idx * 4;
            self.pixel[output] = b;
            self.pixel[output + 1] = g;
            self.pixel[output + 2] = r;
        }
    }

    pub fn reset(&mut self) {
        for v in self.pixel.iter_mut() {
            *v = 0;
        }
        for v in self.zbuff.iter_mut() {
            *v = f32::MIN;
        }
    }
}

pub struct Viewport<'a> {
    pub target: ZBufferedTarget<'a>,
    pub x: Vertex,
    pub y: Vertex,
    pub z: Vertex,
    pub eye: Vertex,
}

impl<'a> Viewport<'a> {
    pub fn x(&self) -> Vertex {
        self.x.clone()
    }
    pub fn y(&self) -> Vertex {
        self.y.clone()
    }
    pub fn z(&self) -> Vertex {
        self.z.clone()
    }
    pub fn eye(&self) -> Vertex {
        self.eye.clone()
    }

    pub fn draw<F: FnOnce() -> (u8, u8, u8)>(&mut self, x: isize, y: isize, z: f32, f: F) {
        self.target.draw(x, y, z, f)
    }

    pub fn xres(&self) -> usize {
        self.target.xres
    }
    pub fn yres(&self) -> u32 {
        self.target.yres as u32
    }

    pub fn draw_triangle<F: FnMut(&mut Viewport, isize, isize)>(
        &mut self,
        vew: &Triangle,
        mut f: F,
    ) {
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
