//! Fuel-bounded sandboxed execution of untrusted witness validators
//! (wasmtime).
//!
//! External plugins and custom witness validators are untrusted code. This
//! court instantiates them in a WASM sandbox with a **guaranteed fuel
//! budget**: the call terminates, always, and reports remaining fuel. A trap
//! — deliberate `unreachable`, division by zero, or fuel exhaustion — is a
//! typed refusal, never a host crash.
//!
//! ## Doctrine: Certify, Don't Decide
//!
//! The sandbox certifies *bounded execution*, not verdict correctness: a
//! plugin that returns without trapping has merely terminated within its
//! budget. Its output is a candidate, admitted or refused by the caller's
//! courts.

use thiserror::Error;
use wasmtime::{Config, Engine, Linker, Module, Store, Trap};

/// Errors and refusals produced by the sandbox court.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum WasmCourtError {
    /// The module failed to compile.
    #[error("wasm compile failed: {0}")]
    Compile(String),
    /// The module failed to instantiate.
    #[error("wasm instantiation failed: {0}")]
    Instantiation(String),
    /// The requested export is missing or has the wrong signature.
    #[error("wasm export not usable: {0}")]
    Export(String),
    /// The module trapped during execution.
    #[error("wasm trap: {message}")]
    Trap {
        /// The trap message.
        message: String,
    },
    /// The module exhausted its fuel budget.
    #[error("fuel exhausted: budget {budget}")]
    OutOfFuel {
        /// The fuel budget that was granted.
        budget: u64,
    },
}

/// A sandbox court with fuel metering enabled engine-wide.
///
/// Construction is fallible (`WasmCourt::new`); there is intentionally no
/// `Default` impl — a panic path in a sandbox boundary would violate the
/// crate's panic-free production-path law.
pub struct WasmCourt {
    engine: Engine,
}

impl WasmCourt {
    /// Construct a court with fuel consumption enabled.
    ///
    /// # Errors
    ///
    /// Returns [`WasmCourtError::Compile`] if engine construction fails
    /// (configuration error).
    pub fn new() -> Result<Self, WasmCourtError> {
        let mut config = Config::new();
        config.consume_fuel(true);
        let engine = Engine::new(&config).map_err(|e| WasmCourtError::Compile(e.to_string()))?;
        Ok(Self { engine })
    }

    /// Execute the module's `export` function (no params, no results) under
    /// a guaranteed fuel budget, returning the fuel remaining.
    ///
    /// # Errors
    ///
    /// Typed refusals for compile, instantiation, export, trap, and fuel
    /// exhaustion. Every error is a refusal; the host never crashes on
    /// guest behavior.
    pub fn run_bounded(
        &self,
        wasm_bytes: &[u8],
        export: &str,
        fuel_budget: u64,
    ) -> Result<u64, WasmCourtError> {
        let module = Module::new(&self.engine, wasm_bytes)
            .map_err(|e| WasmCourtError::Compile(e.to_string()))?;
        let mut store = Store::new(&self.engine, ());
        store
            .set_fuel(fuel_budget)
            .map_err(|e| WasmCourtError::Instantiation(e.to_string()))?;
        let linker: Linker<()> = Linker::new(&self.engine);
        let instance = linker
            .instantiate(&mut store, &module)
            .map_err(|e| WasmCourtError::Instantiation(e.to_string()))?;
        let run = instance
            .get_typed_func::<(), ()>(&mut store, export)
            .map_err(|_| WasmCourtError::Export(export.to_string()))?;

        if let Err(err) = run.call(&mut store, ()) {
            if let Some(trap) = err.downcast_ref::<Trap>() {
                if matches!(trap, Trap::OutOfFuel) {
                    return Err(WasmCourtError::OutOfFuel {
                        budget: fuel_budget,
                    });
                }
                return Err(WasmCourtError::Trap {
                    message: trap.to_string(),
                });
            }
            return Err(WasmCourtError::Trap {
                message: err.to_string(),
            });
        }
        store
            .get_fuel()
            .map_err(|e| WasmCourtError::Instantiation(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TERMINATES: &str = r#"
        (module (func (export "run")))
    "#;

    const TRAPS: &str = r#"
        (module (func (export "run") unreachable))
    "#;

    const BURNS: &str = r#"
        (module
            (func (export "run")
                (loop $spin (br $spin))
            )
        )
    "#;

    #[test]
    fn terminating_module_runs_within_budget() {
        let court = WasmCourt::new().expect("court");
        let remaining = court
            .run_bounded(TERMINATES.as_bytes(), "run", 10_000)
            .expect("run");
        assert!(remaining > 0, "a terminating module must not burn all fuel");
        assert!(remaining <= 10_000);
    }

    #[test]
    fn trapping_module_is_a_typed_refusal() {
        let court = WasmCourt::new().expect("court");
        match court.run_bounded(TRAPS.as_bytes(), "run", 10_000) {
            Err(WasmCourtError::Trap { .. }) => {}
            other => panic!("expected typed trap, got {other:?}"),
        }
    }

    #[test]
    fn infinite_loop_is_killed_by_fuel_exhaustion() {
        let court = WasmCourt::new().expect("court");
        match court.run_bounded(BURNS.as_bytes(), "run", 5_000) {
            Err(WasmCourtError::OutOfFuel { budget }) => assert_eq!(budget, 5_000),
            other => panic!("expected fuel exhaustion, got {other:?}"),
        }
    }

    #[test]
    fn missing_export_is_typed_refusal() {
        let court = WasmCourt::new().expect("court");
        match court.run_bounded(TERMINATES.as_bytes(), "no_such_export", 1_000) {
            Err(WasmCourtError::Export(_)) => {}
            other => panic!("expected export refusal, got {other:?}"),
        }
    }

    #[test]
    fn garbage_bytes_are_compile_refusals() {
        let court = WasmCourt::new().expect("court");
        assert!(matches!(
            court.run_bounded(b"\0garbage", "run", 1_000),
            Err(WasmCourtError::Compile(_))
        ));
    }
}
