//! Pure scores for the synthesized show cues (src/quiz/sounds.js).
use crate::theme::format::SoundPack;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Sound {
    Round,
    Answer,
    Reveal,
    Winner,
}
impl Sound {
    pub const ALL: [Self; 4] = [Self::Round, Self::Answer, Self::Reveal, Self::Winner];
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Wave {
    Sine,
    Sawtooth,
    Square,
    Triangle,
}
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Tone {
    pub from: f64,
    pub to: f64,
    pub start: f64,
    pub length: f64,
    pub wave: Wave,
    pub gain: f64,
    pub attack: f64,
    /// Frequency modulation depth in Hz; the LFO runs at 6 Hz.
    pub vibrato: f64,
    pub lowpass: Option<f64>,
}
impl Tone {
    fn new(midi: f64, start: f64, length: f64, gain: f64) -> Self {
        Self {
            from: note(midi),
            to: note(midi),
            start,
            length,
            wave: Wave::Sine,
            gain,
            attack: 0.01,
            vibrato: 0.,
            lowpass: None,
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Noise {
    pub start: f64,
    pub length: f64,
    pub gain: f64,
    pub highpass: f64,
}
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Voice {
    Tone(Tone),
    Noise(Noise),
}
#[derive(Debug, Clone, PartialEq)]
pub struct Score {
    /// Pack loudness compensation; multiply by the user's volume.
    pub level: f64,
    pub voices: Vec<Voice>,
}
/// MIDI note number to frequency in Hz.
pub fn note(midi: f64) -> f64 {
    440. * 2_f64.powf((midi - 69.) / 12.)
}
/// A white-noise sample with the JS engine's linear decay. `random` is in [0, 1).
pub fn noise_sample(random: f64, index: usize, frames: usize) -> f32 {
    ((random * 2. - 1.) * (1. - index as f64 / frames as f64)) as f32
}
impl Score {
    fn tone(&mut self, tone: Tone) {
        self.voices.push(Voice::Tone(tone));
    }
    fn bell(&mut self, midi: f64, start: f64, length: f64, gain: f64) {
        let tone = Tone::new(midi, start, length, gain);
        self.tone(tone);
        self.tone(Tone {
            from: tone.from * 2.76,
            to: tone.to * 2.76,
            length: length * 0.5,
            gain: gain * 0.3,
            ..tone
        });
    }
    fn noise(&mut self, start: f64, length: f64, gain: f64, highpass: f64) {
        self.voices.push(Voice::Noise(Noise {
            start,
            length,
            gain,
            highpass,
        }));
    }
}
/// Every pack/event combination is defined; a theme with no pack stays silent.
pub fn score(pack: SoundPack, sound: Sound) -> Score {
    use Sound::*;
    use SoundPack::*;
    let mut s = Score {
        level: match pack {
            Spooky => 1.7,
            Arcade => 1.4,
            _ => 1.,
        },
        voices: vec![],
    };
    match (pack, sound) {
        (Chimes, Round) => {
            s.bell(72., 0., 1.2, 0.25);
            s.bell(79., 0.18, 1.4, 0.25);
        }
        (Chimes, Answer) => {
            s.bell(84., 0., 1.6, 0.3);
            s.bell(88., 0.08, 1.4, 0.18);
        }
        (Chimes, Reveal) => {
            s.noise(0., 0.35, 0.06, 3000.);
            s.bell(76., 0.25, 1.2, 0.25);
        }
        (Chimes, Winner) => {
            for (i, n) in [72., 76., 79., 84.].into_iter().enumerate() {
                s.bell(n, i as f64 * 0.14, 1.6, 0.25);
            }
            for (i, n) in [84., 88., 91.].into_iter().enumerate() {
                s.bell(n, 0.7 + i as f64 * 0.05, 2.2, 0.18);
            }
        }
        (Spooky, Round) => {
            s.tone(Tone {
                to: note(57.),
                wave: Wave::Sawtooth,
                attack: 0.3,
                lowpass: Some(700.),
                ..Tone::new(45., 0., 1.6, 0.16)
            });
            s.tone(Tone {
                vibrato: 12.,
                attack: 0.3,
                ..Tone::new(76., 0.4, 1.8, 0.12)
            });
        }
        (Spooky, Answer) => s.tone(Tone {
            to: note(81.),
            vibrato: 14.,
            attack: 0.08,
            ..Tone::new(69., 0., 1.2, 0.18)
        }),
        (Spooky, Reveal) => {
            for n in [57., 60., 63.] {
                s.tone(Tone {
                    wave: Wave::Sawtooth,
                    attack: 0.15,
                    lowpass: Some(900.),
                    ..Tone::new(n, 0., 1.4, 0.08)
                });
            }
        }
        (Spooky, Winner) => {
            for n in [45., 48., 52., 57.] {
                s.tone(Tone {
                    wave: Wave::Sawtooth,
                    attack: 0.4,
                    lowpass: Some(1100.),
                    ..Tone::new(n, 0., 2.6, 0.07)
                });
            }
            s.tone(Tone {
                to: note(93.),
                vibrato: 18.,
                attack: 0.2,
                ..Tone::new(69., 0.3, 2.2, 0.16)
            });
            s.noise(1.6, 0.8, 0.05, 1500.);
        }
        (Festive, Round) => {
            s.noise(0., 0.12, 0.15, 6000.);
            s.noise(0.15, 0.12, 0.15, 6000.);
            s.bell(79., 0.05, 1.2, 0.25);
            s.bell(84., 0.25, 1.4, 0.25);
        }
        (Festive, Answer) => {
            s.noise(0., 0.1, 0.15, 6000.);
            s.bell(88., 0., 1.4, 0.28);
        }
        (Festive, Reveal) => {
            for start in [0., 0.12, 0.24] {
                s.noise(start, 0.1, 0.1, 6000.);
            }
            s.bell(81., 0.3, 1.2, 0.25);
        }
        (Festive, Winner) => {
            for (n, start) in [76., 76., 76., 76., 76., 76., 76., 79., 72., 74., 76.]
                .into_iter()
                .zip([0., 0.18, 0.36, 0.72, 0.9, 1.08, 1.44, 1.62, 1.8, 1.98, 2.16])
            {
                s.bell(n + 12., start, 0.6, 0.18);
            }
            for start in [0., 0.36, 0.72, 1.08, 1.44] {
                s.noise(start, 0.12, 0.09, 6000.);
            }
        }
        (Arcade, Round) => {
            for (i, n) in [60., 64., 67., 72.].into_iter().enumerate() {
                s.tone(Tone {
                    wave: Wave::Square,
                    ..Tone::new(n, i as f64 * 0.07, 0.12, 0.08)
                });
            }
        }
        (Arcade, Answer) => {
            for (n, start, length) in [(84., 0., 0.08), (91., 0.08, 0.22)] {
                s.tone(Tone {
                    wave: Wave::Square,
                    ..Tone::new(n, start, length, 0.09)
                });
            }
        }
        (Arcade, Reveal) => s.tone(Tone {
            wave: Wave::Square,
            to: note(84.),
            ..Tone::new(48., 0., 0.35, 0.07)
        }),
        (Arcade, Winner) => {
            for (n, start) in [
                (72., 0.),
                (72., 0.12),
                (72., 0.24),
                (72., 0.4),
                (68., 0.56),
                (70., 0.72),
                (72., 0.88),
                (70., 1.04),
                (72., 1.16),
            ] {
                s.tone(Tone {
                    wave: Wave::Square,
                    ..Tone::new(n, start, 0.16, 0.08)
                });
            }
            s.tone(Tone {
                wave: Wave::Triangle,
                ..Tone::new(48., 0., 1.4, 0.12)
            });
        }
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        quiz::model::Settings,
        show::{Intro, Role, Show},
    };

    // sounds.test.js's pure routing case; presence expiry is owned by the caller.
    #[test]
    fn presenter_owns_audio_until_it_leaves() {
        let intro = Intro {
            has_rules: false,
            has_prizes: false,
        };
        let mut tv = Show::new(Role::Main, Settings::default(), intro);
        let mut presenter = Show::new(Role::Presenter, Settings::default(), intro);
        for here in [false, true, false] {
            tv.note_presenter(here);
            presenter.note_presenter(here);
            assert_eq!(tv.plays_sounds(), !here);
            assert!(presenter.plays_sounds());
        }
    }
    #[test]
    fn all_pack_scores_have_expected_voice_counts_and_levels() {
        for (pack, counts, level) in [
            (SoundPack::Chimes, [4, 4, 3, 14], 1.),
            (SoundPack::Spooky, [2, 1, 3, 6], 1.7),
            (SoundPack::Festive, [6, 3, 5, 27], 1.),
            (SoundPack::Arcade, [4, 2, 1, 10], 1.4),
        ] {
            for (sound, count) in Sound::ALL.into_iter().zip(counts) {
                let score = score(pack, sound);
                assert_eq!(score.level, level);
                assert_eq!(score.voices.len(), count);
            }
        }
    }
    #[test]
    fn bell_overtones_and_envelopes_match_js() {
        let s = score(SoundPack::Chimes, Sound::Answer);
        let Voice::Tone(base) = s.voices[0] else {
            panic!()
        };
        let Voice::Tone(overtone) = s.voices[1] else {
            panic!()
        };
        assert_eq!(base.from, note(84.));
        assert_eq!(
            (base.start, base.length, base.gain, base.attack),
            (0., 1.6, 0.3, 0.01)
        );
        assert_eq!(overtone.from, base.from * 2.76);
        assert_eq!((overtone.length, overtone.gain), (0.8, 0.09));
    }
    #[test]
    fn glide_vibrato_and_filtered_noise_match_js() {
        let s = score(SoundPack::Spooky, Sound::Winner);
        let Voice::Tone(t) = s.voices[4] else {
            panic!()
        };
        assert_eq!(
            (t.from, t.to, t.start, t.length, t.vibrato, t.attack),
            (440., note(93.), 0.3, 2.2, 18., 0.2)
        );
        assert_eq!(
            s.voices[5],
            Voice::Noise(Noise {
                start: 1.6,
                length: 0.8,
                gain: 0.05,
                highpass: 1500.
            })
        );
        assert_eq!(noise_sample(0., 0, 4), -1.);
        assert_eq!(noise_sample(1., 3, 4), 0.25);
        assert_eq!(noise_sample(0.5, 0, 4), 0.);
    }
}
