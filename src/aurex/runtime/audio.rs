use super::event::{AudioSfx, RuntimeAudioCommand};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum AudioMode {
    Boot,
    Game,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MixProfile {
    Soft,
    Default,
    Arcade,
    Boot,
}

impl MixProfile {
    fn render_gain_q8(self) -> i32 {
        match self {
            Self::Soft => 80,
            Self::Default => 88,
            Self::Arcade => 96,
            Self::Boot => 88,
        }
    }

    fn lp_smoothing(self) -> i32 {
        match self {
            Self::Soft => 5,
            Self::Default => 4,
            Self::Arcade => 3,
            Self::Boot => 6,
        }
    }

    fn hp_decay_q6(self) -> i32 {
        match self {
            Self::Soft => 61,
            Self::Default => 63,
            Self::Arcade => 64,
            Self::Boot => 62,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Soft => "soft",
            Self::Default => "default",
            Self::Arcade => "arcade",
            Self::Boot => "boot",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "soft" => Some(Self::Soft),
            "default" => Some(Self::Default),
            "arcade" => Some(Self::Arcade),
            "boot" => Some(Self::Boot),
            _ => None,
        }
    }
}

const SAMPLE_RATE_HZ: u32 = 48_000;
const AUDIO_RAM_BYTES: usize = 512 * 1024;
const VOICE_COUNT: usize = 24;
const WAVE_SIZE: usize = 512;
const PCM_SAMPLE_RAM_BYTES: usize = 128 * 1024;
const PCM_CHANNEL_COUNT: usize = 16;
const PCM_SLOT_BYTES: usize = 8192;
const WAVETABLE_BYTES: usize = 5 * WAVE_SIZE * 2;
const MIX_SHIFT: i32 = 10;
const TICK_HZ: u32 = 120;
const BOOT_TICK_HZ: u32 = 4;
const PATTERN_STEPS: usize = 16;

const TRACK_BPM: [u16; 6] = [140, 130, 120, 100, 122, 136];
const MASTER_LIMIT: i32 = 28_000;

const WAVE_SINE: usize = 0;
const WAVE_SQUARE: usize = 1;
const WAVE_TRIANGLE: usize = 2;
const WAVE_SAW: usize = 3;
const WAVE_NOISE: usize = 4;

#[derive(Clone, Copy)]
struct Instrument {
    waveform_id: u8,
    attack: u16,
    decay: u16,
    sustain: u16,
    release: u16,
    vibrato_depth: u8,
    vibrato_speed: u8,
}

const INSTRUMENTS: [Instrument; 8] = [
    Instrument {
        waveform_id: WAVE_SQUARE as u8,
        attack: 18,
        decay: 48,
        sustain: 720,
        release: 64,
        vibrato_depth: 2,
        vibrato_speed: 3,
    },
    Instrument {
        waveform_id: WAVE_SAW as u8,
        attack: 8,
        decay: 36,
        sustain: 600,
        release: 52,
        vibrato_depth: 1,
        vibrato_speed: 2,
    },
    Instrument {
        waveform_id: WAVE_TRIANGLE as u8,
        attack: 4,
        decay: 20,
        sustain: 820,
        release: 44,
        vibrato_depth: 0,
        vibrato_speed: 1,
    },
    Instrument {
        waveform_id: WAVE_SINE as u8,
        attack: 12,
        decay: 44,
        sustain: 560,
        release: 64,
        vibrato_depth: 4,
        vibrato_speed: 4,
    },
    Instrument {
        waveform_id: WAVE_NOISE as u8,
        attack: 1,
        decay: 8,
        sustain: 300,
        release: 20,
        vibrato_depth: 0,
        vibrato_speed: 0,
    },
    Instrument {
        waveform_id: WAVE_SQUARE as u8,
        attack: 2,
        decay: 10,
        sustain: 900,
        release: 14,
        vibrato_depth: 0,
        vibrato_speed: 0,
    },
    Instrument {
        waveform_id: WAVE_TRIANGLE as u8,
        attack: 4,
        decay: 28,
        sustain: 850,
        release: 60,
        vibrato_depth: 0,
        vibrato_speed: 0,
    },
    Instrument {
        waveform_id: WAVE_SINE as u8,
        attack: 6,
        decay: 24,
        sustain: 720,
        release: 52,
        vibrato_depth: 1,
        vibrato_speed: 2,
    },
];

#[derive(Clone, Copy, Default)]
struct PcmChannel {
    position: usize,
    length: usize,
    sample_base: usize,
    volume: u16,
    playing: bool,
}

#[derive(Clone, Copy)]
enum EnvelopeState {
    Off,
    Attack,
    Decay,
    Sustain,
    Release,
}

#[derive(Clone, Copy)]
struct Voice {
    waveform_id: u8,
    instrument_id: u8,
    phase: u32,
    pitch: u16,
    volume: u16,
    pan_l: u16,
    pan_r: u16,
    envelope_state: EnvelopeState,
    env_level: u16,
    env_counter: u16,
    vibrato_phase: u8,
    fx: u8,
    delay_line: [i16; 32],
    delay_index: usize,
    lp_state: i32,
    prev_env_gain: u16,
}

impl Voice {
    const fn silent() -> Self {
        Self {
            waveform_id: 0,
            instrument_id: 0,
            phase: 0,
            pitch: 0,
            volume: 0,
            pan_l: 512,
            pan_r: 512,
            envelope_state: EnvelopeState::Off,
            env_level: 0,
            env_counter: 0,
            vibrato_phase: 0,
            fx: 0,
            delay_line: [0; 32],
            delay_index: 0,
            lp_state: 0,
            prev_env_gain: 0,
        }
    }
}

const TRACK0_MELODY: [u16; PATTERN_STEPS] = [
    494, 370, 330, 247, 494, 370, 330, 247, 494, 370, 494, 370, 330, 247, 330, 247,
];
const TRACK0_BASS: [u16; PATTERN_STEPS] = [
    123, 92, 82, 62, 123, 92, 82, 62, 123, 92, 123, 92, 82, 62, 82, 62,
];
const TRACK0_ARP: [u16; PATTERN_STEPS] = [
    494, 370, 330, 247, 494, 370, 330, 0, 370, 494, 370, 494, 330, 247, 494, 370,
];

const TRACK1_MELODY: [u16; PATTERN_STEPS] = [
    233, 311, 349, 466, 233, 311, 349, 466, 349, 311, 233, 0, 466, 349, 311, 233,
];
const TRACK1_BASS: [u16; PATTERN_STEPS] = [
    116, 155, 175, 233, 116, 155, 175, 233, 175, 155, 116, 0, 233, 175, 155, 116,
];
const TRACK1_ARP: [u16; PATTERN_STEPS] = [
    466, 349, 311, 233, 466, 349, 311, 233, 349, 466, 349, 311, 233, 349, 466, 349,
];

const TRACK2_MELODY: [u16; PATTERN_STEPS] = [
    262, 330, 392, 523, 392, 330, 262, 0, 294, 349, 392, 523, 392, 349, 294, 0,
];
const TRACK2_BASS: [u16; PATTERN_STEPS] = [
    131, 165, 196, 262, 196, 165, 131, 0, 147, 175, 196, 262, 196, 175, 147, 0,
];
const TRACK2_ARP: [u16; PATTERN_STEPS] = [
    523, 392, 330, 262, 392, 330, 262, 196, 330, 392, 523, 392, 330, 262, 392, 330,
];

const TRACK3_MELODY: [u16; PATTERN_STEPS] = [
    587, 440, 349, 294, 523, 392, 311, 262, 440, 349, 294, 262, 392, 311, 262, 0,
];
const TRACK3_BASS: [u16; PATTERN_STEPS] = [
    73, 110, 87, 73, 65, 98, 87, 65, 55, 87, 73, 65, 98, 87, 65, 0,
];
const TRACK3_ARP: [u16; PATTERN_STEPS] = [
    294, 349, 440, 587, 349, 262, 311, 392, 440, 349, 294, 262, 392, 311, 262, 220,
];

const TRACK4_MELODY: [u16; PATTERN_STEPS] = [
    165, 110, 123, 82, 165, 110, 123, 82, 110, 165, 110, 82, 123, 165, 123, 82,
];
const TRACK4_BASS: [u16; PATTERN_STEPS] = [
    82, 55, 62, 41, 82, 55, 62, 41, 55, 82, 55, 41, 62, 82, 62, 41,
];
const TRACK4_ARP: [u16; PATTERN_STEPS] = [
    165, 220, 247, 165, 110, 165, 220, 165, 123, 165, 110, 82, 247, 165, 220, 165,
];

const TRACK5_MELODY: [u16; PATTERN_STEPS] = [
    415, 494, 554, 659, 554, 494, 415, 0, 494, 554, 659, 831, 659, 554, 494, 415,
];
const TRACK5_BASS: [u16; PATTERN_STEPS] = [
    104, 123, 139, 165, 139, 123, 104, 0, 123, 139, 165, 208, 165, 139, 123, 104,
];
const TRACK5_ARP: [u16; PATTERN_STEPS] = [
    415, 554, 659, 831, 554, 415, 494, 659, 831, 554, 415, 494, 659, 554, 494, 415,
];

const TRACK0_PERC: [[u16; PATTERN_STEPS]; 4] = [
    [82, 0, 82, 0, 82, 0, 82, 0, 82, 0, 82, 0, 82, 0, 82, 0],
    [0, 0, 0, 0, 350, 0, 0, 0, 0, 0, 0, 0, 350, 0, 0, 0],
    [
        4000, 0, 4000, 0, 4000, 0, 4000, 0, 4000, 0, 4000, 0, 4000, 0, 4000, 0,
    ],
    [
        0, 3000, 0, 3000, 0, 3000, 0, 3000, 0, 3000, 0, 3000, 0, 3000, 0, 3000,
    ],
];
const TRACK4_PERC: [[u16; PATTERN_STEPS]; 4] = [
    [82, 82, 0, 82, 0, 82, 82, 0, 82, 0, 82, 82, 0, 82, 0, 82],
    [0, 0, 350, 0, 0, 0, 0, 350, 0, 0, 350, 0, 0, 0, 0, 350],
    [
        4000, 0, 4000, 0, 4000, 0, 4000, 0, 4000, 0, 4000, 0, 4000, 0, 4000, 0,
    ],
    [0; PATTERN_STEPS],
];
const TRACK5_PERC: [[u16; PATTERN_STEPS]; 4] = [
    [82, 0, 0, 0, 82, 0, 0, 0, 82, 0, 0, 0, 82, 0, 0, 0],
    [0, 0, 0, 0, 350, 0, 0, 0, 0, 0, 0, 0, 350, 0, 0, 0],
    [4000; PATTERN_STEPS],
    [0; PATTERN_STEPS],
];

const TRACK_INST: [[u8; 3]; 6] = [
    [0, 1, 2],
    [1, 1, 2],
    [0, 0, 2],
    [3, 3, 3],
    [5, 1, 2],
    [0, 1, 3],
];
const PERC_INST: [u8; 4] = [4, 4, 5, 5];
const TRACK_FX: [u8; 6] = [0x01, 0x02, 0x00, 0x04, 0x12, 0x03];

const BOOT_MELODY: [u16; PATTERN_STEPS] = [
    147, 147, 147, 147, 196, 196, 196, 196, 262, 262, 294, 349, 392, 392, 349, 0,
];

#[derive(Debug, Clone, Copy)]
pub struct AudioDiagnostics {
    pub frames: usize,
    pub peak_l: i16,
    pub peak_r: i16,
    pub avg_abs_l: i16,
    pub avg_abs_r: i16,
    pub crest_l_q10: u16,
    pub crest_r_q10: u16,
    pub clipped_l: u32,
    pub clipped_r: u32,
    pub boot_beat_step: u8,
}

#[derive(Debug, Clone, Copy)]
pub struct AudioDiagnosticsBaseline {
    pub sample_rate: u32,
    pub frames: usize,
    pub boot: AudioDiagnostics,
    pub game: AudioDiagnostics,
}

impl AudioDiagnosticsBaseline {
    pub fn to_json(&self) -> String {
        format!(
            "{{\"sample_rate\":{},\"frames\":{},\"boot\":{},\"game\":{}}}",
            self.sample_rate,
            self.frames,
            self.boot.to_json(),
            self.game.to_json()
        )
    }
}

impl AudioDiagnostics {
    pub fn to_json(&self) -> String {
        format!(
            "{{\"frames\":{},\"peak_l\":{},\"peak_r\":{},\"avg_abs_l\":{},\"avg_abs_r\":{},\"crest_l_q10\":{},\"crest_r_q10\":{},\"clipped_l\":{},\"clipped_r\":{},\"boot_beat_step\":{}}}",
            self.frames,
            self.peak_l,
            self.peak_r,
            self.avg_abs_l,
            self.avg_abs_r,
            self.crest_l_q10,
            self.crest_r_q10,
            self.clipped_l,
            self.clipped_r,
            self.boot_beat_step
        )
    }
}

pub struct AudioEngine {
    sample_clock: u64,
    sample_rate: u32,
    tick_counter: u32,
    pattern_step: usize,
    track_id: u8,
    voices: [Voice; VOICE_COUNT],
    pcm_channels: [PcmChannel; PCM_CHANNEL_COUNT],
    audio_ram: Box<[u8; AUDIO_RAM_BYTES]>,
    wavetable_base: [usize; 5],
    sfx_play_samples: u32,
    sfx_kind: AudioSfx,
    noise_state: u32,
    mix_lp_l: i32,
    mix_lp_r: i32,
    mix_hp_l: i32,
    mix_hp_r: i32,
    prev_mix_l: i32,
    prev_mix_r: i32,
    mix_profile: MixProfile,
}

impl AudioEngine {
    pub fn new(sample_rate: u32) -> Self {
        Self::new_with_profile(sample_rate, MixProfile::Default)
    }

    pub fn new_with_profile(sample_rate: u32, mix_profile: MixProfile) -> Self {
        let mut audio_ram = Box::new([0u8; AUDIO_RAM_BYTES]);
        let wavetable_base = [
            0,
            WAVE_SIZE * 2,
            WAVE_SIZE * 4,
            WAVE_SIZE * 6,
            WAVE_SIZE * 8,
        ];
        Self::write_wavetables(&mut audio_ram, &wavetable_base);
        Self {
            sample_clock: 0,
            sample_rate,
            tick_counter: 0,
            pattern_step: 0,
            track_id: 0,
            voices: [Voice::silent(); VOICE_COUNT],
            pcm_channels: [PcmChannel::default(); PCM_CHANNEL_COUNT],
            audio_ram,
            wavetable_base,
            sfx_play_samples: 0,
            sfx_kind: AudioSfx::None,
            noise_state: 0xC001_FEED,
            mix_lp_l: 0,
            mix_lp_r: 0,
            mix_hp_l: 0,
            mix_hp_r: 0,
            prev_mix_l: 0,
            prev_mix_r: 0,
            mix_profile,
        }
    }

    fn write_wavetables(ram: &mut [u8; AUDIO_RAM_BYTES], base: &[usize; 5]) {
        for i in 0..WAVE_SIZE {
            let phase = i as i32;
            let half = WAVE_SIZE as i32 / 2;
            let step = 65534 / half.max(1);
            let tri = if i < half as usize {
                -32767 + phase * step
            } else {
                32767 - ((phase - half) * step)
            };
            let saw = -32767 + phase * (65534 / (WAVE_SIZE as i32 - 1).max(1));
            let square = if i < 128 { 28000 } else { -28000 };
            let sine = Self::sine_from_phase(i as u16) as i32;
            let noise_seed = (i as u32).wrapping_mul(1103515245).wrapping_add(12345);
            let noise = ((noise_seed >> 16) as i16) as i32;
            let waves = [sine, square, tri, saw, noise];
            for wave_id in 0..5 {
                let idx = base[wave_id] + i * 2;
                let s = waves[wave_id] as i16;
                ram[idx] = (s as u16 & 0xFF) as u8;
                ram[idx + 1] = ((s as u16 >> 8) & 0xFF) as u8;
            }
        }
    }

    pub fn pattern_step(&self) -> usize {
        self.pattern_step
    }

    pub fn boot_beat_step(&self) -> u8 {
        (self.pattern_step % PATTERN_STEPS) as u8
    }

    pub fn audio_ram_mut(&mut self) -> &mut [u8] {
        &mut self.audio_ram[..]
    }

    pub fn trigger_command(&mut self, cmd: RuntimeAudioCommand) {
        match cmd {
            RuntimeAudioCommand::PlayTrack(track_id) => {
                self.track_id = track_id % TRACK_BPM.len() as u8;
                self.pattern_step = PATTERN_STEPS.wrapping_sub(1);
                let bpm = TRACK_BPM[(self.track_id as usize) % TRACK_BPM.len()].max(1) as u32;
                self.tick_counter = (self.sample_rate * 15 / bpm).max(1);
            }
            RuntimeAudioCommand::PlaySfx(sfx) => {
                self.sfx_kind = sfx;
                self.sfx_play_samples = match sfx {
                    AudioSfx::Launch => self.sample_rate / 3,
                    AudioSfx::Cancel => self.sample_rate / 8,
                    AudioSfx::Confirm => self.sample_rate / 10,
                    AudioSfx::BootChime => self.sample_rate / 12,
                    AudioSfx::PlusChime => self.sample_rate / 8,
                    AudioSfx::None => 0,
                };
            }
            RuntimeAudioCommand::PlayPcm {
                channel,
                sample_id,
                volume,
            } => {
                let ch = (channel as usize) % PCM_CHANNEL_COUNT;
                let slot = (sample_id as usize) % 16;
                self.pcm_channels[ch].sample_base = WAVETABLE_BYTES + slot * PCM_SLOT_BYTES;
                self.pcm_channels[ch].length = PCM_SLOT_BYTES / 2;
                self.pcm_channels[ch].position = 0;
                self.pcm_channels[ch].volume = volume.min(1024);
                self.pcm_channels[ch].playing = true;
            }
            RuntimeAudioCommand::StopTrack => {
                for voice in &mut self.voices {
                    voice.envelope_state = EnvelopeState::Release;
                }
            }
        }
    }

    pub fn diagnostics_for_frames(&self, mode: AudioMode, frames: usize) -> AudioDiagnostics {
        let mut sim =
            Self::new_with_profile(self.sample_rate.max(SAMPLE_RATE_HZ), self.mix_profile);
        sim.track_id = self.track_id;
        let mut peak_l = 0i32;
        let mut peak_r = 0i32;
        let mut abs_sum_l: i64 = 0;
        let mut abs_sum_r: i64 = 0;
        let mut clipped_l: u32 = 0;
        let mut clipped_r: u32 = 0;
        let mut block = [0i16; 512];
        let mut remain = frames;
        while remain > 0 {
            let step = remain.min(block.len() / 2);
            let slice_len = step * 2;
            sim.render_block(mode, &mut block[..slice_len]);
            for i in 0..step {
                let l = block[i * 2] as i32;
                let r = block[i * 2 + 1] as i32;
                peak_l = peak_l.max(l.abs());
                peak_r = peak_r.max(r.abs());
                abs_sum_l += l.abs() as i64;
                abs_sum_r += r.abs() as i64;
                if l.abs() >= 32_000 {
                    clipped_l = clipped_l.saturating_add(1);
                }
                if r.abs() >= 32_000 {
                    clipped_r = clipped_r.saturating_add(1);
                }
            }
            remain -= step;
        }
        let denom = frames.max(1) as i64;
        let avg_abs_l = (abs_sum_l / denom).clamp(i16::MIN as i64, i16::MAX as i64) as i16;
        let avg_abs_r = (abs_sum_r / denom).clamp(i16::MIN as i64, i16::MAX as i64) as i16;
        let crest_l_q10 = ((peak_l.max(1) * 1024) / (avg_abs_l.abs().max(1) as i32))
            .clamp(0, u16::MAX as i32) as u16;
        let crest_r_q10 = ((peak_r.max(1) * 1024) / (avg_abs_r.abs().max(1) as i32))
            .clamp(0, u16::MAX as i32) as u16;
        AudioDiagnostics {
            frames,
            peak_l: peak_l.clamp(i16::MIN as i32, i16::MAX as i32) as i16,
            peak_r: peak_r.clamp(i16::MIN as i32, i16::MAX as i32) as i16,
            avg_abs_l,
            avg_abs_r,
            crest_l_q10,
            crest_r_q10,
            clipped_l,
            clipped_r,
            boot_beat_step: sim.boot_beat_step(),
        }
    }

    pub fn render_block(&mut self, mode: AudioMode, out: &mut [i16]) {
        debug_assert!(self.sample_rate == SAMPLE_RATE_HZ);
        let frames = out.len() / 2;
        for frame in 0..frames {
            self.advance_sequencer(mode);
            let (mut mix_l, mut mix_r) = (0i32, 0i32);
            for i in 0..VOICE_COUNT {
                let (l, r) = self.sample_voice(i);
                mix_l += l;
                mix_r += r;
            }
            let (sfx_l, sfx_r) = self.sfx_sample();
            mix_l += sfx_l;
            mix_r += sfx_r;
            let (pcm_l, pcm_r) = self.sample_pcm();
            mix_l += pcm_l;
            mix_r += pcm_r;

            let profile = if matches!(mode, AudioMode::Boot) {
                MixProfile::Boot
            } else {
                self.mix_profile
            };
            let gain = profile.render_gain_q8();
            mix_l = (mix_l * gain) / 256;
            mix_r = (mix_r * gain) / 256;

            let (out_l, out_r) = if matches!(mode, AudioMode::Boot) {
                (Self::soft_clip(mix_l), Self::soft_clip(mix_r))
            } else {
                let lp = profile.lp_smoothing().max(1);
                self.mix_lp_l += (mix_l - self.mix_lp_l) / lp;
                self.mix_lp_r += (mix_r - self.mix_lp_r) / lp;
                let hp_in_l = self.mix_lp_l - self.prev_mix_l;
                let hp_in_r = self.mix_lp_r - self.prev_mix_r;
                let hp_decay = profile.hp_decay_q6();
                self.mix_hp_l = (self.mix_hp_l * hp_decay + hp_in_l * 64) / 64;
                self.mix_hp_r = (self.mix_hp_r * hp_decay + hp_in_r * 64) / 64;
                self.prev_mix_l = self.mix_lp_l;
                self.prev_mix_r = self.mix_lp_r;
                (
                    Self::soft_clip(self.mix_hp_l),
                    Self::soft_clip(self.mix_hp_r),
                )
            };

            out[frame * 2] = out_l.clamp(i16::MIN as i32, i16::MAX as i32) as i16;
            out[frame * 2 + 1] = out_r.clamp(i16::MIN as i32, i16::MAX as i32) as i16;
            self.sample_clock = self.sample_clock.wrapping_add(1);
        }
    }

    fn advance_sequencer(&mut self, mode: AudioMode) {
        let _tick_samples = if matches!(mode, AudioMode::Boot) {
            (self.sample_rate / BOOT_TICK_HZ).max(1)
        } else {
            let bpm = TRACK_BPM[(self.track_id as usize) % 6].max(1) as u32;
            (self.sample_rate * 15 / bpm).max(1)
        };
        self.tick_counter = self.tick_counter.wrapping_add(1);
        let tick_samples = self.tick_samples_for_mode(mode);
        if self.tick_counter < tick_samples {
            return;
        }
        self.tick_counter = 0;
        self.pattern_step = (self.pattern_step + 1) % PATTERN_STEPS;

        if matches!(mode, AudioMode::Boot) {
            self.advance_boot_sequencer();
            return;
        }

        let tid = (self.track_id as usize) % TRACK_BPM.len();
        let inst = TRACK_INST[tid];
        let (melody, bass, arp) = match tid {
            0 => (&TRACK0_MELODY, &TRACK0_BASS, &TRACK0_ARP),
            1 => (&TRACK1_MELODY, &TRACK1_BASS, &TRACK1_ARP),
            2 => (&TRACK2_MELODY, &TRACK2_BASS, &TRACK2_ARP),
            3 => (&TRACK3_MELODY, &TRACK3_BASS, &TRACK3_ARP),
            4 => (&TRACK4_MELODY, &TRACK4_BASS, &TRACK4_ARP),
            _ => (&TRACK5_MELODY, &TRACK5_BASS, &TRACK5_ARP),
        };

        let has_perc = matches!(tid, 0 | 4 | 5);
        let perc = match tid {
            0 => &TRACK0_PERC,
            4 => &TRACK4_PERC,
            _ => &TRACK5_PERC,
        };

        let s = self.pattern_step;
        for i in 0..VOICE_COUNT {
            let (hz, inst_id) = if i < 4 {
                (melody[(s + i) % PATTERN_STEPS], inst[0])
            } else if i < 8 {
                (bass[(s + (i - 4)) % PATTERN_STEPS], inst[1])
            } else if has_perc && i >= 20 {
                let pidx = i - 20;
                (perc[pidx][s], PERC_INST[pidx])
            } else {
                let arp_idx = if has_perc { (i - 8).min(12) } else { i - 8 };
                (arp[(s + arp_idx * 3) % PATTERN_STEPS], inst[2])
            };
            self.note_on(i, hz, inst_id, mode);
        }
    }

    fn tick_samples_for_mode(&self, mode: AudioMode) -> u32 {
        if matches!(mode, AudioMode::Boot) {
            return (self.sample_rate / BOOT_TICK_HZ).max(1);
        }
        let bpm = TRACK_BPM[(self.track_id as usize) % TRACK_BPM.len()].max(1) as u32;
        (self.sample_rate * 15 / bpm).max(1)
    }

    pub fn diagnostics_baseline(&self, frames: usize) -> AudioDiagnosticsBaseline {
        AudioDiagnosticsBaseline {
            sample_rate: self.sample_rate.max(SAMPLE_RATE_HZ),
            frames,
            boot: self.diagnostics_for_frames(AudioMode::Boot, frames),
            game: self.diagnostics_for_frames(AudioMode::Game, frames),
        }
    }

    fn advance_boot_sequencer(&mut self) {
        let s = self.pattern_step;
        let note = BOOT_MELODY[s];
        self.trigger_voice(0, note, 7, 600, 1024, 1024, 0);
    }

    fn trigger_voice(
        &mut self,
        idx: usize,
        hz: u16,
        instrument_id: u8,
        volume: u16,
        pan_l: u16,
        pan_r: u16,
        fx: u8,
    ) {
        self.note_on(idx, hz, instrument_id, AudioMode::Boot);
        let v = &mut self.voices[idx];
        v.volume = if hz == 0 { 0 } else { volume };
        v.pan_l = pan_l;
        v.pan_r = pan_r;
        v.fx = fx;
    }

    fn note_on(&mut self, idx: usize, hz: u16, instrument_id: u8, mode: AudioMode) {
        let inst = INSTRUMENTS[(instrument_id as usize) % INSTRUMENTS.len()];
        let v = &mut self.voices[idx];
        let default_volume = if matches!(mode, AudioMode::Boot) {
            560
        } else {
            680
        };

        if hz == 0 {
            v.volume = 0;
            if !matches!(
                v.envelope_state,
                EnvelopeState::Off | EnvelopeState::Release
            ) {
                v.envelope_state = EnvelopeState::Release;
            }
        } else {
            let was_off = matches!(v.envelope_state, EnvelopeState::Off);
            let same_note = v.pitch == hz
                && v.instrument_id == instrument_id
                && !matches!(
                    v.envelope_state,
                    EnvelopeState::Off | EnvelopeState::Release
                );
            if !same_note {
                v.envelope_state = EnvelopeState::Attack;
                v.env_counter = 0;
                if was_off {
                    v.phase = 0;
                }
            }
            v.volume = default_volume;
        }

        v.instrument_id = instrument_id;
        v.waveform_id = inst.waveform_id;
        v.pitch = hz;
        v.pan_l = ((VOICE_COUNT - idx) as u16 * 1024 / VOICE_COUNT as u16).clamp(128, 1024);
        v.pan_r = ((idx + 1) as u16 * 1024 / VOICE_COUNT as u16).clamp(128, 1024);
        let base_fx = match (mode, idx % 4) {
            (AudioMode::Boot, _) => 0,
            (AudioMode::Game, 0) => 0b0001,
            (AudioMode::Game, 1) => 0b0010,
            (AudioMode::Game, _) => 0,
        };
        v.fx = base_fx | TRACK_FX[(self.track_id as usize) % TRACK_FX.len()];
    }

    fn sample_voice(&mut self, idx: usize) -> (i32, i32) {
        let (inst, wave_id, pitch, pan_l, pan_r, vol, env_gain, vib_add, fx) = {
            let v = &mut self.voices[idx];
            if matches!(v.envelope_state, EnvelopeState::Off) {
                return (0, 0);
            }
            let inst = INSTRUMENTS[(v.instrument_id as usize) % INSTRUMENTS.len()];
            let env_gain = Self::step_envelope(v, inst);
            v.vibrato_phase = v.vibrato_phase.wrapping_add(inst.vibrato_speed.max(1));
            let vib_src = ((v.vibrato_phase as i16 as i32) >> 5).clamp(-8, 7);
            let vib_add = vib_src * inst.vibrato_depth as i32;
            (
                inst,
                v.waveform_id as usize,
                v.pitch,
                v.pan_l,
                v.pan_r,
                v.volume,
                env_gain,
                vib_add,
                v.fx,
            )
        };

        let hz = (pitch as i32 + vib_add).max(0) as u32;
        let step = self.step_from_hz(hz);
        let phase_idx = {
            let v = &mut self.voices[idx];
            v.phase = v.phase.wrapping_add(step);
            ((v.phase as u64 * WAVE_SIZE as u64) >> 24) as usize
        };
        let wave = self.read_wave(wave_id, phase_idx);
        let mut sample = wave as i32;
        sample = (sample * vol as i32) >> MIX_SHIFT;
        let smooth_gain = {
            let v = &mut self.voices[idx];
            let sg = ((env_gain as u32 + v.prev_env_gain as u32) >> 1) as i32;
            v.prev_env_gain = env_gain;
            sg
        };
        sample = (sample * smooth_gain) >> MIX_SHIFT;
        sample = self.apply_effects(idx, sample, inst, fx);

        let l = (sample * pan_l as i32) >> MIX_SHIFT;
        let r = (sample * pan_r as i32) >> MIX_SHIFT;
        (l, r)
    }

    fn step_envelope(v: &mut Voice, inst: Instrument) -> u16 {
        match v.envelope_state {
            EnvelopeState::Off => v.env_level = 0,
            EnvelopeState::Attack => {
                v.env_counter = v.env_counter.saturating_add(1);
                let inc = ((1024u32 / inst.attack.max(1) as u32).max(1)) as u16;
                v.env_level = v.env_level.saturating_add(inc).min(1024);
                if v.env_level >= 1024 {
                    v.envelope_state = EnvelopeState::Decay;
                }
            }
            EnvelopeState::Decay => {
                v.env_counter = v.env_counter.saturating_add(1);
                let dec = ((1024u32 / inst.decay.max(1) as u32).max(1)) as u16;
                v.env_level = v.env_level.saturating_sub(dec).max(inst.sustain);
                if v.env_level <= inst.sustain {
                    v.envelope_state = EnvelopeState::Sustain;
                }
            }
            EnvelopeState::Sustain => {
                v.env_level = inst.sustain;
                if v.volume == 0 {
                    v.envelope_state = EnvelopeState::Release;
                }
            }
            EnvelopeState::Release => {
                let rel = ((1024u32 / inst.release.max(1) as u32).max(1)) as u16;
                v.env_level = v.env_level.saturating_sub(rel);
                if v.env_level == 0 {
                    v.envelope_state = EnvelopeState::Off;
                }
            }
        }
        v.env_level
    }

    fn apply_effects(&mut self, idx: usize, sample: i32, _inst: Instrument, fx: u8) -> i32 {
        let mut out = sample;
        let v = &mut self.voices[idx];
        if fx & 0b0001 != 0 {
            let delayed = v.delay_line[v.delay_index] as i32;
            out = (out * 3 + delayed) / 4;
            v.delay_line[v.delay_index] = out as i16;
            v.delay_index = (v.delay_index + 1) % v.delay_line.len();
        }
        if fx & 0b0010 != 0 {
            out = (out * 6 / 5).clamp(-26_000, 26_000);
        }
        if fx & 0b0100 != 0 {
            v.lp_state += (out - v.lp_state) / 3;
            out = v.lp_state;
        }
        if fx & 0b1000 != 0 {
            out = (out * 3 / 2).clamp(-30_000, 30_000);
        }
        if fx & 0b10000 != 0 {
            let bits = 6;
            let step = 1i32 << (16 - bits);
            out = (out / step).saturating_mul(step);
        }
        out
    }

    fn read_wave(&self, wave_id: usize, idx: usize) -> i16 {
        let wave = wave_id.min(4);
        let base = self.wavetable_base[wave];
        let i = base + (idx % WAVE_SIZE) * 2;
        i16::from_le_bytes([self.audio_ram[i], self.audio_ram[i + 1]])
    }

    fn step_from_hz(&self, hz: u32) -> u32 {
        (((hz as u64) << 24) / self.sample_rate.max(1) as u64) as u32
    }

    fn sine_from_phase(phase: u16) -> i16 {
        let x = phase as i32;
        let half = (WAVE_SIZE / 2) as i32;
        let step = 65534 / half.max(1);
        let tri = if x < half {
            -32767 + x * step
        } else {
            32767 - (x - half) * step
        };
        let abs_t = tri.abs();
        let shaped = tri * (65535 - abs_t / 2) / 65535;
        shaped.clamp(i16::MIN as i32, i16::MAX as i32) as i16
    }

    fn soft_clip(sample: i32) -> i32 {
        let abs = sample.abs();
        let sign = if sample < 0 { -1 } else { 1 };
        let mag = (abs * MASTER_LIMIT) / (MASTER_LIMIT + abs.max(1));
        sign * mag
    }

    fn sample_pcm(&mut self) -> (i32, i32) {
        let mut mix_l = 0i32;
        let mut mix_r = 0i32;
        for ch in &mut self.pcm_channels {
            if !ch.playing || ch.position >= ch.length {
                continue;
            }
            let idx = ch.sample_base + ch.position * 2;
            if idx + 1 >= self.audio_ram.len() {
                ch.playing = false;
                continue;
            }
            let s = i16::from_le_bytes([self.audio_ram[idx], self.audio_ram[idx + 1]]) as i32;
            let scaled = (s * ch.volume as i32) >> MIX_SHIFT;
            mix_l += scaled;
            mix_r += scaled;
            ch.position += 1;
            if ch.position >= ch.length {
                ch.playing = false;
            }
        }
        (mix_l, mix_r)
    }

    fn sfx_sample(&mut self) -> (i32, i32) {
        if self.sfx_play_samples == 0 {
            return (0, 0);
        }
        let total = match self.sfx_kind {
            AudioSfx::Launch => self.sample_rate / 3,
            AudioSfx::Cancel => self.sample_rate / 8,
            AudioSfx::Confirm => self.sample_rate / 10,
            AudioSfx::BootChime => self.sample_rate / 12,
            AudioSfx::PlusChime => self.sample_rate / 8,
            AudioSfx::None => 1,
        }
        .max(1);
        let elapsed = total.saturating_sub(self.sfx_play_samples);
        self.sfx_play_samples = self.sfx_play_samples.saturating_sub(1);
        if self.sfx_play_samples == 0 {
            self.sfx_kind = AudioSfx::None;
        }

        self.noise_state = self
            .noise_state
            .wrapping_mul(1664525)
            .wrapping_add(1013904223);
        let base = match self.sfx_kind {
            AudioSfx::Launch => 400 + elapsed * 1100 / total,
            AudioSfx::Cancel => 920u32.saturating_sub(elapsed * 500 / total),
            AudioSfx::Confirm => 700 + elapsed * 450 / total,
            AudioSfx::BootChime => 400,
            AudioSfx::PlusChime => 660,
            AudioSfx::None => 0,
        };

        let step = self.step_from_hz(base.max(20));
        let mut pulse = ((self.noise_state >> 24) as i8 as i32) * 48;
        pulse += if (self.sample_clock as u32).wrapping_add(step) & 0x1000 != 0 {
            7200
        } else {
            -7200
        };
        let l = pulse;
        let r = pulse * 3 / 4;
        (l, r)
    }
}

#[cfg(test)]
mod tests {
    use super::{AudioEngine, AudioMode};

    #[test]
    fn wavetable_generation_does_not_overflow_in_debug() {
        let mut engine = AudioEngine::new(48_000);
        let mut block = [0i16; 1600];
        for _ in 0..8 {
            engine.render_block(AudioMode::Boot, &mut block);
            if block.iter().any(|s| *s != 0) {
                return;
            }
        }
        assert!(block.iter().any(|s| *s != 0));
    }

    #[test]
    fn diagnostics_peak_stays_below_hard_clip() {
        let engine = AudioEngine::new(48_000);
        let boot = engine.diagnostics_for_frames(AudioMode::Boot, 48_000);
        let game = engine.diagnostics_for_frames(AudioMode::Game, 48_000);
        assert!(boot.peak_l.abs() < 32_000 && boot.peak_r.abs() < 32_000);
        assert!(game.peak_l.abs() < 32_000 && game.peak_r.abs() < 32_000);
        assert!(boot.crest_l_q10 > 1024 && boot.crest_r_q10 > 1024);
        assert!(game.crest_l_q10 > 1024 && game.crest_r_q10 > 1024);
        assert_eq!(boot.clipped_l, 0);
        assert_eq!(boot.clipped_r, 0);
        assert_eq!(game.clipped_l, 0);
        assert_eq!(game.clipped_r, 0);
    }

    #[test]
    fn same_note_does_not_retrigger_active_voice() {
        let mut engine = AudioEngine::new(48_000);
        engine.note_on(0, 262, 0, AudioMode::Boot);
        engine.voices[0].envelope_state = super::EnvelopeState::Sustain;
        engine.voices[0].env_counter = 7;

        engine.note_on(0, 262, 0, AudioMode::Boot);

        assert!(matches!(
            engine.voices[0].envelope_state,
            super::EnvelopeState::Sustain
        ));
        assert_eq!(engine.voices[0].env_counter, 7);

        engine.note_on(0, 294, 0, AudioMode::Boot);
        assert!(matches!(
            engine.voices[0].envelope_state,
            super::EnvelopeState::Attack
        ));
        assert_eq!(engine.voices[0].env_counter, 0);
    }

    #[test]
    fn boot_voice_density_stays_within_budget() {
        let mut engine = AudioEngine::new(48_000);
        let mut max_active = 0usize;

        for step in 0..super::PATTERN_STEPS {
            engine.pattern_step = step;
            engine.advance_boot_sequencer();
            let active = engine
                .voices
                .iter()
                .filter(|voice| voice.pitch > 0 && voice.volume > 0)
                .count();
            max_active = max_active.max(active);
        }

        assert!(max_active <= 9, "max_active={max_active}");
    }
}
