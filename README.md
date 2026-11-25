# DDA Remote — v0.1.0 (development prototype)

A new Rust workspace for cross-platform remote desktop. This first milestone implements screen capture, JPEG streaming, mouse movement/clicks, and explicit session approval. It is **not** AnyDesk/Parsec feature parity.

## Security status — read first

**NOT READY FOR PUBLIC NETWORKS.** The application protocol has no TLS, peer identity, or 2FA yet. The host deliberately listens on `127.0.0.1` only and the client refuses non-loopback IP addresses. To connect different computers, use an authenticated, encrypted SSH tunnel. Never expose TCP port 45890 to the internet or disable the loopback restrictions. No unattended access. Host approval is mandatory.

## Requirements

- Rust stable with Cargo
- Windows 10/11, or Debian-based Linux with a graphical desktop
- Linux native libraries: `libx11-dev libxext-dev libxrandr-dev libxinerama-dev libxcursor-dev libxi-dev libwayland-dev libxkbcommon-dev pkg-config` (some combinations vary by desktop)
- X11 is the initial Linux target. Wayland support is not guaranteed: capture and input often require portals and compositor-specific APIs.

## Build

```sh
cargo build --release
cargo test --workspace
```

Executable: `target/release/dda-remote` on Linux; `target/release/dda-remote.exe` on Windows. Windows release binary can be copied to a folder/USB drive, but portability must be validated on a clean machine. This is a CLI with a remote-viewer window, not yet a full dashboard.

## Local smoke test (one computer)

Terminal 1:
```sh
cargo run --release -- host
```

Terminal 2:
```sh
cargo run --release -- connect 127.0.0.1:45890
```

At the host terminal, type `YES` to authorize. Remote viewer displays screen; move and click with the mouse; ESC closes the viewer. Keyboard input is not yet enabled. A host capturing its own viewer creates a hall-of-mirrors effect.

## Two computers: SSH tunnel

On the remote Debian computer, start the host on its own loopback:
```sh
./dda-remote host 45890
```

On the controlling computer, forward a local port to the remote machine (requires SSH server on Debian, host key verification and SSH authentication):
```sh
ssh -N -L 45890:127.0.0.1:45890 user@remote-debian-host
```

On the controlling computer, in another terminal:
```sh
./dda-remote connect 127.0.0.1:45890
```

The remote host must type `YES` in its terminal. For Debian -> Windows, run OpenSSH Server on Windows, use an SSH tunnel to Windows loopback and reverse roles. On Windows, the command is `dda-remote.exe`.

## Workspace

- `crates/dda-protocol`: versioned message format and size bounds
- `crates/dda-core`: length-prefixed TCP transport
- `crates/dda-cli`: host/viewer prototype
- `docs/ROADMAP.md`: milestone and security gates
- `scripts/`: platform build helpers

## Known limitations

- No application-layer encryption, authentication, 2FA, device IDs, or relay
- No keyboard forwarding, audio, file transfer, clipboard, multi-monitor selection, or persistent service
- JPEG/JSON is inefficient; ~5 fps target, not low-latency Parsec-class streaming
- Mouse coordinates assume the primary display begins at (0,0); multi-monitor layouts may differ
- Consent is entered through host terminal, not a native approval dialog
- Wayland screen capture and input are not yet implemented reliably
- No CI-validated builds in this delivery: dependencies and code must be compiled/tested on target platforms
