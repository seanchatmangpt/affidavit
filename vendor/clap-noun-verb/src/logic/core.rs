// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Core business logic functions - pure functions reusable by any interface
//!
//! These functions contain the actual business logic and are completely
//! independent of how they are called (CLI, API, Web, etc.).

use crate::error::Result;

/// Trait for core business logic functions
///
/// Core functions are pure functions that implement business logic
/// independently of any interface. They can be called from CLI, API,
/// web apps, or any other interface.
pub trait CoreFunction<I, O>: Send + Sync
where
    I: Send + Sync,
    O: Send + Sync,
{
    /// Execute the core business logic
    ///
    /// # Arguments
    ///
    /// * `input` - Typed input for the function
    ///
    /// # Errors
    ///
    /// Returns an error if business logic execution fails.
    fn execute(&self, input: I) -> Result<O>;
}

/// Type alias for core function implementations
///
/// This allows using function pointers or closures as core functions.
pub type CoreFunctionImpl<I, O> = Box<dyn Fn(I) -> Result<O> + Send + Sync>;

/// Helper to create a core function from a closure
pub fn make_core_function<I, O, F>(f: F) -> CoreFunctionImpl<I, O>
where
    I: Send + Sync + 'static,
    O: Send + Sync + 'static,
    F: Fn(I) -> Result<O> + Send + Sync + 'static,
{
    Box::new(f)
}
