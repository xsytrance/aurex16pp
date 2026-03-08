use super::event::{AudioSfx, RuntimeAudioCommand};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum AudioMode {
    Boot,
    Game,
}

const SAMPLE_RATE_HZ: u32 = 48_000;
const AUDIO_RAM_BYTES: usize = 512 * 1024;
const VOICE_COUNT: usize = 12;
const WAVE_SIZE: usize = 256;
const MIX_SHIFT: i32 = 10;
const TICK_HZ: u32 = 120;
const PATTERN_STEPS: usize = 16;

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

const INSTRUMENTS: [Instrument; 6] = [
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
];

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
        }
    }
}

const TRACK0: [u16; PATTERN_STEPS] = [
    196, 0, 220, 0, 247, 0, 262, 0, 196, 0, 220, 0, 175, 0, 196, 0,
];
const TRACK1: [u16; PATTERN_STEPS] = [
    247, 262, 294, 330, 247, 262, 294, 330, 220, 247, 262, 294, 196, 220, 247, 262,
];
const TRACK2: [u16; PATTERN_STEPS] = [
    98, 98, 110, 98, 123, 123, 110, 98, 82, 82, 98, 82, 73, 73, 82, 73,
];
const TRACK3: [u16; PATTERN_STEPS] = [
    330, 392, 494, 523, 440, 392, 587, 523, 392, 440, 494, 523, 330, 392, 440, 494,
];

#[derive(Debug, Clone, Copy)]
pub struct AudioDiagnostics {
    pub frames: usize,
    pub peak_l: i16,
    pub peak_r: i16,
    pub avg_abs_l: i16,
    pub avg_abs_r: i16,
}

impl AudioDiagnostics {
    pub fn to_json(&self) -> String {
        format!(
            "{{\"frames\":{},\"peak_l\":{},\"peak_r\":{},\"avg_abs_l\":{},\"avg_abs_r\":{}}}",
            self.frames, self.peak_l, self.peak_r, self.avg_abs_l, self.avg_abs_r
        )
    }
}

pub struct AudioEngine {
    sample_clock: u64,
    sample_rate: u32,
    tick_samples: u32,
    tick_counter: u32,
    pattern_step: usize,
    track_id: u8,
    voices: [Voice; VOICE_COUNT],
    audio_ram: Box<[u8; AUDIO_RAM_BYTES]>,
    wavetable_base: [usize; 5],
    sfx_play_samples: u32,
    sfx_kind: AudioSfx,
    noise_state: u32,
}

impl AudioEngine {
    pub fn new(sample_rate: u32) -> Self {
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
            tick_samples: (sample_rate / TICK_HZ).max(1),
            tick_counter: 0,
            pattern_step: 0,
            track_id: 0,
            voices: [Voice::silent(); VOICE_COUNT],
            audio_ram,
            wavetable_base,
            sfx_play_samples: 0,
            sfx_kind: AudioSfx::None,
            noise_state: 0xC001_FEED,
        }
    }

    fn write_wavetables(ram: &mut [u8; AUDIO_RAM_BYTES], base: &[usize; 5]) {
        for i in 0..WAVE_SIZE {
            let phase = i as i32;
            let tri = if i < 128 {
                -32767 + phase * 512
            } else {
                32767 - ((phase - 128) * 512)
            };
            let saw = -32767 + phase * 256;
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

    pub fn trigger_command(&mut self, cmd: RuntimeAudioCommand) {
        match cmd {
            RuntimeAudioCommand::PlayTrack(track_id) => {
                self.track_id = track_id % 4;
                self.pattern_step = 0;
                self.tick_counter = 0;
            }
            RuntimeAudioCommand::PlaySfx(sfx) => {
                self.sfx_kind = sfx;
                self.sfx_play_samples = match sfx {
                    AudioSfx::Launch => self.sample_rate / 3,
                    AudioSfx::Cancel => self.sample_rate / 8,
                    AudioSfx::Confirm => self.sample_rate / 10,
                    AudioSfx::None => 0,
                };
            }
            RuntimeAudioCommand::StopTrack => {
                for voice in &mut self.voices {
                    voice.envelope_state = EnvelopeState::Release;
                }
            }
        }
    }

    pub fn diagnostics_for_frames(&self, mode: AudioMode, frames: usize) -> AudioDiagnostics {
        let mut sim = Self::new(self.sample_rate.max(SAMPLE_RATE_HZ));
        sim.track_id = self.track_id;

        let mut peak_l = 0i32;
        let mut peak_r = 0i32;
        let mut abs_sum_l: i64 = 0;
        let mut abs_sum_r: i64 = 0;
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
            }

            remain -= step;
        }

        let denom = frames.max(1) as i64;
        AudioDiagnostics {
            frames,
            peak_l: peak_l.clamp(i16::MIN as i32, i16::MAX as i32) as i16,
            peak_r: peak_r.clamp(i16::MIN as i32, i16::MAX as i32) as i16,
            avg_abs_l: (abs_sum_l / denom).clamp(i16::MIN as i64, i16::MAX as i64) as i16,
            avg_abs_r: (abs_sum_r / denom).clamp(i16::MIN as i64, i16::MAX as i64) as i16,
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

            out[frame * 2] = mix_l.clamp(i16::MIN as i32, i16::MAX as i32) as i16;
            out[frame * 2 + 1] = mix_r.clamp(i16::MIN as i32, i16::MAX as i32) as i16;
            self.sample_clock = self.sample_clock.wrapping_add(1);
        }
    }

    fn advance_sequencer(&mut self, mode: AudioMode) {
        self.tick_counter = self.tick_counter.wrapping_add(1);
        if self.tick_counter < self.tick_samples {
            return;
        }
        self.tick_counter = 0;
        self.pattern_step = (self.pattern_step + 1) % PATTERN_STEPS;

        let track = match self.track_id {
            0 => &TRACK0,
            1 => &TRACK1,
            2 => &TRACK2,
            _ => &TRACK3,
        };

        for i in 0..VOICE_COUNT {
            let hz = if i < 4 {
                track[(self.pattern_step + i) % PATTERN_STEPS]
            } else if i < 8 {
                track[(self.pattern_step * 2 + i) % PATTERN_STEPS] / 2
            } else {
                track[(self.pattern_step + i * 3) % PATTERN_STEPS]
            };

            let inst = match (mode, i) {
                (AudioMode::Boot, 0..=3) => 3,
                (AudioMode::Boot, 4..=7) => 2,
                (AudioMode::Boot, _) => 4,
                (AudioMode::Game, 0..=3) => 0,
                (AudioMode::Game, 4..=7) => 1,
                (AudioMode::Game, _) => 2,
            };

            self.note_on(i, hz, inst as u8);
        }
    }

    fn note_on(&mut self, idx: usize, hz: u16, instrument_id: u8) {
        let inst = INSTRUMENTS[(instrument_id as usize) % INSTRUMENTS.len()];
        let v = &mut self.voices[idx];
        v.instrument_id = instrument_id;
        v.waveform_id = inst.waveform_id;
        v.pitch = hz;
        v.volume = if hz == 0 { 0 } else { 800 };
        v.envelope_state = if hz == 0 {
            EnvelopeState::Release
        } else {
            EnvelopeState::Attack
        };
        v.env_counter = 0;
        v.pan_l = ((VOICE_COUNT - idx) as u16 * 1024 / VOICE_COUNT as u16).clamp(128, 1024);
        v.pan_r = ((idx + 1) as u16 * 1024 / VOICE_COUNT as u16).clamp(128, 1024);
        v.fx = match idx % 4 {
            0 => 0b0001,
            1 => 0b0010,
            2 => 0b0100,
            _ => 0b1000,
        };
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
            (v.phase >> 24) as usize
        };
        let wave = self.read_wave(wave_id, phase_idx);
        let mut sample = wave as i32;
        sample = (sample * vol as i32) >> MIX_SHIFT;
        sample = (sample * env_gain as i32) >> MIX_SHIFT;
        sample = self.apply_effects(idx, sample, inst, fx);

        let l = (sample * pan_l as i32) >> MIX_SHIFT;
        let r = (sample * pan_r as i32) >> MIX_SHIFT;
        (l, r)
    }

    fn step_envelope(v: &mut Voice, inst: Instrument) -> u16 {
        match v.envelope_state {
            EnvelopeState::Off => {
                v.env_level = 0;
            }
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
            out = (out + (out >> 1)) / 2;
        }

        if fx & 0b0100 != 0 {
            out = (out >> 8) << 8;
        }

        if fx & 0b1000 != 0 {
            out = (out * 3 / 2).clamp(-30_000, 30_000);
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
        let tri = if x < 128 {
            -32767 + x * 512
        } else {
            32767 - (x - 128) * 512
        };
        // Integer parabolic shaping from triangle to pseudo-sine.
        let abs_t = tri.abs();
        let shaped = tri * (65535 - abs_t / 2) / 65535;
        shaped.clamp(i16::MIN as i32, i16::MAX as i32) as i16
    }

    fn sfx_sample(&mut self) -> (i32, i32) {
        if self.sfx_play_samples == 0 {
            return (0, 0);
        }

        let total = match self.sfx_kind {
            AudioSfx::Launch => self.sample_rate / 3,
            AudioSfx::Cancel => self.sample_rate / 8,
            AudioSfx::Confirm => self.sample_rate / 10,
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
        let mut block = [0i16; 64];
        engine.render_block(AudioMode::Boot, &mut block);
        assert!(block.iter().any(|s| *s != 0));
    }
}
