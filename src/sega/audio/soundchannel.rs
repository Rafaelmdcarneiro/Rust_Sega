pub trait SoundGenerator {
    // Data may be from latched or data
    fn set_volume(&mut self, data: u8);
    // Data is from both 'latched' and 'data', may represent noise or tone (depending on channel).
    // It's only updated on a 'data' write.
    fn set_tone(&mut self, latched_data: u8, data: u8);
    fn get_wave(&mut self, length: u32, sample_rate: u32) -> Vec<PlaybackType>;
}

pub struct SoundChannel {}

pub type PlaybackType = u8; // Only 8-bit playback is currently supported.
impl SoundChannel {
    const FREQMULTIPLIER: u32 = 125000;

    pub const MAX_VOLUME_MASK: u8 = 0xF;
    pub const MAX_VOLUME: PlaybackType = 0xFF;

    const NUM_CHANNELS: u8 = 4;

    pub fn get_hertz(frequency: u16) -> u32 {
        SoundChannel::FREQMULTIPLIER / (frequency as u32 + 1)
    }

    pub fn get_volume(volume_reg: u8) -> PlaybackType {
        // Min volume when volume_reg = 0xF
        // Max volume when volume_reg = 0x0
        ((SoundChannel::MAX_VOLUME_MASK ^ volume_reg) << 4) / SoundChannel::NUM_CHANNELS
    }
}

pub struct ToneSoundChannel {
    freq_reg: u16, // DDDDDDddd/(---trr)-trr)
    volume_reg: u8,
    current_level: bool,    // Square wave is 0 or 1
    frequency_counter: u32, // counter remaining before toggle (counts up, as audio isn't at 125000Hz, it can have a remainder).
}

pub struct NoiseSoundChannel {
    noise_period_select: bool,
    noise_shift_register: u16,
    freq_reg: u16,
    volume_reg: u8,
    frequency_counter: u32, // counter remaining before toggle (counts up, as audio isn't at 125000Hz, it can have a remainder).
}

impl ToneSoundChannel {
    const LATCHED_UPPER_FREQ_MASK: u8 = 0x3F;
    const LATCHED_LOWER_FREQ_MASK: u8 = 0x0F;

    pub fn new() -> Self {
        Self {
            freq_reg: 0,
            volume_reg: SoundChannel::MAX_VOLUME_MASK, // Initialise as 'silent'

            current_level: false, // Square wave is 0 or 1
            frequency_counter: 0, // counter remaining before toggle.
        }
    }
    pub fn get_wave(&mut self, length: u32, sample_rate: u32) -> Vec<PlaybackType> {
        // Generate the 'wave' output buffer.
        // First copy what's left of the current 'play buffer', update to the
        // new buffer, if it's changed and copy that until the wave buffer has
        // been fully populated.

        let mut wave = Vec::with_capacity(length as usize);

        for i in 0..length {
            // If the counter has reached zero, then toggle the level.
            if self.frequency_counter > sample_rate {
                self.current_level = !self.current_level;
                self.frequency_counter %= sample_rate;
            } else {
                self.frequency_counter += SoundChannel::get_hertz(self.freq_reg) * 2;
            }
            let volume = if self.current_level {
                SoundChannel::get_volume(self.volume_reg)
            } else {
                0x0
            };
            wave.push(volume);
        }

        wave
    }
}

impl NoiseSoundChannel {
    const NOISE_SHIFT_REGISTER_RESET: u16 = 0x4000;

    pub fn new() -> Self {
        Self {
            noise_period_select: false,
            noise_shift_register: NoiseSoundChannel::NOISE_SHIFT_REGISTER_RESET,
            freq_reg: 0,
            volume_reg: SoundChannel::MAX_VOLUME_MASK, // Initialise as 'silent'
            frequency_counter: 0,
        }
    }

    fn set_data(&mut self, data: u8) {
        if (data & 0x3) < 3 {
            self.freq_reg = match data & 0x3 {
                0 => 0x10,
                1 => 0x20,
                2 => 0x40,
                _ => {
                    panic!("Match for noise frequency not possible");
                }
            };

            // TODO: Not sure how the 'periodic' should sound.
            // Superficially, it sounds better if noise is forced to 'true'
            self.noise_period_select = 0x1 == (data >> 2) & 0x1; // If (---trr) -> t = 1 -> white noise

            // Reset the noise shift register:
            self.noise_shift_register = 0; // Clear the register (will be set on first get).
        } else {
            // TODO: Figure out what 'Tone 3' means
            self.freq_reg = 0;
            println!("Tone 3 not supported for noise channel");
        }
    }

    // Outputs  '1' or '0' on each 'clock'
    pub fn get_shiff_register_output(&mut self, noise: bool, sample_rate: u32) -> PlaybackType {
        let output = self.noise_shift_register & 0x1;

        if 0 == self.noise_shift_register {
            self.noise_shift_register = 0x8000;
        }

        // Shift the register
        self.frequency_counter += SoundChannel::get_hertz(self.freq_reg) * 32; // TODO: fudge factor, not sure if this is correct.
        if self.frequency_counter >= sample_rate {
            self.frequency_counter %= sample_rate;

            if noise {
                let feed_back = (self.noise_shift_register >> 3) & 0x1;
                self.noise_shift_register =
                    (self.noise_shift_register >> 1) | ((output ^ feed_back) << 15);
            } else {
                self.noise_shift_register = (self.noise_shift_register >> 1) | (output << 15);
            }
        }

        if output == 0x1 {
            0
        } else {
            SoundChannel::get_volume(self.volume_reg)
        }
    }
}

impl SoundGenerator for ToneSoundChannel {
    // Data may be from latched or data
    fn set_volume(&mut self, data: u8) {
        self.volume_reg = data & SoundChannel::MAX_VOLUME_MASK;
    }

    // Data is from both 'latched' and 'data', may represent noise or tone (depending on channel).
    // It's only updated on a 'data' write.
    fn set_tone(&mut self, latched_data: u8, data: u8) {
        if (data & 0x80) == 0x00 {
            // Only update the frequencey if the 'data' is non-latch data.
            self.freq_reg = (((data & ToneSoundChannel::LATCHED_UPPER_FREQ_MASK) as u16) << 4)
                | (latched_data & ToneSoundChannel::LATCHED_LOWER_FREQ_MASK) as u16;
        }
    }

    fn get_wave(&mut self, length: u32, sample_rate: u32) -> Vec<PlaybackType> {
        self.get_wave(length, sample_rate)
    }
}

impl SoundGenerator for NoiseSoundChannel {
    fn set_volume(&mut self, data: u8) {
        self.volume_reg = data & SoundChannel::MAX_VOLUME_MASK;
    }

    // Data is from both 'latched' and 'data', may represent noise or tone (depending on channel).
    // It's only updated on a 'data' write.
    fn set_tone(&mut self, latched_data: u8, data: u8) {
        self.set_data(data);
    }

    fn get_wave(&mut self, length: u32, sample_rate: u32) -> Vec<PlaybackType> {
        let mut channel_wave = Vec::<PlaybackType>::new();
        for i in 0..length {
            channel_wave
                .push(self.get_shiff_register_output(self.noise_period_select, sample_rate));
        }

        channel_wave
    }
}
