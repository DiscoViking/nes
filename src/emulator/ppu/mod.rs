mod registers;

use emulator::components::latch;
use emulator::memory;
use emulator::memory::Reader;

// Colours represented as a single byte:
// 76543210
// ||||||||
// ||||++++- Hue (phase, determines NTSC/PAL chroma)
// ||++----- Value (voltage, determines NTSC/PAL luma)
// ++------- Unimplemented, reads back as 0
pub struct Colour {
    byte: u8,
}

impl Colour {
    pub fn hue(&self) -> u8 {
        self.byte & 0b1111
    }

    pub fn brightness(&self) -> u8 {
        (self.byte >> 4) & 0b11
    }
}

pub struct Palette {
    c1: Colour,
    c2: Colour,
    c3: Colour,
}

pub trait VideoOut {
    fn emit(&mut self, c: Colour);
}

pub struct PPU {
    // Device to output rendered pixels to.
    output: Box<VideoOut>,

    // --- Registers.

    // PPUCTRL
    // 7  bit  0
    // ---- ----
    // VPHB SINN
    // |||| ||||
    // |||| ||++- Base nametable address
    // |||| ||    (0 = $2000; 1 = $2400; 2 = $2800; 3 = $2C00)
    // |||| |+--- VRAM address increment per CPU read/write of PPUDATA
    // |||| |     (0: add 1, going across; 1: add 32, going down)
    // |||| +---- Sprite pattern table address for 8x8 sprites
    // ||||       (0: $0000; 1: $1000; ignored in 8x16 mode)
    // |||+------ Background pattern table address (0: $0000; 1: $1000)
    // ||+------- Sprite size (0: 8x8 pixels; 1: 8x16 pixels)
    // |+-------- PPU master/slave select
    // |          (0: read backdrop from EXT pins; 1: output color on EXT pins)
    // +--------- Generate an NMI at the start of the
    //            vertical blanking interval (0: off; 1: on)
    ppuctrl: u8,

    // PPUMASK
    // 7  bit  0
    // ---- ----
    // BGRs bMmG
    // |||| ||||
    // |||| |||+- Greyscale (0: normal color, 1: produce a greyscale display)
    // |||| ||+-- 1: Show background in leftmost 8 pixels of screen, 0: Hide
    // |||| |+--- 1: Show sprites in leftmost 8 pixels of screen, 0: Hide
    // |||| +---- 1: Show background
    // |||+------ 1: Show sprites
    // ||+------- Emphasize red*
    // |+-------- Emphasize green*
    // +--------- Emphasize blue
    ppumask: u8,

    // PPUSTATUS
    // 7  bit  0
    // ---- ----
    // VSO. ....
    // |||| ||||
    // |||+-++++- Least significant bits previously written into a PPU register
    // |||        (due to register not being updated for this address)
    // ||+------- Sprite overflow. The intent was for this flag to be set
    // ||         whenever more than eight sprites appear on a scanline, but a
    // ||         hardware bug causes the actual behavior to be more complicated
    // ||         and generate false positives as well as false negatives; see
    // ||         PPU sprite evaluation. This flag is set during sprite
    // ||         evaluation and cleared at dot 1 (the second dot) of the
    // ||         pre-render line.
    // |+-------- Sprite 0 Hit.  Set when a nonzero pixel of sprite 0 overlaps
    // |          a nonzero background pixel; cleared at dot 1 of the pre-render
    // |          line.  Used for raster timing.
    // +--------- Vertical blank has started (0: not in vblank; 1: in vblank).
    //            Set at dot 1 of line 241 (the line *after* the post-render
    //            line); cleared after reading $2002 and at dot 1 of the
    //            pre-render line.
    ppustatus: u8,
    oamaddr: u8,
    ppuscroll_latch: latch::Latch,
    ppuaddr_latch: latch::Latch,

    // PPU memory is laid out like so:
    // $0000-$0FFF = pattern table 0
    // $1000-$1FFF = pattern table 1
    // $2000-$23FF = name table 0
    // $2400-$27FF = name table 0
    // $2800-$2BFF = name table 0
    // $2C00-$2FFF = name table 0
    // $3000-$3EFF = mirrors of $2000-$2EFF
    // $3F00-$3F1F = palette RAM indexes
    // $3F20-$3FFF = mirrors of $3F00-$3F1F
    memory: memory::Manager,

    // -- Background State --

    // VRAM address.
    v: u16,

    // Temporary VRAM address.
    t: u16,

    // Fine X Scroll.
    fine_x: u8,

    // Two 16-bit shift registers containing bitmap data for 2 tiles.
    // Every 8 cycles the data for the next tile is loaded into the upper 8 bits of the register,
    // meanwhile the pixel to render is fetched from the lower 8 bits.
    tile_register_low: u16,
    tile_register_high: u16,
    tile_latch_low: u8,
    tile_latch_high: u8,

    // Two 8-bit shift registers containing the palette attributes for the lower 8 pixels of the
    // 16-bit register.
    // These registers are fed by a latch which contains the palette attribute for the next tile.
    // Every 8 cycles the latch is loaded with the attribute for the next tile.
    attribute_register_1: u8,
    attribute_register_2: u8,
    attribute_latch_1: u8,
    attribute_latch_2: u8,

    // -- Sprite State --

    // In addition to its main memory, the PPU has 256 bytes of memory known as OAM which determines how sprites are
    // rendered.
    // $00-$0C = Sprite Y coordinate
    // $01-$0D = Sprite tile #
    // $02-$0E = Sprite attribute
    // $03-$0F = Sprite X coordinate
    // TODO: What does this actually mean?
    oam: memory::Manager,

    // Secondary OAM holds 8 sprites to be rendered on the current scanline.
    secondary_oam: memory::Manager,

    // Eight pairs of 8-bit shift registers to hold the bitmap data for 8 sprites to be rendered on
    // the current scanline.

    // Eight latches containing the attribute bytes for the 8 sprites.

    // Eight counters containing the X positions for the 8 sprites.

    // --- Counters for tracking the current rendering stage.

    // There are 262 scanlines in total. 0-239 are visible, 240-260 occur durng vblank, and 261 is
    // idle.
    scanline: u16,

    // Each scanline takes 341 cycles to render.
    cycle: u16,

    // -- Internal State --

    // Byte fetched from nametable indicating which tile to fetch from pattern table.
    tmp_pattern_coords: u8,

    // Which half of pattern tables to use.  0 = 'left', 1 = 'right'.
    pattern_table_side: u8,
}

pub fn new(output: Box<VideoOut>) -> PPU {
    PPU {
        output: output,
        ppuctrl: 0,
        ppumask: 0,
        ppustatus: 0,
        oamaddr: 0,
        ppuscroll_latch: latch::new(),
        ppuaddr_latch: latch::new(),
        memory: memory::new(),
        v: 0,
        t: 0,
        fine_x: 0,
        tile_register_low: 0,
        tile_register_high: 0,
        tile_latch_low: 0,
        tile_latch_high: 0,
        attribute_register_1: 0,
        attribute_register_2: 0,
        attribute_latch_1: 0,
        attribute_latch_2: 0,
        oam: memory::new(),
        secondary_oam: memory::new(),
        scanline: 261,
        cycle:  0,
        tmp_pattern_coords: 0,
        pattern_table_side: 0,
    }
}

impl PPU {
    // Returns how many PPU cycles the tick took.
    pub fn tick(&mut self) -> u16 {
        let cycles = match self.scanline {
            0 ... 239 | 261 => self.tick_render_scanline(),
            240 => self.tick_idle_scanline(),
            241 => self.tick_vblank_scanline(),
            _ => panic!("Scanline index should never exceed 261.  Got {}.", self.scanline),
        };

        self.cycle = self.cycle + cycles;

        if self.cycle > 341 {
            panic!("Cycle index should never exceed 341.  Got: {}.", self.cycle);
        }

        if self.cycle == 341 {
            self.cycle = 0;
            self.scanline = (self.scanline + 1) % 262;
        }

        cycles
    }

    fn tick_render_scanline(&mut self) -> u16 {
        // Rendering stages.
        let cycles = match self.cycle {
            // Cycle 0 is an idle cycle.
            0 => self.tick_idle_cycle(),

            // The data for each tile is fetched durnig this phase.
            // This where the actual pixels for the scanline are output.
            1 ... 256 => self.tick_render_cycle(),

            // The tile data for the sprites on the next scanline are fetched during this phase.
            257 ... 320 => self.tick_sprite_fetch_cycle(),

            // This is where the first two tiles of the next scanline are fetched and loaded into
            // the shift registers.
            321 ... 336 => self.tick_prefetch_tiles_cycle(),

            // Finally, here two bytes are fetched, but the purpose is unknown.
            337 ... 340 => self.tick_unknown_fetch(),

            _ => panic!("PPU cycle index should never exceed 341.  Got {}.", self.cycle),
        };

        // Scrolling.
        self.handle_scrolling();

        // On dot 1 of the pre-render scanline, clear vblank flag.
        if self.scanline == 261 && self.cycle == 1 {
            self.ppustatus &= 0b0111_1111;
        }

        cycles
    }

    fn tick_idle_scanline(&mut self) -> u16 {
        // PPU does nothing on the idle scanline.
        // Just idle for 341 cycles.
        341
    }

    fn tick_vblank_scanline(&mut self) -> u16 {
        if self.scanline == 241 && self.cycle == 1 {
            // Set VBlank flag.
            self.ppustatus |= 0b1000_0000;
        }
        // Otherwise idle.
        1
    }

    fn tick_idle_cycle(&mut self) -> u16 {
        // PPU does nothing during idle cycle.
        1
    }

    fn tick_render_cycle(&mut self) -> u16 {
        // If cycle 1, 9, 17, ..., 257 then reload the shift registers from the latches.
        if self.cycle % 8 == 1 {
            self.reload_shift_registers();
        }

        self.fetch_tile_data();

        // Actually render and emit one pixel.
        // Unless this is scanline 261, which is just a dummy scanline.
        if self.scanline != 261 {
            let pixel = self.render_pixel();
            self.output.emit(pixel);
        }

        // Finally shift all the registers.
        self.shift_registers();
        1
    }

    fn tick_sprite_fetch_cycle(&mut self) -> u16 {
        // TODO: Implement sprites.
        1
    }

    fn tick_prefetch_tiles_cycle(&mut self) -> u16 {
        self.fetch_tile_data();
        1
    }

    fn tick_unknown_fetch(&mut self) -> u16 {
        // These cycles just read the next nametable byte for no reason.
        // This is used by one mapper to detect hblank, so have to include it.
        let addr = self.tile_address();
        self.tmp_pattern_coords = self.memory.read(addr);
        1
    }

    // --- FETCHING
    // Put all the memory fetching logic in one place.

    // Reload shift registers from their associated latches.
    fn reload_shift_registers(&mut self) {
        self.tile_register_low &= 0x00FF;
        self.tile_register_low |= (self.tile_latch_low as u16) << 8;
        self.tile_register_high &= 0x00FF;
        self.tile_register_high |= (self.tile_latch_high as u16) << 8;

        self.attribute_register_1 = self.attribute_latch_1;
        self.attribute_register_2 = self.attribute_latch_2;
    }

    // Shift the registers.
    fn shift_registers(&mut self) {
        self.tile_register_low >>= 1;
        self.tile_register_high >>= 1;
    }

    // Memory accesses for next tile data.
    fn fetch_tile_data(&mut self) {
        // We fetch 4 bytes in turn (each fetch takes 2 cycles):
        // These reads begin on cycle 1.
        // TODO: Fill in attribute table loading.
        match self.cycle % 8 {
            // 1. Nametable byte.
            1 => {
                let addr = self.tile_address();
                self.tmp_pattern_coords = self.memory.read(addr);
            },

            // 2. Attribute table byte.
            3 => (),

            // 3. Tile bitmap low.
            5 => {
                let addr = self.pattern_address_low();
                self.tile_latch_low = self.memory.read(addr);
            },

            // 4. Tile bitmap high.
            7 => {
                let addr = self.pattern_address_high();
                self.tile_latch_high = self.memory.read(addr);
            },

            // Do nothing on inbetween cycles.
            _ => (),
        };
    }

    // --- RENDERING
    // Put all rendering logic in one place.
    fn render_pixel(&self) -> Colour {
        // TODO: Implement palettes properly.
        let palette = Palette {
            c1: Colour { byte: 0x00 },
            c2: Colour { byte: 0x0F },
            c3: Colour { byte: 0xFF },
        };

        let bg_low_bit = (self.tile_register_low >> self.fine_x) & 1;
        let bg_high_bit = (self.tile_register_high >> self.fine_x) & 1;

        let colour_index = (bg_high_bit << 1) & bg_low_bit;

        match colour_index {
            0 => Colour { byte: 0x00 },
            1 => palette.c1,
            2 => palette.c2,
            3 => palette.c3,
            _ => panic!("Got an unexpected colour index: {}", colour_index)
        }
    }

    // --- SCROLLING
    // Put all scrolling logic in one place.
    fn handle_scrolling(&mut self) {
        // No scrolling happens if rendering is disabled.
        if !self.rendering_is_enabled() {
            return;
        }

        // If rendering is enabled, on dot 256 of each scanline, the PPU increments y position.
        if self.cycle == 256 {
            self.increment_y();
        }

        // If rendering is enabled, on dot 257 of each scanline, copy all horizontal bits from t to v.
        if self.cycle == 257 {
            let horizontal_bitmask = 0b0000100_00011111;
            self.v = self.v & !horizontal_bitmask;
            self.v = self.v | (self.t & horizontal_bitmask);
        }

        // If rendering is enabled, between dots 280 to 304 of the pre-render scanline, the PPU repeatedly copies the
        // vertical bits from t to v.
        if self.scanline == 261 && self.cycle >= 280 && self.cycle <= 304 {
            let vertical_bitmask = 0b1111011_11100000;
            self.v = self.v & !vertical_bitmask;
            self.v = self.v | (self.t & vertical_bitmask);
        }

        // Between dot 328 of a scanline, and 256 of the next scanline, x scroll is incremented
        // on every multiple of 8 dots except 0.  i.e. 328, 336, 8, 16, ..., 256.
        if ((self.cycle > 0 && self.cycle <= 256) || self.cycle >= 328) && (self.cycle % 8 == 0) {
            self.increment_coarse_x();
        }
    }

    // During rendering the VRAM address v is laid out like so:
    // yyy NN YYYYY XXXXX
    // ||| || ||||| +++++-- coarse X scroll
    // ||| || +++++-------- coarse Y scroll
    // ||| ++-------------- nametable select
    // +++----------------- fine Y scroll
    //
    // Here are some convenience methods to pull out these values.
    fn fine_y_scroll(&self) -> u16 {
        ((self.v >> 12) & 0b111) as u16
    }

    fn nametable_select(&self) -> u16 {
        ((self.v >> 10) & 0b11) as u16
    }

    fn coarse_y_scroll(&self) -> u16 {
        ((self.v >> 5) & 0b11111) as u16
    }

    fn coarse_x_scroll(&self) -> u16 {
        (self.v & 0b11111) as u16
    }

    // Scrolling is complex, so split out the logic here.
    fn increment_coarse_x(&mut self) {
        if self.coarse_x_scroll() == 31 {
            self.v &= !0x001F;  // Coarse X = 0.
            self.v ^= 0x0400;  // Switch horizontal nametable.
        } else {
            self.v += 1;  // Increment coarse X.
        }
    }

    fn increment_y(&mut self) {
        if self.fine_y_scroll() < 7 {
            self.v += 0x100;  // Increment fine Y.
        } else {
            self.v &= !0x700;  // Fine Y = 0.
            let mut coarse_y = self.coarse_y_scroll();
            if coarse_y == 29 {
                coarse_y = 0;
                self.v ^= 0x0800;  // Switch vertical nametable.
            } else if coarse_y == 31 {
                coarse_y = 0;
            } else {
                coarse_y += 1;
            }

            self.v = (self.v & !0x03E0) | ((coarse_y as u16) << 5);  // Put coarse_y back into v.
        }
    }

    // And then methods to load the tile and attribute addresses to load next.
    fn tile_address(&self) -> u16 {
        0x2000 | (self.v & 0x0FFF)
    }

    fn attribute_address(&self) -> u16 {
        // This formula copied from nesdev wiki.  I should try to understand it later.
        0x23C0 | self.nametable_select() | ((self.v >> 4) & 0x38) | ((self.v >> 2) & 0x07)
    }

    fn pattern_address_low(&self) -> u16 {
        ((self.pattern_table_side as u16) << 12)  // Left or right half of sprite table.
            | ((self.tmp_pattern_coords as u16) << 4)  // Tile coordinates.
            | 0b0000  // Lower bit plane.
            | self.fine_y_scroll()  // Fine Y offset.
    }

    fn pattern_address_high(&self) -> u16 {
        ((self.pattern_table_side as u16) << 12)  // Left or right half of sprite table.
            | ((self.tmp_pattern_coords as u16) << 4)  // Tile coordinates.
            | 0b1000  // Upper bit plane.
            | self.fine_y_scroll()  // Fine Y offset.
    }

    // Utility methods to query internal state.
    fn rendering_is_enabled(&self) -> bool {
        self.ppumask & 0b0001_1000 != 0
    }

    fn is_vblanking(&self) -> bool {
        self.scanline >= 241
    }

    fn is_rendering(&self) -> bool {
        !self.is_vblanking() && self.rendering_is_enabled()
    }
}
