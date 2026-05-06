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
            Self::Soft => 72,
            Self::Default => 84,
            Self::Arcade => 96,
            Self::Boot => 82,
        }
    }

    fn lp_smoothing(self) -> i32 {
        match self {
            Self::Soft => 7,
            Self::Default => 5,
            Self::Arcade => 3,
            Self::Boot => 8,
        }
    }

    fn hp_decay_q6(self) -> i32 {
        match self {
            Self::Soft => 61,
            Self::Default => 62,
            Self::Arcade => 63,
            Self::Boot => 61,
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
const PCM_CHANNEL_COUNT: usize = 16;
const PCM_SLOT_BYTES: usize = 8192;
const WAVETABLE_BYTES: usize = 5 * WAVE_SIZE * 2;
const MIX_SHIFT: i32 = 10;
const PATTERN_STEPS: usize = 16;
const TRACK_BPM: [u16; 6] = [140, 130, 120, 100, 122, 136];
const MASTER_LIMIT: i32 = 27_000;

const WAVE_SINE: usize = 0;
const WAVE_SQUARE: usize = 1;
const WAVE_TRIANGLE: usize = 2;
const WAVE_SAW: usize = 3;
const WAVE_NOISE: usize = 4;

#[derive(Clone, Copy, Default)]
struct PcmChannel {
    position: usize,
    length: usize,
    sample_base: usize,
    volume: u16,
    playing: bool,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum EnvState {
    Off,
    Attack,
    Decay,
    Sustain,
    Release,
}

#[derive(Clone, Copy)]
struct Instrument {
    wave: usize,
    attack_ms: u16,
    decay_ms: u16,
    sustain_q10: u16,
    release_ms: u16,
    vibrato_cents: i16,
    vibrato_hz_x10: u16,
}

const INSTRUMENTS: [Instrument; 8] = [
    Instrument { wave: WAVE_SQUARE, attack_ms: 6, decay_ms: 90, sustain_q10: 650, release_ms: 70, vibrato_cents: 8, vibrato_hz_x10: 48 },
    Instrument { wave: WAVE_SAW, attack_ms: 4, decay_ms: 70, sustain_q10: 560, release_ms: 50, vibrato_cents: 2, vibrato_hz_x10: 35 },
    Instrument { wave: WAVE_TRIANGLE, attack_ms: 3, decay_ms: 55, sustain_q10: 780, release_ms: 55, vibrato_cents: 0, vibrato_hz_x10: 0 },
    Instrument { wave: WAVE_SINE, attack_ms: 14, decay_ms: 130, sustain_q10: 500, release_ms: 110, vibrato_cents: 11, vibrato_hz_x10: 62 },
    Instrument { wave: WAVE_NOISE, attack_ms: 1, decay_ms: 28, sustain_q10: 0, release_ms: 22, vibrato_cents: 0, vibrato_hz_x10: 0 },
    Instrument { wave: WAVE_SQUARE, attack_ms: 1, decay_ms: 20, sustain_q10: 850, release_ms: 18, vibrato_cents: 0, vibrato_hz_x10: 0 },
    Instrument { wave: WAVE_TRIANGLE, attack_ms: 8, decay_ms: 40, sustain_q10: 710, release_ms: 60, vibrato_cents: 4, vibrato_hz_x10: 20 },
    Instrument { wave: WAVE_SINE, attack_ms: 12, decay_ms: 40, sustain_q10: 740, release_ms: 80, vibrato_cents: 0, vibrato_hz_x10: 0 },
];

#[derive(Clone, Copy)]
struct Voice {
    active: bool,
    phase: u32,
    phase_inc: u32,
    pitch_hz: u16,
    instrument_id: u8,
    volume_q10: u16,
    pan_l_q10: u16,
    pan_r_q10: u16,
    env_state: EnvState,
    env_q10: u16,
    glide_target_env_q10: u16,
    gate_ramp_left: u16,
    vib_phase: u32,
    fx: u8,
    lp_state: i32,
    delay_line: [i16; 32],
    delay_index: usize,
}

impl Voice {
    const fn silent() -> Self {
        Self {
            active: false,
            phase: 0,
            phase_inc: 0,
            pitch_hz: 0,
            instrument_id: 0,
            volume_q10: 0,
            pan_l_q10: 512,
            pan_r_q10: 512,
            env_state: EnvState::Off,
            env_q10: 0,
            glide_target_env_q10: 0,
            gate_ramp_left: 0,
            vib_phase: 0,
            fx: 0,
            lp_state: 0,
            delay_line: [0; 32],
            delay_index: 0,
        }
    }
}

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
    tick_phase_samples: u32,
    tick_samples: u32,
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
    dc_x_prev_l: i32,
    dc_x_prev_r: i32,
    dc_y_prev_l: i32,
    dc_y_prev_r: i32,
    mix_profile: MixProfile,
}

impl AudioEngine {
    pub fn new(sample_rate: u32) -> Self {
        Self::new_with_profile(sample_rate, MixProfile::Default)
    }

    pub fn new_with_profile(sample_rate: u32, mix_profile: MixProfile) -> Self {
        let mut audio_ram = Box::new([0u8; AUDIO_RAM_BYTES]);
        let wavetable_base = [0, WAVE_SIZE * 2, WAVE_SIZE * 4, WAVE_SIZE * 6, WAVE_SIZE * 8];
        Self::write_wavetables(&mut audio_ram, &wavetable_base);
        let sr = sample_rate.max(1);
        let tick_samples = Self::tick_samples_for_track(sr, 0, AudioMode::Game);
        Self {
            sample_clock: 0,
            sample_rate: sr,
            tick_phase_samples: tick_samples.saturating_sub(1),
            tick_samples,
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
            dc_x_prev_l: 0,
            dc_x_prev_r: 0,
            dc_y_prev_l: 0,
            dc_y_prev_r: 0,
            mix_profile,
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
                self.tick_samples = Self::tick_samples_for_track(self.sample_rate, self.track_id, AudioMode::Game);
                self.tick_phase_samples = self.tick_samples.saturating_sub(1);
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
            RuntimeAudioCommand::PlayPcm { channel, sample_id, volume } => {
                let ch = (channel as usize) % PCM_CHANNEL_COUNT;
                let slot = (sample_id as usize) % 16;
                self.pcm_channels[ch].sample_base = WAVETABLE_BYTES + slot * PCM_SLOT_BYTES;
                self.pcm_channels[ch].length = PCM_SLOT_BYTES / 2;
                self.pcm_channels[ch].position = 0;
                self.pcm_channels[ch].volume = volume.min(1024);
                self.pcm_channels[ch].playing = true;
            }
            RuntimeAudioCommand::StopTrack => {
                for v in &mut self.voices {
                    if v.env_state != EnvState::Off {
                        v.env_state = EnvState::Release;
                        v.gate_ramp_left = Self::ms_to_samples(self.sample_rate, 6);
                    }
                }
            }
        }
    }

    pub fn diagnostics_for_frames(&self, mode: AudioMode, frames: usize) -> AudioDiagnostics {
        let mut sim = Self::new_with_profile(self.sample_rate.max(SAMPLE_RATE_HZ), self.mix_profile);
        sim.track_id = self.track_id;
        sim.tick_samples = Self::tick_samples_for_track(sim.sample_rate, sim.track_id, mode);
        let mut peak_l = 0i32;
        let mut peak_r = 0i32;
        let mut abs_sum_l: i64 = 0;
        let mut abs_sum_r: i64 = 0;
        let mut clipped_l = 0u32;
        let mut clipped_r = 0u32;
        let mut block = [0i16; 512];
        let mut remain = frames;
        while remain > 0 {
            let step = remain.min(block.len() / 2);
            sim.render_block(mode, &mut block[..step * 2]);
            for i in 0..step {
                let l = block[i * 2] as i32;
                let r = block[i * 2 + 1] as i32;
                peak_l = peak_l.max(l.abs());
                peak_r = peak_r.max(r.abs());
                abs_sum_l += l.abs() as i64;
                abs_sum_r += r.abs() as i64;
                if l.abs() >= 32_000 { clipped_l = clipped_l.saturating_add(1); }
                if r.abs() >= 32_000 { clipped_r = clipped_r.saturating_add(1); }
            }
            remain -= step;
        }
        let denom = frames.max(1) as i64;
        let avg_abs_l = (abs_sum_l / denom).clamp(i16::MIN as i64, i16::MAX as i64) as i16;
        let avg_abs_r = (abs_sum_r / denom).clamp(i16::MIN as i64, i16::MAX as i64) as i16;
        let crest_l_q10 = ((peak_l.max(1) * 1024) / (avg_abs_l.abs().max(1) as i32)).clamp(0, u16::MAX as i32) as u16;
        let crest_r_q10 = ((peak_r.max(1) * 1024) / (avg_abs_r.abs().max(1) as i32)).clamp(0, u16::MAX as i32) as u16;
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

    pub fn diagnostics_baseline(&self, frames: usize) -> AudioDiagnosticsBaseline {
        AudioDiagnosticsBaseline {
            sample_rate: self.sample_rate.max(SAMPLE_RATE_HZ),
            frames,
            boot: self.diagnostics_for_frames(AudioMode::Boot, frames),
            game: self.diagnostics_for_frames(AudioMode::Game, frames),
        }
    }

    pub fn render_block(&mut self, mode: AudioMode, out: &mut [i16]) {
        let frames = out.len() / 2;
        self.tick_samples = Self::tick_samples_for_track(self.sample_rate, self.track_id, mode);
        for frame in 0..frames {
            self.tick_phase_samples = self.tick_phase_samples.saturating_add(1);
            if self.tick_phase_samples >= self.tick_samples {
                self.tick_phase_samples = 0;
                self.pattern_step = (self.pattern_step + 1) % PATTERN_STEPS;
                if matches!(mode, AudioMode::Boot) {
                    self.schedule_boot_step();
                } else {
                    self.schedule_game_step();
                }
            }

            let mut mix_l = 0i32;
            let mut mix_r = 0i32;
            for i in 0..VOICE_COUNT {
                let (l, r) = self.sample_voice(i);
                mix_l += l;
                mix_r += r;
            }
            let (pcm_l, pcm_r) = self.sample_pcm();
            mix_l += pcm_l;
            mix_r += pcm_r;
            let (sfx_l, sfx_r) = self.sfx_sample();
            mix_l += sfx_l;
            mix_r += sfx_r;

            let profile = if matches!(mode, AudioMode::Boot) { MixProfile::Boot } else { self.mix_profile };
            let gain = profile.render_gain_q8();
            mix_l = (mix_l * gain) / 256;
            mix_r = (mix_r * gain) / 256;

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

            let pre_l = Self::soft_clip((self.mix_lp_l * 3 + self.mix_hp_l * 2) / 5);
            let pre_r = Self::soft_clip((self.mix_lp_r * 3 + self.mix_hp_r * 2) / 5);

            let dc_a_q10 = 1019;
            let y_l = pre_l - self.dc_x_prev_l + (self.dc_y_prev_l * dc_a_q10) / 1024;
            let y_r = pre_r - self.dc_x_prev_r + (self.dc_y_prev_r * dc_a_q10) / 1024;
            self.dc_x_prev_l = pre_l;
            self.dc_x_prev_r = pre_r;
            self.dc_y_prev_l = y_l;
            self.dc_y_prev_r = y_r;

            out[frame * 2] = y_l.clamp(i16::MIN as i32, i16::MAX as i32) as i16;
            out[frame * 2 + 1] = y_r.clamp(i16::MIN as i32, i16::MAX as i32) as i16;
            self.sample_clock = self.sample_clock.wrapping_add(1);
        }
    }

    fn schedule_boot_step(&mut self) {
        const BOOT: [u16; PATTERN_STEPS] = [147, 147, 196, 196, 220, 220, 262, 262, 294, 294, 330, 349, 392, 392, 330, 0];
        let note = BOOT[self.pattern_step];
        self.note_on(0, note, 7, AudioMode::Boot);
    }

    fn schedule_game_step(&mut self) {
        let tid = (self.track_id as usize) % 6;
        let (melody, bass, arp) = Self::track_patterns(tid);
        let step = self.pattern_step;

        for i in 0..VOICE_COUNT {
            if i < 4 {
                self.note_on(i, melody[(step + i) % PATTERN_STEPS], Self::track_instruments(tid).0, AudioMode::Game);
            } else if i < 8 {
                self.note_on(i, bass[(step + i - 4) % PATTERN_STEPS], Self::track_instruments(tid).1, AudioMode::Game);
            } else if i < 16 {
                self.note_on(i, arp[(step + (i - 8) * 2) % PATTERN_STEPS], Self::track_instruments(tid).2, AudioMode::Game);
            } else if i < 20 {
                let ghost = arp[(step + i) % PATTERN_STEPS] / 2;
                self.note_on(i, ghost, 6, AudioMode::Game);
            } else {
                let hit = Self::perc_for_track(tid, i - 20, step);
                self.note_on(i, hit, if i % 2 == 0 { 4 } else { 5 }, AudioMode::Game);
            }
        }
    }

    fn note_on(&mut self, idx: usize, hz: u16, instrument_id: u8, mode: AudioMode) {
        let phase_inc = self.hz_to_phase_inc(hz as u32);
        let v = &mut self.voices[idx];
        let inst_id = (instrument_id as usize) % INSTRUMENTS.len();
        let inst = INSTRUMENTS[inst_id];
        let old_pitch = v.pitch_hz;

        if hz == 0 {
            if v.env_state != EnvState::Off {
                v.env_state = EnvState::Release;
                v.glide_target_env_q10 = 0;
                v.gate_ramp_left = Self::ms_to_samples(self.sample_rate, 6);
            }
            return;
        }

        let hold_same = old_pitch == hz && v.instrument_id == instrument_id && (v.env_state == EnvState::Sustain || v.env_state == EnvState::Decay);
        if !hold_same {
            v.active = true;
            v.instrument_id = instrument_id;
            v.pitch_hz = hz;
            v.env_state = EnvState::Attack;
            v.glide_target_env_q10 = 1024;
            v.gate_ramp_left = Self::ms_to_samples(self.sample_rate, 6);
            if v.phase == 0 { v.phase = ((idx as u32 + 1) * 0x0100_0000) / VOICE_COUNT as u32; }
        }

        let vol = if matches!(mode, AudioMode::Boot) {
            560
        } else if idx < 4 {
            700
        } else if idx < 8 {
            640
        } else if idx < 16 {
            520
        } else {
            450
        };
        v.volume_q10 = vol;

        v.pan_l_q10 = ((VOICE_COUNT - idx) as u16 * 1024 / VOICE_COUNT as u16).clamp(128, 1024);
        v.pan_r_q10 = ((idx + 1) as u16 * 1024 / VOICE_COUNT as u16).clamp(128, 1024);
        let fx_lane = match idx % 4 { 0 => 0b0001, 1 => 0b0010, 2 => 0b0100, _ => 0 };
        v.fx = if matches!(mode, AudioMode::Boot) { 0 } else { fx_lane | (((self.track_id as usize + idx) as u8) & 0b1_0000) };
        v.phase_inc = phase_inc;

        if inst.vibrato_hz_x10 == 0 { v.vib_phase = 0; }
    }

    fn sample_voice(&mut self, idx: usize) -> (i32, i32) {
        let (inst, phase_idx, volume_q10, env_q10, gate_ramp_left, pan_l_q10, pan_r_q10) = {
            let v = &mut self.voices[idx];
            if !v.active || v.env_state == EnvState::Off {
                return (0, 0);
            }
            let inst = INSTRUMENTS[(v.instrument_id as usize) % INSTRUMENTS.len()];
            Self::step_envelope(v, inst, self.sample_rate);
            if v.env_state == EnvState::Off {
                v.active = false;
                return (0, 0);
            }

            let mut phase_inc = v.phase_inc;
            if inst.vibrato_hz_x10 > 0 && inst.vibrato_cents != 0 {
                v.vib_phase = v.vib_phase.wrapping_add(((inst.vibrato_hz_x10 as u64 * (1u64 << 24)) / (self.sample_rate as u64 * 10)) as u32);
                let vib = ((v.vib_phase >> 16) as i16 as i32) >> 8;
                let cents = vib * inst.vibrato_cents as i32 / 128;
                phase_inc = ((phase_inc as i64 * (12_000 + cents as i64)) / 12_000).max(0) as u32;
            }
            v.phase = v.phase.wrapping_add(phase_inc);
            let phase_idx = ((v.phase as u64 * WAVE_SIZE as u64) >> 32) as usize;
            (inst, phase_idx, v.volume_q10, v.env_q10, v.gate_ramp_left, v.pan_l_q10, v.pan_r_q10)
        };

        let mut sample = self.read_wave(inst.wave, phase_idx) as i32;
        sample = (sample * volume_q10 as i32) >> MIX_SHIFT;
        sample = (sample * env_q10 as i32) >> MIX_SHIFT;

        {
            let v = &mut self.voices[idx];
            if gate_ramp_left > 0 {
                let total = Self::ms_to_samples(self.sample_rate, 6).max(1);
                let ramp = (((total.saturating_sub(gate_ramp_left)) as u32 * 1024) / total as u32) as i32;
                sample = (sample * ramp) >> 10;
                v.gate_ramp_left = v.gate_ramp_left.saturating_sub(1);
            }
            sample = Self::apply_effects(v, sample);
        }

        let l = (sample * pan_l_q10 as i32) >> MIX_SHIFT;
        let r = (sample * pan_r_q10 as i32) >> MIX_SHIFT;
        (l, r)
    }

    fn apply_effects(v: &mut Voice, sample: i32) -> i32 {
        let mut out = sample;
        if v.fx & 0b0001 != 0 {
            let delayed = v.delay_line[v.delay_index] as i32;
            out = (out * 5 + delayed * 2) / 7;
            v.delay_line[v.delay_index] = (out / 2).clamp(i16::MIN as i32, i16::MAX as i32) as i16;
            v.delay_index = (v.delay_index + 1) % v.delay_line.len();
        }
        if v.fx & 0b0010 != 0 {
            out = (out * 11 / 10).clamp(-24_000, 24_000);
        }
        if v.fx & 0b0100 != 0 {
            v.lp_state += (out - v.lp_state) / 4;
            out = v.lp_state;
        }
        if v.fx & 0b1_0000 != 0 {
            let step = 1i32 << 7;
            out = (out / step) * step;
        }
        out
    }

    fn step_envelope(v: &mut Voice, inst: Instrument, sample_rate: u32) {
        let a = Self::ms_to_samples(sample_rate, inst.attack_ms).max(1) as u32;
        let d = Self::ms_to_samples(sample_rate, inst.decay_ms).max(1) as u32;
        let r = Self::ms_to_samples(sample_rate, inst.release_ms).max(1) as u32;
        match v.env_state {
            EnvState::Off => v.env_q10 = 0,
            EnvState::Attack => {
                let inc = (1024 / a).max(1) as u16;
                v.env_q10 = v.env_q10.saturating_add(inc).min(1024);
                if v.env_q10 >= 1024 { v.env_state = EnvState::Decay; }
            }
            EnvState::Decay => {
                let diff = 1024u16.saturating_sub(inst.sustain_q10);
                let dec = ((diff as u32 / d).max(1)) as u16;
                v.env_q10 = v.env_q10.saturating_sub(dec).max(inst.sustain_q10);
                if v.env_q10 <= inst.sustain_q10 { v.env_state = EnvState::Sustain; }
            }
            EnvState::Sustain => {
                v.env_q10 = inst.sustain_q10;
                if v.volume_q10 == 0 { v.env_state = EnvState::Release; }
            }
            EnvState::Release => {
                let dec = (1024 / r).max(1) as u16;
                v.env_q10 = v.env_q10.saturating_sub(dec);
                if v.env_q10 == 0 { v.env_state = EnvState::Off; }
            }
        }
    }

    fn sample_pcm(&mut self) -> (i32, i32) {
        let mut l = 0i32;
        let mut r = 0i32;
        for ch in &mut self.pcm_channels {
            if !ch.playing || ch.position >= ch.length { continue; }
            let idx = ch.sample_base + ch.position * 2;
            if idx + 1 >= self.audio_ram.len() {
                ch.playing = false;
                continue;
            }
            let s = i16::from_le_bytes([self.audio_ram[idx], self.audio_ram[idx + 1]]) as i32;
            let v = (s * ch.volume as i32) >> MIX_SHIFT;
            l += v;
            r += v;
            ch.position += 1;
            if ch.position >= ch.length { ch.playing = false; }
        }
        (l, r)
    }

    fn sfx_sample(&mut self) -> (i32, i32) {
        if self.sfx_play_samples == 0 { return (0, 0); }
        let total = match self.sfx_kind {
            AudioSfx::Launch => self.sample_rate / 3,
            AudioSfx::Cancel => self.sample_rate / 8,
            AudioSfx::Confirm => self.sample_rate / 10,
            AudioSfx::BootChime => self.sample_rate / 12,
            AudioSfx::PlusChime => self.sample_rate / 8,
            AudioSfx::None => 1,
        }.max(1);
        let elapsed = total.saturating_sub(self.sfx_play_samples);
        self.sfx_play_samples = self.sfx_play_samples.saturating_sub(1);
        let kind = self.sfx_kind;
        if self.sfx_play_samples == 0 { self.sfx_kind = AudioSfx::None; }

        self.noise_state = self.noise_state.wrapping_mul(1664525).wrapping_add(1013904223);
        let noise = ((self.noise_state >> 16) as i16 as i32) / 4;
        let base_hz = match kind {
            AudioSfx::Launch => 320 + elapsed * 1400 / total,
            AudioSfx::Cancel => 1100u32.saturating_sub(elapsed * 700 / total),
            AudioSfx::Confirm => 640 + elapsed * 500 / total,
            AudioSfx::BootChime => 440 + elapsed * 180 / total,
            AudioSfx::PlusChime => 760,
            AudioSfx::None => 0,
        };
        let pulse = if ((self.sample_clock as u32).wrapping_mul(base_hz.max(1))) & 0x2000 != 0 { 6200 } else { -6200 };
        let env = (((total.saturating_sub(elapsed)) * 1024) / total).min(1024) as i32;
        let s = ((pulse + noise) * env) >> 10;
        (s, s * 3 / 4)
    }

    fn tick_samples_for_track(sample_rate: u32, track_id: u8, mode: AudioMode) -> u32 {
        if matches!(mode, AudioMode::Boot) {
            return (sample_rate / 4).max(1);
        }
        let bpm = TRACK_BPM[(track_id as usize) % TRACK_BPM.len()].max(1) as u32;
        (sample_rate * 15 / bpm).max(1)
    }

    fn hz_to_phase_inc(&self, hz: u32) -> u32 {
        (((hz as u64) << 32) / self.sample_rate.max(1) as u64) as u32
    }

    fn ms_to_samples(sample_rate: u32, ms: u16) -> u16 {
        ((sample_rate as u64 * ms as u64) / 1000).clamp(1, u16::MAX as u64) as u16
    }

    fn read_wave(&self, wave_id: usize, idx: usize) -> i16 {
        let base = self.wavetable_base[wave_id.min(4)];
        let i = base + (idx % WAVE_SIZE) * 2;
        i16::from_le_bytes([self.audio_ram[i], self.audio_ram[i + 1]])
    }

    fn write_wavetables(ram: &mut [u8; AUDIO_RAM_BYTES], base: &[usize; 5]) {
        for i in 0..WAVE_SIZE {
            let ph = i as f32 / WAVE_SIZE as f32;
            let sine = (ph * core::f32::consts::TAU).sin() * 29_000.0;
            let square = if i < WAVE_SIZE / 2 { 27_000.0 } else { -27_000.0 };
            let tri = (2.0 * (2.0 * ph - (2.0 * ph + 0.5).floor()).abs() - 1.0) * 28_000.0;
            let saw = ((2.0 * ph) - 1.0) * 26_000.0;
            let noise_seed = (i as u32).wrapping_mul(747796405).wrapping_add(2891336453);
            let noise = ((noise_seed >> 16) as i16 as f32) * 0.8;
            let waves = [sine as i16, square as i16, tri as i16, saw as i16, noise as i16];
            for w in 0..5 {
                let idx = base[w] + i * 2;
                let s = waves[w].to_le_bytes();
                ram[idx] = s[0];
                ram[idx + 1] = s[1];
            }
        }
    }

    fn soft_clip(sample: i32) -> i32 {
        let abs = sample.abs();
        let sign = if sample < 0 { -1 } else { 1 };
        let mag = (abs * MASTER_LIMIT) / (MASTER_LIMIT + abs.max(1));
        sign * mag
    }

    fn track_instruments(track: usize) -> (u8, u8, u8) {
        match track {
            0 => (0, 1, 2),
            1 => (1, 6, 2),
            2 => (0, 2, 3),
            3 => (3, 6, 0),
            4 => (5, 1, 2),
            _ => (0, 3, 6),
        }
    }

    fn track_patterns(track: usize) -> (&'static [u16; PATTERN_STEPS], &'static [u16; PATTERN_STEPS], &'static [u16; PATTERN_STEPS]) {
        const M0: [u16; PATTERN_STEPS] = [494, 370, 330, 247, 494, 370, 330, 247, 494, 370, 494, 370, 330, 247, 330, 247];
        const B0: [u16; PATTERN_STEPS] = [123, 92, 82, 62, 123, 92, 82, 62, 123, 92, 123, 92, 82, 62, 82, 62];
        const A0: [u16; PATTERN_STEPS] = [494, 370, 330, 247, 370, 494, 370, 247, 330, 247, 494, 370, 330, 247, 494, 370];

        const M1: [u16; PATTERN_STEPS] = [233, 311, 349, 466, 233, 311, 349, 466, 349, 311, 233, 0, 466, 349, 311, 233];
        const B1: [u16; PATTERN_STEPS] = [116, 155, 175, 233, 116, 155, 175, 233, 175, 155, 116, 0, 233, 175, 155, 116];
        const A1: [u16; PATTERN_STEPS] = [466, 349, 311, 233, 311, 466, 349, 233, 349, 311, 466, 311, 233, 349, 466, 349];

        const M2: [u16; PATTERN_STEPS] = [262, 330, 392, 523, 392, 330, 262, 0, 294, 349, 392, 523, 392, 349, 294, 0];
        const B2: [u16; PATTERN_STEPS] = [131, 165, 196, 262, 196, 165, 131, 0, 147, 175, 196, 262, 196, 175, 147, 0];
        const A2: [u16; PATTERN_STEPS] = [523, 392, 330, 262, 330, 392, 523, 392, 262, 330, 392, 523, 392, 330, 262, 196];

        const M3: [u16; PATTERN_STEPS] = [587, 440, 349, 294, 523, 392, 311, 262, 440, 349, 294, 262, 392, 311, 262, 0];
        const B3: [u16; PATTERN_STEPS] = [73, 110, 87, 73, 65, 98, 87, 65, 55, 87, 73, 65, 98, 87, 65, 0];
        const A3: [u16; PATTERN_STEPS] = [294, 349, 440, 587, 349, 262, 311, 392, 440, 349, 294, 262, 392, 311, 262, 220];

        const M4: [u16; PATTERN_STEPS] = [165, 110, 123, 82, 165, 110, 123, 82, 110, 165, 110, 82, 123, 165, 123, 82];
        const B4: [u16; PATTERN_STEPS] = [82, 55, 62, 41, 82, 55, 62, 41, 55, 82, 55, 41, 62, 82, 62, 41];
        const A4: [u16; PATTERN_STEPS] = [165, 220, 247, 165, 110, 165, 220, 165, 123, 165, 110, 82, 247, 165, 220, 165];

        const M5: [u16; PATTERN_STEPS] = [415, 494, 554, 659, 554, 494, 415, 0, 494, 554, 659, 831, 659, 554, 494, 415];
        const B5: [u16; PATTERN_STEPS] = [104, 123, 139, 165, 139, 123, 104, 0, 123, 139, 165, 208, 165, 139, 123, 104];
        const A5: [u16; PATTERN_STEPS] = [415, 554, 659, 831, 554, 415, 494, 659, 831, 554, 415, 494, 659, 554, 494, 415];

        match track {
            0 => (&M0, &B0, &A0),
            1 => (&M1, &B1, &A1),
            2 => (&M2, &B2, &A2),
            3 => (&M3, &B3, &A3),
            4 => (&M4, &B4, &A4),
            _ => (&M5, &B5, &A5),
        }
    }

    fn perc_for_track(track: usize, lane: usize, step: usize) -> u16 {
        const K: [u16; PATTERN_STEPS] = [82, 0, 82, 0, 82, 0, 82, 0, 82, 0, 82, 0, 82, 0, 82, 0];
        const S: [u16; PATTERN_STEPS] = [0, 0, 0, 0, 350, 0, 0, 0, 0, 0, 0, 0, 350, 0, 0, 0];
        const H: [u16; PATTERN_STEPS] = [4000, 0, 4000, 0, 4000, 0, 4000, 0, 4000, 0, 4000, 0, 4000, 0, 4000, 0];
        const O: [u16; PATTERN_STEPS] = [0, 3000, 0, 3000, 0, 3000, 0, 3000, 0, 3000, 0, 3000, 0, 3000, 0, 3000];
        if track == 4 {
            match lane {
                0 => [82, 82, 0, 82, 0, 82, 82, 0, 82, 0, 82, 82, 0, 82, 0, 82][step],
                1 => [0, 0, 350, 0, 0, 0, 0, 350, 0, 0, 350, 0, 0, 0, 0, 350][step],
                2 => H[step],
                _ => 0,
            }
        } else if track == 5 {
            match lane {
                0 => [82, 0, 0, 0, 82, 0, 0, 0, 82, 0, 0, 0, 82, 0, 0, 0][step],
                1 => S[step],
                2 => 4000,
                _ => 0,
            }
        } else {
            match lane {
                0 => K[step],
                1 => S[step],
                2 => H[step],
                _ => O[step],
            }
        }
    }
}
