use sdl2::EventPump;
use sdl2::event::{Event, WindowEvent};
use super::geom::Vertex;

pub struct RawMouseIterator<'a>(&'a mut EventPump);

impl<'a> Iterator for RawMouseIterator<'a> {
    type Item = (f32, f32);

    fn next(&mut self) -> Option<(f32, f32)> {
        loop {
            let event = self.0.wait_event();
            match event {
                Event::Quit { timestamp: _ } => return None,
                Event::Window { timestamp: _, window_id: _, win_event: WindowEvent::Exposed } =>
                    return Some((0.0, 0.0)),
                Event::Window { timestamp: _, window_id: _, win_event: WindowEvent::Close } =>
                    return None,
                Event::MouseMotion {
                    timestamp: _, window_id: _, which: _, mousestate: _, x: _, y: _, xrel, yrel,
                } => return Some((xrel as f32, yrel as f32)),
                _ => (),
            };
        }
    }
}

pub fn mouse_iter(pump: &mut EventPump) -> RawMouseIterator {
    RawMouseIterator(pump)
}

pub struct MouseCameraIterator<'a> {
    inner: RawMouseIterator<'a>,
    x: f32,
    y: f32,
    sens: f32,
}

impl<'a> Iterator for MouseCameraIterator<'a> {
    type Item = (Vertex, Vertex);

    fn next(&mut self) -> Option<(Vertex, Vertex)> {
        self.inner.next().map(|(xrel, yrel)| {
            self.x -= self.sens * xrel;
            self.y += self.sens * yrel;
            // Mouse rotates:
            let eye = Vertex { x: self.x.sin(), y: self.y.sin(), z: self.x.cos() };
            let eye_dir = eye.clone();
            // Mouse translates:
            //let eye = Vertex { x: self.x, y: self.y, z: 1.0 };
            //let eye_dir = Vertex { x: 0.0, y: 0.0, z: 1.0 };
            (eye, eye_dir)
        })
    }
}

pub fn mouse_camera(pump: &mut EventPump) -> MouseCameraIterator {
    MouseCameraIterator {
        inner: mouse_iter(pump),
        x: 0.0,
        y: 0.0,
        sens: 0.005,
    }
}
