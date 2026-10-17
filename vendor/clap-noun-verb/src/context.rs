// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Global application context and state management
//!
//! This module provides a type-safe way to share state across all commands.
//!
//! # Example
//!
//! ```rust,ignore
//! use clap_noun_verb::context::AppContext;
//! use std::sync::Arc;
//!
//! #[derive(Clone)]
//! struct AppState {
//!     db_connection: Arc<Database>,
//!     config: AppConfig,
//! }
//!
//! // Create context once at startup
//! let state = AppState {
//!     db_connection: Arc::new(Database::connect().await?),
//!     config: load_config()?,
//! };
//!
//! let context = AppContext::new(state);
//!
//! // Pass to all handlers
//! // Handlers can access via: context.state()
//! ```

use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::fmt;
use std::sync::{Arc, RwLock};

/// Global application context for sharing state across commands
///
/// Type-safe container for application state that can be accessed by all handlers.
#[derive(Clone)]
pub struct AppContext {
    /// Thread-safe storage for application state
    state: Arc<RwLock<ContextData>>,
}

/// Internal context data storage
struct ContextData {
    /// Map of TypeId to Any for type-erased storage
    values: HashMap<TypeId, Box<dyn Any + Send + Sync>>,
}

impl AppContext {
    /// Create a new empty context
    pub fn new() -> Self {
        Self { state: Arc::new(RwLock::new(ContextData { values: HashMap::new() })) }
    }

    /// Insert a value into the context
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let ctx = AppContext::new();
    /// ctx.insert(42_i32);
    /// ctx.insert("hello".to_string());
    /// ```
    pub fn insert<T: Send + Sync + 'static>(&self, value: T) -> Result<(), ContextError> {
        let type_id = TypeId::of::<T>();
        let mut state = self.state.write().map_err(|_| ContextError::PoisonedLock)?;

        state.values.insert(type_id, Box::new(value));
        Ok(())
    }

    /// Get a reference to a value from the context
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let ctx = AppContext::new();
    /// ctx.insert(42_i32)?;
    /// let value: i32 = ctx.get::<i32>()?;
    /// assert_eq!(value, 42);
    /// ```
    pub fn get<T>(&self) -> Result<T, ContextError>
    where
        T: Clone + 'static,
    {
        let type_id = TypeId::of::<T>();
        let state = self.state.read().map_err(|_| ContextError::PoisonedLock)?;

        state
            .values
            .get(&type_id)
            .and_then(|v| v.downcast_ref::<T>())
            .cloned()
            .ok_or(ContextError::TypeNotFound(std::any::type_name::<T>().to_string()))
    }

    /// Check if a value exists in the context
    pub fn contains<T: 'static>(&self) -> Result<bool, ContextError> {
        let type_id = TypeId::of::<T>();
        let state = self.state.read().map_err(|_| ContextError::PoisonedLock)?;

        Ok(state.values.contains_key(&type_id))
    }

    /// Remove a value from the context
    pub fn remove<T>(&self) -> Result<Option<T>, ContextError>
    where
        T: 'static,
    {
        let type_id = TypeId::of::<T>();
        let mut state = self.state.write().map_err(|_| ContextError::PoisonedLock)?;

        Ok(state.values.remove(&type_id).and_then(|v| v.downcast::<T>().ok()).map(|b| *b))
    }

    /// Execute a closure with read access to a value
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let ctx = AppContext::new();
    /// ctx.insert("hello".to_string())?;
    /// ctx.with::<String, _, _>(|s| println!("{}", s))?;
    /// ```
    pub fn with<T, F, R>(&self, f: F) -> Result<R, ContextError>
    where
        T: Clone + 'static,
        F: FnOnce(T) -> R,
    {
        let value = self.get::<T>()?;
        Ok(f(value))
    }

    /// Get number of values stored
    pub fn len(&self) -> Result<usize, ContextError> {
        let state = self.state.read().map_err(|_| ContextError::PoisonedLock)?;
        Ok(state.values.len())
    }

    /// Check if context is empty
    pub fn is_empty(&self) -> Result<bool, ContextError> {
        Ok(self.len()? == 0)
    }

    /// Clear all values from context
    pub fn clear(&self) -> Result<(), ContextError> {
        let mut state = self.state.write().map_err(|_| ContextError::PoisonedLock)?;
        state.values.clear();
        Ok(())
    }
}

impl Default for AppContext {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Debug for AppContext {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AppContext").field("values_count", &self.len().unwrap_or(0)).finish()
    }
}

/// Errors that can occur when working with AppContext
#[derive(Debug, Clone)]
pub enum ContextError {
    /// The requested type is not in the context
    TypeNotFound(String),
    /// The internal lock is poisoned
    PoisonedLock,
}

impl fmt::Display for ContextError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ContextError::TypeNotFound(name) => {
                write!(f, "Type '{}' not found in application context", name)
            }
            ContextError::PoisonedLock => {
                write!(f, "Application context lock was poisoned")
            }
        }
    }
}

impl std::error::Error for ContextError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_and_get() {
        let ctx = AppContext::new();
        ctx.insert(42_i32).unwrap();

        let value: i32 = ctx.get().unwrap();
        assert_eq!(value, 42);
    }

    #[test]
    fn test_multiple_types() {
        let ctx = AppContext::new();
        ctx.insert(42_i32).unwrap();
        ctx.insert("hello".to_string()).unwrap();

        let int_val: i32 = ctx.get().unwrap();
        let str_val: String = ctx.get().unwrap();

        assert_eq!(int_val, 42);
        assert_eq!(str_val, "hello");
    }

    #[test]
    fn test_contains() {
        let ctx = AppContext::new();
        ctx.insert(42_i32).unwrap();

        assert!(ctx.contains::<i32>().unwrap());
        assert!(!ctx.contains::<String>().unwrap());
    }

    #[test]
    fn test_remove() {
        let ctx = AppContext::new();
        ctx.insert(42_i32).unwrap();

        let removed: Option<i32> = ctx.remove().unwrap();
        assert_eq!(removed, Some(42));
        assert!(!ctx.contains::<i32>().unwrap());
    }

    #[test]
    fn test_with_closure() {
        let ctx = AppContext::new();
        ctx.insert("hello".to_string()).unwrap();

        let result = ctx.with::<String, _, _>(|s| format!("{} world", s)).unwrap();
        assert_eq!(result, "hello world");
    }

    #[test]
    fn test_clear() {
        let ctx = AppContext::new();
        ctx.insert(42_i32).unwrap();
        ctx.insert("hello".to_string()).unwrap();

        assert_eq!(ctx.len().unwrap(), 2);
        ctx.clear().unwrap();
        assert!(ctx.is_empty().unwrap());
    }

    #[test]
    fn test_type_not_found() {
        let ctx = AppContext::new();
        let result: Result<i32, _> = ctx.get();
        assert!(matches!(result, Err(ContextError::TypeNotFound(_))));
    }
}
