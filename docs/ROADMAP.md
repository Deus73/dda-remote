# Development plan and acceptance gates

## Milestone 0 — repository and baseline (week 1)
- Rust workspace, protocol, TCP framing, capture, viewer, mouse input
- Compile on Windows and Debian/X11; smoke test and fix platform issues
- Confirm session approval and loopback-only restrictions

## Milestone 1 — security foundation (weeks 2–4)
- TLS 1.3 or QUIC with pinned device identity and authenticated key exchange
- Persistent per-device Ed25519 keys stored using OS credential facilities
- TOTP enrollment/verification, backup codes, anti-replay, lockouts and audit
- Explicit user consent dialog, session permission scopes, instant revocation
- Threat model, dependency audit, security tests; only then LAN listening

## Milestone 2 — remote UX (weeks 5–7)
- Tauri UI, address book, pairing, session tabs, monitor selection
- Keyboard scan-code handling, clipboard, file transfer, permissions
- Reliable reconnection, bitrate adaptation, error reporting

## Milestone 3 — performance and Linux (weeks 8–11)
- H.264 hardware encoder/decoder, optional AV1, adaptive 30/60fps
- DXGI/WGC on Windows, PipeWire portal on Wayland, X11 fallback
- High DPI, multiple monitors, cursor shape, latency telemetry

## Milestone 4 — internet connectivity (weeks 12–15)
- Self-hosted signaling, STUN/TURN-like traversal, authenticated relay
- E2E session encryption across relays, device registration, rate limits
- Capacity/load testing; no artificial session caps, resource-based limits

## Milestone 5 — deployment (weeks 16–18)
- Windows portable build and optional service installer
- Signed binaries, update channel, Linux AppImage/DEB
- CI Windows/Debian matrix, integration tests, reproducible release checks

## Later
- Android and iOS clients sharing Rust core; mobile host capability depends on OS permissions
- Audio, gamepads, unattended access policies, enterprise admin features

Timelines are planning estimates, not delivery commitments.
