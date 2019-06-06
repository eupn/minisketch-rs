# minisketch-rs

[![Crates.io](https://img.shields.io/crates/v/minisketch-rs.svg)](https://crates.io/crates/minisketch-rs)
[![Crates.io](https://img.shields.io/crates/d/minisketch-rs.svg)](https://crates.io/crates/minisketch-rs)
[![Docs.rs](https://docs.rs/minisketch-rs/badge.svg)](https://docs.rs/minisketch-rs/)
[![Build Status](https://travis-ci.com/eupn/minisketch-rs.svg?branch=master)](https://travis-ci.com/eupn/minisketch-rs)

`minisketch-rs` is a wrapper around [minisketch](https://github.com/sipa/minisketch),
a C library by [Pieter Wuille](https://github.com/sipa) for efficient set reconciliation.

> minisketch is proposed as a part of an [Erlay](https://arxiv.org/abs/1905.10518) technique for bandwidth-efficient TX propagation in Bitcoin.

This library exposes type-safe Rust bindings to all `minisketch` functions by providing `Minisketch` structure.

## Usage

Add dependency in Cargo.toml:
```toml
[dependencies]
minisketch-rs = "0.1"
```

Generate sketches from your sets of data, serialize those sketches and send them around. Reconcile sets between peers by merging sketches.

## Examples

See the [examples](examples).
