pub mod tokens;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum PaneVariant {
    #[default]
    Content,
    Widget,
    Overlay,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum PaneTint {
    #[default]
    Warm,
    Dark,
    Neutral,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SourceColor {
    Amber,
    Blue,
    Green,
    Copper,
    Gray,
    Teal,
    Violet,
    Peach,
}

impl SourceColor {
    pub const ALL: [Self; 8] = [
        Self::Amber,
        Self::Blue,
        Self::Green,
        Self::Copper,
        Self::Gray,
        Self::Teal,
        Self::Violet,
        Self::Peach,
    ];

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Amber => "amber",
            Self::Blue => "blue",
            Self::Green => "green",
            Self::Copper => "copper",
            Self::Gray => "gray",
            Self::Teal => "teal",
            Self::Violet => "violet",
            Self::Peach => "peach",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum BadgeVariant {
    NeedsYou,
    Review,
    Handled,
    Ceremony,
    Anchor,
}

impl BadgeVariant {
    pub const ALL: [Self; 5] = [
        Self::NeedsYou,
        Self::Review,
        Self::Handled,
        Self::Ceremony,
        Self::Anchor,
    ];

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::NeedsYou => "needs-you",
            Self::Review => "review",
            Self::Handled => "handled",
            Self::Ceremony => "ceremony",
            Self::Anchor => "anchor",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PaneProps {
    #[serde(default)]
    pub variant: PaneVariant,
    #[serde(default)]
    pub tint: PaneTint,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceAvatarProps {
    pub color: SourceColor,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BadgeProps {
    pub variant: BadgeVariant,
    pub children: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RowProps {
    pub sender: String,
    pub subject: String,
    pub snippet: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub badge: Option<BadgeProps>,
    pub timestamp: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TabChip {
    pub id: String,
    pub label: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub count: Option<u32>,
    #[serde(default)]
    pub highlighted: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TabBarProps {
    pub chips: Vec<TabChip>,
    pub selected: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommandPillAction {
    pub id: String,
    pub label: String,
    #[serde(default)]
    pub disabled: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommandPillProps {
    pub actions: Vec<CommandPillAction>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WidgetProps {
    pub title: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DotTrailProps {
    pub label: String,
    pub filled: u8,
    #[serde(default = "default_dot_total")]
    pub total: u8,
    pub color: SourceColor,
}

fn default_dot_total() -> u8 {
    5
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SparklineProps {
    pub data: Vec<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub highlight_index: Option<usize>,
    pub label: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct HoldRingProps {
    pub progress: f32,
}

impl HoldRingProps {
    pub fn normalized_progress(self) -> f32 {
        if self.progress.is_nan() {
            0.0
        } else {
            self.progress.clamp(0.0, 1.0)
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CeremonyEvidenceItem {
    pub label: String,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CeremonyCardProps {
    pub label: String,
    pub headline: String,
    pub subtitle: String,
    pub evidence: Vec<CeremonyEvidenceItem>,
    pub provenance: Vec<CeremonyEvidenceItem>,
    pub rollback: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OmnibarResultType {
    Memory,
    Card,
    Agent,
    Pack,
    Action,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OmnibarResult {
    pub id: String,
    #[serde(rename = "type")]
    pub result_type: OmnibarResultType,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subtitle: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OmnibarPaletteProps {
    pub query: String,
    pub results: Vec<OmnibarResult>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InboxItem {
    pub id: String,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub badge: Option<BadgeProps>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InboxSection {
    pub id: String,
    pub title: String,
    pub items: Vec<InboxItem>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InboxDrawerProps {
    pub sections: Vec<InboxSection>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ChatRole {
    User,
    Assistant,
    Agent,
    System,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChatMessage {
    pub id: String,
    pub role: ChatRole,
    pub sender: String,
    pub body: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChatPaneProps {
    pub messages: Vec<ChatMessage>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MenubarGroup {
    pub label: String,
    pub items: Vec<MenubarItem>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MenubarItem {
    pub action: String,
    pub label: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shortcut: Option<String>,
    #[serde(default)]
    pub disabled: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MenubarExtra {
    pub label: String,
    #[serde(default)]
    pub active: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MenubarProps {
    pub groups: Vec<MenubarGroup>,
    pub extras: Vec<MenubarExtra>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BoxGrid {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub columns: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rows: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gap: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum BoxFlexDirection {
    Row,
    Column,
    RowReverse,
    ColumnReverse,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BoxFlex {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub direction: Option<BoxFlexDirection>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gap: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BoxProps {
    #[serde(default = "default_box_as")]
    pub as_element: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub grid: Option<BoxGrid>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flex: Option<BoxFlex>,
}

fn default_box_as() -> String {
    "div".to_owned()
}
