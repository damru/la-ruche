---
description: 'Licensing rules: dual AGPL-3.0 / commercial license, AGPL-compatible dependencies, zero telemetry, mandatory CLA.'
applyTo: 'Cargo.toml,frontend/package.json,LICENSE,NOTICE,docs/LEGAL.md'
---

# License & compliance

- The project is under a **dual license**: AGPL-3.0 for non-commercial use, a commercial license for commercial use (see `LICENSE`, `docs/LEGAL.md`).
- Any new dependency (Rust or npm) must be **AGPL-3.0 compatible**; check the license before adding to the manifest, and update `NOTICE` if required.
- **Zero telemetry**: no usage data collection, no tracking dependency, no non-functional outbound network call.
- Any external contribution goes through the CLA (`CLA.md`).
- Never introduce code whose license imposes obligations incompatible with the dual license.
