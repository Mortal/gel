use geom::*;
use std::fmt;
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::Path;
use std::result;
use super::{Viewport, TextureShader, Pixels, draw_shaded_triangle};

#[derive(Debug)]
pub enum Error {
    Io(io::Error),
    Scan(String),
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Error {
        Error::Io(e)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::Io(ref e) => write!(f, "I/O error: {}", e),
            Error::Scan(ref s) => write!(f, "Parse error: {}", s),
        }
    }
}

pub type Result<T> = result::Result<T, Error>;

pub struct Obj {
    pub vsv: Vec<Vertex>,
    pub vsn: Vec<Vertex>,
    pub vst: Vec<Vertex>,
    pub fs: Vec<Face>,
}

// scan! from https://stackoverflow.com/a/31048103/1570972
macro_rules! scan {
    ( $string:expr, $sep:expr, $( $x:ty ),+ ) => {(|| {
        let mut iter = $string.split($sep);
        let r = ($(match iter.next() {
            Some(v) => match v.parse::<$x>() {
                Ok(v) => v,
                Err(e) => return Err(Error::Scan(format!("parse error {:?}", e))),
            },
            None => return Err(Error::Scan("unexpected end-of-string".to_owned())),
        },)*);
        match iter.next() {
            Some(s) => return Err(Error::Scan(format!("unexpected token {}", s))),
            None => (),
        };
        Ok(r)
    })()}
}

impl Obj {
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        let mut vsv = Vec::new();
        let mut vsn = Vec::new();
        let mut vst = Vec::new();
        let mut fs = Vec::new();
        for line in BufReader::new(File::open(path)?).lines() {
            let line = line?;
            if line.len() == 0 || line.starts_with("#") {
                continue;
            }
            if line.starts_with("f ") {
                let (_, sx, sy, sz) =
                    scan!(line, ' ', String, String, String, String).expect(&line);
                let (va, ta, na) = scan!(sx, '/', usize, usize, usize)?;
                let (vb, tb, nb) = scan!(sy, '/', usize, usize, usize)?;
                let (vc, tc, nc) = scan!(sz, '/', usize, usize, usize)?;
                fs.push(Face {
                    va: va - 1,
                    vb: vb - 1,
                    vc: vc - 1,
                    ta: ta - 1,
                    tb: tb - 1,
                    tc: tc - 1,
                    na: na - 1,
                    nb: nb - 1,
                    nc: nc - 1,
                });
            } else if line.starts_with("vn ") {
                let (_, x, y, z) = scan!(line, ' ', String, f32, f32, f32).expect(&line);
                let v = Vertex { x: x, y: y, z: z };
                vsn.push(v);
            } else if line.starts_with("vt ") {
                let (_, x, y) = scan!(line, ' ', String, f32, f32).expect(&line);
                let v = Vertex {
                    x: x,
                    y: y,
                    z: 0f32,
                };
                vst.push(v);
            } else if line.starts_with("v ") {
                let (_, x, y, z) = scan!(line, ' ', String, f32, f32, f32).expect(&line);
                let v = Vertex { x: x, y: y, z: z };
                vsv.push(v);
            } else if line.starts_with("mtllib ")
                || line.starts_with("g ")
                || line.starts_with("usemtl ")
                || line.starts_with("s ")
                || line.starts_with("o ")
            {
                continue;
            } else {
                println!("Unexpected line {:?}", line);
                continue;
            }
        }
        Ok(Obj {
            vsv: vsv,
            vsn: vsn,
            vst: vst,
            fs: fs,
        })
    }

    pub fn tvgen(&self) -> Vec<Triangle> {
        let scale = self.vsv.iter().map(|v| v.len()).fold(0.0, f32::max);
        self.fs
            .iter()
            .map(|f| {
                Triangle::new(
                    self.vsv[f.va].clone(),
                    self.vsv[f.vb].clone(),
                    self.vsv[f.vc].clone(),
                ) * (1f32 / scale)
            })
            .collect()
    }

    pub fn tngen(&self) -> Vec<Triangle> {
        self.fs
            .iter()
            .map(|f| {
                Triangle::new(
                    self.vsn[f.na].clone(),
                    self.vsn[f.nb].clone(),
                    self.vsn[f.nc].clone(),
                )
            })
            .collect()
    }

    pub fn ttgen(&self) -> Vec<Triangle> {
        self.fs
            .iter()
            .map(|f| {
                Triangle::new(
                    self.vst[f.ta].clone(),
                    self.vst[f.tb].clone(),
                    self.vst[f.tc].clone(),
                )
            })
            .collect()
    }

    pub fn draw_shaded<P: Pixels>(&self, viewport: &mut Viewport, shader: &TextureShader<P>) {
        let scale = self.vsv.iter().map(|v| v.len()).fold(0.0, f32::max);
        let vertices = self.fs
            .iter()
            .map(|f| {
                Triangle::new(
                    self.vsv[f.va].clone(),
                    self.vsv[f.vb].clone(),
                    self.vsv[f.vc].clone(),
                ) * (1f32 / scale)
            });
        let normals = self.fs
            .iter()
            .map(|f| {
                Triangle::new(
                    self.vsn[f.na].clone(),
                    self.vsn[f.nb].clone(),
                    self.vsn[f.nc].clone(),
                )
            });
        let textures = self.fs
            .iter()
            .map(|f| {
                Triangle::new(
                    self.vst[f.ta].clone(),
                    self.vst[f.tb].clone(),
                    self.vst[f.tc].clone(),
                )
            });
        for ((nrm, tex), tri) in normals.zip(textures).zip(vertices) {
            let nrm = nrm.clone()
                .view_normal(&viewport.x(), &viewport.y(), &viewport.z())
                .unit();
            let tri = tri.clone().view_triangle(
                &viewport.x(),
                &viewport.y(),
                &viewport.z(),
                &viewport.eye(),
            );
            let per = tri.perspective();
            let vew = per.viewport(viewport.xres() as u32, viewport.yres() as u32);
            draw_shaded_triangle(viewport, &vew, &nrm, &tex, &shader);
        }
    }
}
