//! Trait-based node definitions for external DSP nodes.

#![forbid(unsafe_code)]

use crate::graph::Port;
use std::any::Any;

/// Object-safe node definition for external nodes.
pub trait NodeDefDyn: Send + Sync {
    fn input_ports(&self) -> &'static [Port];
    fn output_ports(&self) -> &'static [Port];
    fn required_inputs(&self) -> usize;
    fn init_state(&self, sample_rate: f32, block_size: usize) -> Box<dyn Any + Send>;
    fn process_block(
        &self,
        state: &mut dyn Any,
        inputs: &[&[f32]],
        outputs: &mut [Vec<f32>],
        sample_rate: f32,
    ) -> Result<(), &'static str>;
}

/// Generic node definition; implement this for your DSP nodes.
pub trait NodeDef: Send + Sync + 'static {
    type State: Send + 'static;
    fn input_ports(&self) -> &'static [Port];
    fn output_ports(&self) -> &'static [Port];
    fn required_inputs(&self) -> usize;
    fn init_state(&self, sample_rate: f32, block_size: usize) -> Self::State;
    fn process_block(
        &self,
        state: &mut Self::State,
        inputs: &[&[f32]],
        outputs: &mut [Vec<f32>],
        sample_rate: f32,
    ) -> Result<(), &'static str>;
}

impl<T: NodeDef> NodeDefDyn for T {
    fn input_ports(&self) -> &'static [Port] {
        <T as NodeDef>::input_ports(self)
    }

    fn output_ports(&self) -> &'static [Port] {
        <T as NodeDef>::output_ports(self)
    }

    fn required_inputs(&self) -> usize {
        <T as NodeDef>::required_inputs(self)
    }

    fn init_state(&self, sample_rate: f32, block_size: usize) -> Box<dyn Any + Send> {
        Box::new(<T as NodeDef>::init_state(self, sample_rate, block_size))
    }

    fn process_block(
        &self,
        state: &mut dyn Any,
        inputs: &[&[f32]],
        outputs: &mut [Vec<f32>],
        sample_rate: f32,
    ) -> Result<(), &'static str> {
        // Downcast to concrete state; if type mismatch, return error.
        if let Some(typed) = state.downcast_mut::<<T as NodeDef>::State>() {
            <T as NodeDef>::process_block(self, typed, inputs, outputs, sample_rate)
        } else {
            // Type mismatch: this indicates a wiring bug in runtime state initialization.
            Err("State type mismatch in External node process_block - this indicates a wiring bug")
        }
    }
}
