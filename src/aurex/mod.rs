pub mod clock;
pub mod dma;
pub mod pdu;
pub mod ppu;
pub mod vm32;
pub mod wram;

use crate::aurex::ppu::ppu::Ppu;
use clock::Clock;
use dma::controller::DmaController;
use pdu::Pdu;
use ppu::vram::Vram;
use vm32::core::Vm32;
use wram::Wram;

pub struct Aurex {
    clock: Clock,
    pdu: Pdu,
    wram: Wram,
    vm: Vm32,
    dma: DmaController,
    vram: Vram,
    fb: ppu::framebuffer::Framebuffer,
    ppu: Ppu,
}

impl Aurex {
    pub fn new() -> Self {
        let mut s = Self {
            clock: Clock::new(),
            pdu: Pdu::new(),
            wram: Wram::new(),
            vm: Vm32::new(),
            dma: DmaController::new(),
            vram: Vram::new(),
            fb: ppu::framebuffer::Framebuffer::new(),
            ppu: Ppu::new(),
        };

        // =====================================================================
        // TEMP TEST: Seed VRAM with a simple BG0 tilemap + palette
        // Removal: delete this block once cartridge/SDK uploads real assets via DMA
        // =====================================================================
        #[cfg(debug_assertions)]
        {
            seed_test_bg0(&mut s.vram);
        }

        #[cfg(debug_assertions)]
        {
            seed_test_bg0(&mut s.vram);
        }

        s
    }

    pub fn run(&mut self) -> ! {
        loop {
            self.run_frame();
        }
    }

    pub fn run_frame(&mut self) {
        self.clock.begin_frame();
        self.pdu.begin_frame();

        use crate::aurex::ppu::framebuffer::rgb555;

        // Clear to black each frame (v0.1)
        self.fb.clear(rgb555(0, 0, 0));

        // ---------------------------------------------------------------------
        // PPU FRAME RENDER
        // ---------------------------------------------------------------------
        self.ppu.render_frame(&self.vram, &mut self.fb);

        // =====================================================================
        // PPU → PDU TELEMETRY BRIDGE
        // ---------------------------------------------------------------------
        // The PPU latches hardware events during rendering (e.g. sprite overflow).
        // The PDU collects per-frame telemetry for debugging / future SDK hooks.
        // This keeps rendering logic isolated from diagnostics logic.
        // =====================================================================
        self.pdu.ingest_ppu(
            self.ppu.sprite_overflow_latched(),
            self.ppu.sprite_overflow_scanlines(),
        );

        // ---------------------------------------------------------------------
        // DMA + CPU
        // ---------------------------------------------------------------------
        self.dma.begin_frame();

        // CPU execution for this frame
        self.vm.run_frame(&mut self.pdu);

        // =====================================================================
        // TEMP TEST: Moving test sprite
        // Removal: replace with cartridge logic later
        // =====================================================================
        #[cfg(debug_assertions)]
        {
            let frame = self.pdu.frame_index() as u16;

            let x = (frame % 400) as u16;
            let y = 120;

            self.ppu.write_sprite(
                0, // sprite index
                x, y, 0,     // base tile index (top-left of 2x2 block)
                0,     // palette
                0,     // priority
                true,  // 16x16 enabled
                true,  // hflip
                false, // vflip
            );
        }

        // =====================================================================
        // TEMP TEST: CPU writes to PPU scroll register
        // Removal: replace with proper memory-mapped register writes
        // =====================================================================
        #[cfg(debug_assertions)]
        {
            let scroll = (self.pdu.frame_index() as u16).wrapping_mul(1);
            self.ppu.set_bg0_scroll(scroll, 0);
        }

        // Apply accepted DMA transfers to hardware memory
        self.dma.apply(&self.wram, &mut self.vram);

        // Aggregate telemetry into PDU
        self.pdu.ingest_dma(
            self.dma.commands_used(),
            self.dma.vram_bytes_used(),
            0, // audio not implemented yet
            self.dma.rejects_this_frame(),
        );

        self.pdu.end_frame();
        self.clock.end_frame();
    }
    pub fn framebuffer(&self) -> &crate::aurex::ppu::framebuffer::Framebuffer {
        &self.fb
    }
}

// =====================================================================
// TEMP TEST: VRAM seed helpers
// Removal: delete once cartridges upload real assets via DMA
// =====================================================================
#[cfg(debug_assertions)]
fn seed_test_bg0(vram: &mut crate::aurex::ppu::vram::Vram) {
    // Palette: 256 RGB555 entries (LE). We'll fill a small useful subset.
    // Entry 0: black
    // Entry 1: white
    // Entry 2: red
    // Entry 3: green
    // Entry 4: blue
    // Entry 5: yellow
    // Entry 6: magenta
    // Entry 7: cyan
    let colors: [u16; 8] = [
        0b0_00000_00000_00000,
        0b0_11111_11111_11111,
        0b0_11111_00000_00000,
        0b0_00000_11111_00000,
        0b0_00000_00000_11111,
        0b0_11111_11111_00000,
        0b0_11111_00000_11111,
        0b0_00000_11111_11111,
    ];

    for (i, c) in colors.iter().enumerate() {
        let ofs = i * 2;
        vram.palettes[ofs] = (*c & 0xFF) as u8;
        vram.palettes[ofs + 1] = (*c >> 8) as u8;
    }

    // Tile 0: checker using palette indices 0 and 1
    // 4bpp packed: two pixels per byte, hi nibble then lo nibble
    // We'll alternate 0 and 1 across rows.
    for row in 0..8 {
        let base = row * 4;
        // pattern: 0,1,0,1,0,1,0,1
        // bytes: [0x01, 0x01, 0x01, 0x01] if (0,1) repeating per byte hi/lo
        let b = if row % 2 == 0 { 0x01 } else { 0x10 }; // flip nibble order per row for visible variation
        vram.bg_tiles[base + 0] = b;
        vram.bg_tiles[base + 1] = b;
        vram.bg_tiles[base + 2] = b;
        vram.bg_tiles[base + 3] = b;
    }

    // Tile 1: solid red (index 2)
    for row in 0..8 {
        let base = 32 + row * 4; // tile 1 starts at byte 32
        // pixel value 2 => nibble 0x2
        vram.bg_tiles[base + 0] = 0x22;
        vram.bg_tiles[base + 1] = 0x22;
        vram.bg_tiles[base + 2] = 0x22;
        vram.bg_tiles[base + 3] = 0x22;
    }

    // Tilemap: 64x64 entries. We'll fill the visible area with alternating tiles.
    // Entry bits: tile_index (0..9), pal_sel (10..11), flips, priority.
    // Use pal_sel = 0 for now.
    let tiles_x = (crate::aurex::ppu::framebuffer::FB_W + 7) / 8;
    let tiles_y = (crate::aurex::ppu::framebuffer::FB_H + 7) / 8;

    for y in 0..tiles_y {
        for x in 0..tiles_x {
            let tile = if (x + y) % 2 == 0 { 0 } else { 1 };
            let entry: u16 = tile; // pal=0, no flips, pri=0
            let idx = (y * 64 + x) * 2;
            vram.bg0_tilemap[idx] = (entry & 0xFF) as u8;
            vram.bg0_tilemap[idx + 1] = (entry >> 8) as u8;
        }
    }

    // -----------------------------------------------------------------------------
    // TEMP TEST — Build 16x16 sprite (tiles 0–3)
    // Layout:
    // [0][1]
    // [2][3]
    // -----------------------------------------------------------------------------

    // Tile 0 (top-left) — color index 1
    for row in 0..8 {
        let base = row * 4;
        for i in 0..4 {
            vram.sprite_tiles[base + i] = 0x11;
        }
    }

    // Tile 1 (top-right) — color index 2
    for row in 0..8 {
        let base = 32 + row * 4;
        for i in 0..4 {
            vram.sprite_tiles[base + i] = 0x22;
        }
    }

    // Tile 2 (bottom-left) — color index 3
    for row in 0..8 {
        let base = 64 + row * 4;
        for i in 0..4 {
            vram.sprite_tiles[base + i] = 0x33;
        }
    }

    // Tile 3 (bottom-right) — color index 4
    for row in 0..8 {
        let base = 96 + row * 4;
        for i in 0..4 {
            vram.sprite_tiles[base + i] = 0x44;
        }
    }

    // ---------------------------------------------------------------------
    // Seed sprite palette entry 1 (bright red)
    // ---------------------------------------------------------------------
    if vram.palettes.len() >= 4 {
        let green = crate::aurex::ppu::framebuffer::rgb555(0, 31, 0);
        vram.palettes[2] = (green & 0xFF) as u8;
        vram.palettes[3] = (green >> 8) as u8;
    }
}
