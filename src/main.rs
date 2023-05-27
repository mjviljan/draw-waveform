#![deny(clippy::all)]
#![forbid(unsafe_code)]

use error_iter::ErrorIter as _;
use log::error;
use pixels::{Error, Pixels, SurfaceTexture};
use rand::distributions::Uniform;
use rand::Rng;
use winit::{
    dpi::LogicalSize,
    window::WindowBuilder,
};
use winit::event::{Event, VirtualKeyCode};
use winit::event_loop::{ControlFlow, EventLoop};
use winit_input_helper::WinitInputHelper;

const WIDTH: u32 = 1024;
const HEIGHT: u32 = 256;

struct WaveformDisplay {
    values: [i16; WIDTH as usize],
}

fn main() -> Result<(), Error> {
    env_logger::init();
    let event_loop = EventLoop::new();
    let mut input = WinitInputHelper::new();
    let window = {
        let size = LogicalSize::new(WIDTH as f64, HEIGHT as f64);
        WindowBuilder::new()
            .with_title("Hello Pixels")
            .with_inner_size(size)
            .with_min_inner_size(size)
            .build(&event_loop)
            .unwrap()
    };

    let mut pixels = {
        let window_size = window.inner_size();
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
        Pixels::new(WIDTH, HEIGHT, surface_texture)?
    };

    let display = WaveformDisplay::new();
    event_loop.run(move |event, _, control_flow| {
        if let Event::RedrawRequested(_) = event {
            display.draw(pixels.frame_mut());
            if let Err(err) = pixels.render() {
                log_error("pixels.render", err);
                *control_flow = ControlFlow::Exit;
                return;
            }
        }

        if input.update(&event) {
            if input.key_pressed(VirtualKeyCode::Escape) || input.close_requested() {
                *control_flow = ControlFlow::Exit;
                return;
            }

            if let Some(size) = input.window_resized() {
                if let Err(err) = pixels.resize_surface(size.width, size.height) {
                    log_error("pixels.resize_surface", err);
                    *control_flow = ControlFlow::Exit;
                    return;
                }
            }

            window.request_redraw();
        }
    });
}

impl WaveformDisplay {
    fn new() -> Self {
        // let value_range: Uniform<i16> = Uniform::new_inclusive(-120, 120);
        let value_range: Uniform<i16> = Uniform::new_inclusive(-10, 10);

        // let randoms: Vec<i16> = (0..WIDTH as usize)
        //     .map(|_| {
        //         rng.gen_range(-120..121) as i16
        //     })
        //     .collect();
        Self {
            // values: randoms.try_into().unwrap()
            values: rand::thread_rng()
                .sample_iter(value_range)
                .take(WIDTH as usize)
                .collect::<Vec<i16>>()
                .try_into()
                .unwrap()
        }
    }

    fn draw(&self, frame: &mut [u8]) {
        println!("val: {}", self.values[0 as usize]);
        for (i, pixel) in frame.chunks_exact_mut(4).enumerate() {
            let x = (i % WIDTH as usize) as i16;
            let y = (i / WIDTH as usize) as i16 - 120;

            let rgba = if y == self.values[x as usize] { [0x0, 0x7f, 0x7f, 0xff] } else { [0x0, 0x0, 0x0, 0xff] };

            pixel.copy_from_slice(&rgba);
        }
    }
}

fn log_error<E: std::error::Error + 'static>(method_name: &str, err: E) {
    error!("{method_name}() failed: {err}");
    for source in err.sources().skip(1) {
        error!("  Caused by: {source}");
    }
}
