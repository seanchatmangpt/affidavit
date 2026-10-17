// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Async handler support for verbs
//!
//! This module provides utilities for using async operations within verb handlers.

use crate::Result;

/// Helper for running async code from sync contexts using tokio runtime
pub fn run_async<F, T>(future: F) -> Result<T>
where
    F: std::future::Future<Output = Result<T>>,
{
    let rt = tokio::runtime::Builder::new_current_thread().enable_all().build().map_err(|e| {
        crate::error::NounVerbError::execution_error(format!("Failed to create runtime: {}", e))
    })?;

    rt.block_on(future)
}

/// Create a tokio runtime that can be reused for multiple async operations
pub fn create_runtime() -> Result<tokio::runtime::Runtime> {
    tokio::runtime::Builder::new_current_thread().enable_all().build().map_err(|e| {
        crate::error::NounVerbError::execution_error(format!("Failed to create runtime: {}", e))
    })
}
