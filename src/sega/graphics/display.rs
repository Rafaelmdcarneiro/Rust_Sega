use sdl2::event;
use sdl2::pixels;
use sdl2::render;
use sdl2::video;

// Splitting Console Size and windows size, as the console size if 'fixed',
// only window size changes/is scalable.
pub struct ConsoleSize {
    pub console_width: u16,
    pub console_height: u16,
}

impl ConsoleSize {
    pub fn new(console_width: u16, console_height: u16) -> Self {
        Self {
            console_width,
            console_height,
        }
    }
}

pub struct WindowSize {
    pub frame_width: u16,
    pub frame_height: u16,
    pub fullscreen: bool,
    pub console_size: ConsoleSize,
}

impl WindowSize {
    pub fn new(
        frame_width: u16,
        frame_height: u16,
        console_size: ConsoleSize,
        fullscreen: bool,
    ) -> Self {
        Self {
            frame_width,
            frame_height,
            fullscreen,
            console_size,
        }
    }
}

#[derive(Clone, Copy)]
pub struct Colour {
    // Simple RGB store and conversion at a per colour level.
    r: u8,
    g: u8,
    b: u8,
}

impl Colour {
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    pub fn convert_rgb444(&self, dst: &mut [u8]) {
        // RGB444
        dst[0] = (self.g & 0xF0) | (self.b >> 4);
        dst[1] = self.r >> 4;
    }

    pub fn convert_rgb24(&self, dst: &mut [u8]) {
        dst[0] = self.r;
        dst[1] = self.g;
        dst[2] = self.b;
    }

    pub fn convert_rgb888(&self, dst: &mut [u8]) {
        dst[0] = self.b;
        dst[1] = self.g;
        dst[2] = self.r;
    }
}

pub struct SDLUtility {}

impl SDLUtility {
    pub const PIXEL_FORMAT: pixels::PixelFormatEnum = pixels::PixelFormatEnum::RGB888;

    pub fn bytes_per_pixel() -> u16 {
        SDLUtility::PIXEL_FORMAT.byte_size_per_pixel() as u16
    }

    pub fn create_canvas(
        sdl_context: &mut sdl2::Sdl,
        name: &str,
        frame_width: u16,
        frame_height: u16,
        fullscreen: bool,
    ) -> Option<render::Canvas<video::Window>> {
        let video_subsystem = sdl_context.video().unwrap();
        let mut renderer = video_subsystem.window(name, frame_width as u32, frame_height as u32);

        // Just playing with if statement (to toggle full screen)
        let window = if fullscreen {
            renderer.fullscreen()
        } else {
            renderer.position_centered().resizable()
        };

        match window.build().map_err(|e| e.to_string()) {
            Ok(built_window) => {
                match built_window
                    .into_canvas()
                    .accelerated()
                    .build()
                    .map_err(|e| e.to_string())
                {
                    Ok(canvas) => Some(canvas),
                    Err(e) => {
                        println!(
                            "Error while building accelerated canvas, will leave canvas empty. {}",
                            e
                        );
                        None
                    }
                }
            }
            Err(e) => {
                println!(
                    "Error while building window, will leave canvas empty. {}",
                    e
                );
                None
            }
        }
    }

    pub fn texture_creator(
        canvas: &render::Canvas<video::Window>,
    ) -> render::TextureCreator<video::WindowContext> {
        canvas.texture_creator()
    }

    pub fn create_texture(
        texture_creator: &render::TextureCreator<video::WindowContext>,
        pixel_format: pixels::PixelFormatEnum,
        frame_width: u16,
        frame_height: u16,
    ) -> render::Texture {
        texture_creator
            .create_texture_streaming(pixel_format, frame_width as u32, frame_height as u32)
            .map_err(|e| e.to_string())
            .unwrap()
    }

    pub fn handle_events(event: &event::Event) {
        // Handle window events.
        if let event::Event::Window {
            win_event: event::WindowEvent::Resized(w, h),
            ..
        } = event
        {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sdl2::pixels;

    use sdl2::event;
    use sdl2::keyboard;
    use sdl2::rect;
    use sdl2::video;

    struct Colours {
        pub colour_lookup: Vec<Colour>,
    }

    impl Colours {
        // The colours palette is specified by a file with 128 lines.
        // R G B # Comment
        // The index in the display is then used with this as a lookup for the raw
        // RGB used for the video mode that's been set.
        const PALETTE_SIZE: u16 = 128;

        pub fn new() -> Self {
            Self {
                colour_lookup: Vec::new(),
            }
        }
    }

    pub struct DisplayGenerator {
        current_k: u16,
        pub pixel_format: pixels::PixelFormatEnum,
        pitch: u16,
    }

    impl DisplayGenerator {
        pub fn new(width: u16, height: u16, pixel_format: pixels::PixelFormatEnum) -> Self {
            let pitch = match pixel_format {
                SDLUtility::PIXEL_FORMAT => width * SDLUtility::bytes_per_pixel(),
                _ => 0,
            };
            Self {
                current_k: 0,
                pixel_format,
                pitch,
            }
        }

        pub fn update_display(&mut self, buffer: &mut [u8]) {
            // Clear the buffer
            buffer.iter_mut().for_each(|x| *x = 0);

            // Draw the display
            for i in 0..0xFF {
                for j in 0..0xFF {
                    let offset = (i + 100 + (self.current_k as usize % 200))
                        * (self.pitch as usize)
                        + (j + 100 + (self.current_k as usize % 200)) * 3_usize;
                    buffer[offset] = 0xFF * (self.current_k as usize & 0x0) as u8;
                    buffer[offset + 1] = j as u8;
                    buffer[offset + 2] = i as u8;
                }
            }
            self.current_k += 1;
        }

        pub fn new_generate_display(&mut self, buffer: &mut [u8], pitch: usize) {
            assert_eq!(self.pitch as usize, pitch);
            // Update the graphics.
            self.update_display(buffer);
        }

        pub fn get_generate_display_closure<'l>(&'l mut self) -> impl FnMut(&mut [u8], usize) + 'l {
            |buffer, pitch| self.new_generate_display(buffer, pitch)
        }
    }

    pub struct SDLDisplay {}

    impl SDLDisplay {
        pub fn new() -> Self {
            Self {}
        }

        pub fn draw_loop<'a, F: FnMut(&mut [u8], usize)>(
            &'a mut self,
            canvas: &mut render::Canvas<video::Window>,
            pixel_format: pixels::PixelFormatEnum,
            frame_width: u16,
            frame_height: u16,
            mut generate_display: F,
            iterations: u32,
        ) {
            // Creating the texture creator and texture is slow, so perform multiple display updates per creation.
            let texture_creator = SDLUtility::texture_creator(canvas);
            let mut texture = SDLUtility::create_texture(
                &texture_creator,
                pixel_format,
                frame_width,
                frame_height,
            );

            for _k in 0..iterations {
                texture
                    .with_lock(None, |buffer: &mut [u8], pitch: usize| {
                        generate_display(buffer, pitch)
                    })
                    .unwrap();

                canvas.clear();
                canvas
                    .copy(
                        &texture,
                        None,
                        Some(rect::Rect::new(
                            0,
                            0,
                            frame_width as u32,
                            frame_height as u32,
                        )),
                    )
                    .unwrap();
                canvas.present();
            }
        }

        // Main entry point, intention is to call 'once'.
        pub fn main_loop<'a>(
            &'a mut self,
            frame_width: u16,
            frame_height: u16,
            fullscreen: bool,
            generator: &mut DisplayGenerator,
        ) {
            let mut sdl_context = sdl2::init().unwrap();

            let mut canvas = SDLUtility::create_canvas(
                &mut sdl_context,
                "rust-sdl2 demo: Video",
                frame_width,
                frame_height,
                fullscreen,
            );

            canvas
                .expect("canvas is unexpectedly None")
                .info()
                .texture_formats
                .iter()
                .for_each(|x| match x {
                    pixels::PixelFormatEnum::Unknown => {
                        println!("Unknown");
                    }
                    pixels::PixelFormatEnum::Index1LSB => {
                        println!("Index1LSB");
                    }
                    pixels::PixelFormatEnum::Index1MSB => {
                        println!("Index1MSB");
                    }
                    pixels::PixelFormatEnum::Index4LSB => {
                        println!("Index4LSB");
                    }
                    pixels::PixelFormatEnum::Index4MSB => {
                        println!("Index4MSB");
                    }
                    pixels::PixelFormatEnum::Index8 => {
                        println!("Index8");
                    }
                    pixels::PixelFormatEnum::RGB332 => {
                        println!("RGB332");
                    }
                    pixels::PixelFormatEnum::RGB444 => {
                        println!("RGB444");
                    }
                    pixels::PixelFormatEnum::RGB555 => {
                        println!("RGB555");
                    }
                    pixels::PixelFormatEnum::BGR555 => {
                        println!("BGR555");
                    }
                    pixels::PixelFormatEnum::ARGB4444 => {
                        println!("ARGB4444");
                    }
                    pixels::PixelFormatEnum::RGBA4444 => {
                        println!("RGBA4444");
                    }
                    pixels::PixelFormatEnum::ABGR4444 => {
                        println!("ABGR4444");
                    }
                    pixels::PixelFormatEnum::BGRA4444 => {
                        println!("BGRA4444");
                    }
                    pixels::PixelFormatEnum::ARGB1555 => {
                        println!("ARGB1555");
                    }
                    pixels::PixelFormatEnum::RGBA5551 => {
                        println!("RGBA5551");
                    }
                    pixels::PixelFormatEnum::ABGR1555 => {
                        println!("ABGR1555");
                    }
                    pixels::PixelFormatEnum::BGRA5551 => {
                        println!("BGRA5551");
                    }
                    pixels::PixelFormatEnum::RGB565 => {
                        println!("RGB565");
                    }
                    pixels::PixelFormatEnum::BGR565 => {
                        println!("BGR565");
                    }
                    pixels::PixelFormatEnum::RGB24 => {
                        println!("RGB24");
                    }
                    pixels::PixelFormatEnum::BGR24 => {
                        println!("BGR24");
                    }
                    pixels::PixelFormatEnum::RGB888 => {
                        println!("RGB888");
                    }
                    pixels::PixelFormatEnum::RGBX8888 => {
                        println!("RGBX8888");
                    }
                    pixels::PixelFormatEnum::BGR888 => {
                        println!("BGR888");
                    }
                    pixels::PixelFormatEnum::BGRX8888 => {
                        println!("BGRX8888");
                    }
                    pixels::PixelFormatEnum::ARGB8888 => {
                        println!("ARGB8888");
                    }
                    pixels::PixelFormatEnum::RGBA8888 => {
                        println!("RGBA8888");
                    }
                    pixels::PixelFormatEnum::ABGR8888 => {
                        println!("ABGR8888");
                    }
                    pixels::PixelFormatEnum::BGRA8888 => {
                        println!("BGRA8888");
                    }
                    pixels::PixelFormatEnum::ARGB2101010 => {
                        println!("ARGB2101010");
                    }
                    pixels::PixelFormatEnum::YV12 => {
                        println!("YV12");
                    }
                    pixels::PixelFormatEnum::IYUV => {
                        println!("IYUV");
                    }
                    pixels::PixelFormatEnum::YUY2 => {
                        println!("YUY2");
                    }
                    pixels::PixelFormatEnum::UYVY => {
                        println!("UYVY");
                    }
                    pixels::PixelFormatEnum::YVYU => {
                        println!("YVYU");
                    }
                });

            let mut event_pump = sdl_context.event_pump().unwrap();

            'running: loop {
                for event in event_pump.poll_iter() {
                    match event {
                        event::Event::Quit { .. }
                        | event::Event::KeyDown {
                            keycode: Some(keyboard::Keycode::Escape),
                            ..
                        } => break 'running,
                        _ => {}
                    }
                }

                // First loop, draw 30 frames at a time.
                self.draw_loop(
                    &mut canvas.unwrap(),
                    generator.pixel_format,
                    frame_width,
                    frame_height,
                    generator.get_generate_display_closure(),
                    60,
                );
            }
        }
    }

    #[test]
    fn test_open_display() {
        const WINDOW_WIDTH: u16 = 800;
        const WINDOW_HEIGHT: u16 = 600; // MAX HEIGHT

        let mut display_generator =
            DisplayGenerator::new(WINDOW_WIDTH, WINDOW_HEIGHT, SDLUtility::PIXEL_FORMAT);

        let mut sdl_display = SDLDisplay::new();
        sdl_display.main_loop(WINDOW_WIDTH, WINDOW_HEIGHT, false, &mut display_generator);
    }
}
