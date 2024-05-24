use super::super::clocks;
use super::super::ports;
use super::display;

#[derive(Clone, Copy, Default)]
pub struct TileAttribute {
    priority_cleared: bool,
    priority: u8,
    palette_select: bool,
    vertical_flip: bool,
    horizontal_flip: bool,
    tile_number: u16,
}

#[derive(Clone, Default)]
pub struct HorizontalScroll {
    column_offset: u8,
    fine_scroll: u8,
    x_offset: u16,
}

#[derive(Clone, Default)]
pub struct Sprite {
    tile_number: u16,
    x: u16,
    y: u16,
}

#[derive(Clone, Default)]
pub struct PatternInfo {
    colours: u16, // TODO: Check, looks as though this should be a u8.
}

pub struct Mode1Settings {
    y_scroll: u8,
    x_scroll: u16,
    h_sync_interrupt_enabled: bool,
    start_x: u8,
    display_mode_1: u8,
}

impl Mode1Settings {
    pub fn new() -> Self {
        Self {
            y_scroll: 0,
            x_scroll: 0,
            h_sync_interrupt_enabled: false,
            start_x: 0,
            display_mode_1: 0,
        }
    }

    pub fn update_mode_1_settings(&mut self, mode_1_input: u8) {
        Mode1Settings::mode_1_settings(
            mode_1_input,
            &mut self.y_scroll,
            &mut self.x_scroll,
            &mut self.h_sync_interrupt_enabled,
            &mut self.start_x,
            &mut self.display_mode_1,
        );
    }

    fn mode_1_settings(
        mode_1_input: u8,
        y_scroll: &mut u8,
        x_scroll: &mut u16,
        h_sync_interrupt_enabled: &mut bool,
        start_x: &mut u8,
        display_mode_1: &mut u8,
    ) {
        // Set first scrolling line
        if 0 != (mode_1_input & Constants::VDP0DISHSCROLL) {
            *y_scroll = 16;
        } else {
            *y_scroll = 0;
        }

        // Set last scrolling position
        // TODO: disable vertical scroll flag not used/implemented
        if 0 != (mode_1_input & Constants::VDP0DISVSCROLL) {
            *x_scroll = 192;
        } else {
            *x_scroll = Constants::SMS_WIDTH;
        }

        *h_sync_interrupt_enabled = 0 != (mode_1_input & Constants::VDP0LINEINTENABLE);

        if 0 != (mode_1_input & Constants::VDP0COL0OVERSCAN) {
            *start_x = Constants::PATTERNWIDTH;
        } else {
            *start_x = 0;
        }

        // TODO: Add additional sprite control
        //            if (mode_1_input & Constants::VDP0SHIFTSPRITES) {
        //                errors.warning("Sprite shift not implemented")
        //
        //            if (mode_1_input & Constants::VDP0NOSYNC) {
        //                errors.warning("No sync, not implemented")

        *display_mode_1 = 0;
        if 0 != (mode_1_input & Constants::VDP0M4) {
            *display_mode_1 |= 8;
        }
        if 0 != (mode_1_input & Constants::VDP0M2) {
            *display_mode_1 |= 2;
        }
    }
}

pub struct Mode2Settings {
    v_sync_interrupt_enabled: bool,
    enable_display: bool,
    sprite_height: u8,
    sprite_width: u8,
    display_mode_2: u8,
}

impl Mode2Settings {
    fn new() -> Self {
        Self {
            v_sync_interrupt_enabled: false,
            enable_display: false,
            sprite_height: 8,
            sprite_width: 8,
            display_mode_2: 0,
        }
    }

    pub fn update_mode_2_settings(&mut self, mode_2_input: u8) {
        Mode2Settings::mode_2_settings(
            mode_2_input,
            &mut self.v_sync_interrupt_enabled,
            &mut self.enable_display,
            &mut self.sprite_height,
            &mut self.display_mode_2,
        );
    }

    fn mode_2_settings(
        mode_2_input: u8,
        v_sync_interrupt_enabled: &mut bool,
        enable_display: &mut bool,
        sprite_height: &mut u8,
        display_mode_2: &mut u8,
    ) {
        *v_sync_interrupt_enabled = 0 != (mode_2_input & Constants::VDP1VSYNC);

        *enable_display = 0 != (mode_2_input & Constants::VDP1ENABLEDISPLAY);

        if 0 != (mode_2_input & Constants::VDP1BIGSPRITES) {
            *sprite_height = 16;
        } else {
            *sprite_height = 8;
        }

        // TODO: Add double sprites
        //            if (mode_2_input & Constants::VDP1DOUBLESPRITES) {
        //                errors.warning("Double sprites not implemented")

        *display_mode_2 = 0;
        if 0 != (mode_2_input & Constants::VDP1M3) {
            *display_mode_2 |= 4;
        }
        if 0 != (mode_2_input & Constants::VDP1M1) {
            *display_mode_2 |= 1;
        }
    }
}

pub struct Constants {}

impl Constants {
    const RAMSIZE: u16 = 0x4000;
    const CRAMSIZE: u8 = 0x20;
    // 3Mhz CPU, 50Hz refresh ~= 60000 ticks
    const VSYNCCYCLETIME: u16 = 65232;
    const BLANKTIME: u16 = ((Constants::VSYNCCYCLETIME as u32 * 72) / 262) as u16;
    const VFRAMETIME: u16 =
        ((Constants::VSYNCCYCLETIME as u32 * Constants::SMS_HEIGHT as u32) / 262) as u16;
    const HSYNCCYCLETIME: u16 = 216;

    const REGISTERMASK: u8 = 0x0F;
    const REGISTERUPDATEMASK: u8 = 0xF0;
    const REGISTERUPDATEVALUE: u8 = 0x80;
    const NUMVDPREGISTERS: u8 = 16;

    // Vdp status register
    const VSYNCFLAG: u8 = 0x80;

    // Vdp register 0
    const MODE_CONTROL_NO_1: u8 = 0x0;
    const VDP0DISVSCROLL: u8 = 0x80;
    const VDP0DISHSCROLL: u8 = 0x40;
    const VDP0COL0OVERSCAN: u8 = 0x20;
    const VDP0LINEINTENABLE: u8 = 0x10;
    const VDP0SHIFTSPRITES: u8 = 0x08;
    const VDP0M4: u8 = 0x04;
    const VDP0M2: u8 = 0x02;
    const VDP0NOSYNC: u8 = 0x01;

    // Vdp register 1 /* bit 7 unused. */
    const MODE_CONTROL_NO_2: u8 = 0x1;
    const VDP1ENABLEDISPLAY: u8 = 0x40;
    const VDP1VSYNC: u8 = 0x20;
    const VDP1M1: u8 = 0x10;
    const VDP1M3: u8 = 0x08;
    const VDP1BIGSPRITES: u8 = 0x02;
    const VDP1DOUBLESPRITES: u8 = 0x01;

    const NUMSPRITES: u8 = 64;

    pub const SMS_WIDTH: u16 = 256;
    pub const SMS_HEIGHT: u16 = 192; // MAX HEIGHT, TODO: Consider changing to 'u8'.
    const SMS_COLOR_DEPTH: u8 = 16;

    const MAXPATTERNS: u16 = 512;
    const PATTERNWIDTH: u8 = 8;
    const PATTERNHEIGHT: u8 = 8;
    const PATTERNSIZE: u8 = 64;

    const MAXPALETTES: u8 = 2;

    const NUMTILEATTRIBUTES: u16 = 0x700;
    const TILEATTRIBUTEMASK: u16 = 0x7FF;
    const TILEATTRIBUTESADDRESSMASK: u16 = 0x3800;
    const TILEATTRIBUTESTILEMASK: u16 = 0x07FE;
    const TILESHIFT: u8 = 1;
    const TILEATTRIBUTESHMASK: u16 = 0x0001;
    const TILEPRIORITYSHIFT: u8 = 4;
    const TILEPALETTESHIFT: u8 = 3;
    const TILEVFLIPSHIFT: u8 = 2;
    const TILEHFLIPSHIFT: u8 = 1;

    const YTILES: u8 = 28;
    const XTILES: u8 = 32;
    const NUMTILES: u16 = Constants::XTILES as u16 * Constants::YTILES as u16;

    const SPRITEATTRIBUTESADDRESSMASK: u16 = 0x3F00;
    const SPRITEATTRIBUTESMASK: u16 = 0x00FF;
    const NUMSPRITEATTRIBUTES: u16 = 0x00FF;

    const SPRITETILEMASK: u16 = 0x0001;

    const LASTSPRITETOKEN: u8 = 0xD0;
    const SPRITEXNMASK: u16 = 0x0080;
    const MAXSPRITES: u8 = 64;
    const MAXSPRITESPERSCANLINE: u8 = 8;

    const PATTERNADDRESSLIMIT: u16 = 0x4000;
}

#[derive(Clone)]
pub struct ScanLines {
    scan_line: Vec<display::Colour>,
}

impl ScanLines {
    pub fn new(width: u16) -> Self {
        Self {
            scan_line: vec![display::Colour::new(0, 0, 0); width as usize],
        }
    }
}

#[derive(Clone)]
pub struct SpriteScanLines {
    scan_line: Vec<u8>,
    num_sprites: u16,
    sprites: Vec<u8>,
}

impl SpriteScanLines {
    pub fn new(width: u16) -> Self {
        Self {
            scan_line: vec![0; width as usize],
            num_sprites: 0,
            sprites: vec![0; Constants::MAXSPRITES as usize],
        }
    }
}

#[derive(Clone)]
pub struct BackgroundScanLines {
    scan_line: Vec<u8>,
}

impl BackgroundScanLines {
    pub fn new(width: u16) -> Self {
        Self {
            scan_line: vec![0; width as usize],
        }
    }
}

#[derive(Clone)]
pub struct PriorityScanLines {
    scan_line: Vec<bool>,
    has_priority: bool,
}

impl PriorityScanLines {
    pub fn new(width: u16) -> Self {
        Self {
            scan_line: vec![false; width as usize],
            has_priority: false,
        }
    }
}

pub struct DisplayBuffers {
    background_scan_lines: Vec<ScanLines>,
    forground_scan_lines: Vec<PriorityScanLines>,
    scan_lines: Vec<ScanLines>,
    sprite_scan_lines: Vec<SpriteScanLines>,
}

impl DisplayBuffers {
    pub fn new() -> Self {
        Self {
            background_scan_lines: vec![
                ScanLines::new(
                    (Constants::PATTERNWIDTH as u16)
                        * (Constants::XTILES as u16)
                        * (Constants::YTILES as u16)
                );
                (Constants::YTILES * Constants::PATTERNHEIGHT) as usize
            ],
            forground_scan_lines: vec![
                PriorityScanLines::new(
                    (Constants::PATTERNWIDTH as u16)
                        * (Constants::XTILES as u16)
                        * (Constants::YTILES as u16)
                );
                (Constants::YTILES * Constants::PATTERNHEIGHT) as usize
            ],
            scan_lines: vec![ScanLines::new(Vdp::FRAME_WIDTH); Constants::SMS_HEIGHT as usize],
            sprite_scan_lines: vec![
                SpriteScanLines::new(Vdp::FRAME_WIDTH);
                Constants::SMS_HEIGHT as usize
            ],
        }
    }
}

// Create a dummy Vdp, to try out hooking into ports.
pub struct Vdp {
    ram: Vec<u8>,
    c_ram: Vec<u8>,

    vdp_register: [u8; Constants::NUMVDPREGISTERS as usize],

    screen_buffer_pending: bool,

    horizontal_scroll_info: Vec<HorizontalScroll>,
    vertical_scroll_info: Vec<u8>,

    last_horizontal_scroll_info: Vec<HorizontalScroll>,
    last_vertical_scroll_info: Vec<u8>,

    pattern_info: Vec<PatternInfo>,
    screen_palette: Vec<display::Colour>,

    display_buffers: DisplayBuffers,

    patterns4: Vec<u8>,
    patterns16: Vec<Vec<display::Colour>>,
    tile_attributes: Vec<TileAttribute>,
    sprites: Vec<Sprite>,

    total_sprites: u8,

    mode_1_control: Mode1Settings,
    mode_2_control: Mode2Settings,

    // address attributes.
    current_address: u16,
    sprite_attributes_address: u16,
    tile_attributes_address: u16,
    write_bf_low_address: u8, // Holds the 'low' byte of the address when a write to bf occurs.

    border_colour: u8,

    code_register: u8,

    read_be_latch: u8,
    address_latch: bool,

    display_mode: u8,

    interrupt_handler: VDPInterrupts,

    sprite_tile_shift: u16,
    horizontal_scroll: u8,
    vertical_scroll: u8,

    // Used internally for debugging/diagnostics
    debug_name_table_offset: u16,
    debug_sprite_information_table_offset: u16,
}

pub struct VDPInterrupts {
    vdp_status_register: u8,
    v_sync: u16,
    y_end: u16,
    current_y_pos: u16,
    last_v_sync_clock: clocks::Clock,
    line_int_time: u32,
    line_interrupt: u16,
    line_interrupt_latch: u16,

    h_int_pending: bool,
    v_int_pending: bool,
    v_sync_interrupt_enabled: bool,
    h_sync_interrupt_enabled: bool,

    frame_updated: bool,
}

impl Vdp {
    const FRAME_WIDTH: u16 = Constants::SMS_WIDTH;
    const FRAME_HEIGHT: u16 = Constants::SMS_HEIGHT;

    pub fn new() -> Self {
        Self {
            ram: vec![0; Constants::RAMSIZE as usize],
            c_ram: vec![0; Constants::CRAMSIZE as usize],
            vdp_register: [0; Constants::NUMVDPREGISTERS as usize],
            screen_buffer_pending: false,

            // One entry per scan line for horizontal and vertical scroll info.
            horizontal_scroll_info: vec![
                HorizontalScroll::default();
                Constants::SMS_HEIGHT as usize
            ],
            vertical_scroll_info: vec![0; Constants::SMS_HEIGHT as usize],

            last_horizontal_scroll_info: vec![
                HorizontalScroll::default();
                Constants::SMS_HEIGHT as usize
            ],
            last_vertical_scroll_info: vec![0; Constants::SMS_HEIGHT as usize],

            pattern_info: vec![PatternInfo::default(); Constants::MAXPATTERNS as usize],
            screen_palette: vec![display::Colour::new(0, 0, 0); Constants::CRAMSIZE as usize],
            display_buffers: DisplayBuffers::new(),
            patterns4: vec![0; (Constants::MAXPATTERNS * (Constants::PATTERNSIZE as u16)) as usize],
            patterns16: vec![
                vec![
                    display::Colour::new(0, 0, 0);
                    (Constants::MAXPATTERNS * (Constants::PATTERNSIZE as u16)) as usize
                ];
                Constants::MAXPALETTES as usize
            ],
            tile_attributes: vec![TileAttribute::default(); Constants::NUMTILEATTRIBUTES as usize],
            sprites: vec![Sprite::default(); Constants::MAXSPRITES as usize],

            total_sprites: Constants::MAXSPRITES,

            mode_1_control: Mode1Settings::new(),
            mode_2_control: Mode2Settings::new(),
            current_address: 0,
            sprite_attributes_address: 0,
            tile_attributes_address: 0,

            code_register: 0,

            read_be_latch: 0,
            write_bf_low_address: 0,
            border_colour: 0,
            address_latch: false,
            display_mode: 0,
            interrupt_handler: VDPInterrupts::new(),

            sprite_tile_shift: 0,
            horizontal_scroll: 0,
            vertical_scroll: 0,

            debug_name_table_offset: 0,
            debug_sprite_information_table_offset: 0,
        }
    }

    pub fn read_port_7e(&mut self, clock: &clocks::Clock) -> u8 {
        self.address_latch = false; // Address is unlatched during port read

        let v_counter: u8 = ((clock.cycles - self.interrupt_handler.last_v_sync_clock.cycles)
            as u32
            / Constants::HSYNCCYCLETIME as u32) as u8;
        self.interrupt_handler.current_y_pos =
            (((clock.cycles - self.interrupt_handler.last_v_sync_clock.cycles) as u32
                / Constants::HSYNCCYCLETIME as u32)
                + 1) as u16;

        // I can't think of an ellegant solution, so this is as good as it gets
        // for now (fudge factor and all)
        // TODO: Add joystick (light gun)        self.inputs.joystick.setYpos(vCounter+10)
        v_counter
    }

    pub fn read_port_7f(&mut self, _clock: &clocks::Clock) -> u8 {
        self.address_latch = false; // Address is unlatched during port read

        // TODO: Add/fix joystick (light gun)
        // I can't think of an ellegant solution, so this is as good as it gets
        // for now (fudge factor and all)
        // hCounter = ((self.inputs.joystick.getXpos() + 0x28)/2 & 0x7F)
        0
    }

    pub fn read_port_be(&mut self, _clock: &clocks::Clock) -> u8 {
        self.address_latch = false; // Address is unlatched during port read

        let data = self.read_be_latch;

        self.current_address = (self.current_address + 1) & 0x3FFF; // Should be ok without this
        self.read_be_latch = self.ram[self.current_address as usize];

        data
    }

    pub fn set_palette(&mut self, address: u16, data: u8) {
        // uint16 colour
        // uint8 r, g, b

        let addr = address as u8 % Constants::CRAMSIZE;

        if self.c_ram[addr as usize] != data {
            self.c_ram[addr as usize] = data;

            // Generate 8-bit RGB components, just to be generic
            let r = ((data as u16 & 0x3) * 0xFF) / 0x3;
            let g = (((data as u16 >> 2) & 0x3) * 0xFF) / 0x3;
            let b = (((data as u16 >> 4) & 0x3) * 0xFF) / 0x3;

            let colour = display::Colour::new(r as u8, g as u8, b as u8);

            self.screen_palette[addr as usize] = colour;
        }
    }

    pub fn update_tile_attributes(&mut self, address: u16, old_data: u8, data: u8) {
        // Only update if altered
        if old_data != data {
            let tile = (address & Constants::TILEATTRIBUTESTILEMASK) >> Constants::TILESHIFT;

            // Alteration of the high byte
            if 0 != address & Constants::TILEATTRIBUTESHMASK {
                if 0 != self.tile_attributes[tile as usize].priority
                    && 0 == (data >> Constants::TILEPRIORITYSHIFT)
                {
                    self.tile_attributes[tile as usize].priority_cleared = true;
                }

                self.tile_attributes[tile as usize].priority = data >> Constants::TILEPRIORITYSHIFT;
                self.tile_attributes[tile as usize].palette_select =
                    0 != (data >> Constants::TILEPALETTESHIFT) & 0x1;
                self.tile_attributes[tile as usize].vertical_flip =
                    0 != (data >> Constants::TILEVFLIPSHIFT) & 0x1;
                self.tile_attributes[tile as usize].horizontal_flip =
                    0 != (data >> Constants::TILEHFLIPSHIFT) & 0x1;
                self.tile_attributes[tile as usize].tile_number =
                    (self.tile_attributes[tile as usize].tile_number & 0xFF)
                        | (((data as u16) & 0x1) << 8);
            } else {
                self.tile_attributes[tile as usize].tile_number =
                    (self.tile_attributes[tile as usize].tile_number & 0x100) | (data as u16);
            }
        }
    }

    pub fn update_sprite_attributes(&mut self, address: u16, old_data: u8, data: u8) {
        // Only update if need be
        if old_data != data {
            let mut sprite_num = (address & Constants::SPRITEATTRIBUTESMASK) as u8;

            // See if it's an x, or tile number attributes
            if 0 != (sprite_num & Constants::SPRITEXNMASK as u8) {
                sprite_num = (sprite_num >> 1) ^ Constants::MAXSPRITES;
                if 0 != (address & Constants::SPRITETILEMASK) {
                    // Changing tile
                    self.sprites[sprite_num as usize].tile_number =
                        data as u16 | self.sprite_tile_shift;
                } else {
                    // Changing x position
                    self.sprites[sprite_num as usize].x = data as u16;
                }
            } else if sprite_num < Constants::MAXSPRITES {
                // Updating y attribute
                // The number of sprites has changed, do some work updating
                // the appropriate scanlines

                // If inserting a new token earlier then previous, remove tiles
                if data == Constants::LASTSPRITETOKEN {
                    if sprite_num < self.total_sprites {
                        for i in (sprite_num..self.total_sprites).rev() {
                            // Not the most efficient, but fairly robust
                            for y in (self.sprites[i as usize].y)
                                ..(self.sprites[i as usize].y
                                    + (self.mode_2_control.sprite_height as u16))
                            {
                                self.remove_sprite_to_scan_lines(y, i);
                            }
                        }
                        self.total_sprites = sprite_num;
                    }

                    self.sprites[sprite_num as usize].y = (data as u16) + 1;
                } else if (self.sprites[sprite_num as usize].y
                    == (Constants::LASTSPRITETOKEN + 1) as u16)
                    && (sprite_num == self.total_sprites)
                {
                    // Removing token, so extend the number of sprites

                    self.total_sprites += 1;
                    while (self.total_sprites < Constants::MAXSPRITES)
                        && (self.sprites[self.total_sprites as usize].y
                            != (Constants::LASTSPRITETOKEN + 1) as u16)
                    {
                        self.total_sprites += 1;
                    }

                    self.sprites[sprite_num as usize].y = (data as u16) + 1;

                    // Not the most efficient, but fairly robust
                    for i in sprite_num..self.total_sprites {
                        for y in self.sprites[i as usize].y
                            ..(self.sprites[i as usize].y
                                + (self.mode_2_control.sprite_height as u16))
                        {
                            self.add_sprite_to_scan_lines(y, i);
                        }
                    }
                } else if sprite_num < self.total_sprites {
                    // Remove from previous scanlines, add to new scanlines
                    // Not the most efficient, but fairly robust
                    for y in self.sprites[sprite_num as usize].y
                        ..(self.sprites[sprite_num as usize].y
                            + (self.mode_2_control.sprite_height as u16))
                    {
                        self.remove_sprite_to_scan_lines(y, sprite_num);
                    }

                    self.sprites[sprite_num as usize].y = (data as u16) + 1;

                    for y in (self.sprites[sprite_num as usize].y)
                        ..(self.sprites[sprite_num as usize].y
                            + (self.mode_2_control.sprite_height as u16))
                    {
                        self.add_sprite_to_scan_lines(y, sprite_num);
                    }
                } else {
                    self.sprites[sprite_num as usize].y = (data as u16) + 1;
                }
            }
        }
    }

    pub fn add_sprite_to_scan_lines(&mut self, scan_line_number: u16, sprite_number: u8) {
        let scan_line_number = scan_line_number & 0xFF;

        if scan_line_number < self.interrupt_handler.y_end {
            assert!(
                self.display_buffers.sprite_scan_lines[scan_line_number as usize].num_sprites
                    != Constants::MAXSPRITES as u16
            );

            if self.display_buffers.sprite_scan_lines[scan_line_number as usize].num_sprites
                != Constants::MAXSPRITES as u16
            {
                let mut i =
                    self.display_buffers.sprite_scan_lines[scan_line_number as usize].num_sprites;
                self.display_buffers.sprite_scan_lines[scan_line_number as usize].num_sprites += 1;
                while i > 0 {
                    if self.display_buffers.sprite_scan_lines[scan_line_number as usize].sprites
                        [(i - 1) as usize]
                        < sprite_number
                    {
                        self.display_buffers.sprite_scan_lines[scan_line_number as usize].sprites
                            [i as usize] = sprite_number;
                        return;
                    }

                    self.display_buffers.sprite_scan_lines[scan_line_number as usize].sprites
                        [i as usize] = self.display_buffers.sprite_scan_lines
                        [scan_line_number as usize]
                        .sprites[(i - 1) as usize];
                    i -= 1;
                }

                self.display_buffers.sprite_scan_lines[scan_line_number as usize].sprites
                    [0_usize] = sprite_number;
            } else {
                panic!("Max sprite limit reached.");
            }
        }
    }

    pub fn remove_sprite_to_scan_lines(&mut self, scan_line_number: u16, sprite_number: u8) {
        let scan_line_number = scan_line_number & 0xFF;

        let mut shift = 0;

        if scan_line_number < self.interrupt_handler.y_end {
            let num_sprites =
                self.display_buffers.sprite_scan_lines[scan_line_number as usize].num_sprites;

            for i in 0..(num_sprites - shift) {
                if self.display_buffers.sprite_scan_lines[scan_line_number as usize].sprites
                    [i as usize]
                    == sprite_number
                {
                    shift += 1;
                    self.display_buffers.sprite_scan_lines[scan_line_number as usize]
                        .num_sprites -= 1;
                }

                if i + shift < Constants::MAXSPRITES as u16 {
                    self.display_buffers.sprite_scan_lines[scan_line_number as usize].sprites
                        [i as usize] = self.display_buffers.sprite_scan_lines
                        [scan_line_number as usize]
                        .sprites[(i + shift) as usize];
                } else {
                    println!("Index exceeds range of MAXSPRITES");
                }
            }
        }
    }

    pub fn update_horizontal_scroll_info(&mut self) {
        let column_offset = (0x20 - (self.horizontal_scroll >> 3)) & 0x1F;
        let fine_scroll = self.horizontal_scroll & 0x7;

        let pattern_offset = (column_offset as u16) * (Constants::PATTERNWIDTH as u16);
        let x_offset = if pattern_offset > fine_scroll as u16 {
            (pattern_offset - (fine_scroll as u16)) % Constants::SMS_WIDTH
        } else {
            ((pattern_offset + Constants::SMS_WIDTH) - (fine_scroll as u16)) % Constants::SMS_WIDTH
        };

        for y in
            self.interrupt_handler.current_y_pos as usize..self.interrupt_handler.y_end as usize
        {
            self.horizontal_scroll_info[y].column_offset = column_offset;
            self.horizontal_scroll_info[y].fine_scroll = fine_scroll;
            self.horizontal_scroll_info[y].x_offset = x_offset;
        }
    }

    pub fn update_vertical_scroll_info(&mut self) {
        for y in
            self.interrupt_handler.current_y_pos as usize..self.interrupt_handler.y_end as usize
        {
            self.vertical_scroll_info[y] = self.vertical_scroll;
        }
    }

    pub fn update_pattern(&mut self, address: u16, old_data: u8, data: u8) {
        let mut change = old_data ^ data; // Flip only the bits that have changed
        if change != 0 {
            let index = (address & 0x3FFC) << 1; // Base index (pattern + row)

            let mask = 1 << (address & 0x3); // Bit position to flip

            // Only update if the data has changed
            // From right to left
            let mut x = 7;
            while 0 != change {
                // Flip the bit position if required
                if 0 != change & 0x1 {
                    self.patterns4[(index + x) as usize] ^= mask;
                }

                x = x.saturating_sub(1);
                change >>= 1;
            }
        }
    }

    pub fn write_port_be(&mut self, _clock: &clocks::Clock, data: u8) {
        self.address_latch = false; // Address is unlatched during port read

        if self.code_register == 0x3 {
            // Write to video ram
            self.set_palette(self.current_address, data);
        } else {
            if ((self.current_address & Constants::TILEATTRIBUTESADDRESSMASK)
                == self.tile_attributes_address)
                && ((self.current_address & Constants::TILEATTRIBUTEMASK)
                    < Constants::NUMTILEATTRIBUTES)
            {
                self.update_tile_attributes(
                    self.current_address,
                    self.ram[self.current_address as usize],
                    data,
                );
            } else if ((self.current_address & Constants::SPRITEATTRIBUTESADDRESSMASK)
                == self.sprite_attributes_address)
                && ((self.current_address & Constants::SPRITEATTRIBUTESMASK)
                    < Constants::NUMSPRITEATTRIBUTES)
            {
                self.update_sprite_attributes(
                    self.current_address,
                    self.ram[self.current_address as usize],
                    data,
                );
            }
            if self.current_address < Constants::PATTERNADDRESSLIMIT {
                self.update_pattern(
                    self.current_address,
                    self.ram[self.current_address as usize],
                    data,
                );
            }

            self.ram[self.current_address as usize] = data; // Update after function call
            self.read_be_latch = data;
        }

        self.current_address = (self.current_address + 1) & 0x3FFF; // Should be ok without this
    }

    pub fn write_register(&mut self, register_number: u8, data: u8) {
        self.vdp_register[register_number as usize] = data; // Update register data

        // Only need to react immediately to some register changes
        match register_number {
            0 => {
                self.update_mode_1_control();
            }
            1 => {
                self.update_mode_2_control();
            }
            2 => {
                self.tile_attributes_address = ((data as u16) & 0xE) << 10;
                self.debug_name_table_offset = self.tile_attributes_address;
            }
            5 => {
                self.sprite_attributes_address = ((data as u16) & 0x7E) << 7;
                self.debug_sprite_information_table_offset = self.sprite_attributes_address;
            }

            6 => {
                // self._tileDefinitions = &self._vdpRAM[(data & 0x4) << 11]
                //  Probably should do more when this changes, as all the
                //  sprite tile numbers should change... maybe later
                self.sprite_tile_shift = ((data as u16) & 0x4) << 6;
            }

            7 => {
                println!("Using border colour: {:x}", data);
                self.border_colour = data & 0xf;
            }

            8 => {
                self.horizontal_scroll = data;
                self.update_horizontal_scroll_info();
            }

            9 => {
                self.vertical_scroll = data;
                self.update_vertical_scroll_info();
            }

            10 => {
                self.interrupt_handler.line_interrupt = data as u16;
            }
            _ => {
                println!(
                    "write_register: unsupported write register: {} ",
                    register_number
                );
            }
        }
    }

    pub fn write_port_bf(&mut self, _clock: &clocks::Clock, data: u8) {
        if !self.address_latch {
            self.write_bf_low_address = data;
            self.address_latch = true;
        } else {
            if (data & Constants::REGISTERUPDATEMASK) == Constants::REGISTERUPDATEVALUE {
                self.write_register(data & Constants::REGISTERMASK, self.write_bf_low_address)
            }

            self.current_address =
                ((self.write_bf_low_address as u16) + ((data as u16) << 8)) & 0x3FFF; // Should limit current_address to ram size
            self.code_register = data >> 6;
            self.address_latch = false;

            self.read_be_latch = self.ram[self.current_address as usize];
        }
    }

    pub fn read_port_bf(&mut self, _clock: &clocks::Clock) -> u8 {
        self.address_latch = false; // Address is unlatched during port read

        let original_value = self.interrupt_handler.vdp_status_register;
        self.interrupt_handler.vdp_status_register = 0;
        self.interrupt_handler.h_int_pending = false;
        self.interrupt_handler.v_int_pending = false;

        original_value
    }

    pub fn update_mode_1_control(&mut self) {
        self.mode_1_control
            .update_mode_1_settings(self.vdp_register[Constants::MODE_CONTROL_NO_1 as usize]);
        self.interrupt_handler.h_sync_interrupt_enabled =
            self.mode_1_control.h_sync_interrupt_enabled;
        self.update_display_mode(
            self.mode_1_control.display_mode_1,
            self.mode_2_control.display_mode_2,
        );
    }

    pub fn update_mode_2_control(&mut self) {
        self.mode_2_control
            .update_mode_2_settings(self.vdp_register[Constants::MODE_CONTROL_NO_2 as usize]);
        self.interrupt_handler.v_sync_interrupt_enabled =
            self.mode_2_control.v_sync_interrupt_enabled;
        self.update_display_mode(
            self.mode_1_control.display_mode_1,
            self.mode_2_control.display_mode_2,
        );
    }

    fn update_display_mode(&mut self, display_mode_1: u8, display_mode_2: u8) {
        self.display_mode = display_mode_1 | display_mode_2;

        // Need to see what the modes do/mean.
        if (self.display_mode == 0x8) || (self.display_mode == 0xA) {
            self.interrupt_handler.y_end = Constants::SMS_HEIGHT;
        } else {
            self.interrupt_handler.y_end = 0;
            println!("Mode not supported");
        }
    }

    fn single_scan(&mut self, y: u16) {
        let mut fine_scroll = 0;
        let mut x_offset = 0;

        let sprite_scan_y = &self.display_buffers.sprite_scan_lines[y as usize];
        let vertical_offset = self.vertical_scroll_info[y as usize];
        let v_y = vertical_offset as u16 + y;
        let tile_offset = v_y % ((Constants::YTILES * Constants::PATTERNHEIGHT) as u16);
        let horizontal_info_y = &self.horizontal_scroll_info[y as usize];
        let background_scan_y = &self.display_buffers.background_scan_lines[tile_offset as usize];
        let background_scan_y_line = &background_scan_y.scan_line;

        self.last_horizontal_scroll_info[y as usize].x_offset = horizontal_info_y.x_offset;
        self.last_vertical_scroll_info[y as usize] = self.vertical_scroll_info[y as usize];

        let scan_y_lines = &mut self.display_buffers.scan_lines[y as usize].scan_line;
        let forground_scan_y = &mut self.display_buffers.forground_scan_lines[tile_offset as usize];

        if y >= self.mode_1_control.y_scroll as u16 {
            fine_scroll = horizontal_info_y.fine_scroll;
            x_offset = horizontal_info_y.x_offset;
        }

        let mut x = fine_scroll;
        if self.mode_1_control.start_x > fine_scroll {
            x = self.mode_1_control.start_x;
        }

        // Copy background,  'x' is either [0, 8].  Split into 2 loops to avoid modulus
        // Copying the brackground appears to be the slowest sections of this function.
        let x_wrap_around = Constants::SMS_WIDTH - x_offset;
        for i in (x as u16)..x_wrap_around {
            scan_y_lines[i as usize] = background_scan_y_line[(x_offset + i) as usize];
        }

        let offset = (Constants::SMS_WIDTH as i16) - (x_offset as i16);
        for i in std::cmp::max(x as u16, x_wrap_around)..Constants::SMS_WIDTH {
            scan_y_lines[i as usize] = background_scan_y_line[(i as i16 - offset) as usize];
        }

        if sprite_scan_y.num_sprites > 0 {
            // If there is a transparent forground on this line
            if forground_scan_y.has_priority {
                for i in 0..sprite_scan_y.num_sprites {
                    let sprite_number = sprite_scan_y.sprites[i as usize];

                    let x_start = self.sprites[sprite_number as usize].x;
                    let x_end = std::cmp::min(
                        self.sprites[sprite_number as usize].x
                            + (self.mode_2_control.sprite_width as u16),
                        Constants::SMS_WIDTH,
                    );
                    let x_wrap_around = Constants::SMS_WIDTH - x_offset;

                    for x in x_start..x_wrap_around {
                        if (sprite_scan_y.scan_line[x as usize] != 0)
                            && !forground_scan_y.scan_line[(x + x_offset) as usize]
                        {
                            scan_y_lines[x as usize] = self.screen_palette
                                [(sprite_scan_y.scan_line[x as usize] | 0x10) as usize];
                        }
                    }

                    for x in std::cmp::max(x_start, x_wrap_around)..x_end {
                        if (sprite_scan_y.scan_line[x as usize] != 0)
                            && !forground_scan_y.scan_line
                                [(x + x_offset - Constants::SMS_WIDTH) as usize]
                        {
                            scan_y_lines[x as usize] = self.screen_palette
                                [(sprite_scan_y.scan_line[x as usize] | 0x10) as usize];
                        }
                    }
                }
            } else {
                for i in 0..sprite_scan_y.num_sprites {
                    let sprite_number = sprite_scan_y.sprites[i as usize];
                    for x in (self.sprites[sprite_number as usize].x)
                        ..std::cmp::min(
                            self.sprites[sprite_number as usize].x
                                + self.mode_2_control.sprite_width as u16,
                            Constants::SMS_WIDTH,
                        )
                    {
                        if sprite_scan_y.scan_line[x as usize] != 0 {
                            scan_y_lines[x as usize] = self.screen_palette
                                [(sprite_scan_y.scan_line[x as usize] | 0x10) as usize];
                        }
                    }
                }
            }

            for i in 0..self.mode_1_control.start_x {
                scan_y_lines[i as usize] = display::Colour::new(0, 0, 0);
            }
        }
    }

    fn update_display(&mut self, _raw_display: &mut [u8]) {
        // The 'export' function is now used to update display, when the display is being draw to screen (lazy/basically pull vs push).
    }

    fn clear_display(&mut self, _raw_display: &mut [u8]) {
        // The 'export' function is now used to update display, when the display is being draw to screen (lazy/basically pull vs push).
    }

    fn driver_update_display(&mut self, raw_display: &mut [u8]) {
        let mut index = 0;
        for y in &self.display_buffers.scan_lines {
            for x in &y.scan_line {
                x.convert_rgb888(
                    &mut raw_display
                        [index..(index + display::SDLUtility::bytes_per_pixel() as usize)],
                );
                index += display::SDLUtility::bytes_per_pixel() as usize;
            }
        }
    }

    fn draw_scan_lines(&mut self) {
        if self.mode_2_control.enable_display {
            // Only draw if 'enable_display' has been set.
            for y in 0..self.interrupt_handler.y_end {
                self.single_scan(y);
            }
        }
    }

    fn draw_buffer(&mut self) {
        self.draw_background();
        self.draw_sprites();

        // Draw the scan lines here (not in export), so scroll locations are locked in.
        self.draw_scan_lines();

        self.screen_buffer_pending = true;

        //        self.draw_patterns() // For debuging purposes
    }

    fn update_screen_pattern(&mut self, pattern_number: u16) {
        let mut index = pattern_number << 6;

        for _py in 0..Constants::PATTERNHEIGHT {
            for _px in 0..Constants::PATTERNWIDTH {
                let pixel4 = self.patterns4[index as usize];

                self.patterns16[0][index as usize] = self.screen_palette[pixel4 as usize];
                self.patterns16[1][index as usize] =
                    self.screen_palette[(pixel4 | (1 << 4)) as usize];
                index += 1;
            }
        }
    }

    fn draw_background(&mut self) {
        let mut tile = 0;

        for y in 0..(Constants::YTILES * Constants::PATTERNHEIGHT) {
            self.display_buffers.forground_scan_lines[y as usize].has_priority = false;
        }

        for y in (0..(Constants::YTILES * Constants::PATTERNHEIGHT) as u16)
            .step_by(Constants::PATTERNHEIGHT as usize)
        {
            for x in (0..(Constants::XTILES as u16) * (Constants::PATTERNWIDTH as u16))
                .step_by(Constants::PATTERNWIDTH as usize)
            {
                let mut tile_attribute = self.tile_attributes[tile as usize];

                if !self.display_buffers.forground_scan_lines[y as usize].has_priority
                    && (tile_attribute.priority != 0)
                {
                    for py in y..(y + (Constants::PATTERNHEIGHT as u16)) {
                        self.display_buffers.forground_scan_lines[py as usize].has_priority = true;
                    }
                }

                {
                    self.update_screen_pattern(tile_attribute.tile_number);

                    let patterns16_palette =
                        &self.patterns16[tile_attribute.palette_select as usize];

                    let mut patterns16_offset: i16 = (tile_attribute.tile_number as i16) << 6;
                    let mut pattern_y_delta: i16 = 0;
                    let mut pattern_x_delta: i16 = 1;

                    if tile_attribute.horizontal_flip {
                        patterns16_offset += (Constants::PATTERNWIDTH as i16) - 1;
                        pattern_x_delta = -1;
                        pattern_y_delta = (Constants::PATTERNWIDTH as i16) * 2;
                    }

                    if tile_attribute.vertical_flip {
                        patterns16_offset += ((Constants::PATTERNHEIGHT as i16) - 1) << 3;
                        pattern_y_delta -= 2 * Constants::PATTERNWIDTH as i16;
                    }

                    if tile_attribute.priority != 0 {
                        let mut pattern4_offset: i16 = (tile_attribute.tile_number as i16) << 6;

                        if tile_attribute.horizontal_flip {
                            pattern4_offset += Constants::PATTERNWIDTH as i16 - 1;
                        }

                        if tile_attribute.vertical_flip {
                            pattern4_offset += (Constants::PATTERNHEIGHT as i16 - 1) << 3;
                        }

                        for py in y..(y + (Constants::PATTERNHEIGHT as u16)) {
                            let background_y_line =
                                &mut self.display_buffers.background_scan_lines[py as usize];
                            let forground_y_line =
                                &mut self.display_buffers.forground_scan_lines[py as usize];
                            for px in x..(x + (Constants::PATTERNWIDTH as u16)) {
                                background_y_line.scan_line[px as usize] =
                                    patterns16_palette[patterns16_offset as usize];

                                // Indicate a forground pixel if the value is
                                // non-zero and it is set as a forground pixel...
                                // well tile
                                forground_y_line.scan_line[px as usize] = false;

                                // 'priority' is always true in this branch
                                if self.patterns4[pattern4_offset as usize] != 0x0 {
                                    forground_y_line.scan_line[px as usize] = true;
                                }

                                patterns16_offset += pattern_x_delta;
                                pattern4_offset += pattern_x_delta;
                            }
                            patterns16_offset += pattern_y_delta;
                            pattern4_offset += pattern_y_delta;
                        }
                    } else {
                        if tile_attribute.priority_cleared {
                            for py in y..(y + (Constants::PATTERNHEIGHT as u16)) {
                                let forground_y_line =
                                    &mut self.display_buffers.forground_scan_lines[py as usize];
                                for px in x..(x + (Constants::PATTERNWIDTH as u16)) {
                                    forground_y_line.scan_line[px as usize] = false;
                                }
                            }
                            tile_attribute.priority_cleared = false;
                        }

                        for py in y..(y + (Constants::PATTERNHEIGHT as u16)) {
                            let background_y_line =
                                &mut self.display_buffers.background_scan_lines[py as usize];
                            if pattern_x_delta == 1 {
                                for i in 0..(Constants::PATTERNWIDTH as u16) {
                                    background_y_line.scan_line[(x + i) as usize] =
                                        patterns16_palette[(patterns16_offset as u16 + i) as usize];
                                }
                                patterns16_offset += Constants::PATTERNWIDTH as i16;
                            } else {
                                for px in 0..(Constants::PATTERNWIDTH as u16) {
                                    background_y_line.scan_line[(x + px) as usize] =
                                        patterns16_palette[patterns16_offset as usize];
                                    patterns16_offset += pattern_x_delta;
                                }
                            }
                            patterns16_offset += pattern_y_delta;
                        }
                    }
                }

                self.tile_attributes[tile as usize] = tile_attribute;

                tile += 1;
            }
        }
    }

    fn draw_sprites(&mut self) {
        // Check for any sprite alterations
        for i in 0..self.total_sprites {
            let mut y = self.sprites[i as usize].y;
            while (y < self.sprites[i as usize].y + (self.mode_2_control.sprite_height as u16))
                && (y < Constants::SMS_HEIGHT)
            {
                y += 1;
            }
        }

        for y in 0..self.interrupt_handler.y_end {
            for i in 0..Constants::SMS_WIDTH {
                self.display_buffers.sprite_scan_lines[y as usize].scan_line[i as usize] = 0;
            }

            let mut i = 0;
            while (i < self.display_buffers.sprite_scan_lines[y as usize].num_sprites)
                && (i < Constants::MAXSPRITESPERSCANLINE as u16)
            {
                let sprite_num =
                    self.display_buffers.sprite_scan_lines[y as usize].sprites[i as usize];

                // Adding check to avoid out of bounds from tiley index
                if (y > self.sprites[sprite_num as usize].y)
                    || ((y + Constants::SMS_HEIGHT) > self.sprites[sprite_num as usize].y)
                {
                    // FIXME, loosing motivation, this is better but still
                    // not quite right
                    let tiley = if self.sprites[sprite_num as usize].y > Constants::SMS_HEIGHT {
                        y - self.sprites[sprite_num as usize].y + Constants::SMS_HEIGHT
                    } else {
                        y - self.sprites[sprite_num as usize].y
                    };

                    let tile_addr =
                        (self.sprites[sprite_num as usize].tile_number << 6) | (tiley << 3);
                    for x in 0..self.mode_2_control.sprite_width {
                        // If the line is clear
                        if ((self.sprites[sprite_num as usize].x + x as u16) < Constants::SMS_WIDTH)
                            && (self.display_buffers.sprite_scan_lines[y as usize].scan_line
                                [(self.sprites[sprite_num as usize].x + x as u16) as usize]
                                == 0)
                        {
                            self.display_buffers.sprite_scan_lines[y as usize].scan_line
                                [(self.sprites[sprite_num as usize].x + x as u16) as usize] =
                                self.patterns4[(tile_addr | x as u16) as usize];
                        }
                    }
                }

                i += 1;
            }
        }
    }

    fn print_debug_info(&mut self) {
        println!(
            "{} {}",
            self.debug_name_table_offset, self.debug_sprite_information_table_offset
        );
    }

    // Draw the background tiles
    fn draw_patterns(&mut self) {
        let mut pattern: u16 = 0;
        let palette_select = 1;

        for y in 0..16 {
            for x in 0..Constants::XTILES {
                for py in 0..Constants::PATTERNHEIGHT {
                    for px in 0..Constants::PATTERNWIDTH {
                        let pixel4 = self.patterns4
                            [((pattern << 6) | ((py as u16) << 3) | (px as u16)) as usize]
                            | (palette_select << 4);
                        self.display_buffers.background_scan_lines
                            [(Constants::PATTERNHEIGHT * y + py) as usize]
                            .scan_line[(Constants::PATTERNWIDTH * x + px) as usize] =
                            self.screen_palette[pixel4 as usize];
                        self.display_buffers.scan_lines
                            [(Constants::PATTERNHEIGHT * y + py) as usize]
                            .scan_line[(Constants::PATTERNWIDTH * x + px) as usize] =
                            self.screen_palette[pixel4 as usize];
                    }
                }
                pattern += 1;
            }
        }
    }
}

impl ports::Device for Vdp {
    fn port_read(&mut self, clock: &clocks::Clock, port_address: u8) -> Option<u8> {
        match port_address {
            // Even values: 0x80 -> 0xBE
            addr if (addr & 0xC1) == 0x80 => Some(self.read_port_be(clock)),
            // Odd values: 0x81 -> 0xBF
            addr if (addr & 0xC1) == 0x81 => Some(self.read_port_bf(clock)),
            // Add the vdp to port `7E' plus all the mirror ports, vdp v_counter
            n if (n & 0xC1 == 0x40) => Some(self.read_port_7e(clock)),
            // Add the vdp to port `7F' plus all the mirror ports, vdp h_counter
            n if (n & 0xC1 == 0x41) => Some(self.read_port_7f(clock)),

            _ => {
                None /* Unhandled */
            }
        }
    }
    fn port_write(&mut self, clock: &clocks::Clock, port_address: u8, value: u8) {
        match port_address {
            // Even values: 0x80 -> 0xBE
            addr if (addr & 0xC1) == 0x80 => {
                self.write_port_be(clock, value);
            }
            // Odd values: 0x81 -> 0xBF
            addr if (addr & 0xC1) == 0x81 => {
                self.write_port_bf(clock, value);
            }
            _ => {}
        }
    }

    fn poll_interrupts(&mut self, raw_display: &mut Vec<u8>, clock: &clocks::Clock) -> bool {
        self.interrupt_handler.update_in_frame_timing(clock);

        self.interrupt_handler.update_post_frame_timing();

        if self.interrupt_handler.v_sync >= Constants::VFRAMETIME {
            if self.mode_2_control.enable_display {
                self.update_display(raw_display);
            } else {
                self.clear_display(raw_display);
            }
        }

        self.interrupt_handler.update_vsync_timing(clock);

        // If v-sync has finished, then draw the buffer and update scroll info
        if self.interrupt_handler.v_sync == 0 {
            self.draw_buffer();
            self.update_horizontal_scroll_info();
            self.update_vertical_scroll_info();
        }

        self.interrupt_handler.poll_interrupts()
    }

    fn export(&mut self, raw_display: &mut Vec<u8>) -> bool {
        if self.screen_buffer_pending {
            self.driver_update_display(raw_display);

            // Clear the pending flag.
            self.screen_buffer_pending = false;

            true // Return an indication that the buffer was updated
        } else {
            false
        }
    }
}

impl VDPInterrupts {
    pub fn new() -> Self {
        Self {
            vdp_status_register: 0,
            v_sync: 0,
            last_v_sync_clock: clocks::Clock::new(),
            y_end: 0,
            current_y_pos: 0,
            line_int_time: 0,
            line_interrupt: 0,
            line_interrupt_latch: 0,
            h_int_pending: false,
            v_int_pending: false,
            v_sync_interrupt_enabled: true,
            h_sync_interrupt_enabled: true,

            frame_updated: false,
        }
    }
}

impl VDPInterrupts {
    fn update_in_frame_timing(&mut self, clock: &clocks::Clock) {
        self.v_sync = (clock.cycles - self.last_v_sync_clock.cycles) as u16;

        if (self.line_int_time < Constants::VFRAMETIME as u32)
            && (self.v_sync as u32 >= self.line_int_time)
        {
            self.current_y_pos = (((clock.cycles - self.last_v_sync_clock.cycles) as u32
                / Constants::HSYNCCYCLETIME as u32)
                + 1) as u16;

            self.line_interrupt_latch = self.line_interrupt + 1;
            self.line_int_time += (self.line_interrupt_latch * Constants::HSYNCCYCLETIME) as u32;

            self.h_int_pending = true;
        }
    }

    fn update_post_frame_timing(&mut self) {
        if !self.frame_updated && self.v_sync >= Constants::VFRAMETIME {
            self.frame_updated = true;
            self.v_int_pending = true;
            self.current_y_pos = self.y_end;

            self.vdp_status_register |= Constants::VSYNCFLAG;
        }
    }

    fn update_vsync_timing(&mut self, clock: &clocks::Clock) {
        if self.v_sync >= Constants::VSYNCCYCLETIME {
            self.frame_updated = false;
            self.last_v_sync_clock.cycles = clock.cycles;
            self.v_sync = 0;
            self.current_y_pos = 0;

            self.line_interrupt_latch = self.line_interrupt;
            self.line_int_time = (self.line_interrupt_latch * Constants::HSYNCCYCLETIME) as u32;
        }
    }

    fn poll_interrupts(&mut self) -> bool {
        (self.v_sync_interrupt_enabled && self.v_int_pending)
            || (self.h_sync_interrupt_enabled && self.h_int_pending)
    }
}

#[cfg(test)]
mod tests {
    use crate::sega::graphics::vdp;
    use sdl2::event;
    use sdl2::keyboard; // Keycode
    use sdl2::pixels;
    use sdl2::rect;

    const PIXEL_WIDTH: u16 = 2;
    const PIXEL_HEIGHT: u16 = 2;

    impl vdp::Vdp {
        pub fn driver_open_display(&mut self) {
            use rand::Rng;

            let sdl_context = sdl2::init().unwrap();
            let video_subsystem = sdl_context.video().unwrap();

            let window = video_subsystem
                .window(
                    "Rusty Sega",
                    (vdp::Vdp::FRAME_WIDTH * PIXEL_WIDTH) as u32,
                    (vdp::Vdp::FRAME_HEIGHT * PIXEL_HEIGHT) as u32,
                )
                .position_centered()
                .build()
                .unwrap();

            let mut canvas = window.into_canvas().build().unwrap();

            let mut event_pump = sdl_context.event_pump().unwrap();
            let mut i = 0;
            let mut rng = rand::thread_rng();

            canvas.set_draw_color(pixels::Color::RGB(0, 0, 0));
            canvas.clear();
            i = (i + 1) % 255;
            canvas.set_draw_color(pixels::Color::RGB(i, 64, 255 - i));
            let (w, h) = canvas.output_size().unwrap();
            let mut points = [rect::Point::new(0, 0); 256];

            'running: loop {
                for event in event_pump.poll_iter() {
                    match event {
                        event::Event::Quit { .. } => break 'running,
                        event::Event::KeyDown {
                            keycode: Some(keyboard::Keycode::Q),
                            repeat: false,
                            ..
                        } => break 'running,
                        event::Event::KeyDown { .. } => {
                            points.fill_with(|| {
                                rect::Point::new(
                                    rng.gen_range(0..w as i32),
                                    rng.gen_range(0..h as i32),
                                )
                            });
                            canvas.draw_points(points.as_slice()).unwrap();
                            canvas.present();
                        }
                        event::Event::KeyUp { .. } => {}
                        _ => {}
                    }
                }
            }
        }
    }

    #[test]
    #[ignore]
    fn test_open_display() {
        let mut vdp = vdp::Vdp::new();

        vdp.driver_open_display();
    }

    #[test]
    fn test_check_constants() {
        assert_eq!(vdp::Constants::NUMTILES, 896);
        assert_eq!(vdp::Constants::BLANKTIME, 17926);
        assert_eq!(vdp::Constants::VFRAMETIME, 47803);
    }
}

// set_colour
// get_colour
// setCycle
// pollInterupts

// _populateMode1Control
// _populateMode2Control
// setInterupt -> set_interrupt
// getNextInterupt -> get_next_interrupt
// openDisplay -> open_display
// printSpriteInformation -> print_sprite_information
// printNameTable -> print_name_table
