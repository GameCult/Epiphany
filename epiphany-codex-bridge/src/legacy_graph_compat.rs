use codex_protocol::protocol::EpiphanyThreadState;
use epiphany_core::EpiphanyMemoryGraphSnapshot;
use epiphany_core::epiphany_graphs_from_memory_graph;

pub fn thread_state_with_legacy_graph_projection(
    state: Option<&EpiphanyThreadState>,
    memory_graph_snapshot: Option<&EpiphanyMemoryGraphSnapshot>,
) -> Option<EpiphanyThreadState> {
    state.zip(memory_graph_snapshot).map(|(state, snapshot)| {
        let mut state = state.clone();
        state.graphs = epiphany_graphs_from_memory_graph(snapshot);
        state
    })
}
