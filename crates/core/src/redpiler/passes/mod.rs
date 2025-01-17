mod clamp_weights;
mod coalesce;
mod constant_coalesce;
mod constant_fold;
mod dedup_links;
mod identify_nodes;
mod input_search;
mod unreachable_output;

use super::compile_graph::CompileGraph;
use super::{CompilerInput, CompilerOptions};
use std::time::Instant;
use tracing::trace;

pub const DEFAULT_PASS_MANAGER: PassManager<'_> = PassManager::new(&[
    &identify_nodes::IdentifyNodes,
    &input_search::InputSearch,
    &clamp_weights::ClampWeights,
    &dedup_links::DedupLinks,
    &constant_fold::ConstantFold,
    &unreachable_output::UnreachableOutput,
    &constant_coalesce::ConstantCoalesce,
    &coalesce::Coalesce,
]);

pub struct PassManager<'p> {
    passes: &'p [&'p dyn Pass],
}

impl<'p> PassManager<'p> {
    pub const fn new(passes: &'p [&dyn Pass]) -> Self {
        Self { passes }
    }

    pub fn run_passes(&self, options: &CompilerOptions, input: CompilerInput<'_>) -> CompileGraph {
        let mut graph = CompileGraph::new();

        for &pass in self.passes {
            if !pass.should_run(options) {
                trace!("Skipping pass: {}", pass.name());
                continue;
            }

            trace!("Running pass: {}", pass.name());
            let start = Instant::now();

            pass.run_pass(&mut graph, options, &input);

            trace!("Completed pass in {:?}", start.elapsed());
            trace!("node_count: {}", graph.node_count());
            trace!("edge_count: {}", graph.edge_count());
        }

        graph
    }
}

pub trait Pass {
    fn run_pass(
        &self,
        graph: &mut CompileGraph,
        options: &CompilerOptions,
        input: &CompilerInput<'_>,
    );

    /// This name should only be use for debugging purposes,
    /// it is not a valid identifier of the pass.
    fn name(&self) -> &'static str {
        std::any::type_name::<Self>()
    }

    fn should_run(&self, options: &CompilerOptions) -> bool {
        // Run passes for optimized builds by default
        options.optimize
    }
}
