# paperjam docs

Documentation site built with [Docusaurus](https://docusaurus.io/) and a WASM playground.

## Development

```bash
pnpm install
pnpm start
```

## Build

```bash
# Build WASM (from project root)
wasm-pack build crates/paperjam-wasm --target web --release --out-dir ../../docs/static/wasm

# Build site
pnpm run build
```

## Linting

```bash
pnpm run lint        # biome check
pnpm run lint:fix    # biome auto-fix
pnpm run typecheck   # typescript check
```
