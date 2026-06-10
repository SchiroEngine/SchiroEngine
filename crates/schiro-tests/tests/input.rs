//! Tests for `schiro_input::InputState` and `InputSystem` defaults.

use schiro_input::{InputState, InputSystem};

#[test]
fn input_state_default_is_zero() {
    let s = InputState::default();
    assert_eq!(s.cursor_position, glam::Vec2::ZERO);
    assert_eq!(s.cursor_delta, glam::Vec2::ZERO);
    assert_eq!(s.scroll_delta, glam::Vec2::ZERO);
}

#[test]
fn input_state_clone_copies_all_fields() {
    let s = InputState {
        cursor_position: glam::Vec2::new(1.0, 2.0),
        cursor_delta: glam::Vec2::new(0.5, 0.0),
        scroll_delta: glam::Vec2::new(0.0, -3.0),
    };
    let c = s.clone();
    assert_eq!(c.cursor_position, s.cursor_position);
    assert_eq!(c.cursor_delta, s.cursor_delta);
    assert_eq!(c.scroll_delta, s.scroll_delta);
}

#[test]
fn input_system_default_state_is_zero() {
    let sys = InputSystem::default();
    let s = sys.state();
    assert_eq!(s.cursor_position, glam::Vec2::ZERO);
}

#[test]
fn input_system_process_event_does_not_panic() {
    // The current implementation is a no-op stub, but the API must
    // be callable with any WindowEvent without panicking. We pass
    // a synthetic WindowEvent to exercise the path.
    let sys = InputSystem::new();
    use winit::event::WindowEvent;
    sys.process_window_event(&WindowEvent::CloseRequested);
}
