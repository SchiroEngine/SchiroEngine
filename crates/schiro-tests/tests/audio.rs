//! Smoke tests for `schiro_audio::AudioSystem`.

use schiro_audio::AudioSystem;

#[test]
fn audio_system_new_marks_initialized() {
    let sys = AudioSystem::new();
    assert!(sys.is_initialized());
}

#[test]
fn audio_system_default_is_initialized() {
    let sys = AudioSystem::default();
    assert!(sys.is_initialized());
}

#[test]
fn multiple_systems_are_independent() {
    let a = AudioSystem::new();
    let b = AudioSystem::new();
    assert!(a.is_initialized());
    assert!(b.is_initialized());
}
