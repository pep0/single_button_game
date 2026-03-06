/// WASM-only: resumes any suspended Web AudioContext on user interaction.
///
/// Web browsers (Chrome, Safari on iOS) implement an autoplay policy that
/// suspends all `AudioContext` instances until a user gesture has occurred.
/// Bevy's audio backend creates its `AudioContext` lazily, so it may be
/// suspended by the time the first sound fires.
///
/// The primary fix is the JavaScript patch in `index.html` which wraps the
/// `AudioContext` constructor and resumes all tracked instances on every
/// `pointerdown`, `touchstart`, and `keydown` event.
///
/// This Rust module provides a belt-and-suspenders backup: a Bevy system
/// that calls a JS `eval` snippet to resume tracked contexts whenever user
/// input is detected.  This handles the edge case where the AudioContext is
/// created between two JS event firings.
use bevy::prelude::*;

#[cfg(target_arch = "wasm32")]
mod inner {
    use bevy::prelude::*;

    /// One-shot flag: set after the first resume attempt following a gesture.
    #[derive(Resource, Default)]
    pub struct AudioResumeState {
        pub attempted: bool,
    }

    /// Bevy system: on the first frame where a user gesture is detected,
    /// call into JS to resume any suspended AudioContext instances tracked
    /// by the `index.html` patch.
    pub fn resume_audio_on_interaction(
        mut state: ResMut<AudioResumeState>,
        keyboard: Res<ButtonInput<KeyCode>>,
        touches: Res<Touches>,
    ) {
        if state.attempted {
            return;
        }

        let gesture_detected = keyboard.get_just_pressed().count() > 0
            || touches.any_just_pressed();

        if !gesture_detected {
            return;
        }

        state.attempted = true;

        // The index.html patch wraps AudioContext so every created instance
        // is pushed into a closure-scoped array and resumed on pointer/key
        // events.  This eval call is a Rust-side trigger for the same
        // resume logic, covering the exact frame of the first Bevy input.
        let _ = js_sys::eval(
            r#"(function() {
                // Attempt to resume via the patch installed by index.html.
                // The patch sets up event listeners that call ctx.resume()
                // on all tracked contexts; here we fire that same logic
                // immediately from the Rust/WASM side.
                if (typeof window !== 'undefined') {
                    var CtxClass = window.AudioContext || window.webkitAudioContext;
                    if (CtxClass && CtxClass._instances) {
                        CtxClass._instances.forEach(function(c) {
                            if (c.state === 'suspended') c.resume();
                        });
                    }
                }
            })()"#,
        );
    }
}

#[cfg(target_arch = "wasm32")]
pub use inner::*;

/// Plugin that registers the WASM audio resume system.
/// On non-WASM targets this plugin is a no-op.
pub struct WasmAudioPlugin;

impl Plugin for WasmAudioPlugin {
    fn build(&self, app: &mut App) {
        #[cfg(target_arch = "wasm32")]
        {
            app.init_resource::<inner::AudioResumeState>()
                .add_systems(Update, inner::resume_audio_on_interaction);
        }
        let _ = app; // suppress unused warning on non-wasm
    }
}
