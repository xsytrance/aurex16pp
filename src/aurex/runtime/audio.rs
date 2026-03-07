use crate::aurex::game::AudioCue;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum AudioMode {
    Boot,
    Game,
}

pub struct AudioEngine {
    sample_clock: u64,
    bass_phase: u32,
    lead_phase: u32,
    arp_phase: u32,
    sample_rate: u32,
    confirm_samples_left: u32,
    eat_samples_left: u32,
    fail_samples_left: u32,
    noise_state: u32,
    track_index: usize,
}

impl AudioEngine {
    pub fn new(sample_rate: u32) -> Self {
        Self {
            sample_clock: 0,
            bass_phase: 0,
            lead_phase: 0,
            arp_phase: 0,
            sample_rate,
            confirm_samples_left: 0,
            eat_samples_left: 0,
            fail_samples_left: 0,
            noise_state: 0xA5A5_1357,
            track_index: 0,
        }
    }

    fn step_from_hz(&self, hz: u32) -> u32 {
        (((hz as u64) << 32) / self.sample_rate as u64) as u32
    }

    pub fn trigger_cue(&mut self, cue: AudioCue) {
        match cue {
            AudioCue::Eat => self.eat_samples_left = self.sample_rate / 9,
            AudioCue::Fail => self.fail_samples_left = self.sample_rate / 4,
            AudioCue::TrackNext => {
                self.track_index = (self.track_index + 1) % 3;
                self.confirm_samples_left = self.sample_rate / 10;
            }
            AudioCue::TrackPrev => {
                self.track_index = (self.track_index + 2) % 3;
                self.confirm_samples_left = self.sample_rate / 10;
            }
            AudioCue::None => {}
        }
    }

    pub fn render_block(&mut self, mode: AudioMode, out: &mut [i16]) {
        for s in out.iter_mut() {
            let music = match mode {
                AudioMode::Boot => self.boot_sample(),
                AudioMode::Game => self.game_sample(),
            };

            let sfx = self.sfx_sample();
            *s = (music + sfx).clamp(i16::MIN as i32, i16::MAX as i32) as i16;
            self.sample_clock = self.sample_clock.wrapping_add(1);
        }
    }

    fn boot_sample(&mut self) -> i32 {
        // ~4.5s Yuzo-inspired driving intro with bass + lead + arp + noise hats.
        const BOOT_SAMPLES: u64 = 44_100 * 9 / 2;
        if self.sample_clock >= BOOT_SAMPLES {
            return 0;
        }

        const BPM: u32 = 154;
        const BASS: [u32; 16] = [
            73, 73, 98, 73, 82, 82, 110, 82, 73, 73, 98, 73, 65, 65, 87, 65,
        ];
        const LEAD: [u32; 16] = [
            294, 392, 440, 392, 349, 392, 523, 392, 294, 392, 440, 392, 262, 330, 392, 330,
        ];

        let body = self.pattern_sample(BPM, &BASS, &LEAD, 8200, 6400, true);
        let tail = (BOOT_SAMPLES - self.sample_clock).min(44_100) as i32;
        (body * tail) / 44_100
    }

    fn game_sample(&mut self) -> i32 {
        match self.track_index {
            0 => {
                const BPM: u32 = 110;
                const BASS: [u32; 16] = [
                    49, 49, 55, 49, 65, 65, 55, 49, 49, 49, 55, 49, 41, 41, 49, 41,
                ];
                const LEAD: [u32; 16] = [
                    196, 247, 294, 247, 220, 247, 330, 247, 196, 247, 294, 247, 175, 196, 247, 196,
                ];
                self.pattern_sample(BPM, &BASS, &LEAD, 5200, 3400, true)
            }
            1 => {
                const BPM: u32 = 124;
                const BASS: [u32; 16] = [
                    55, 55, 65, 55, 73, 73, 65, 55, 55, 55, 65, 55, 49, 49, 55, 49,
                ];
                const LEAD: [u32; 16] = [
                    220, 277, 330, 277, 247, 277, 370, 277, 220, 277, 330, 277, 196, 220, 277, 220,
                ];
                self.pattern_sample(BPM, &BASS, &LEAD, 5000, 3200, true)
            }
            _ => {
                const BPM: u32 = 136;
                const BASS: [u32; 16] = [
                    65, 65, 73, 65, 82, 82, 73, 65, 65, 65, 73, 65, 55, 55, 65, 55,
                ];
                const LEAD: [u32; 16] = [
                    262, 330, 392, 330, 294, 330, 440, 330, 262, 330, 392, 330, 220, 262, 330, 262,
                ];
                self.pattern_sample(BPM, &BASS, &LEAD, 5000, 3300, true)
            }
        }
    }

    fn pattern_sample(
        &mut self,
        bpm: u32,
        bass: &[u32; 16],
        lead: &[u32; 16],
        bass_amp: i32,
        lead_amp: i32,
        with_arp: bool,
    ) -> i32 {
        let spb = (self.sample_rate * 60) / bpm;
        let beat = (self.sample_clock / spb as u64) as usize;
        let step = beat % 16;
        let sub = (self.sample_clock % spb as u64) as u32;

        self.bass_phase = self.bass_phase.wrapping_add(self.step_from_hz(bass[step]));
        let bass_wave = if self.bass_phase < 0x8000_0000 {
            bass_amp
        } else {
            -bass_amp
        };

        let lead_hz = lead[step];
        let lead_wave = if lead_hz == 0 {
            0
        } else {
            let vib = if with_arp {
                let lfo = ((self.sample_clock >> 10) & 0x0F) as i32 - 8;
                lfo * 3
            } else {
                0
            };
            let hz = (lead_hz as i32 + vib).max(40) as u32;
            self.lead_phase = self.lead_phase.wrapping_add(self.step_from_hz(hz));
            let pulse = if (self.lead_phase >> 28) < 6 {
                lead_amp
            } else {
                -(lead_amp / 3)
            };
            if sub < spb / 2 { pulse } else { pulse / 2 }
        };

        let arp_wave = if with_arp {
            const ARP: [u32; 8] = [659, 784, 988, 784, 659, 988, 1175, 988];
            let arp_step = ((beat * 2) + ((sub > (spb / 2)) as usize)) % ARP.len();
            self.arp_phase = self
                .arp_phase
                .wrapping_add(self.step_from_hz(ARP[arp_step]));
            if self.arp_phase < 0x8000_0000 {
                2200
            } else {
                -800
            }
        } else {
            0
        };

        let kick_env = (spb.saturating_sub(sub)).min(spb / 6) as i32;
        let kick = (kick_env * 6) - 3500;

        let hat_window = spb / 8;
        let hat = if sub < hat_window || (sub > spb / 2 && sub < (spb / 2 + hat_window / 2)) {
            self.noise_state = self
                .noise_state
                .wrapping_mul(1664525)
                .wrapping_add(1013904223);
            ((self.noise_state >> 24) as i8 as i32) * 38
        } else {
            0
        };

        (bass_wave + lead_wave + arp_wave + kick + hat) / 2
    }

    fn sfx_sample(&mut self) -> i32 {
        if self.confirm_samples_left > 0 {
            let t = self.sample_rate / 3 - self.confirm_samples_left;
            let hz = 700 + (t * 900 / (self.sample_rate / 3).max(1));
            self.confirm_samples_left -= 1;
            self.lead_phase = self.lead_phase.wrapping_add(self.step_from_hz(hz));
            return if self.lead_phase < 0x8000_0000 {
                9000
            } else {
                -9000
            };
        }

        if self.fail_samples_left > 0 {
            let t = self.sample_rate / 4 - self.fail_samples_left;
            let hz = 420_u32.saturating_sub(t / 180);
            self.fail_samples_left -= 1;
            self.bass_phase = self.bass_phase.wrapping_add(self.step_from_hz(hz.max(90)));
            return if self.bass_phase < 0x8000_0000 {
                11000
            } else {
                -11000
            };
        }

        if self.eat_samples_left > 0 {
            let t = self.sample_rate / 9 - self.eat_samples_left;
            let hz = 1800_u32.saturating_sub(t * 6).max(500);
            self.eat_samples_left -= 1;
            self.lead_phase = self.lead_phase.wrapping_add(self.step_from_hz(hz));
            return if self.lead_phase < 0x8000_0000 {
                7000
            } else {
                -2000
            };
        }

        0
    }
}
