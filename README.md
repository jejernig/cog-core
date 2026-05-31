# Cog — AI Coworker Engine

[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)

Cog is an open-source game engine middleware purpose-built for cooperative AI gameplay. It gives NPCs goals, memory, relationships, opinions, and the ability to learn from the player — not just dialogue, but genuine cooperative reasoning.

**Built for games where AI agents ARE the mechanic, not the flavor.**

## Architecture

```
Cog Core (Rust) → FFI → Unity Package / Unreal Plugin / Web SDK
                            ↓
                      Inference Proxy
                            ↓
                  LLM Providers (OpenAI, Anthropic, etc.)
```

## Repositories

| Repo | Description |
|---|---|
| `cog-core` | Rust engine: agent runtime, memory, personality, relationships, world-state |
| `cog-unity` | Unity package: `CogNPC` MonoBehaviour, editor tools |
| `cog-unreal` | Unreal plugin: `ACogNPC` Actor, Blueprint nodes |
| `cog-web` | WASM SDK for WebGL games |
| `cog-docs` | Documentation site |
| `cog-demo` | Library Organization Simulator — reference implementation |

## Status

**Phase 0 — Foundation.** Spikes and prototype in progress.

## License

Apache 2.0 — see [LICENSE](LICENSE) for details.

---

Built by [Team Teddy Development](https://github.com/jejernig)
