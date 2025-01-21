#![feature(core_intrinsics)]

#![no_main]
#![no_std]

extern crate alloc;

mod ball;

use alloc::vec;
use alloc::vec::Vec;
use core::mem;
use log::info;
use uefi::prelude::*;
use uefi::proto::console::gop::{BltOp, BltPixel, BltRegion, GraphicsOutput};
use uefi::proto::rng::Rng;
use uefi::{boot, Result};
use uefi::proto::network::{self, IpAddress};
use uefi::proto::network::snp::SimpleNetwork;
use uefi::boot::ScopedProtocol;
#[derive(Clone, Copy)]
struct Point {
    x: f32,
    y: f32,
}

impl Point {
    fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

struct Buffer {
    width: usize,
    height: usize,
    pixels: Vec<BltPixel>,
}

impl Buffer {
    /// Create a new `Buffer`.
    fn new(width: usize, height: usize) -> Self {
        Buffer {
            width,
            height,
            pixels: vec![BltPixel::new(0, 0, 0); width * height],
        }
    }

    /// Get a single pixel.
    fn pixel(&mut self, x: usize, y: usize) -> Option<&mut BltPixel> {
        self.pixels.get_mut(y * self.width + x)
    }

    /// Blit the buffer to the framebuffer.
    fn blit(&self, gop: &mut GraphicsOutput) -> Result {
        gop.blt(BltOp::BufferToVideo {
            buffer: &self.pixels,
            src: BltRegion::Full,
            dest: (0, 0),
            dims: (self.width, self.height),
        })
    }

    /// Update only a pixel to the framebuffer.
    fn blit_pixel(&self, gop: &mut GraphicsOutput, coords: (usize, usize)) -> Result {
        gop.blt(BltOp::BufferToVideo {
            buffer: &self.pixels,
            src: BltRegion::SubRectangle {
                coords,
                px_stride: self.width,
            },
            dest: coords,
            dims: (1, 1),
        })
    }
}

/// Get a random `usize` value.
fn get_random_usize(rng: &mut Rng) -> usize {
    let mut buf = [0; mem::size_of::<usize>()];
    rng.get_rng(None, &mut buf).expect("get_rng failed");
    usize::from_le_bytes(buf)
}

fn draw_green_poker_background() -> Result {
    // Open graphics output protocol.
    let gop_handle = boot::get_handle_for_protocol::<GraphicsOutput>()?;
    let mut gop = boot::open_protocol_exclusive::<GraphicsOutput>(gop_handle)?;

    // Open random number generator protocol.
    let rng_handle = boot::get_handle_for_protocol::<Rng>()?;
    let mut rng = boot::open_protocol_exclusive::<Rng>(rng_handle)?;

    // Create a buffer to draw into.
    let (width, height) = gop.current_mode_info().resolution();
    let mut buffer = Buffer::new(width, height);

    let (width, height) = (600, 600);

    // Initialize the buffer with a simple gradient background.
    for y in 0..width {
        for x in 0..height {
            let fx = x as f32 / (width as f32);
            let fy = y as f32 / (height as f32);
            let fy = 1.0f32 - fy;
            let col = ball::mainImage(fx * 2f32 - 1f32, fy * 2f32 - 1f32);

            let pixel = buffer.pixel(x, y).unwrap();
            pixel.red = (col.x * 255f32) as u8;
            pixel.green = (col.y * 255f32) as u8;
            pixel.blue = (col.z * 255f32) as u8;
        }
    }

    loop {
        // Draw background.
        buffer.blit(&mut gop)?;
    }

    Ok(())
}

fn network_hello(network_boy: &ScopedProtocol<SimpleNetwork>) {
    let buf = vec!(1 as u8, 2, 3);
    // network_boy.transmit(0, buf, );
}

#[entry]
fn main() -> Status {
    uefi::helpers::init().unwrap();
    //let network_boy_handle = boot::get_handle_for_protocol::<SimpleNetwork>().unwrap();
    //let network_boy = boot::open_protocol_exclusive::<SimpleNetwork>(network_boy_handle).unwrap();
    //network_boy.initialize(0,0).expect("INITIALIZE FUCKED");
    //network_boy.start().expect("START FUCKED!");
    //network_hello(&network_boy);
    //info!("initialized network");
    loop {
        let res = draw_green_poker_background();
        info!("got results: {res:?}");
        res.unwrap();
    }
}
