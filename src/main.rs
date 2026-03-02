mod aurex;

use aurex::ppu::framebuffer::{FB_H, FB_W};
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use std::time::{Duration, Instant};

fn rgb555_to_argb8888(c: u16) -> u32 {
    // c is 0b0_RRRRR_GGGGG_BBBBB (15-bit)
    let r5 = ((c >> 10) & 0x1F) as u32;
    let g5 = ((c >> 5) & 0x1F) as u32;
    let b5 = (c & 0x1F) as u32;

    // Expand 5-bit to 8-bit (bit replication)
    let r8 = (r5 << 3) | (r5 >> 2);
    let g8 = (g5 << 3) | (g5 >> 2);
    let b8 = (b5 << 3) | (b5 >> 2);

    // ARGB8888
    (0xFF << 24) | (r8 << 16) | (g8 << 8) | b8
}

fn main() {
    // SDL host (monitor cable)
    let sdl = sdl2::init().expect("SDL init failed");
    let video = sdl.video().expect("SDL video init failed");

    let scale: u32 = 3; // change to 2/3/4 as you like
    let win_w = (FB_W as u32) * scale;
    let win_h = (FB_H as u32) * scale;

    let window = video
        .window("Aurex-16++", win_w, win_h)
        .position_centered()
        .build()
        .expect("window build failed");

    let mut canvas = window
        .into_canvas()
        .present_vsync() // host pacing only; core determinism remains internal
        .build()
        .expect("canvas build failed");

    let texture_creator = canvas.texture_creator();
    let mut texture = texture_creator
        .create_texture_streaming(
            sdl2::pixels::PixelFormatEnum::ARGB8888,
            FB_W as u32,
            FB_H as u32,
        )
        .expect("texture create failed");

    let mut pump = sdl.event_pump().expect("event pump failed");

    let mut system = aurex::Aurex::new();

    // Simple host pacing (60Hz-ish). Not part of core determinism.
    let target = Duration::from_nanos(16_666_667);
    let mut last = Instant::now();

    'running: loop {
        for e in pump.poll_iter() {
            match e {
                Event::Quit { .. } => break 'running,
                Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                _ => {}
            }
        }

        // Run exactly one console frame
        system.run_frame();

        // Upload framebuffer to texture
        let src = system.framebuffer().pixels();

        texture
            .with_lock(None, |dst: &mut [u8], pitch: usize| {
                for y in 0..FB_H {
                    let row = &src[y * FB_W..(y + 1) * FB_W];
                    let out = &mut dst[y * pitch..y * pitch + FB_W * 4];

                    for (x, &c) in row.iter().enumerate() {
                        let argb = rgb555_to_argb8888(c);
                        let o = x * 4;
                        // SDL ARGB8888 byte order in memory is platform-dependent,
                        // but SDL expects the buffer in the texture's native format.
                        out[o + 0] = (argb & 0xFF) as u8; // B
                        out[o + 1] = ((argb >> 8) & 0xFF) as u8; // G
                        out[o + 2] = ((argb >> 16) & 0xFF) as u8; // R
                        out[o + 3] = ((argb >> 24) & 0xFF) as u8; // A
                    }
                }
            })
            .expect("texture lock failed");

        canvas.clear();
        canvas
            .copy(&texture, None, None)
            .expect("canvas copy failed");
        canvas.present();

        // Host pacing (optional)
        let elapsed = last.elapsed();
        if elapsed < target {
            std::thread::sleep(target - elapsed);
        }
        last = Instant::now();
    }
}
