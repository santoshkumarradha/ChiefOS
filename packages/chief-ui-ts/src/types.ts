import type { CSSProperties, ElementType, ReactNode } from "react";
import type { BadgeVariant, PaneTint, PaneVariant, SourceColor } from "./guards.js";

export interface ChiefPrimitiveProps {
  id?: string;
  children?: ReactNode;
}

export interface PaneProps extends ChiefPrimitiveProps {
  variant?: PaneVariant;
  tint?: PaneTint;
}

export interface SourceAvatarProps {
  color: SourceColor;
  label?: string;
}

export interface BadgeProps extends ChiefPrimitiveProps {
  variant: BadgeVariant;
}

export interface RowProps {
  avatar: ReactNode;
  sender: string;
  subject: string;
  snippet: string;
  badge?: ReactNode;
  timestamp: string;
  onOpen?: () => void;
}

export interface SectionHeaderProps extends ChiefPrimitiveProps {}

export interface TabChip {
  id: string;
  label: string;
  count?: number;
  highlighted?: boolean;
}

export interface TabBarProps {
  chips: readonly TabChip[];
  selected: string;
  onSelect: (id: string) => void;
}

export interface CommandPillAction {
  id: string;
  label: string;
  icon?: ReactNode;
  disabled?: boolean;
  onSelect: () => void;
}

export interface CommandPillProps {
  actions: readonly CommandPillAction[];
}

export interface WidgetProps extends ChiefPrimitiveProps {
  title: string;
}

export interface DotTrailProps {
  label: string;
  filled: number;
  total?: number;
  color: SourceColor;
}

export interface SparklineProps {
  data: readonly number[];
  highlightIndex?: number;
  label: string;
}

export interface HoldRingProps {
  progress: number;
  label: string;
  onHold?: () => void;
  onRelease?: () => void;
}

export interface CeremonyEvidenceItem {
  label: string;
  value: ReactNode;
}

export interface CeremonyCardProps {
  label: string;
  headline: string;
  subtitle: string;
  evidence: readonly CeremonyEvidenceItem[];
  provenance: readonly CeremonyEvidenceItem[];
  rollback: string;
  onApprove: () => void;
}

export type OmnibarResultType = "Memory" | "Card" | "Agent" | "Pack" | "Action";

export interface OmnibarResult {
  id: string;
  type: OmnibarResultType;
  title: string;
  subtitle?: string;
}

export interface OmnibarPaletteProps {
  query: string;
  onQueryChange: (query: string) => void;
  results: readonly OmnibarResult[];
  onSelect: (result: OmnibarResult) => void;
}

export interface InboxItem {
  id: string;
  title: string;
  detail?: string;
  badge?: ReactNode;
  timestamp?: string;
  onOpen?: () => void;
}

export interface InboxSection {
  id: string;
  title: string;
  items: readonly InboxItem[];
}

export interface InboxDrawerProps {
  sections: readonly InboxSection[];
}

export interface ChatMessage {
  id: string;
  role: "user" | "assistant" | "agent" | "system";
  sender: string;
  body: ReactNode;
  timestamp?: string;
}

export interface ChatPaneProps {
  messages: readonly ChatMessage[];
  composer: ReactNode;
}

export interface MenubarProps extends ChiefPrimitiveProps {}

export interface MenubarGroupProps extends ChiefPrimitiveProps {
  label: string;
}

export interface MenubarItemProps extends ChiefPrimitiveProps {
  action: string;
  shortcut?: string;
  disabled?: boolean;
  onSelect?: () => void;
}

export interface MenubarExtraProps extends ChiefPrimitiveProps {
  label: string;
  active?: boolean;
}

export interface BoxGridOptions {
  columns?: string;
  rows?: string;
  gap?: string;
  align?: CSSProperties["alignItems"];
  justify?: CSSProperties["justifyContent"];
}

export interface BoxFlexOptions {
  direction?: "row" | "column" | "row-reverse" | "column-reverse";
  wrap?: "nowrap" | "wrap" | "wrap-reverse";
  gap?: string;
  align?: CSSProperties["alignItems"];
  justify?: CSSProperties["justifyContent"];
}

export interface BoxProps {
  as?: ElementType;
  grid?: BoxGridOptions;
  flex?: BoxFlexOptions;
  children?: ReactNode;
}
