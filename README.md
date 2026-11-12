# lineage-rust

A Lineage II Chronicle 4 client written in Rust on top of [wgpu](https://wgpu.rs): engine, renderer,
network protocol (C4, protocol 656) and the in-game interface. It runs natively on desktop, in the
browser (WebAssembly + WebGPU) and on mobile (iOS, Android), and ships with a companion bot
(Tauri + React).

**Play it in your browser: [play.unkbot.com](https://play.unkbot.com)** (Chrome or Edge with WebGPU). No install needed;
the demo connects to the L2Killer server, so you log in with an L2Killer account.

The repository **contains no game files and needs none of the original DLLs**. It reads the data of
a C4 client you already own: maps, meshes, textures, animations, `.dat` tables, `.ini`/`.int` files
and UnrealScript `.u` packages.

> **Legal notice.** Independent, non-profit project, not affiliated with NCSoft or Epic Games.
> "Lineage" is a trademark of NCSoft. You need your own copy of the C4 client; the game data belongs
> to its owners. The code reimplements observed behaviour for interoperability.

| | |
|---|---|
| Platforms | macOS (Apple Silicon, Metal) is the tested target. Windows/Linux through wgpu are not validated yet. Web, iOS and Android are experimental. |
| Language | Rust stable 1.95, edition 2024 |
| Status | Work in progress: login, lobby, world, movement, combat, and most C4 windows work; not everything is ported |
| License | MIT (vendored UE2 readers: Canastra Source License 1.0) |

## Requirements

- Rust **1.95** or newer (`rustup toolchain install 1.95.0`)
- A Lineage II **C4 client folder** on disk (the one that contains `system/`, `maps/`,
  `animations/`, `textures/`, `systextures/`, ...)
- A C4 (protocol 656) server to log in to. Without one you can still explore the world offline.

Per platform you also need:

| Target | Extra tools |
|---|---|
| Web | Python 3, `wasm-bindgen-cli` 0.2.128, the `wasm32-unknown-unknown` target, a browser with WebGPU and OffscreenCanvas in Workers (Chrome/Edge) |
| iOS | Xcode command line tools, the `aarch64-apple-ios` (device) or `aarch64-apple-ios-sim` (simulator) target |
| Android | Android SDK + NDK, `cargo-apk` 0.10.0, the `aarch64-linux-android` target |
| Bot | Node.js 22, pnpm 10 |

## Quick start (desktop)

```sh
git clone https://github.com/w3nder/lineage-rust.git
cd lineage-rust
export KILLER_CLIENT_DIR=/path/to/your/c4/client     # folder with system/, maps/, animations/...
cargo build --release -p killer
./target/release/killer                                # login -> character selection -> world
```

The first build takes a while (wgpu). On machines with little RAM use
`CARGO_INCREMENTAL=0 cargo build -j2 --release -p killer`.

### No server? Run it offline

```sh
./target/release/killer --tile 22_22          # open a map tile (Giran) without networking
./target/release/killer --tile 22_22 --demo   # Giran town centre with a crowd of 300 characters
./target/release/killer --lobby login         # lobby screens with synthetic data
```

### Connecting to a server

The login server address (host and port) is edited with the gear button on the login screen and
saved to:

- macOS: `~/Library/Application Support/killer/config.toml`
- Windows: `%APPDATA%\killer\config.toml`
- Linux: `~/.config/killer/config.toml`

```toml
[login]
host = "your.server.example"
port = 2106
```

Type the account and password in the login form. `KILLER_ACCOUNT` pre-fills the account name.
Credentials are never written to logs.

### Command line options

| Option | What it does |
|---|---|
| `--client <dir>` / `KILLER_CLIENT_DIR` | Path to the C4 client folder |
| `--client-url <url>` / `KILLER_CLIENT_URL` | Read the client on demand from an HTTP bucket instead of a local folder (see [Hosting the client data](#hosting-the-client-data)) |
| `--client-token <token>` / `KILLER_CLIENT_TOKEN` | Optional `Authorization: Bearer` token for that bucket |
| `--tile 22_22` | Open one map tile offline |
| `--demo`, `--demo-crowd N` | With `--tile`: stress scene with N characters (default 300) |
| `--lobby login\|servers\|characters\|create` | Open a lobby screen with synthetic data |
| `--hud classic\|modern` | Original C4 interface or the modern theme; the choice is remembered |
| `--chronicle c4\|interlude` / `KILLER_CHRONICLE` | Asset set to use (the protocol stays C4) |
| `--hour 12` | Fix the time of day |
| `--screenshot out.png` | Render one frame, save it and exit |

`RUST_LOG=killer=info,killer_world=info` prints what arrives from the server (tables, shortcuts,
inventory, skills, unhandled or malformed packets).

### Controls

- Left click: move / select. Drag with any button: orbit the camera. Wheel: zoom.
- The C4 Alt shortcuts open the windows (`Alt+V`, `Alt+K`, `Alt+T`, `Alt+X`, `Alt+B`, `Alt+C`, ...).
- `F1`-`F12` use the shortcut bar, `Alt+F1`-`Alt+F10` switch its page.
- `F12` saves a screenshot to `~/Pictures/killer/`.

## Web (WebAssembly + WebGPU)

A hosted build of this client runs at [play.unkbot.com](https://play.unkbot.com). To run your own:

```sh
rustup target add wasm32-unknown-unknown
cargo install wasm-bindgen-cli --version 0.2.128 --locked
python3 scripts/web.py run            # builds the wasm and serves http://127.0.0.1:8080
```

Open `http://127.0.0.1:8080` in Chrome or Edge and pick your C4 client folder with the
"open client folder" button. The files are read locally in the browser; nothing is uploaded.
`/?mode=demo` opens the offline Giran preview.

Browsers cannot open raw TCP sockets, so online play goes through a small WebSocket gateway. Run it
in a second terminal, pointing at the login and game servers:

```sh
KILLER_LOGIN_TARGET=your.server.example:2106 \
KILLER_GAME_TARGET=your.server.example:7777 \
cargo run --release -p killer-gateway -- --origin http://127.0.0.1:8080
```

It listens on `127.0.0.1:8081`, the default gateway in `web/config.mjs`. `--origin` accepts a
comma-separated list of allowed origins.

To publish a static build:

```sh
python3 scripts/web_dist.py --client-url https://your-bucket.example/c4 --gateway wss://play.example.com/gw
```

The output goes to `dist/web`. `workers/gateway` is the same gateway as a Cloudflare Worker
(`npm install && npx wrangler deploy` after editing the placeholders in `wrangler.toml`; set the
session secret with `npx wrangler secret put SESSION_SECRET` if you enable `REQUIRE_LOGIN`).
Remote hosting requires HTTPS/WSS; do not expose the development server as production.

## iOS

```sh
rustup target add aarch64-apple-ios-sim aarch64-apple-ios
sh scripts/ios_bundle.sh                        # simulator build -> target/ios/Killer.app
sh scripts/ios_bundle.sh aarch64-apple-ios      # device build
```

The app has no local client folder, so pass the bucket at build time with `KILLER_CLIENT_URL`
(and optionally `KILLER_CLIENT_TOKEN`). Device builds are signed with `IOS_SIGN_IDENTITY`
(ad-hoc by default). `.github/workflows/ios.yml` builds an `.ipa` in CI.

## Android

```sh
rustup target add aarch64-linux-android
cargo install cargo-apk --locked --version 0.10.0
export ANDROID_NDK_ROOT=/path/to/ndk
cargo apk build --release -p killer --lib      # -> target/release/apk/Killer.apk
```

`cargo-apk` needs a release keystore (`CARGO_APK_RELEASE_KEYSTORE` and
`CARGO_APK_RELEASE_KEYSTORE_PASSWORD`); see `.github/workflows/android.yml` for a complete example.

## Bot (Tauri + React)

`apps/killer-bot-ui` is a desktop bot that speaks the same protocol, built on `killer-bot-core`
(session, automation) and `killer-bot-script` (script language).

```sh
cd apps/killer-bot-ui/ui
pnpm install
pnpm build                                      # type-check, tests and the production bundle
cd ..
ui/node_modules/.bin/tauri dev                  # run in development
ui/node_modules/.bin/tauri build                # package (.app/.dmg on macOS, NSIS on Windows)
```

The bot needs only the `.dat` tables and icons from the client. Point it at a local client with
`KILLER_CLIENT_DIR`, or at a bucket with `KILLER_CLIENT_URL`. Passwords are stored in the system
keychain, or read from `KILLER_BOT_PASSWORD` when no keychain is available.

## Command line tools

```sh
cargo run --release -p killer-cli -- --help
```

Subcommands: `login` (test a login against a server), `package`, `texture`, `object`, `level`,
`system` (inspect client files), `census` and `render` (headless capture of a map tile).

## Hosting the client data

The web, mobile and bot builds can read the client over HTTP instead of from a local folder.
`scripts/client_upload.py` generates the `manifest.json` and uploads a client folder to an
S3-compatible bucket (Cloudflare R2) with the `aws` CLI:

```sh
R2_ACCOUNT_ID=... R2_BUCKET=... R2_PREFIX=c4 python3 scripts/client_upload.py --client "$KILLER_CLIENT_DIR"
```

Use the resulting base URL as `KILLER_CLIENT_URL` / `--client-url` / `clientUrl` in
`web/config.mjs`. No default bucket is configured in this repository: you host your own copy of
the client, under your own responsibility.

## Project layout

```
login server --Blowfish--> l2-net --> game server --XOR C4--> l2-net (typed packets)
                                                                  |
                     killer-world (entities, clock, target, inventory, skills, party, movement)
                                                                  |
C4 client --> l2-crypto --> ue2-package / ue2-assets / ue2-level --> l2-catalog --> killer-assets
                                                                  |
                     killer-render (wgpu: BSP, terrain, static and skinned meshes, materials, UI pass)
                                                                  |
                     killer-ui (the game windows: status, chat, target, shortcuts, inventory, ...)
                                                                  |
                     apps/killer (window, input, camera, main loop)
```

| Crate | Purpose |
|---|---|
| `ue2-core`, `ue2-package`, `ue2-assets`, `ue2-level`, `l2-crypto`, `l2-catalog` | Unreal Engine 2 readers vendored from CanastraEngine, with the changes listed in [`crates/VENDORED.md`](crates/VENDORED.md) |
| `l2-dat` | Readers for the `.dat` tables (`npcgrp`, `armorgrp`, `weapongrp`, `skillgrp`, `systemmsg`, ...) |
| `l2-net` | C4 protocol 656: login (Blowfish, RSA), game (XOR with rolling key), all packets |
| `killer-world` | World state and engine rules: movement, session, inventory, chat, animation, party, pets |
| `killer-assets` | The game client as a data source: catalog, scene, characters, animation, options |
| `killer-render` | wgpu renderer: materials, GPU skinning, lighting, fog, sky, 2D UI pass, headless capture |
| `killer-ui`, `killer-font` | The interface: bitmap fonts, draw lists, frames, and one module per window |
| `killer-platform`, `killer-download` | Platform glue and on-demand client download |
| `killer-bot-core`, `killer-bot-script` | Bot engine and its scripting language |
| `apps/killer` | The client (desktop, iOS, Android) |
| `apps/killer-web` | The WebAssembly build of the client; `web/` holds the page that loads it |
| `apps/killer-gateway` | WebSocket-to-TCP gateway for the web client |
| `apps/killer-cli` | Inspection and capture tools |
| `apps/killer-bot-ui` | The bot (Tauri + React) |

## Tests

```sh
cargo test --workspace
cargo clippy --workspace --all-targets
```

Tests that read real game data are marked `#[ignore]` and need the client:

```sh
KILLER_CLIENT_DIR=/path/to/your/c4/client cargo test --workspace -- --ignored
```

Some of those ignored tests compare against reference scripts and capture folders that are not part
of this repository and will fail without them.

## Contributing

- Work on a branch and open a pull request.
- No `unsafe`; `unwrap()`/`expect()` only in tests; no clippy warnings.
- Never commit game files or credentials.

## Licenses

Original code is under the MIT license ([`LICENSE`](LICENSE)). The vendored CanastraEngine crates
are under the Canastra Source License 1.0 ([`crates/LICENSE-CANASTRA`](crates/LICENSE-CANASTRA));
details in [`crates/VENDORED.md`](crates/VENDORED.md). Game data is not part of this repository.
