//! Minimal stub for `wasm4pm` crate used by reference tests.
#![allow(unused)]

use std::collections::HashMap;

pub mod models {
    use std::collections::HashMap;

    #[derive(Debug, Clone, PartialEq)]
    pub enum AttributeValue {
        String(std::string::String),
        Int(i64),
        Float(f64),
        Date(std::string::String),
        Boolean(bool),
        List(Vec<AttributeValue>),
        Container(HashMap<std::string::String, AttributeValue>),
        Null,
    }

    #[derive(Debug, Clone)]
    pub struct Event {
        pub attributes: HashMap<std::string::String, AttributeValue>,
    }

    impl Event {
        pub fn new(attributes: HashMap<std::string::String, AttributeValue>) -> Self {
            Event { attributes }
        }
    }

    #[derive(Debug, Clone)]
    pub struct Trace {
        pub events: Vec<Event>,
        pub attributes: HashMap<std::string::String, AttributeValue>,
    }

    #[derive(Debug, Clone)]
    pub struct DFGNode {
        pub id: std::string::String,
        pub label: std::string::String,
        pub frequency: usize,
    }

    #[derive(Debug, Clone)]
    pub struct DirectlyFollowsRelation {
        pub from: std::string::String,
        pub to: std::string::String,
        pub frequency: usize,
    }

    #[derive(Debug, Clone, Default)]
    pub struct DFG {
        pub nodes: Vec<DFGNode>,
        pub edges: Vec<DirectlyFollowsRelation>,
        pub start_activities: HashMap<std::string::String, usize>,
        pub end_activities: HashMap<std::string::String, usize>,
    }

    impl DFG {
        pub fn new() -> Self { DFG::default() }
    }
}
