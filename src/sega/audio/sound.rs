use super::soundchannel;
use sdl2::audio;

pub type SoundQueueType = audio::AudioQueue<soundchannel::PlaybackType>;
pub struct SDLUtility {}

impl SDLUtility {
    // TODO: Fix up values, make them more dynamic, do better comparisons
    // Not sure how they compare on different PCs
    const TARGET_QUEUE_LENGTH: u32 = 8192; // This drives the 'delay' in audio, but too small for the speed and they aren't filled fast enough
    const AUDIO_SAMPLE_SIZE: u16 = 1024; // 'Desired' sample size, too small and SDL buffer doesn't stay filled (pops/crackles).
    const FRACTION_FILL: f32 = 0.05; // TODO: FUDGE FACTOR.  Don't completely fill, samples a removed 1 at a time, don't fill them immediately.

    const MONO_STERO_FLAG: u8 = 2; // TODO: Make this configurable 1 - mono, 2 - stereo

    pub fn get_audio_queue(sdl_context: &mut sdl2::Sdl) -> Option<Box<SoundQueueType>> {
        let audio_subsystem = sdl_context.audio().unwrap();

        let desired_spec = audio::AudioSpecDesired {
            freq: Some(Sound::SAMPLERATE as i32),
            channels: Some(SDLUtility::MONO_STERO_FLAG), // mono
            samples: Some(SDLUtility::AUDIO_SAMPLE_SIZE),
        };

        match audio_subsystem.open_queue::<soundchannel::PlaybackType, _>(None, &desired_spec) {
            Ok(audio_queue) => {
                audio_queue.clear();
                audio_queue.resume(); // Start the audio (nothing in the queue at this point).

                Some(Box::new(audio_queue))
            }
            Err(e) => {
                println!(
                    "Error while opening audio.  Setting audio queue to None/no audio. {}",
                    e
                );
                None
            }
        }
    }

    pub fn top_up_audio_queue<F>(audio_queue: &mut SoundQueueType, mut get_additional_buffer: F)
    where
        F: FnMut(u32) -> Vec<soundchannel::PlaybackType>,
    {
        assert!(audio_queue.size() <= SDLUtility::TARGET_QUEUE_LENGTH);
        let fill_size = ((SDLUtility::TARGET_QUEUE_LENGTH - audio_queue.size()) as f32
            * SDLUtility::FRACTION_FILL) as u32;
        // If 'stereo' the buffer is twice as large, so just as for half as much.
        let sound_buffer = get_additional_buffer(fill_size / (SDLUtility::MONO_STERO_FLAG as u32));
        audio_queue.queue_audio(&sound_buffer).unwrap();
    }
}

pub enum ChannelTypeEnum {
    ToneType,
    VolumeType,
}

impl From<u8> for ChannelTypeEnum {
    fn from(orig: u8) -> Self {
        match (orig >> 4) & 0x1 {
            0 => ChannelTypeEnum::ToneType,
            1 => ChannelTypeEnum::VolumeType,
            _ => panic!("Invalid sound channel type.  This case shouldn't be possible."),
        }
    }
}

pub enum ChannelEnum {
    Channel0,
    Channel1,
    Channel2,
    Channel3,
}

impl From<u8> for ChannelEnum {
    fn from(orig: u8) -> Self {
        match (orig >> 5) & 0x3 {
            0 => ChannelEnum::Channel0,
            1 => ChannelEnum::Channel1,
            2 => ChannelEnum::Channel2,
            3 => ChannelEnum::Channel3,
            _ => panic!("Invalid sound channel.  This case shouldn't be possible."),
        }
    }
}

#[derive(Default)]
pub struct LatchSoundReg {
    data: u8,
}

impl LatchSoundReg {
    const LATCH_DATA_MASK: u8 = 0xF;

    pub fn set_reg_value(&mut self, data: u8) {
        self.data = data;
    }

    pub fn get_channel(&self) -> ChannelEnum {
        ChannelEnum::from(self.data)
    }

    pub fn get_channel_type(&self) -> ChannelTypeEnum {
        ChannelTypeEnum::from(self.data)
    }

    pub fn get_data(&self) -> u8 {
        self.data & LatchSoundReg::LATCH_DATA_MASK
    }
}

pub struct Sound {
    channels: Vec<Box<dyn soundchannel::SoundGenerator>>,

    latched_reg: LatchSoundReg,
}

impl Sound {
    //    const SAMPLERATE:u32 = 32050;
    const SAMPLERATE: u32 = 44100;
    const CHANNELS: u8 = 4;
    const BITS: u8 = 8;
    pub fn new() -> Self {
        Self {
            channels: vec![
                Box::new(soundchannel::ToneSoundChannel::new()),
                Box::new(soundchannel::ToneSoundChannel::new()),
                Box::new(soundchannel::ToneSoundChannel::new()),
                Box::new(soundchannel::NoiseSoundChannel::new()),
            ],
            latched_reg: LatchSoundReg::default(),
        }
    }

    pub fn get_next_audio_chunk(&mut self, length: u32) -> Vec<soundchannel::PlaybackType> {
        let mut stream = Vec::with_capacity((2 * length) as usize);
        if length > 0 {
            for i in 0..(length * (SDLUtility::MONO_STERO_FLAG as u32)) {
                stream.push(0x0); // Neutral volume
            }

            for c in 0..Sound::CHANNELS {
                let channel_wave = self.channels[c as usize].get_wave(length, Sound::SAMPLERATE);

                if c % SDLUtility::MONO_STERO_FLAG == 0 {
                    for i in 0..length {
                        stream[(i * (SDLUtility::MONO_STERO_FLAG as u32)) as usize] +=
                            channel_wave[i as usize];
                    }
                } else {
                    // This will only be called if 'MONO_STEREO_FLAG' is set to '2'
                    for i in 0..length {
                        stream[(i * (SDLUtility::MONO_STERO_FLAG as u32) + 1) as usize] +=
                            channel_wave[i as usize];
                    }
                }
            }
        }

        stream
    }

    pub fn write_port(&mut self, data: u8) {
        // Dispatch the data to perform the specified audio function (frequency,
        // channel frequency, volume).

        if (data & 0x80) == 0x80 {
            // Set the 'latched' register information.
            self.latched_reg.set_reg_value(data);
        }

        match self.latched_reg.get_channel_type() {
            ChannelTypeEnum::VolumeType => {
                // set volume: 1rr1dddd
                // or          0-DDDDDD
                self.channels[self.latched_reg.get_channel() as usize]
                    .set_volume(data & soundchannel::SoundChannel::MAX_VOLUME_MASK);
            }
            ChannelTypeEnum::ToneType => {
                // The 'set tone' needs needs to handle if 'data' is data or latched.
                self.channels[self.latched_reg.get_channel() as usize]
                    .set_tone(self.latched_reg.get_data(), data);
            }
        }
    }
}
