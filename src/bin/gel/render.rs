use gel::fps::SumWindow;
use gel::{asciidigits, Vertex, Viewport, ZBufferedTarget};
use mouse::mouse_camera;
use sdl2;
use sdl2::pixels::PixelFormatEnum;
use sdl2::rect::Rect;
use std::time::Instant;

///Set up SDL2 with mouse controls and an FPS counter to repeatedly render frames.
pub fn render_loop<R, F>(title: &str, xres: usize, yres: usize, mut render_frame: F) -> Option<R>
where
    F: FnMut(&mut Viewport) -> Option<R>,
{
    let sdl = sdl2::init().unwrap();
    let video = sdl.video().unwrap();
    let window = video
        .window(title, xres as u32, yres as u32)
        .build()
        .unwrap();
    let mut canvas = window.into_canvas().present_vsync().build().unwrap();
    let renderer = canvas.texture_creator();
    // Notice the flip between xres and yres - the renderer is on its side to maximize cache effeciency.
    let mut texture = renderer
        .create_texture_streaming(PixelFormatEnum::ARGB8888, yres as u32, xres as u32)
        .unwrap();

    let mut zbuff = Vec::new();
    zbuff.resize((xres * yres) as usize, 0f32);
    sdl.mouse().set_relative_mouse_mode(false);
    let mut fps_counter = SumWindow::new(30);
    let mut result = None;
    for (eye, eye_dir) in mouse_camera(&mut sdl.event_pump().unwrap()) {
        texture
            .with_lock(None, |pixel, _pitch| {
                let t = Instant::now();
                let mut target = ZBufferedTarget::new_column_major(xres, yres, pixel, &mut zbuff);
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
                        let o = (x + i * 6) * yres as usize + (yres as usize - 1 - y);
                        viewport.target.pixel[4 * o] = 255;
                        viewport.target.pixel[4 * o + 1] = 255;
                        viewport.target.pixel[4 * o + 2] = 255;
                    });
                }
            })
            .unwrap();
        match result {
            Some(r) => return Some(r),
            None => (),
        };
        let dst = Rect::new(
            (xres as i32 - yres as i32) / 2,
            (yres as i32 - xres as i32) / 2,
            yres as u32,
            xres as u32,
        );
        canvas
            .copy_ex(&texture, None, dst, -90f64, None, false, false)
            .unwrap();
        canvas.present();
    }
    None
}
