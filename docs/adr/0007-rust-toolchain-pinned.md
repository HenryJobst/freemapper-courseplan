# 0007. The Rust toolchain is pinned in rust-toolchain.toml

Date: 2026-11-08
Status: Accepted

## Context

In freemapper, CI installed a newer stable Rust than the local one, and a new clippy lint failed every push.

## Decision

`rust-toolchain.toml` pins the same channel as freemapper and freemapper-print (1.99.1) with `rustfmt` and
`clippy`. Local builds, CI and the release workflow use it. The pin moves by hand, together with the siblings.

## Consequences

- Local and CI lints agree, and the suite builds with one compiler.
- Cost if wrong: the project stays on an older compiler until someone moves the pin.

Source: freemapper-print ADR 0007; freemapper ADR 0018.
