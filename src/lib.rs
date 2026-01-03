//! # Auxide
//!
//! A real-time-safe, deterministic, block-based audio graph kernel for building audio tools.
//!
//! ## Architecture
//!
//! The core flow is: **Graph → Plan → Runtime**.
//!
//! - **Graph**: Define nodes and edges.
//! - **Plan**: Compile the graph into an execution schedule.
//! - **Runtime**: Process audio blocks deterministically.
//!
//! ## Real-Time Safety
//!
//! RT paths (e.g., `Runtime::process_block`) avoid allocations and locking. Graph mutation and plan compilation may allocate.
//!
//! ## Determinism
//!
//! Given the same graph, plan, and inputs, outputs are identical (modulo floating-point precision).
//!
//! ## Invariants
//!
//! - Only one edge may write to a given input port (single-writer rule).
//! - No cycles unless involving delay nodes.
//! - All ports must have compatible rates.
//!
//! ## Example
//!
//! ```rust
//! use auxide::graph::{Graph, NodeType, PortId, Rate};
//! use auxide::plan::Plan;
//! use auxide::rt::Runtime;
//!
//! let mut graph = Graph::new();
//! let osc = graph.add_node(NodeType::SineOsc { freq: 440.0 });
//! let sink = graph.add_node(NodeType::OutputSink);
//! graph.add_edge(auxide::graph::Edge {
//!     from_node: osc,
//!     from_port: PortId(0),
//!     to_node: sink,
//!     to_port: PortId(0),
//!     rate: Rate::Audio,
//! }).unwrap();
//!
//! let plan = Plan::compile(&graph, 64).unwrap();
//! let mut runtime = Runtime::new(plan, &graph, 44100.0);
//! let mut out = vec![0.0; 64];
//! runtime.process_block(&mut out);
//! ```

#![doc(
    html_logo_url = "https://raw.githubusercontent.com/Michael-A-Kuykendall/auxide/master/assets/auxide-logo.png"
)]

pub mod dsl;
pub mod graph;
pub mod plan;
pub mod rt;
