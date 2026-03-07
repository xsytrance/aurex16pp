use crate::aurex::ppu::framebuffer::{FB_H, FB_W};
use sdl2::render::{Canvas, Texture};
use sdl2::video::Window;

#[inline]
fn rgb555_to_argb8888(c: u16) -> u32 {
    let r5 = ((c >> 10) & 0x1F) as u32;
    let g5 = ((c >> 5) & 0x1F) as u32;
    let b5 = (c & 0x1F) as u32;

    let r8 = (r5 << 3) | (r5 >> 2);
    let g8 = (g5 << 3) | (g5 >> 2);
    let b8 = (b5 << 3) | (b5 >> 2);

    (0xFF << 24) | (r8 << 16) | (g8 << 8) | b8
}

pub fn present_frame(
    canvas: &mut Canvas<Window>,
    texture: &mut Texture,
    src: &[u16],
) -> Result<(), String> {
    texture.with_lock(None, |dst: &mut [u8], pitch: usize| {
        for y in 0..FB_H {
            let row = &src[y * FB_W..(y + 1) * FB_W];
            let out = &mut dst[y * pitch..y * pitch + FB_W * 4];
            for (x, &c) in row.iter().enumerate() {
                let argb = rgb555_to_argb8888(c);
                let o = x * 4;
                out[o] = (argb & 0xFF) as u8;
                out[o + 1] = ((argb >> 8) & 0xFF) as u8;
                out[o + 2] = ((argb >> 16) & 0xFF) as u8;
                out[o + 3] = ((argb >> 24) & 0xFF) as u8;
            }
        }
    })?;

    canvas.clear();
    canvas.copy(texture, None, None)?;
    canvas.present();
    Ok(())
}
