//! Web Audio renderer for quiz-core's scores. No audio files are used by the JS engine.
use leptos::prelude::*;
use quiz_core::{
    quiz::model::Settings,
    sounds::{Noise, Sound, Tone, Voice, Wave, noise_sample, score},
    theme::{Theme, format::SoundPack},
};
use std::cell::RefCell;
use wasm_bindgen::{JsCast, JsValue, closure::Closure};
use web_sys::{AudioContext, AudioContextState, BiquadFilterType, GainNode, OscillatorType};

#[derive(Clone, Copy)]
struct Player {
    owns_audio: Signal<bool>,
    settings: RwSignal<Settings>,
    theme: Signal<Theme>,
}
thread_local! {
    static PLAYER: RefCell<Option<Player>> = const { RefCell::new(None) };
    static AUDIO: RefCell<Option<AudioContext>> = const { RefCell::new(None) };
    // One pair of listeners for the lifetime of this browser window, including previews.
    static UNLOCK: RefCell<Option<Closure<dyn FnMut(web_sys::Event)>>> = const { RefCell::new(None) };
}

/// Install once in LoadedShow after its settings and theme contexts are provided:
/// `provide_player(Signal::derive(move || reducer.with_value(|r| r.plays_sounds())));`
/// The caller also owns presenter heartbeat expiry, as specified by the show reducer.
/// Capturing signals here lets scene effects, callbacks and timers call `play` directly.
pub fn provide_player(owns_audio: Signal<bool>) {
    let player = Player {
        owns_audio,
        settings: crate::settings::use_settings().settings,
        theme: expect_context::<Signal<Theme>>(),
    };
    PLAYER.with(|p| *p.borrow_mut() = Some(player));
    install_unlock();
    on_cleanup(|| PLAYER.with(|p| *p.borrow_mut() = None));
}

/// Play the current theme's cue only in the owning, enabled show window.
/// Without an installed show player this is deliberately silent.
pub fn play(sound: Sound) {
    let Some(player) = PLAYER.with(|p| *p.borrow()) else {
        return;
    };
    if !player.owns_audio.get_untracked() {
        return;
    }
    let settings = player.settings.get_untracked();
    if settings
        .fields
        .get("sounds")
        .and_then(serde_json::Value::as_bool)
        == Some(false)
    {
        return;
    }
    if let Some(pack) = player.theme.get_untracked().sounds.pack {
        let volume = settings
            .fields
            .get("soundVolume")
            .and_then(serde_json::Value::as_f64)
            .unwrap_or(0.6);
        let _ = preview(pack, sound, volume);
    }
}

/// Explicit local audition for settings and the theme editor, independent of show ownership.
/// The settings button should call `preview(pack, Sound::Answer, volume)`.
/// Returns false when audio is unavailable, volume is zero, or Web Audio setup fails.
/// As in JS, true means scheduled, not proof that autoplay policy allowed audible output.
pub fn preview(pack: SoundPack, sound: Sound, volume: f64) -> bool {
    install_unlock();
    if !volume.is_finite() || volume <= 0. {
        return false;
    }
    let Some(context) = audio() else { return false };
    resume(&context);
    render(&context, pack, sound, volume).is_ok()
}

fn audio() -> Option<AudioContext> {
    if web_sys::window().is_none() {
        return None;
    }
    AUDIO.with(|stored| {
        let mut stored = stored.borrow_mut();
        if stored.is_none() {
            *stored = AudioContext::new().ok();
        }
        stored.clone()
    })
}
fn resume(context: &AudioContext) {
    if context.state() == AudioContextState::Suspended
        && let Ok(promise) = context.resume()
    {
        wasm_bindgen_futures::spawn_local(async move {
            let _ = wasm_bindgen_futures::JsFuture::from(promise).await;
        });
    }
}
fn install_unlock() {
    UNLOCK.with(|stored| {
        let mut stored = stored.borrow_mut();
        if stored.is_some() {
            return;
        }
        let Some(window) = web_sys::window() else {
            return;
        };
        let unlock = Closure::<dyn FnMut(web_sys::Event)>::new(|_| {
            if let Some(context) = audio() {
                resume(&context);
            }
        });
        for event in ["pointerdown", "keydown"] {
            let _ = window.add_event_listener_with_callback(event, unlock.as_ref().unchecked_ref());
        }
        *stored = Some(unlock);
    });
}
fn render(
    context: &AudioContext,
    pack: SoundPack,
    sound: Sound,
    volume: f64,
) -> Result<(), JsValue> {
    let score = score(pack, sound);
    let out = context.create_gain()?;
    out.gain().set_value((volume * score.level) as f32);
    out.connect_with_audio_node(&context.destination())?;
    // Use one clock snapshot so every voice stays aligned even during graph construction.
    let now = context.current_time();
    for voice in score.voices {
        match voice {
            Voice::Tone(tone) => tone_voice(context, &out, now, tone)?,
            Voice::Noise(noise) => noise_voice(context, &out, now, noise)?,
        }
    }
    Ok(())
}
fn tone_voice(context: &AudioContext, out: &GainNode, now: f64, tone: Tone) -> Result<(), JsValue> {
    let start = now + tone.start;
    let osc = context.create_oscillator()?;
    osc.set_type(match tone.wave {
        Wave::Sine => OscillatorType::Sine,
        Wave::Sawtooth => OscillatorType::Sawtooth,
        Wave::Square => OscillatorType::Square,
        Wave::Triangle => OscillatorType::Triangle,
    });
    osc.frequency().set_value_at_time(tone.from as f32, start)?;
    if tone.to != tone.from {
        osc.frequency()
            .exponential_ramp_to_value_at_time(tone.to as f32, start + tone.length * 0.8)?;
    }
    if tone.vibrato != 0. {
        let lfo = context.create_oscillator()?;
        let depth = context.create_gain()?;
        lfo.frequency().set_value(6.);
        depth.gain().set_value(tone.vibrato as f32);
        lfo.connect_with_audio_node(&depth)?;
        depth.connect_with_audio_param(&osc.frequency())?;
        lfo.start_with_when(start)?;
        lfo.stop_with_when(start + tone.length + 0.1)?;
    }
    let env = context.create_gain()?;
    env.gain().set_value_at_time(0., start)?;
    env.gain()
        .linear_ramp_to_value_at_time(tone.gain as f32, start + tone.attack)?;
    env.gain()
        .exponential_ramp_to_value_at_time(0.0001, start + tone.length)?;
    if let Some(frequency) = tone.lowpass {
        let filter = context.create_biquad_filter()?;
        filter.set_type(BiquadFilterType::Lowpass);
        filter.frequency().set_value(frequency as f32);
        osc.connect_with_audio_node(&filter)?;
        filter.connect_with_audio_node(&env)?;
    } else {
        osc.connect_with_audio_node(&env)?;
    }
    env.connect_with_audio_node(out)?;
    osc.start_with_when(start)?;
    osc.stop_with_when(start + tone.length + 0.05)?;
    Ok(())
}
fn noise_voice(
    context: &AudioContext,
    out: &GainNode,
    now: f64,
    noise: Noise,
) -> Result<(), JsValue> {
    let frames = (f64::from(context.sample_rate()) * noise.length).ceil() as u32;
    let buffer = context.create_buffer(1, frames, context.sample_rate())?;
    let samples: Vec<f32> = (0..frames)
        .map(|i| noise_sample(js_sys::Math::random(), i as usize, frames as usize))
        .collect();
    buffer.copy_to_channel(&samples, 0)?;
    let source = context.create_buffer_source()?;
    source.set_buffer(Some(&buffer));
    let filter = context.create_biquad_filter()?;
    filter.set_type(BiquadFilterType::Highpass);
    filter.frequency().set_value(noise.highpass as f32);
    let env = context.create_gain()?;
    env.gain().set_value(noise.gain as f32);
    source.connect_with_audio_node(&filter)?;
    filter.connect_with_audio_node(&env)?;
    env.connect_with_audio_node(out)?;
    source.start_with_when(now + noise.start)?;
    Ok(())
}
