use chief_ui::tokens;
use chief_ui::{BadgeVariant, HoldRingProps, PaneProps, PaneTint, PaneVariant, SourceColor};

#[test]
fn source_color_palette_is_closed() {
    let names: Vec<&'static str> = SourceColor::ALL
        .into_iter()
        .map(SourceColor::as_str)
        .collect();
    assert_eq!(
        names,
        ["amber", "blue", "green", "copper", "gray", "teal", "violet", "peach"]
    );
}

#[test]
fn badge_variants_are_closed() {
    let names: Vec<&'static str> = BadgeVariant::ALL
        .into_iter()
        .map(BadgeVariant::as_str)
        .collect();
    assert_eq!(
        names,
        ["needs-you", "review", "handled", "ceremony", "anchor"]
    );
}

#[test]
fn token_constants_match_brand_spec() {
    assert_eq!(tokens::CHIEF_BG_0, "#0c0e11");
    assert_eq!(tokens::CHIEF_BG_1, "#14171b");
    assert_eq!(tokens::CHIEF_BG_2, "#1b1f24");
    assert_eq!(tokens::CHIEF_FG_PRIMARY, "#e8ecf1");
    assert_eq!(tokens::CHIEF_AMBER, "#d4a574");
    assert_eq!(tokens::CHIEF_BLUE, "#5294e2");
    assert_eq!(tokens::CHIEF_GREEN, "#73c991");
    assert_eq!(tokens::CHIEF_COPPER, "#c68858");
    assert_eq!(tokens::CHIEF_GRAY, "#8a8f99");
}

#[test]
fn motion_presets_match_binding_values() {
    assert_eq!(tokens::MOTION_CARD_REVEAL.stiffness, 220.0);
    assert_eq!(tokens::MOTION_CARD_REVEAL.damping, 28.0);
    assert_eq!(tokens::MOTION_TRUST_GROW.stiffness, 180.0);
    assert_eq!(tokens::MOTION_OMNIBAR_SUMMON.mass, 0.9);
    assert_eq!(tokens::MOTION_CEREMONY_BEGIN.mass, 1.2);
}

#[test]
fn serde_uses_the_public_kebab_case_contract() {
    let props = PaneProps {
        variant: PaneVariant::Overlay,
        tint: PaneTint::Neutral,
    };
    let json = serde_json::to_string(&props).expect("serialize pane props");
    assert_eq!(json, r#"{"variant":"overlay","tint":"neutral"}"#);
}

#[test]
fn hold_ring_progress_is_bounded() {
    assert_eq!(HoldRingProps { progress: -0.2 }.normalized_progress(), 0.0);
    assert_eq!(HoldRingProps { progress: 0.4 }.normalized_progress(), 0.4);
    assert_eq!(HoldRingProps { progress: 4.0 }.normalized_progress(), 1.0);
    assert_eq!(
        HoldRingProps { progress: f32::NAN }.normalized_progress(),
        0.0
    );
}
