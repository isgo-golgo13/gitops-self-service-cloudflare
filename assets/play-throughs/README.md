# Edge Factory play-throughs

Three self-contained HTML animations (no network, no assets): a ratatui terminal play-through of
the SSIP request, the dispatch path for that variant, and the Cloudflare stack being drawn as the
profile layer resolves the six fields.

| Directory | Dispatch path shown |
| --- | --- |
| `tui-edge-factory-ngo/` | request file → PR → plan → gate → exact apply (GitHub Actions) |
| `tui-edge-factory-go/` | request file → make push → Flux pull → tofu-controller converges (k3s) |
| `tui-edge-factory-xp/` | AppStack → SSIP: OIDC, schema, signed freight, admission → Crossplane composes |

Open `index.html` in any browser. Space = play/pause, ← → = ±2 s, Home = restart; the slider scrubs.
Palette: dronegrid.io tactical green on black; the animation is a deterministic `frame(t)` over inline SVG.
