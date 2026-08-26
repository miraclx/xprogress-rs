use crate::pulse::Pulse;

fn count(line: &str, glyph: char) -> usize {
    line.chars().filter(|&c| c == glyph).count()
}

#[test]
fn block_length_is_constant_and_width_is_exact() {
    let pulse = Pulse::new(6);
    for phase in 0..30u64 {
        let line = pulse.render(20, phase);
        assert_eq!(
            line.chars().count(),
            20,
            "phase {phase} is exactly the width"
        );
        assert_eq!(count(&line, '█'), 6, "phase {phase} keeps the block length");
    }
}

#[test]
fn block_starts_left_and_reaches_right() {
    let pulse = Pulse::new(4); // width 10, travel 6
    assert!(
        pulse.render(10, 0).starts_with('█'),
        "phase 0 sits at the left"
    );
    assert!(
        pulse.render(10, 6).ends_with('█'),
        "phase == travel sits at the right"
    );
}

#[test]
fn it_bounces_back_rather_than_jumping() {
    let pulse = Pulse::new(4); // width 10, travel 6, period 12
    assert_eq!(
        pulse.render(10, 12),
        pulse.render(10, 0),
        "a full period returns to the start"
    );
    assert_eq!(
        pulse.render(10, 8),
        pulse.render(10, 4),
        "returning passes through the same spots"
    );
}

#[test]
fn a_block_wider_than_the_bar_fills_it() {
    assert_eq!(count(&Pulse::new(50).render(8, 3), '█'), 8);
}
