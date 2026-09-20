# merle

A RAW photo culler: browse a shoot folder fast, keep or reject, and persist the
decisions. Nothing is implemented yet — this repository currently holds the
workspace skeleton only.

## Crates

| Crate          | Kind   | Purpose                                             |
| -------------- | ------ | --------------------------------------------------- |
| `merle-raw`    | lib    | RAW parsing and preview extraction                  |
| `merle-loader` | lib    | Folder listing, decoding, preload cache             |
| `merle-core`   | lib    | Culling logic: decisions, undo, persist, tournament |
| `merle-view`   | bin    | Minimal viewer                                      |
| `merle-app`    | bin    | The culler                                          |

Dependency direction:

```
merle-view -> merle-raw, merle-loader
merle-app  -> merle-raw, merle-loader, merle-core
```

`merle-loader` builds on `merle-raw`; `merle-core` is a leaf crate.

## Building

With Nix (devShell provides the toolchain, `cargo-nextest`, `bacon`,
`cargo-deny` and `typos`):

```fish
direnv allow    # or: nix develop
cargo build --workspace
```

Without Nix, `rust-toolchain.toml` pins 1.98.1 for rustup users:

```fish
cargo build --workspace
cargo test --workspace
```

## Checks

`just` wraps the common tasks — `just` on its own lists them:

```fish
just fmt        # format
just lint       # clippy, warnings as errors
just test       # nextest
just ci         # everything CI runs: fmt-check, lint, test, deny, typos
```

## License

Copyright (C) 2026 Daniel Mauderer.

The library crates — `merle-raw`, `merle-loader`, `merle-core` — are licensed
under the **Mozilla Public License 2.0** ([LICENSE-MPL](LICENSE-MPL)): you may
use them in a closed-source project, but changes to merle's own files must be
published under the MPL.

The binaries — `merle-app` and `merle-view` — are licensed under the **GNU
General Public License v3.0 or later** ([LICENSE-GPL](LICENSE-GPL)): a program
built from them, modified or not, stays free software.

Both licenses require that the copyright and license notices be kept in any
copy or derivative work.
