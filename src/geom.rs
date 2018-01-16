use std::ops::{Sub, Mul};

#[derive(Debug, Clone)]
pub struct Vertex {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vertex {
    pub fn center() -> Self {
        Vertex { x: 0.0, y: 0.0, z: 0.0 }
    }

    pub fn upward() -> Self {
        Vertex { x: 0.0, y: 1.0, z: 0.0 }
    }

    pub fn len(&self) -> f32 {
        (self.x*self.x + self.y*self.y + self.z*self.z).sqrt()
    }

    pub fn unit(self) -> Self {
        let l = self.len();
        self * (1f32 / l)
    }

    pub fn dot(&self, other: &Vertex) -> f32 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    pub fn cross(self, other: Vertex) -> Self {
        Vertex {
            x: self.y * other.z - self.z * other.y,
            y: self.z * other.x - self.x * other.z,
            z: self.x * other.y - self.y * other.x,
        }
    }

    pub fn view_normal(&self, x: &Vertex, y: &Vertex, z: &Vertex) -> Self {
        Vertex {
            x: self.dot(x),
            y: self.dot(y),
            z: self.dot(z),
        }
    }

    pub fn view_triangle(&self, x: &Vertex, y: &Vertex, z: &Vertex, eye: &Vertex) -> Self {
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

pub struct Face {
    pub va: usize,
    pub vb: usize,
    pub vc: usize,
    pub ta: usize,
    pub tb: usize,
    pub tc: usize,
    pub na: usize,
    pub nb: usize,
    pub nc: usize,
}

#[derive(Debug, Clone)]
pub struct Triangle {
    pub a: Vertex,
    pub b: Vertex,
    pub c: Vertex,
}

impl Triangle {
    pub fn new(a: Vertex, b: Vertex, c: Vertex) -> Triangle {
        Triangle {
            a: a,
            b: b,
            c: c,
        }
    }

    pub fn unit(self) -> Triangle {
        Triangle {
            a: self.a.unit(),
            b: self.b.unit(),
            c: self.c.unit(),
        }
    }

    pub fn view_normal(self, x: &Vertex, y: &Vertex, z: &Vertex) -> Self {
        Triangle {
            a: self.a.view_normal(x, y, z),
            b: self.b.view_normal(x, y, z),
            c: self.c.view_normal(x, y, z),
        }
    }

    pub fn view_triangle(self, x: &Vertex, y: &Vertex, z: &Vertex, eye: &Vertex) -> Self {
        Triangle {
            a: self.a.view_triangle(x, y, z, eye),
            b: self.b.view_triangle(x, y, z, eye),
            c: self.c.view_triangle(x, y, z, eye),
        }
    }

    pub fn perspective(mut self) -> Self {
        let c = 3.0;
        let za = 1.0 - self.a.z / c;
        let zb = 1.0 - self.b.z / c;
        let zc = 1.0 - self.c.z / c;
        self.a = self.a * (1.0 / za);
        self.b = self.b * (1.0 / zb);
        self.c = self.c * (1.0 / zc);
        self
    }

    pub fn viewport(self, xres: u32, yres: u32) -> Self {
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

    pub fn barycenter(self, x: isize, y: isize) -> Vertex {
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
