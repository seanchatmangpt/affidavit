//! 1000X COMBINATORIAL MAXIMALISM: DX Innovation
//! Time-Travel Debugger for Receipts.
//!
//! This module implements a macro-based harness that allows developers to step
//! backward and forward through a receipt's state transitions in a REPL,
//! inspecting memory at each event tick.

use crate::types::{Receipt, OperationEvent};
use std::io::{self, Write};
use std::fmt::Debug;

/// The Time-Travel Debugger engine.
///
/// It pre-calculates the state at every "tick" (event) in the receipt chain
/// to allow instantaneous O(1) backward and forward navigation.
pub struct TimeTravelDebugger<'a, S> {
    receipt: &'a Receipt,
    states: Vec<S>,
    cursor: usize,
}

impl<'a, S: Debug + Clone> TimeTravelDebugger<'a, S> {
    /// Create a new debugger instance.
    ///
    /// # Arguments
    /// * `receipt` - The receipt to debug.
    /// * `initial_state` - The state of the system before any events are processed.
    /// * `transition` - A pure function: (current_state, event) -> next_state.
    pub fn new<F>(receipt: &'a Receipt, initial_state: S, transition: F) -> Self
    where
        F: Fn(&S, &OperationEvent) -> S,
    {
        let mut states = Vec::with_capacity(receipt.events.len() + 1);
        states.push(initial_state);

        let mut current = states[0].clone();
        for event in &receipt.events {
            current = transition(&current, event);
            states.push(current.clone());
        }

        Self {
            receipt,
            states,
            cursor: 0,
        }
    }

    /// Enter the interactive REPL.
    pub fn run_repl(&mut self) -> io::Result<()> {
        println!("\n\x1b[1;36m1000X TIME-TRAVEL DEBUGGER\x1b[0m");
        println!("============================");
        println!("Receipt: {}", self.receipt.chain_hash.as_hex());
        println!("Events:  {}", self.receipt.events.len());
        println!("Ticks:   0 to {} (0 is GENESIS)", self.states.len() - 1);
        println!("Type \x1b[1;33m'help'\x1b[0m for commands.\n");

        loop {
            self.print_current_tick();
            
            print!("\x1b[1;32m(tt-dbg @ {})\x1b[0m > ", self.cursor);
            io::stdout().flush()?;

            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            let parts: Vec<&str> = input.trim().split_whitespace().collect();
            
            if parts.is_empty() {
                continue;
            }

            match parts[0] {
                "f" | "forward" | "n" | "next" => {
                    if self.cursor < self.states.len() - 1 {
                        self.cursor += 1;
                    } else {
                        println!("\x1b[1;31m[!] Already at the end of history.\x1b[0m");
                    }
                }
                "b" | "backward" | "p" | "prev" => {
                    if self.cursor > 0 {
                        self.cursor -= 1;
                    } else {
                        println!("\x1b[1;31m[!] Already at GENESIS.\x1b[0m");
                    }
                }
                "g" | "goto" => {
                    if let Some(tick_str) = parts.get(1) {
                        if let Ok(tick) = tick_str.parse::<usize>() {
                            if tick < self.states.len() {
                                self.cursor = tick;
                            } else {
                                println!("\x1b[1;31m[!] Tick {} is out of range (0-{}).\x1b[0m", tick, self.states.len() - 1);
                            }
                        } else {
                            println!("\x1b[1;31m[!] Invalid tick number: {}\x1b[0m", tick_str);
                        }
                    } else {
                        println!("\x1b[1;31m[!] Usage: goto <tick_number>\x1b[0m");
                    }
                }
                "m" | "memory" | "state" | "inspect" => {
                    self.inspect_memory();
                }
                "h" | "help" | "?" => {
                    self.print_help();
                }
                "q" | "quit" | "exit" => {
                    println!("Exiting Time-Travel Debugger.");
                    break;
                }
                _ => {
                    println!("\x1b[1;31m[!] Unknown command: '{}'. Type 'help'.\x1b[0m", parts[0]);
                }
            }
        }

        Ok(())
    }

    fn print_current_tick(&self) {
        let is_genesis = self.cursor == 0;
        
        println!("\x1b[1;34m--- TICK {} ---\x1b[0m", self.cursor);
        if is_genesis {
            println!("Event: \x1b[33m[GENESIS]\x1b[0m");
            println!("Hash:  {}", crate::chain::GENESIS_SEED.escape_ascii());
        } else {
            let event = &self.receipt.events[self.cursor - 1];
            println!("Event: \x1b[1;33m{}\x1b[0m", event.event_type);
            println!("Seq:   {}", event.seq);
            println!("ID:    {}", event.id);
            println!("Comm:  {}", event.payload_commitment.as_hex());
        }
        println!();
    }

    fn inspect_memory(&self) {
        println!("\x1b[1;35m--- MEMORY INSPECTION ---\x1b[0m");
        println!("{:#?}", self.states[self.cursor]);
        println!();
    }

    fn print_help(&self) {
        println!("\x1b[1;33mAVAILABLE COMMANDS:\x1b[0m");
        println!("  \x1b[1;32mf, forward, n, next\x1b[0m    : Step forward to next event");
        println!("  \x1b[1;32mb, backward, p, prev\x1b[0m   : Step backward to previous state");
        println!("  \x1b[1;32mg, goto <tick>\x1b[0m         : Jump to a specific point in time");
        println!("  \x1b[1;32mm, memory, inspect\x1b[0m     : Dump current state memory");
        println!("  \x1b[1;32mh, help\x1b[0m                : Show this help");
        println!("  \x1b[1;32mq, quit, exit\x1b[0m          : Exit REPL");
        println!();
    }
}

/// Macro-based harness for zero-config time-travel debugging.
///
/// Usage:
/// ```rust
/// time_travel_harness!(receipt, initial_state, |state, event| {
///     // transition logic
///     new_state
/// });
/// ```
#[macro_export]
macro_rules! time_travel_harness {
    ($receipt:expr, $initial_state:expr, $transition:expr) => {
        {
            let mut dbg = $crate::TimeTravelDebugger::new($receipt, $initial_state, $transition);
            dbg.run_repl().expect("Time-Travel Debugger REPL failed");
        }
    };
}

// --- SPEC / EXAMPLE USAGE ---

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ocel::{build_event, object_ref, SeqCounter};
    use crate::chain::ChainAssembler;

    #[derive(Debug, Clone, Default)]
    struct MockSystemState {
        counter: i32,
        last_op: String,
        objects_seen: Vec<String>,
    }

    #[test]
    #[ignore] // This test requires user interaction for REPL
    fn test_time_travel_debugger_manual() {
        let mut asm = ChainAssembler::new();
        let mut counter = SeqCounter::new();

        let e1 = build_event("increment", vec![object_ref("c1", "counter")], b"1", &mut counter).unwrap();
        let e2 = build_event("increment", vec![object_ref("c1", "counter")], b"5", &mut counter).unwrap();
        let e3 = build_event("decrement", vec![object_ref("c1", "counter")], b"2", &mut counter).unwrap();

        asm.append(e1).unwrap();
        asm.append(e2).unwrap();
        asm.append(e3).unwrap();

        let receipt = asm.finalize();

        let initial_state = MockSystemState::default();

        // The transition function simulates how our system evolves based on receipt events
        let transition = |state: &MockSystemState, event: &OperationEvent| {
            let mut next = state.clone();
            next.last_op = event.event_type.clone();
            
            for obj in &event.objects {
                if !next.objects_seen.contains(&obj.id) {
                    next.objects_seen.push(obj.id.clone());
                }
            }

            match event.event_type.as_str() {
                "increment" => next.counter += 1, // simplified for mock
                "decrement" => next.counter -= 1,
                _ => {}
            }
            next
        };

        // Invoke the debugger harness
        // In a real DX scenario, this would be triggered via a --debug flag in the CLI
        let mut dbg = TimeTravelDebugger::new(&receipt, initial_state, transition);
        // dbg.run_repl().unwrap();
        
        // Automated verification of state calculation
        assert_eq!(dbg.states.len(), 4);
        assert_eq!(dbg.states[0].counter, 0); // Genesis
        assert_eq!(dbg.states[1].counter, 1); // After e1
        assert_eq!(dbg.states[2].counter, 2); // After e2
        assert_eq!(dbg.states[3].counter, 1); // After e3 (decrement)
    }
}
}
}
