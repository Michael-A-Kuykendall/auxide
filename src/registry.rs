//! UGen registry: maps UGen names to factories that build kernel `NodeType`s.
//!
//! This module lives in the kernel (`auxide`) on purpose: both `auxide-server`
//! (which owns the registry instance) and `auxide-dsp` (which populates it via
//! `register_dsp_ugens`) depend on `auxide`, so the registry type is the one
//! shared root they can both name without inverting the dependency direction.
//!
//! A factory turns a [`ParamMap`] (named, string-keyed parameters) into a
//! concrete [`auxide::graph::NodeType`] — either a built-in variant (e.g.
//! `SineOsc`) or an `External` wrapping an `Arc<dyn NodeDefDyn>`.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

use crate::graph::NodeType;
use crate::node::NodeDef;
use std::collections::HashMap;
use std::sync::Arc;

/// Parameters passed to a UGen factory when instantiating a node.
///
/// Keys are UGen-specific (e.g. `"freq"`, `"cutoff"`, `"resonance"`). Missing
/// keys fall back to sensible defaults inside each factory, so a `SynthDef`
/// only needs to specify the parameters it cares about.
pub type ParamMap = HashMap<String, f32>;

/// A factory that builds a kernel [`NodeType`] from a parameter map.
///
/// Stored boxed in the [`Registry`]. Any closure `for<'a> Fn(&'a ParamMap) ->
/// NodeType` qualifies, so registering a UGen is a one-liner:
///
/// ```ignore
/// reg.register("sine", |p: &ParamMap| NodeType::SineOsc {
///     freq: param_or(p, "freq", 440.0),
/// });
/// ```
pub type UgenFactory = Box<dyn for<'a> Fn(&'a ParamMap) -> NodeType + Send + Sync>;

/// Errors that can occur when looking up or using a registry entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RegistryError {
    /// No UGen is registered under the requested name.
    UnknownUgen(String),
}

/// A registry mapping UGen names (e.g. `"sine"`, `"svf"`) to factories.
///
/// `auxide-server` owns the instance; `auxide-dsp::register_dsp_ugens` and the
/// server's own built-in registrations populate it at init.
pub struct Registry {
    factories: HashMap<String, UgenFactory>,
}

impl Default for Registry {
    fn default() -> Self {
        Self::new()
    }
}

impl Registry {
    /// Create an empty registry.
    pub fn new() -> Self {
        Registry {
            factories: HashMap::new(),
        }
    }

    /// Register a UGen factory under `name`. Re-registration replaces.
    pub fn register<F>(&mut self, name: &str, factory: F)
    where
        F: for<'a> Fn(&'a ParamMap) -> NodeType + Send + Sync + 'static,
    {
        self.factories.insert(name.to_string(), Box::new(factory));
    }

    /// Look up a factory by name.
    pub fn get(&self, name: &str) -> Option<&UgenFactory> {
        self.factories.get(name)
    }

    /// Whether a UGen is registered under `name`.
    pub fn contains(&self, name: &str) -> bool {
        self.factories.contains_key(name)
    }

    /// All registered UGen names.
    pub fn names(&self) -> impl Iterator<Item = &String> {
        self.factories.keys()
    }

    /// Number of registered UGens.
    pub fn len(&self) -> usize {
        self.factories.len()
    }

    /// Whether the registry is empty.
    pub fn is_empty(&self) -> bool {
        self.factories.is_empty()
    }

    /// Convenience: build a `NodeType` from a registered UGen name + params.
    pub fn create(&self, name: &str, params: &ParamMap) -> Result<NodeType, RegistryError> {
        self.factories
            .get(name)
            .map(|f| f(params))
            .ok_or_else(|| RegistryError::UnknownUgen(name.to_string()))
    }
}

/// Helper for factories: fetch a `f32` parameter with a default.
pub fn param_or(params: &ParamMap, key: &str, default: f32) -> f32 {
    params.get(key).copied().unwrap_or(default)
}

/// Helper for factories: build an `External` node type from any `NodeDef`.
pub fn external<T: NodeDef + 'static>(def: T) -> NodeType {
    NodeType::External { def: Arc::new(def) }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::NodeType;

    #[test]
    fn register_and_create() {
        let mut reg = Registry::new();
        reg.register("sine", |p: &ParamMap| NodeType::SineOsc {
            freq: param_or(p, "freq", 440.0),
        });
        assert!(reg.contains("sine"));
        assert_eq!(reg.len(), 1);
        let nt = reg.create("sine", &ParamMap::new()).unwrap();
        match nt {
            NodeType::SineOsc { freq } => assert_eq!(freq, 440.0),
            _ => panic!("wrong node type"),
        }
        // Missing name -> error.
        assert!(matches!(
            reg.create("nope", &ParamMap::new()),
            Err(RegistryError::UnknownUgen(_))
        ));
    }
}
