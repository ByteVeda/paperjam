# paperjam docs

Documentation site built with [Docusaurus](https://docusaurus.io/) and a WASM playground.

## Development

```bash
npm ci
npm start
```

## Build

```bash
# Build WASM (from project root)
wasm-pack build crates/paperjam-wasm --target web --release --out-dir ../../docs-site/static/wasm

# Build site
npm run build
```

## Linting

```bash
npm run lint        # biome check
npm run lint:fix    # biome auto-fix
npm run typecheck   # typescript check
```
