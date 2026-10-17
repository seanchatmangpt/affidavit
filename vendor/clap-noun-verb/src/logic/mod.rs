// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Business logic layer - reusable functions independent of CLI
//!
//! This module contains pure business logic functions that can be used
//! by any interface: CLI, API, web apps, etc.
//!
//! ## Design Principle
//!
//! Business logic functions are completely independent of CLI implementation.
//! They accept typed inputs and return typed outputs, making them reusable
//! across different interfaces.

pub mod core;
pub mod handler;

pub use core::CoreFunction;
pub use handler::{CommandHandler, HandlerContext, HandlerInput, HandlerOutput};
