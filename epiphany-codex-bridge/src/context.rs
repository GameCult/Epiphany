use epiphany_core::EpiphanyContextParams;
use epiphany_core::EpiphanyContextView;
use epiphany_core::EpiphanyGraphQuery;
use epiphany_core::EpiphanyGraphQueryView;
use epiphany_core::EpiphanyPlanningView;
use epiphany_core::derive_context;
use epiphany_core::derive_graph_query;
use epiphany_core::derive_planning_view;
use epiphany_state_model::EpiphanyThreadState;

pub fn derive_epiphany_context(
    state: Option<&EpiphanyThreadState>,
    params: &EpiphanyContextParams,
) -> EpiphanyContextView {
    derive_context(state, params)
}

pub fn derive_epiphany_planning(state: Option<&EpiphanyThreadState>) -> EpiphanyPlanningView {
    derive_planning_view(state)
}

pub fn derive_epiphany_graph_query(
    state: Option<&EpiphanyThreadState>,
    query: &EpiphanyGraphQuery,
) -> EpiphanyGraphQueryView {
    derive_graph_query(state, query)
}
