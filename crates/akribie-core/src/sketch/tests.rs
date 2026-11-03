//! Testes do traço: determinismo, acabamento à mão e acabamento técnico.

use super::*;
use crate::geometry::Rect;
use crate::num::format as num;

#[test]
fn seed_is_stable_and_well_distributed() {
    assert_eq!(seed_for("api"), seed_for("api"));
    assert_ne!(seed_for("api"), seed_for("auth"));
    assert_ne!(seed_for("a"), seed_for("b"));
}

#[test]
fn rng_is_deterministic_per_seed() {
    let mut first = Rng::new(7);
    let mut second = Rng::new(7);
    for _ in 0..8 {
        assert_eq!(first.next_u32(), second.next_u32());
    }
    assert!((-1.0..1.0).contains(&Rng::new(7).signed()));
}

#[test]
fn generators_are_identical_for_the_same_seed() {
    let frame = Rect::new(0.0, 0.0, 100.0, 40.0);
    assert_eq!(
        rect(frame, 12.0, Ink::Rough(42)),
        rect(frame, 12.0, Ink::Rough(42))
    );
    assert_eq!(
        ellipse(frame, Ink::Rough(42)),
        ellipse(frame, Ink::Rough(42))
    );
    assert_ne!(
        rect(frame, 12.0, Ink::Rough(42)),
        rect(frame, 12.0, Ink::Rough(43))
    );
}

#[test]
fn paths_are_closed_and_use_no_scientific_notation() {
    let path = rect(Rect::new(10.25, 20.5, 100.0, 40.0), 12.0, Ink::Rough(1));
    assert!(path.contains('Z'));
    assert!(!path.contains('e'), "{path}");
    assert_eq!(num(1.2), "1.2");
    assert_eq!(num(-0.0), "0");
    assert_eq!(num(1.234), "1.23");
    assert_eq!(num(10.0), "10");
}

#[test]
fn exact_rect_is_geometry_with_arcs_and_a_single_pass() {
    let frame = Rect::new(10.0, 20.0, 100.0, 40.0);
    let path = rect(frame, 12.0, Ink::Exact);
    assert!(path.starts_with("M22 20"), "{path}");
    assert_eq!(path.matches('M').count(), 1, "{path}");
    assert_eq!(path.matches('A').count(), 4, "{path}");
    assert!(path.ends_with(" Z"), "{path}");
    assert_eq!(path, rect(frame, 12.0, Ink::Exact));
    // O traço à mão tem duas passagens; o exato, uma só.
    assert_eq!(rect(frame, 12.0, Ink::Rough(1)).matches('M').count(), 2);
}

#[test]
fn exact_shapes_use_arcs() {
    let frame = Rect::new(0.0, 0.0, 120.0, 60.0);
    assert!(ellipse(frame, Ink::Exact).contains('A'));
    assert!(cylinder(frame, Ink::Exact).contains('A'));
    assert!(cloud(frame, Ink::Exact).contains('C'));
    assert!(note_shape(frame, Ink::Exact).contains('L'));
}

#[test]
fn exact_ink_is_a_single_exact_pass() {
    assert_eq!(
        polyline(&[(0.0, 0.0), (10.0, 10.0)], Trace::Open, Ink::Exact),
        "M0 0 L10 10"
    );
}

#[test]
fn hatch_covers_the_whole_rect() {
    let path = hatch(Rect::new(0.0, 0.0, 80.0, 40.0), 10.0, Tilt::Forward);
    let count = path.matches('M').count();
    assert!(count >= 8, "{count}");
}

#[test]
fn arrow_head_points_at_the_tip() {
    let path = arrow_head((100.0, 0.0), (0.0, 0.0));
    assert!(path.contains("100 0"), "{path}");
    assert!(path.ends_with('Z'));
}
