---
id: brand-sound
title: "Sound — system sounds, licensing, behavior"
status: draft
owners: [santosh]
last_updated: 2026-04-21
related: [brand-visual-language, apple-design]
tags: [brand, sound, audio]
---

# Sound

> Binding reference. Source of truth for sound semantics is [`docs/brand/visual-language.md`](../docs/brand/visual-language.md) §11.

Three system sounds. Nothing more. Sound is chrome — use it sparingly enough that each one retains meaning.

## Sound palette

| Sound | Purpose | Duration | Level | Default |
|---|---|---|---|---|
| **Ship chime** | One-shot, plays on every shipped approval (action completed after Ceremony) | 200 ms | ~−24 dB | On |
| **Approval-needed cluck** | Plays when a new item lands in HAX Inbox that needs your attention | 80 ms | ~−28 dB | On |
| **Ceremony-begin tone** | Held drone, plays when Ceremony surface appears (Region 7/8) | 900 ms | ~−30 dB | On |

## Sound design principles

1. **Three or nothing.** Every additional system sound dilutes the meaning of the existing ones. If a new sound is proposed, retire or merge an existing one.
2. **Restraint over delight.** These sounds are not ringtones; they are informational. Calm over cheerful. Short over sustained (except Ceremony, which must feel weighty).
3. **Physical, not synthetic.** Physical materials (wood, bell, breath) over synthesized tones. Acoustic recording or high-quality physical-modeling synthesis.
4. **Timbre carves distinction, not pitch.** The three sounds should feel unmistakably different — bell, wood, drone — even when heard partially through room noise.
5. **Layer with the OS mute.** Respect the platform's "Do Not Disturb" / focus mode settings — sounds suppressed automatically.

## v0 source

Placeholder sounds from royalty-free libraries (Freesound, Zapsplat). Acceptable for v0; must be replaced before public launch.

## v1 target

- **Commissioned** by a sound designer, or
- **Licensed** set from a premium library (e.g., BOOM Library, Soundly curated packs).

Budget: `{TBD}` once v1 scope locked.

## Technical requirements

1. Played via **native audio** through Tauri's audio plugin (or OS audio framework directly) — NOT HTML `<audio>` tags. Web-audio introduces 100+ ms latency; native is instant.
2. Format: **16-bit / 48 kHz AAC** or **Ogg Vorbis** for portability; bundled into the Tauri binary.
3. Per-user mute in Settings, discoverable via Omnibar search ("sound", "mute", "quiet").
4. Per-sound level controls v2; single master on/off at v0.
5. Respect platform accessibility preferences (reduce-audio, mono-audio, etc.).

## Haptics companion (trackpad-equipped hosts)

Sounds pair with haptics where available (macOS trackpads, select Linux devices):

| Event | Sound | Haptic |
|---|---|---|
| Ship chime | bell, 200 ms | light tap on release |
| Approval-needed cluck | wood-block, 80 ms | subtle ping |
| Ceremony-begin tone | drone, 900 ms | progressive hum rising to a firm tap on sign |

See `brand/motion.md` and [`docs/brand/visual-language.md`](./visual-language.md) §12 for haptic details.

## Decision log

- **2026-04-21** — Three-sound palette locked. v0 uses royalty-free placeholders; v1 decision (commission vs license) deferred to post-scope.

## Related

- [`brand/visual-language.md`](./visual-language.md) §11 — sound semantics
- [`brand/motion.md`](./motion.md) — motion/haptic companions
- [`docs/05-surfaces.md`](../docs/05-surfaces.md) — surfaces that emit each sound
