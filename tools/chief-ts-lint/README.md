# chief-ts-lint

`chief-ts-lint` enforces ADR-0011 at pack build time.
It scans pack TypeScript, TSX, and CSS before a signed artifact is produced.
Packs must consume the public Chief OS surface, not internal UI or shell APIs.

## Usage

```bash
chief-ts-lint --pack-root prototypes/my-pack
chief-ts-lint --pack-root prototypes/my-pack --json
chief-ts-lint --pack-root prototypes/my-pack --fail-on-violation
chief-ts-lint --pack-root prototypes/my-pack --allowlist chief-ts-lint.toml
```

Exit code `0` means no violations.
Exit code `1` means forbidden imports were found.
`--json` emits `{ "violations": [...], "total": n }` for CI parsers.

## Default Allowlist

Allowed package imports are `@chief-os/sdk`, `@chief-os/ui`, `react`,
`react-jsx-runtime`, `zod`, `date-fns`, `ulid`, `yaml`, `lodash`, and
`classnames`. Relative imports beginning with `./` or `../` are allowed.

Hard bans always win: `@radix-ui/*`, `@chief-os/ui/internal/*`, shadcn
paths, `tauri`, `@tauri-apps/*`, `react-dom`, and runtime font URLs.

## Custom Allowlist

A project can extend the defaults with TOML:

```toml
[allowed]
exact = ["nanoid"]
wildcard = ["@scope/pkg/*"]
patterns = ["@company/shared-*"]
```

Use this for reviewed utilities only. It cannot override hard bans.

## Pack Integration

Add the scanner before versioning or packaging:

```json
{
  "scripts": {
    "preversion": "chief-ts-lint --pack-root . --fail-on-violation"
  }
}
```

Keep pack UI on `@chief-os/ui`; extend the kit when the public surface is
missing a needed primitive.
