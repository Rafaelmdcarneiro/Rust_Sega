use sdl2::pixels;
use sdl2::rect;
use sdl2::render;
use sdl2::video;

use super::audio::sound;
use super::clocks;
use super::cpu;
use super::graphics;
use super::inputs;
use super::interruptor;
use super::memory;
use super::ports;

pub struct Sega {
    core: cpu::core::Core<memory::memory::MemoryAbsolute>,
    debug: bool,
    realtime: bool,
    stop_clock: clocks::ClockType,
    fullscreen: bool,

    pub powered: bool,

    // These appear as 'Options' to simplify delayed initialisation.
    sdl_context: Option<sdl2::Sdl>,
    canvas: Option<render::Canvas<video::Window>>,
    audio_queue: Option<Box<sound::SoundQueueType>>,
}

impl Sega {
    const DISPLAY_UPDATES_PER_KEY_EVENT: u32 = 1; // Number of display updates per key press event. (reduces texture creation overhead).
    const CPU_STEPS_PER_AUDIO_UPDATE: u32 = 50; // Number of times to step the CPU before updating the audio.

    pub fn build_sega(cartridge_name: &str) -> cpu::core::Core<memory::memory::MemoryAbsolute> {
        let clock = clocks::Clock::new();
        let mut memory = memory::memory::MemoryAbsolute::new();
        let pc_state = cpu::pc_state::PcState::new();
        let vdp = graphics::vdp::Vdp::new();
        let mut ports = ports::Ports::new();
        let interruptor = interruptor::Interruptor::new();

        // Add the graphics device to the list of ports.
        // Joysticks are held directly, not as a 'device' (don't need to pass to ports).
        ports.add_device(Box::new(vdp));

        memory.reset(cartridge_name);

        cpu::core::Core::new(clock, memory, pc_state, ports, interruptor)
    }

    pub fn get_console_size() -> graphics::display::ConsoleSize {
        graphics::display::ConsoleSize::new(
            graphics::vdp::Constants::SMS_WIDTH,
            graphics::vdp::Constants::SMS_HEIGHT,
        )
    }

    pub fn reset(&mut self, cartridge_name: &str) {
        self.core.memory.reset(cartridge_name);
        self.core.reset();
    }

    pub fn run_sega(me: &mut Sega) -> bool {
        let console_size = Self::get_console_size();

        let pixel_format = graphics::display::SDLUtility::PIXEL_FORMAT;
        let mut event_pump = me
            .sdl_context
            .as_mut()
            .expect("Should be here")
            .event_pump()
            .unwrap();

        for event in event_pump.poll_iter() {
            graphics::display::SDLUtility::handle_events(&event);

            if !inputs::Input::handle_events(event, &mut me.core.ports.joysticks) {
                return false;
            };
        }

        // First loop, draw FRAMES_PER_KEY_EVENT frames at a time.
        if !me.draw_loop(
            pixel_format,
            &console_size,
            Sega::DISPLAY_UPDATES_PER_KEY_EVENT,
        ) {
            return false;
        }
        true
    }

    pub fn power_sega(&mut self) {
        let mut frame_width = graphics::vdp::Constants::SMS_WIDTH;
        // If not in full screen, default to using a bigger window.
        if !self.fullscreen {
            frame_width *= 3;
        }
        let frame_height = ((frame_width as u32) * (graphics::vdp::Constants::SMS_HEIGHT as u32)
            / (graphics::vdp::Constants::SMS_WIDTH as u32)) as u16;

        println!("powering on Sega Emulator.");
        inputs::Input::print_keys();

        let console_size = Self::get_console_size();
        let window_size = graphics::display::WindowSize::new(
            frame_width,
            frame_height,
            console_size,
            self.fullscreen,
        );

        self.configure_sdl(window_size, graphics::display::SDLUtility::PIXEL_FORMAT);
        self.powered = true;
    }

    pub fn new(
        debug: bool,
        realtime: bool,
        stop_clock: clocks::ClockType,
        cartridge_name: &str,
        fullscreen: bool,
    ) -> Self {
        let core = Self::build_sega(cartridge_name);
        Self {
            core,
            debug,
            realtime,
            stop_clock,
            fullscreen,
            powered: false,
            sdl_context: None,
            canvas: None,
            audio_queue: None,
        }
    }

    pub fn draw_loop(
        &mut self,
        pixel_format: pixels::PixelFormatEnum,
        console_size: &graphics::display::ConsoleSize,
        iterations: u32,
    ) -> bool {
        // Number of iterations to do before getting a new texture.
        // These loops will update the display, but currently events aren't checked in this time.

        if self.canvas.is_some() {
            let canvas = self.canvas.as_mut().expect("Optional canvas not set");

            // Creating the texture creator and texture is slow, so perform multiple display updates per creation.
            let texture_creator = graphics::display::SDLUtility::texture_creator(canvas);
            let mut texture;
            texture = graphics::display::SDLUtility::create_texture(
                &texture_creator,
                pixel_format,
                console_size.console_width,
                console_size.console_height,
            );

            let mut audio_steps = 0;
            let mut display_refreshes = 0;
            while display_refreshes < iterations {
                if self.stop_clock > 0 && self.core.clock.cycles > self.stop_clock {
                    return false;
                }
                self.core.step(self.debug, self.realtime);

                if 0 == audio_steps % Sega::CPU_STEPS_PER_AUDIO_UPDATE {
                    // Top-up the audio queue
                    let audio_queue = self.audio_queue.as_mut().expect("Optional audio not set");
                    sound::SDLUtility::top_up_audio_queue(audio_queue, |fill_size| {
                        self.core.ports.audio.get_next_audio_chunk(fill_size)
                    });
                }
                audio_steps += 1;

                // If an 'export' occurred (buffer was draw), then update the texture.
                if self.core.export() {
                    texture
                        .with_lock(None, |buffer: &mut [u8], _pitch: usize| {
                            self.core.generate_display(buffer)
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
                                console_size.console_width as u32,
                                console_size.console_height as u32,
                            )),
                        )
                        .unwrap();
                    canvas.present();

                    display_refreshes += 1;
                }
            }
            true
        } else {
            let mut audio_steps = 0;
            let mut display_refreshes = 0;
            while display_refreshes < iterations {
                if self.stop_clock > 0 && self.core.clock.cycles > self.stop_clock {
                    return false;
                }
                self.core.step(self.debug, self.realtime);

                if 0 == audio_steps % Sega::CPU_STEPS_PER_AUDIO_UPDATE {
                    // Top-up the audio queue
                    let audio_queue = self.audio_queue.as_mut().expect("Optional audio not set");
                    sound::SDLUtility::top_up_audio_queue(audio_queue, |fill_size| {
                        self.core.ports.audio.get_next_audio_chunk(fill_size)
                    });
                }
                audio_steps += 1;

                display_refreshes += 1;
            }
            true
        }
    }

    pub fn configure_sdl(
        &mut self,
        window_size: graphics::display::WindowSize,
        pixel_format: pixels::PixelFormatEnum,
    ) {
        let mut sdl_context = sdl2::init().unwrap();

        self.canvas = graphics::display::SDLUtility::create_canvas(
            &mut sdl_context,
            "rust-sega emulator",
            window_size.frame_width,
            window_size.frame_height,
            window_size.fullscreen,
        );

        if let Some(ref mut v) = self.canvas {
            v.set_logical_size(
                window_size.console_size.console_width as u32,
                window_size.console_size.console_height as u32,
            )
            .unwrap();
        }

        self.audio_queue = sound::SDLUtility::get_audio_queue(&mut sdl_context);
        self.sdl_context = Some(sdl_context);
    }
}
