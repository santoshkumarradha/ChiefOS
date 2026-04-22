import { createElement } from "react";
import type { ReactElement } from "react";
import type {
  BadgeProps,
  BoxProps,
  CeremonyCardProps,
  ChatPaneProps,
  CommandPillProps,
  DotTrailProps,
  HoldRingProps,
  InboxDrawerProps,
  MenubarExtraProps,
  MenubarGroupProps,
  MenubarItemProps,
  MenubarProps,
  OmnibarPaletteProps,
  PaneProps,
  RowProps,
  SectionHeaderProps,
  SourceAvatarProps,
  SparklineProps,
  TabBarProps,
  WidgetProps,
} from "./types.js";
import { clampProgress } from "./guards.js";

function h(type: unknown, props: Record<string, unknown> | null, ...children: unknown[]): ReactElement | null {
  const make = createElement as (...args: unknown[]) => ReactElement | null;
  return make(type, props, ...children);
}

function keyActivate(event: unknown, action: () => void): void {
  const keyboard = event as { key?: string; preventDefault?: () => void };
  if (keyboard.key === "Enter" || keyboard.key === " ") {
    keyboard.preventDefault?.();
    action();
  }
}

export function Pane({ id, variant = "content", tint = "warm", children }: PaneProps) {
  return h(
    "section",
    {
      id,
      className: "chief-pane",
      "data-chief": "pane",
      "data-variant": variant,
      "data-tint": tint,
    },
    children,
  );
}

export function SourceAvatar({ color, label }: SourceAvatarProps) {
  return h("span", {
    className: "chief-source-avatar",
    "data-chief": "source-avatar",
    "data-color": color,
    "aria-label": label ?? `${color} source`,
    role: "img",
  });
}

export function Badge({ id, variant, children }: BadgeProps) {
  return h(
    "span",
    {
      id,
      className: "chief-badge",
      "data-chief": "badge",
      "data-variant": variant,
    },
    children,
  );
}

export function Row({ avatar, sender, subject, snippet, badge, timestamp, onOpen }: RowProps) {
  const content = [
    h("div", { className: "chief-row__avatar", key: "avatar" }, avatar),
    h(
      "div",
      { className: "chief-row__body", key: "body" },
      h("div", { className: "chief-row__sender" }, sender),
      h("div", { className: "chief-row__subject" }, subject),
      h("div", { className: "chief-row__snippet" }, snippet),
    ),
    h(
      "div",
      { className: "chief-row__meta", key: "meta" },
      badge,
      h("time", { className: "chief-row__time" }, timestamp),
    ),
  ];

  if (!onOpen) {
    return h("article", { className: "chief-row", "data-chief": "row" }, content);
  }

  return h(
    "button",
    {
      className: "chief-row chief-row--button",
      "data-chief": "row",
      type: "button",
      onClick: onOpen,
    },
    content,
  );
}

export function SectionHeader({ id, children }: SectionHeaderProps) {
  return h(
    "div",
    {
      id,
      className: "chief-section-header",
      "data-chief": "section-header",
      role: "heading",
    },
    children,
  );
}

export function TabBar({ chips, selected, onSelect }: TabBarProps) {
  return h(
    "nav",
    { className: "chief-tabbar", "data-chief": "tabbar", role: "tablist" },
    chips.map((chip) =>
      h(
        "button",
        {
          key: chip.id,
          className: "chief-tabbar__chip",
          "data-highlighted": chip.highlighted ? "true" : undefined,
          type: "button",
          role: "tab",
          "aria-selected": chip.id === selected,
          onClick: () => onSelect(chip.id),
        },
        h("span", { className: "chief-tabbar__label" }, chip.label),
        typeof chip.count === "number" ? h("span", { className: "chief-tabbar__count" }, chip.count) : null,
      ),
    ),
  );
}

export function CommandPill({ actions }: CommandPillProps) {
  return h(
    "div",
    { className: "chief-command-pill", "data-chief": "command-pill", role: "toolbar" },
    actions.map((action) =>
      h(
        "button",
        {
          key: action.id,
          className: "chief-command-pill__action",
          type: "button",
          disabled: action.disabled,
          "aria-label": action.label,
          onClick: action.onSelect,
        },
        action.icon ? h("span", { className: "chief-command-pill__icon" }, action.icon) : null,
        h("span", { className: "chief-command-pill__label" }, action.label),
      ),
    ),
  );
}

export function Widget({ id, title, children }: WidgetProps) {
  return h(
    "section",
    { id, className: "chief-widget", "data-chief": "widget" },
    h("header", { className: "chief-widget__title" }, title),
    h("div", { className: "chief-widget__body" }, children),
  );
}

export function DotTrail({ label, filled, total = 5, color }: DotTrailProps) {
  const boundedTotal = Math.max(1, Math.floor(total));
  const boundedFilled = Math.min(boundedTotal, Math.max(0, Math.floor(filled)));

  return h(
    "div",
    {
      className: "chief-dot-trail",
      "data-chief": "dot-trail",
      "data-color": color,
      "aria-label": `${label}: ${boundedFilled} of ${boundedTotal}`,
    },
    h("span", { className: "chief-dot-trail__label" }, label),
    h(
      "span",
      { className: "chief-dot-trail__dots", role: "meter", "aria-valuemin": 0, "aria-valuemax": boundedTotal, "aria-valuenow": boundedFilled },
      Array.from({ length: boundedTotal }, (_, index) =>
        h("span", {
          key: index,
          className: "chief-dot-trail__dot",
          "data-filled": index < boundedFilled ? "true" : "false",
        }),
      ),
    ),
  );
}

export function Sparkline({ data, highlightIndex, label }: SparklineProps) {
  const max = Math.max(1, ...data);

  return h(
    "figure",
    { className: "chief-sparkline", "data-chief": "sparkline", "aria-label": label },
    h(
      "div",
      { className: "chief-sparkline__bars", role: "img" },
      data.map((value, index) =>
        h("span", {
          key: index,
          className: "chief-sparkline__bar",
          "data-highlighted": index === highlightIndex ? "true" : "false",
          style: { "--chief-sparkline-height": `${Math.max(4, Math.round((value / max) * 100))}%` },
        }),
      ),
    ),
    h("figcaption", { className: "chief-sparkline__label" }, label),
  );
}

export function HoldRing({ progress, label, onHold, onRelease }: HoldRingProps) {
  const value = clampProgress(progress);

  return h(
    "button",
    {
      className: "chief-hold-ring",
      "data-chief": "hold-ring",
      type: "button",
      style: { "--chief-hold-progress": `${value}` },
      "aria-label": label,
      "aria-valuemin": 0,
      "aria-valuemax": 1,
      "aria-valuenow": value,
      onMouseDown: onHold,
      onMouseUp: onRelease,
      onMouseLeave: onRelease,
      onTouchStart: onHold,
      onTouchEnd: onRelease,
    },
    h("span", { className: "chief-hold-ring__core" }, label),
  );
}

export function CeremonyCard({
  label,
  headline,
  subtitle,
  evidence,
  provenance,
  rollback,
  onApprove,
}: CeremonyCardProps) {
  return h(
    "section",
    { className: "chief-ceremony-card", "data-chief": "ceremony-card", "data-motion": "ceremony-begin" },
    h("p", { className: "chief-ceremony-card__label" }, label),
    h("h1", { className: "chief-ceremony-card__headline" }, headline),
    h("p", { className: "chief-ceremony-card__subtitle" }, subtitle),
    h(
      "dl",
      { className: "chief-ceremony-card__evidence" },
      evidence.map((item) => [
        h("dt", { key: `${item.label}-term` }, item.label),
        h("dd", { key: `${item.label}-value` }, item.value),
      ]),
    ),
    h(
      "dl",
      { className: "chief-ceremony-card__provenance" },
      provenance.map((item) => [
        h("dt", { key: `${item.label}-term` }, item.label),
        h("dd", { key: `${item.label}-value` }, item.value),
      ]),
    ),
    h("p", { className: "chief-ceremony-card__rollback" }, rollback),
    h(
      "button",
      { className: "chief-ceremony-card__approve", type: "button", onClick: onApprove },
      "Hold complete: approve",
    ),
  );
}

export function OmnibarPalette({ query, onQueryChange, results, onSelect }: OmnibarPaletteProps) {
  return h(
    "section",
    { className: "chief-omnibar", "data-chief": "omnibar-palette", "data-motion": "omnibar-summon" },
    h("input", {
      className: "chief-omnibar__input",
      value: query,
      type: "search",
      autoFocus: true,
      "aria-label": "Omnibar",
      onChange: (event: unknown) => onQueryChange((event as { currentTarget?: { value?: string } }).currentTarget?.value ?? ""),
    }),
    h(
      "div",
      { className: "chief-omnibar__results", role: "listbox" },
      results.map((result) =>
        h(
          "button",
          {
            key: result.id,
            className: "chief-omnibar__result",
            type: "button",
            role: "option",
            onClick: () => onSelect(result),
          },
          h("span", { className: "chief-omnibar__type" }, result.type),
          h("span", { className: "chief-omnibar__title" }, result.title),
          result.subtitle ? h("span", { className: "chief-omnibar__subtitle" }, result.subtitle) : null,
        ),
      ),
    ),
  );
}

export function InboxDrawer({ sections }: InboxDrawerProps) {
  return h(
    "aside",
    { className: "chief-inbox-drawer", "data-chief": "inbox-drawer" },
    sections.map((section) =>
      h(
        "section",
        { key: section.id, className: "chief-inbox-drawer__section" },
        h("header", { className: "chief-inbox-drawer__title" }, section.title),
        section.items.map((item) =>
          h(
            "button",
            {
              key: item.id,
              className: "chief-inbox-drawer__item",
              type: "button",
              onClick: item.onOpen,
              disabled: !item.onOpen,
            },
            h("span", { className: "chief-inbox-drawer__item-title" }, item.title),
            item.detail ? h("span", { className: "chief-inbox-drawer__item-detail" }, item.detail) : null,
            item.badge,
            item.timestamp ? h("time", { className: "chief-inbox-drawer__time" }, item.timestamp) : null,
          ),
        ),
      ),
    ),
  );
}

export function ChatPane({ messages, composer }: ChatPaneProps) {
  return h(
    "aside",
    { className: "chief-chat-pane", "data-chief": "chat-pane" },
    h(
      "div",
      { className: "chief-chat-pane__messages" },
      messages.map((message) =>
        h(
          "article",
          { key: message.id, className: "chief-chat-pane__message", "data-role": message.role },
          h("header", { className: "chief-chat-pane__sender" }, message.sender),
          h("div", { className: "chief-chat-pane__body" }, message.body),
          message.timestamp ? h("time", { className: "chief-chat-pane__time" }, message.timestamp) : null,
        ),
      ),
    ),
    h("div", { className: "chief-chat-pane__composer" }, composer),
  );
}

export function Menubar({ id, children }: MenubarProps) {
  return h("nav", { id, className: "chief-menubar", "data-chief": "menubar", role: "menubar" }, children);
}

export function MenubarGroup({ id, label, children }: MenubarGroupProps) {
  return h(
    "section",
    { id, className: "chief-menubar__group", "data-chief": "menubar-group", "aria-label": label },
    h("span", { className: "chief-menubar__group-label" }, label),
    children,
  );
}

export function MenubarItem({ id, action, shortcut, disabled, onSelect, children }: MenubarItemProps) {
  return h(
    "button",
    {
      id,
      className: "chief-menubar__item",
      "data-chief": "menubar-item",
      "data-action": action,
      type: "button",
      role: "menuitem",
      disabled,
      onClick: onSelect,
    },
    h("span", { className: "chief-menubar__item-label" }, children),
    shortcut ? h("kbd", { className: "chief-menubar__shortcut" }, shortcut) : null,
  );
}

export function MenubarExtra({ id, label, active, children }: MenubarExtraProps) {
  return h(
    "button",
    {
      id,
      className: "chief-menubar__extra",
      "data-chief": "menubar-extra",
      "data-active": active ? "true" : undefined,
      type: "button",
      "aria-current": active ? "page" : undefined,
      onKeyDown: (event: unknown) => keyActivate(event, () => undefined),
    },
    children ?? label,
  );
}

export function Box({ as = "div", grid, flex, children }: BoxProps) {
  const style: Record<string, string | number | undefined> = {};
  const dataLayout = grid ? "grid" : flex ? "flex" : "block";

  if (grid) {
    style.display = "grid";
    style.gridTemplateColumns = grid.columns;
    style.gridTemplateRows = grid.rows;
    style.gap = grid.gap;
    style.alignItems = grid.align;
    style.justifyContent = grid.justify;
  }

  if (flex) {
    style.display = "flex";
    style.flexDirection = flex.direction;
    style.flexWrap = flex.wrap;
    style.gap = flex.gap;
    style.alignItems = flex.align;
    style.justifyContent = flex.justify;
  }

  return h(as, { className: "chief-box", "data-chief": "box", "data-layout": dataLayout, style }, children);
}
