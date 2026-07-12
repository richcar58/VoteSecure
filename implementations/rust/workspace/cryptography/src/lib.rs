// SPDX-License-Identifier: Apache-2.0
// Copyright 2025 Free & Fair
// See LICENSE.md for details

//! Cryptography library for the VoteSecure project

#![allow(dead_code)]
// The nightly features below are needed only so that custom_warning_macro
// attributes can appear in statement/expression position. Gating them (and
// every such attribute use, via cfg_attr) behind the `custom-warnings`
// feature lets this crate compile on stable Rust by default; enabling
// `custom-warnings` requires a nightly toolchain, as before.
#![cfg_attr(
    feature = "custom-warnings",
    feature(stmt_expr_attributes, proc_macro_hygiene)
)]
#![doc = include_str!("../README.md")]
#![cfg_attr(coverage_nightly, feature(coverage_attribute))]

// final pass
// #![warn(clippy::restriction)]

/// Defines implementation choices for key cryptographic functionalities.
pub mod context;
pub mod cryptosystem;
#[cfg_attr(
    feature = "custom-warnings",
    crate::warning("This module is not optimized.")
)]
pub mod dkgd;
pub mod groups;
/// Abstractions for curve arithmetic, groups, elements and scalars.
pub mod traits;
/// Utilities such as random number generation, hashing, signatures and serialization.
pub mod utils;
pub mod zkp;

pub use custom_warning_macro::warning;
pub use vser_derive::VSerializable;

/// Create the `cryptography` alias that points to `crate`
///
/// This alias allows applying the vser_derive macro within this crate:
///
/// `vser_derive` refers to its target traits with `cryptography::`, but
/// _within_ this crate, that reference will not resolve to anything
/// unless we add this alias. Other crates will resolve correctly
/// as they will be importing `cryptography` as a dependency.
#[doc(hidden)]
extern crate self as cryptography;
