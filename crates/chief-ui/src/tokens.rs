#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ColorToken {
    pub name: &'static str,
    pub value: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MotionPreset {
    pub name: &'static str,
    pub stiffness: f32,
    pub damping: f32,
    pub mass: f32,
}

pub const CHIEF_BG_0: &str = "#0c0e11";
pub const CHIEF_BG_1: &str = "#14171b";
pub const CHIEF_BG_2: &str = "#1b1f24";
pub const CHIEF_BORDER: &str = "rgba(255,255,255,0.06)";

pub const CHIEF_FG_PRIMARY: &str = "#e8ecf1";
pub const CHIEF_FG_SECONDARY: &str = "#98a2b0";
pub const CHIEF_FG_TERTIARY: &str = "#5c6470";

pub const CHIEF_AMBER: &str = "#d4a574";
pub const CHIEF_BLUE: &str = "#5294e2";
pub const CHIEF_GREEN: &str = "#73c991";
pub const CHIEF_COPPER: &str = "#c68858";
pub const CHIEF_GRAY: &str = "#8a8f99";

pub const CHIEF_SRC_AMBER: &str = "#d4a574";
pub const CHIEF_SRC_BLUE: &str = "#7a9cc6";
pub const CHIEF_SRC_GREEN: &str = "#8fb093";
pub const CHIEF_SRC_COPPER: &str = "#c68858";
pub const CHIEF_SRC_GRAY: &str = "#8a8f99";
pub const CHIEF_SRC_TEAL: &str = "#8ec0b8";
pub const CHIEF_SRC_VIOLET: &str = "#9d89b8";
pub const CHIEF_SRC_PEACH: &str = "#e0b090";

pub const CHIEF_RADIUS_CARD: &str = "10px";
pub const CHIEF_RADIUS_BUTTON: &str = "6px";
pub const CHIEF_RADIUS_PILL: &str = "999px";

pub const CHIEF_FONT_DISPLAY: &str = "Inter Tight";
pub const CHIEF_FONT_BODY: &str = "Inter";
pub const CHIEF_FONT_MONO: &str = "JetBrains Mono";

pub const CHIEF_SIZE_HEADLINE: &str = "34pt";
pub const CHIEF_SIZE_SECTION: &str = "11pt";
pub const CHIEF_SIZE_BODY: &str = "13pt";
pub const CHIEF_SIZE_META: &str = "11pt";
pub const CHIEF_TRACKING_SECTION: &str = "0.08em";

pub const CHIEF_SPACE_XS: &str = "4px";
pub const CHIEF_SPACE_S: &str = "8px";
pub const CHIEF_SPACE_M: &str = "12px";
pub const CHIEF_SPACE_L: &str = "20px";
pub const CHIEF_SPACE_XL: &str = "40px";

pub const COLOR_TOKENS: &[ColorToken] = &[
    ColorToken {
        name: "--chief-bg-0",
        value: CHIEF_BG_0,
    },
    ColorToken {
        name: "--chief-bg-1",
        value: CHIEF_BG_1,
    },
    ColorToken {
        name: "--chief-bg-2",
        value: CHIEF_BG_2,
    },
    ColorToken {
        name: "--chief-border",
        value: CHIEF_BORDER,
    },
    ColorToken {
        name: "--chief-fg-primary",
        value: CHIEF_FG_PRIMARY,
    },
    ColorToken {
        name: "--chief-fg-secondary",
        value: CHIEF_FG_SECONDARY,
    },
    ColorToken {
        name: "--chief-fg-tertiary",
        value: CHIEF_FG_TERTIARY,
    },
    ColorToken {
        name: "--chief-amber",
        value: CHIEF_AMBER,
    },
    ColorToken {
        name: "--chief-blue",
        value: CHIEF_BLUE,
    },
    ColorToken {
        name: "--chief-green",
        value: CHIEF_GREEN,
    },
    ColorToken {
        name: "--chief-copper",
        value: CHIEF_COPPER,
    },
    ColorToken {
        name: "--chief-gray",
        value: CHIEF_GRAY,
    },
    ColorToken {
        name: "--chief-src-amber",
        value: CHIEF_SRC_AMBER,
    },
    ColorToken {
        name: "--chief-src-blue",
        value: CHIEF_SRC_BLUE,
    },
    ColorToken {
        name: "--chief-src-green",
        value: CHIEF_SRC_GREEN,
    },
    ColorToken {
        name: "--chief-src-copper",
        value: CHIEF_SRC_COPPER,
    },
    ColorToken {
        name: "--chief-src-gray",
        value: CHIEF_SRC_GRAY,
    },
    ColorToken {
        name: "--chief-src-teal",
        value: CHIEF_SRC_TEAL,
    },
    ColorToken {
        name: "--chief-src-violet",
        value: CHIEF_SRC_VIOLET,
    },
    ColorToken {
        name: "--chief-src-peach",
        value: CHIEF_SRC_PEACH,
    },
];

pub const MOTION_CARD_REVEAL: MotionPreset = MotionPreset {
    name: "card-reveal",
    stiffness: 220.0,
    damping: 28.0,
    mass: 1.0,
};

pub const MOTION_TRUST_GROW: MotionPreset = MotionPreset {
    name: "trust-grow",
    stiffness: 180.0,
    damping: 22.0,
    mass: 1.0,
};

pub const MOTION_OMNIBAR_SUMMON: MotionPreset = MotionPreset {
    name: "omnibar-summon",
    stiffness: 260.0,
    damping: 30.0,
    mass: 0.9,
};

pub const MOTION_CEREMONY_BEGIN: MotionPreset = MotionPreset {
    name: "ceremony-begin",
    stiffness: 120.0,
    damping: 20.0,
    mass: 1.2,
};

pub const MOTION_PRESETS: &[MotionPreset] = &[
    MOTION_CARD_REVEAL,
    MOTION_TRUST_GROW,
    MOTION_OMNIBAR_SUMMON,
    MOTION_CEREMONY_BEGIN,
];
