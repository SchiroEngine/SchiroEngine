//! Tests for `schiro_input::InputSystem`.

use schiro_input::InputSystem;

#[test]
fn input_system_default_creates_empty_state() {
    let sys = InputSystem::default();
    let mouse = sys.mouse();
    assert_eq!(mouse.position, glam::Vec2::ZERO);
    assert!(!mouse.left);
    assert!(!mouse.right);
}

#[test]
fn multiple_systems_are_independent() {
    let a = InputSystem::new();
    let b = InputSystem::new();
    let _ = (a, b);
}
