#![deny(clippy::all)]
#![forbid(unsafe_code)]

use std::fs::File;
use std::io::prelude::*;

use error_iter::ErrorIter as _;
use log::error;
use pixels::{Error, Pixels, SurfaceTexture};
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
    values: [u8; WIDTH as usize],
}

fn main() -> Result<(), Error> {
    env_logger::init();

    let mut f = File::open("test_input.dat").unwrap();
    let mut buffer: [u8; WIDTH as usize] = [127; WIDTH as usize];
    f.read(&mut buffer).expect("Cannot read source file");

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

    let display = WaveformDisplay::new(buffer);
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
    fn new(buffer: [u8; WIDTH as usize]) -> Self {
        Self {
            values: buffer
        }
    }

    fn draw(&self, frame: &mut [u8]) {
        // println!("val: {}", self.values[0 as usize]);
        for (i, pixel) in frame.chunks_exact_mut(4).enumerate() {
            let x = i % WIDTH as usize;
            let y = 255u8 - (i / WIDTH as usize) as u8;

            let rgba = if y == self.values[x] { [0x0, 0x7f, 0x7f, 0xff] } else { [0x0, 0x0, 0x0, 0xff] };

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
