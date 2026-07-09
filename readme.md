# Notify API

Proxy service for notification channels (email, firebase notification, sms).

## Prerequisites

- [Devbox](https://www.jetify.com/devbox)

## Toolchain

- **Rust:** 1.96.0
- **Cargo:** Managed via `rustup`
- **Edition:** 2024

Rust is managed using `rustup` inside Devbox to ensure exact version parity with official Rust releases.

---

## Getting Started

1. Enter the Devbox environment:

    ```bash
    devbox shell
    ```

2. Build the project:

   ```bash
   cargo build
   ```

## Notes

- Do not install Rust globally; always use devbox shell.
- Rust versions are pinned via rustup, not nixpkgs, to avoid version lag.
- RUSTUP_HOME location is on .rustup
- LIBRARY_PATH location is on .rustup/nix/profile/default/lib
