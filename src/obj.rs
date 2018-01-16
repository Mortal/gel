use std::io::{self, BufRead, BufReader};
use std::result;
use std::fs::File;
use std::path::Path;
use geom::*;

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
                let (_, sx, sy, sz) = scan!(line, ' ', String, String, String, String).expect(&line);
                let (va, ta, na) = scan!(sx, '/', usize, usize, usize)?;
                let (vb, tb, nb) = scan!(sy, '/', usize, usize, usize)?;
                let (vc, tc, nc) = scan!(sz, '/', usize, usize, usize)?;
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
        Ok(Obj {
            vsv: vsv,
            vsn: vsn,
            vst: vst,
            fs: fs,
        })
    }

    pub fn tvgen(&self) -> Vec<Triangle> {
        let scale = self.vsv.iter().map(|v| v.len()).fold(0.0, f32::max);
        self.fs.iter().map(
            |f| Triangle::new(self.vsv[f.va].clone(),
                              self.vsv[f.vb].clone(),
                              self.vsv[f.vc].clone()) * (1f32 / scale)).collect()
    }

    pub fn tngen(&self) -> Vec<Triangle> {
        self.fs.iter().map(
            |f| Triangle::new(self.vsn[f.na].clone(),
                              self.vsn[f.nb].clone(),
                              self.vsn[f.nc].clone())).collect()
    }

    pub fn ttgen(&self) -> Vec<Triangle> {
        self.fs.iter().map(
            |f| Triangle::new(self.vst[f.ta].clone(),
                              self.vst[f.tb].clone(),
                              self.vst[f.tc].clone())).collect()
    }
}
