use std::collections::BTreeMap;

use anyhow::{Result, ensure};
use chrono::DateTime;
use cultcache_rs::DatabaseEntry;
use serde::{Deserialize, Serialize};

use super::contracts::{
    ATLAS_PROJECTION_SCHEMA, AtlasBodyEvidenceRef, AtlasCompatibility, AtlasContractDescriptor,
    AtlasContractRequirement, AtlasCycleClass, AtlasEntanglementKind, AtlasEntanglementProjection,
    AtlasFailureSemantics, AtlasImpactScope, AtlasOfferLifecycle, AtlasProjectedEntanglement,
    AtlasPublicationFreshness, AtlasRepositoryIdentity, AtlasVerificationState,
};
use super::projector::atlas_projection_digest;

pub const EVE_SURFACE_SCHEMA: &str = "gamecult.eve.surface.v1";
pub const EVE_COMMAND_SCHEMA: &str = "gamecult.eve.command.v1";
pub const EVE_PROVIDER_ADVERTISEMENT_SCHEMA: &str = "gamecult.eve.provider_advertisement.v1";

pub const MODEL_ATLAS_PROVIDER_ID: &str = "epiphany.model-atlas";
pub const MODEL_ATLAS_SERVICE_ID: &str = "epiphany.model-entanglement-projector";
pub const MODEL_ATLAS_SURFACE_ID: &str = "epiphany.model-atlas.surface";
pub const MODEL_ATLAS_VERSE_ID: &str = "gamecult-local";
pub const MODEL_ATLAS_CANONICAL_SERVICE: &str = "asgard.epiphany.model-atlas";
pub const MODEL_ATLAS_LOCATED_SERVICE: &str = "asgard.starfire.epiphany.model-atlas";
pub const MODEL_ATLAS_CULTMESH_ADDRESS: &str =
    "cultmesh://gamecult-local/asgard/starfire/epiphany/model-atlas";
pub const MODEL_ATLAS_PROJECTION_SOURCE: &str =
    "cultmesh://gamecult-local/asgard/starfire/epiphany/model-atlas/entanglements";
pub const MODEL_ATLAS_DOMAIN_MUTATION_ROUTE: &str =
    "gamecult://swarm/{swarm_id}/workspace/{workspace_id}/epiphany/modeling";
pub const MODEL_ATLAS_SELECT_COMMAND: &str = "epiphany.model-atlas.presentation.select";
pub const MODEL_ATLAS_FILTER_COMMAND: &str = "epiphany.model-atlas.presentation.filter";
pub const MODEL_ATLAS_PRESENTATION_AUTHORITY: &str = "epiphany.model-atlas.presentation";

// These are foreign contract evidence owned by GameCult/Eve. They are not
// Epiphany-local schemas or renderer-owned state.
pub const EVE_SURFACE_SCHEMA_REF: &str = "schemas/gamecult.eve.surface.v1.schema.json";
pub const EVE_MODEL_ATLAS_CONFORMANCE_REF: &str = "web/fixtures/epiphany-model-atlas-surface.json";
pub const EVE_MODEL_ATLAS_ADVERTISEMENT_CONFORMANCE_REF: &str =
    "web/fixtures/epiphany-model-atlas.provider-advertisement.json";

const MODEL_ATLAS_TITLE: &str = "Epiphany Model Atlas";
const MODEL_ATLAS_PROVIDER_KIND: &str = "modeling.projection";
const MODEL_ATLAS_SURFACE_KEY: &str = "epiphany:surface:model-atlas";
const MODEL_ATLAS_ROOT_ID: &str = "epiphany.model-atlas.root";
const MODEL_ATLAS_ATTENTION_ID: &str = "epiphany.model-atlas.attention";
const MODEL_ATLAS_DRILLDOWN_ID: &str = "epiphany.model-atlas.drilldown";
const MODEL_ATLAS_GRAPH_ID: &str = "epiphany.model-atlas.graph";
const PROVIDER_ADVERTISEMENT_MAX_AGE_MS: u64 = 15_000;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AtlasEveDocuments {
    pub advertisement: EveProviderAdvertisement,
    pub surface: EveSurfaceDocument,
}

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct AtlasEvePresentationState {
    pub attention_filter: AtlasEveAttentionFilter,
    pub selected_entanglement_id: Option<String>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum AtlasEveAttentionFilter {
    #[default]
    NeedsAttention,
    Degraded,
    Disputed,
    Stale,
    All,
}

impl AtlasEveAttentionFilter {
    fn as_wire_name(self) -> &'static str {
        match self {
            Self::NeedsAttention => "needs-attention",
            Self::Degraded => "degraded",
            Self::Disputed => "disputed",
            Self::Stale => "stale",
            Self::All => "all",
        }
    }

    fn accepts(self, state: AtlasEveEntanglementState) -> bool {
        match self {
            Self::NeedsAttention => matches!(
                state,
                AtlasEveEntanglementState::Degraded
                    | AtlasEveEntanglementState::Disputed
                    | AtlasEveEntanglementState::Stale
            ),
            Self::Degraded => state == AtlasEveEntanglementState::Degraded,
            Self::Disputed => state == AtlasEveEntanglementState::Disputed,
            Self::Stale => state == AtlasEveEntanglementState::Stale,
            Self::All => true,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum AtlasEveEntanglementState {
    Current,
    Degraded,
    Disputed,
    Stale,
    Retired,
}

impl AtlasEveEntanglementState {
    fn as_wire_name(self) -> &'static str {
        match self {
            Self::Current => "current",
            Self::Degraded => "degraded",
            Self::Disputed => "disputed",
            Self::Stale => "stale",
            Self::Retired => "retired",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EveProviderAdvertisement {
    pub schema: String,
    pub provider_id: String,
    pub service_id: String,
    pub verse_id: String,
    pub root_verse: String,
    pub canonical_service: String,
    pub located_service: String,
    pub cult_mesh_address: String,
    pub title: String,
    pub kind: String,
    pub updated_at_utc: String,
    pub freshness: EveProviderFreshness,
    pub schemas: Vec<String>,
    pub witnesses: Vec<EveContractWitness>,
    pub surfaces: Vec<EveSurfaceAdvertisement>,
    pub commands: Vec<EveAdvertisedCommand>,
}

impl DatabaseEntry for EveProviderAdvertisement {
    const TYPE: &'static str = "gamecult.eve.provider_advertisement";
    const SCHEMA_NAME: &'static str = EVE_PROVIDER_ADVERTISEMENT_SCHEMA;
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EveProviderFreshness {
    pub state: String,
    pub last_seen_at_utc: String,
    pub max_age_ms: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EveContractWitness {
    pub kind: String,
    #[serde(rename = "ref")]
    pub reference: String,
    pub summary: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EveSurfaceAdvertisement {
    pub schema: String,
    pub surface_id: String,
    pub key: String,
    pub transport: String,
    pub status: String,
    pub audience: String,
    pub mode: String,
    pub surface_kind: String,
    pub interaction_model: String,
    pub lowering_targets: Vec<String>,
    pub state_schemas: Vec<String>,
    pub ownership: String,
    pub domain_state_owner: String,
    pub domain_mutation_route: String,
    pub summary: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EveAdvertisedCommand {
    pub command: String,
    pub schema: String,
    pub transport: String,
    pub authority: String,
    pub presentation_only: bool,
    pub domain_state_effects: String,
    pub summary: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EveSurfaceDocument {
    #[serde(rename = "type")]
    pub document_type: String,
    pub schema: String,
    pub provider_id: String,
    pub provider_kind: String,
    pub title: String,
    pub version: u64,
    pub updated_at_utc: String,
    pub surface: EveRetainedSurface,
    pub commands: Vec<EveCommandDescriptor>,
}

impl DatabaseEntry for EveSurfaceDocument {
    const TYPE: &'static str = "gamecult.eve.surface";
    const SCHEMA_NAME: &'static str = EVE_SURFACE_SCHEMA;
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EveRetainedSurface {
    pub id: String,
    pub root: EveComponent,
    pub styles: EveStyleSheet,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EveComponent {
    pub id: String,
    pub kind: String,
    pub props: EveComponentProps,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub state_bindings: Vec<EveStateBinding>,
    pub children: Vec<EveComponent>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum EveComponentProps {
    Root(EveRootProps),
    Select(EveSelectProps),
    Graph(EveGraphProps),
    Panel(EvePanelProps),
    List(EveListProps),
    Text(EveTextProps),
    Metric(EveMetricProps),
    Rail(EveRailProps),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EveRootProps {
    pub default_view: String,
    pub state_authority: String,
    pub presentation_state: String,
    pub view_order: Vec<String>,
    pub domain_state_owner: String,
    pub domain_mutation_route: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EvePanelProps {
    pub title: String,
    pub view_id: String,
    pub role: String,
    #[serde(default, skip_serializing_if = "is_false")]
    pub default: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EveRailProps {
    pub role: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EveSelectProps {
    pub label: String,
    pub value: String,
    pub options: Vec<String>,
    pub command: String,
    pub command_id: String,
    pub presentation_only: bool,
    pub domain_state_effects: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EveListProps {
    pub title: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
    pub items: Vec<EveListItem>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EveTextProps {
    pub text: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EveMetricProps {
    pub label: String,
    pub value: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EveGraphProps {
    pub title: String,
    pub view_id: String,
    pub role: String,
    pub presentation_fallback: String,
    pub nodes: Vec<EveGraphNode>,
    pub edges: Vec<EveGraphEdge>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EveListItem {
    pub label: String,
    pub status: String,
    pub detail: String,
    pub badges: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EveGraphNode {
    pub id: String,
    pub label: String,
    pub owner: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EveGraphEdge {
    pub id: String,
    pub from: String,
    pub to: String,
    pub state: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EveStateBinding {
    pub target_prop: String,
    pub pointer_id: String,
    pub source_id: String,
    pub schema_id: String,
    pub route_kind: String,
    pub route_description: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EveStyleSheet {
    pub tokens: EveStyleTokens,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EveStyleTokens {
    pub color_background: String,
    pub color_panel: String,
    pub color_panel_alt: String,
    pub color_text: String,
    pub color_muted: String,
    pub color_accent: String,
    pub color_warning: String,
    pub color_danger: String,
    pub font_body: String,
    pub font_title: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EveCommandDescriptor {
    pub schema: String,
    pub command: String,
    pub label: String,
    pub surface_id: String,
    pub transport: String,
    pub authority: String,
    pub result: String,
    pub presentation_only: bool,
    pub domain_state_effects: String,
}

/// Projects one typed Modeling snapshot into the single retained tree lowered
/// by both Eve GUI and TUI runtimes. The presentation input may select and
/// filter that tree; it has no authority to mutate Atlas domain state.
pub fn project_model_atlas_eve_documents(
    projection: &AtlasEntanglementProjection,
    presentation: &AtlasEvePresentationState,
    surface_version: u64,
    updated_at_utc: &str,
) -> Result<AtlasEveDocuments> {
    validate_projection(projection)?;
    validate_utc_timestamp(updated_at_utc)?;

    Ok(AtlasEveDocuments {
        advertisement: provider_advertisement(updated_at_utc),
        surface: surface_document(projection, presentation, surface_version, updated_at_utc),
    })
}

fn provider_advertisement(updated_at_utc: &str) -> EveProviderAdvertisement {
    EveProviderAdvertisement {
        schema: EVE_PROVIDER_ADVERTISEMENT_SCHEMA.into(),
        provider_id: MODEL_ATLAS_PROVIDER_ID.into(),
        service_id: MODEL_ATLAS_SERVICE_ID.into(),
        verse_id: MODEL_ATLAS_VERSE_ID.into(),
        root_verse: "asgard".into(),
        canonical_service: MODEL_ATLAS_CANONICAL_SERVICE.into(),
        located_service: MODEL_ATLAS_LOCATED_SERVICE.into(),
        cult_mesh_address: MODEL_ATLAS_CULTMESH_ADDRESS.into(),
        title: MODEL_ATLAS_TITLE.into(),
        kind: MODEL_ATLAS_PROVIDER_KIND.into(),
        updated_at_utc: updated_at_utc.into(),
        freshness: EveProviderFreshness {
            state: "fresh".into(),
            last_seen_at_utc: updated_at_utc.into(),
            max_age_ms: PROVIDER_ADVERTISEMENT_MAX_AGE_MS,
        },
        schemas: vec![
            ATLAS_PROJECTION_SCHEMA.into(),
            EVE_SURFACE_SCHEMA.into(),
            EVE_COMMAND_SCHEMA.into(),
        ],
        witnesses: vec![
            EveContractWitness {
                kind: "schema".into(),
                reference: EVE_SURFACE_SCHEMA_REF.into(),
                summary: "Canonical retained-surface wire contract owned by GameCult/Eve."
                    .into(),
            },
            EveContractWitness {
                kind: "fixture".into(),
                reference: EVE_MODEL_ATLAS_CONFORMANCE_REF.into(),
                summary: "Eve conformance witness for an Epiphany-owned cross-repo Modeling projection. The fixture is presentation, not entanglement authority."
                    .into(),
            },
            EveContractWitness {
                kind: "fixture".into(),
                reference: EVE_MODEL_ATLAS_ADVERTISEMENT_CONFORMANCE_REF.into(),
                summary: "Eve conformance witness for the Model Atlas provider advertisement."
                    .into(),
            },
        ],
        surfaces: vec![EveSurfaceAdvertisement {
            schema: EVE_SURFACE_SCHEMA.into(),
            surface_id: MODEL_ATLAS_SURFACE_ID.into(),
            key: MODEL_ATLAS_SURFACE_KEY.into(),
            transport: "cultmesh-record".into(),
            status: "available".into(),
            audience: "operator".into(),
            mode: "interactive".into(),
            surface_kind: "model-atlas".into(),
            interaction_model: "provider-presentation-commands".into(),
            lowering_targets: vec!["gui".into(), "tui".into()],
            state_schemas: vec![ATLAS_PROJECTION_SCHEMA.into()],
            ownership:
                "epiphany-owns-entanglement-projection-eve-renders-one-retained-tree"
                    .into(),
            domain_state_owner: MODEL_ATLAS_CANONICAL_SERVICE.into(),
            domain_mutation_route: MODEL_ATLAS_DOMAIN_MUTATION_ROUTE.into(),
            summary: "Attention-first cross-repo Modeling view with drilldown and a secondary graph from one typed CultMesh projection."
                .into(),
        }],
        commands: presentation_command_advertisements(),
    }
}

fn surface_document(
    projection: &AtlasEntanglementProjection,
    presentation: &AtlasEvePresentationState,
    surface_version: u64,
    updated_at_utc: &str,
) -> EveSurfaceDocument {
    let selected = selected_entanglement(projection, presentation);
    let selected_id = selected.map(entanglement_id).unwrap_or_default();
    let options = projection
        .entanglements
        .iter()
        .map(entanglement_id)
        .collect::<Vec<_>>();

    EveSurfaceDocument {
        document_type: "surface-state".into(),
        schema: EVE_SURFACE_SCHEMA.into(),
        provider_id: MODEL_ATLAS_PROVIDER_ID.into(),
        provider_kind: MODEL_ATLAS_PROVIDER_KIND.into(),
        title: MODEL_ATLAS_TITLE.into(),
        version: surface_version,
        updated_at_utc: updated_at_utc.into(),
        surface: EveRetainedSurface {
            id: MODEL_ATLAS_SURFACE_ID.into(),
            root: EveComponent {
                id: MODEL_ATLAS_ROOT_ID.into(),
                kind: "surface".into(),
                props: EveComponentProps::Root(EveRootProps {
                    default_view: "attention".into(),
                    state_authority: ATLAS_PROJECTION_SCHEMA.into(),
                    presentation_state: "provider-owned-ephemeral".into(),
                    view_order: vec!["attention".into(), "drilldown".into(), "graph".into()],
                    domain_state_owner: MODEL_ATLAS_CANONICAL_SERVICE.into(),
                    domain_mutation_route: MODEL_ATLAS_DOMAIN_MUTATION_ROUTE.into(),
                }),
                state_bindings: Vec::new(),
                children: vec![
                    attention_component(projection, presentation, selected_id, options),
                    drilldown_component(projection, selected),
                    graph_component(projection),
                ],
            },
            styles: model_atlas_styles(),
        },
        commands: presentation_command_descriptors(),
    }
}

fn attention_component(
    projection: &AtlasEntanglementProjection,
    presentation: &AtlasEvePresentationState,
    selected_id: String,
    options: Vec<String>,
) -> EveComponent {
    let items = projection
        .entanglements
        .iter()
        .filter(|edge| {
            presentation
                .attention_filter
                .accepts(entanglement_state(edge))
        })
        .map(|edge| entanglement_item(projection, edge))
        .collect();

    EveComponent {
        id: MODEL_ATLAS_ATTENTION_ID.into(),
        kind: "panel".into(),
        props: EveComponentProps::Panel(EvePanelProps {
            title: "Attention".into(),
            view_id: "attention".into(),
            role: "primary".into(),
            default: true,
        }),
        state_bindings: Vec::new(),
        children: vec![
            EveComponent {
                id: "epiphany.model-atlas.attention.controls".into(),
                kind: "rail".into(),
                props: EveComponentProps::Rail(EveRailProps {
                    role: "presentation-controls".into(),
                }),
                state_bindings: Vec::new(),
                children: vec![
                    EveComponent {
                        id: "epiphany.model-atlas.attention.filter".into(),
                        kind: "control.select".into(),
                        props: EveComponentProps::Select(EveSelectProps {
                            label: "Filter".into(),
                            value: presentation.attention_filter.as_wire_name().into(),
                            options: vec![
                                "needs-attention".into(),
                                "degraded".into(),
                                "disputed".into(),
                                "stale".into(),
                                "all".into(),
                            ],
                            command: MODEL_ATLAS_FILTER_COMMAND.into(),
                            command_id: MODEL_ATLAS_FILTER_COMMAND.into(),
                            presentation_only: true,
                            domain_state_effects: "none".into(),
                        }),
                        state_bindings: Vec::new(),
                        children: Vec::new(),
                    },
                    EveComponent {
                        id: "epiphany.model-atlas.attention.select".into(),
                        kind: "control.select".into(),
                        props: EveComponentProps::Select(EveSelectProps {
                            label: "Selected entanglement".into(),
                            value: selected_id,
                            options,
                            command: MODEL_ATLAS_SELECT_COMMAND.into(),
                            command_id: MODEL_ATLAS_SELECT_COMMAND.into(),
                            presentation_only: true,
                            domain_state_effects: "none".into(),
                        }),
                        state_bindings: Vec::new(),
                        children: Vec::new(),
                    },
                ],
            },
            EveComponent {
                id: "epiphany.model-atlas.attention.items".into(),
                kind: "list".into(),
                props: EveComponentProps::List(EveListProps {
                    title: "Relationships requiring attention".into(),
                    role: None,
                    items,
                }),
                state_bindings: vec![binding(
                    "items",
                    "epiphany.model-atlas.attention.items",
                    "attention",
                    "CultMesh projection of entanglements currently requiring operator or owner attention.",
                )],
                children: Vec::new(),
            },
        ],
    }
}

fn drilldown_component(
    projection: &AtlasEntanglementProjection,
    selected: Option<&AtlasProjectedEntanglement>,
) -> EveComponent {
    let title = selected
        .map(entanglement_label)
        .unwrap_or_else(|| "Selected entanglement".into());
    let summary = selected
        .map(|edge| entanglement_summary(projection, edge))
        .unwrap_or_else(|| "No entanglement is present in the current projection.".into());
    let status = selected
        .map(entanglement_state)
        .map(AtlasEveEntanglementState::as_wire_name)
        .unwrap_or("empty");
    let claims = selected.map(claim_items).unwrap_or_default();
    let evidence = selected
        .map(|edge| evidence_items(projection, edge))
        .unwrap_or_default();

    EveComponent {
        id: MODEL_ATLAS_DRILLDOWN_ID.into(),
        kind: "panel".into(),
        props: EveComponentProps::Panel(EvePanelProps {
            title,
            view_id: "drilldown".into(),
            role: "detail".into(),
            default: false,
        }),
        state_bindings: vec![binding(
            "title",
            "epiphany.model-atlas.selected.title",
            "selected.title",
            "Provider-authored title for the selected entanglement.",
        )],
        children: vec![
            EveComponent {
                id: "epiphany.model-atlas.drilldown.summary".into(),
                kind: "text".into(),
                props: EveComponentProps::Text(EveTextProps { text: summary }),
                state_bindings: vec![binding(
                    "text",
                    "epiphany.model-atlas.selected.summary",
                    "selected.summary",
                    "Provider-authored relationship summary.",
                )],
                children: Vec::new(),
            },
            EveComponent {
                id: "epiphany.model-atlas.drilldown.status".into(),
                kind: "metric".into(),
                props: EveComponentProps::Metric(EveMetricProps {
                    label: "State".into(),
                    value: status.into(),
                }),
                state_bindings: vec![binding(
                    "value",
                    "epiphany.model-atlas.selected.state",
                    "selected.state",
                    "Derived entanglement state owned by Epiphany Modeling.",
                )],
                children: Vec::new(),
            },
            EveComponent {
                id: "epiphany.model-atlas.drilldown.claims".into(),
                kind: "list".into(),
                props: EveComponentProps::List(EveListProps {
                    title: "Owned claims".into(),
                    role: None,
                    items: claims,
                }),
                state_bindings: vec![binding(
                    "items",
                    "epiphany.model-atlas.selected.claims",
                    "selected.claims",
                    "Endpoint-owned claims retained without merging their authority.",
                )],
                children: Vec::new(),
            },
            EveComponent {
                id: "epiphany.model-atlas.drilldown.evidence".into(),
                kind: "list".into(),
                props: EveComponentProps::List(EveListProps {
                    title: "Verification".into(),
                    role: None,
                    items: evidence,
                }),
                state_bindings: vec![binding(
                    "items",
                    "epiphany.model-atlas.selected.evidence",
                    "selected.evidence",
                    "Verification evidence projected by Epiphany Modeling.",
                )],
                children: Vec::new(),
            },
        ],
    }
}

fn graph_component(projection: &AtlasEntanglementProjection) -> EveComponent {
    let (nodes, edges) = graph_values(projection);
    let linear_items = projection
        .entanglements
        .iter()
        .map(|edge| entanglement_item(projection, edge))
        .collect();

    EveComponent {
        id: MODEL_ATLAS_GRAPH_ID.into(),
        kind: "graph".into(),
        props: EveComponentProps::Graph(EveGraphProps {
            title: "Swarm topology".into(),
            view_id: "graph".into(),
            role: "secondary".into(),
            presentation_fallback: "children".into(),
            nodes,
            edges,
        }),
        state_bindings: vec![
            binding(
                "nodes",
                "epiphany.model-atlas.graph.nodes",
                "graph.nodes",
                "Derived repository nodes for graphical lowering.",
            ),
            binding(
                "edges",
                "epiphany.model-atlas.graph.edges",
                "graph.edges",
                "Derived entanglement edges for graphical lowering.",
            ),
        ],
        children: vec![EveComponent {
            id: "epiphany.model-atlas.graph.linear".into(),
            kind: "list".into(),
            props: EveComponentProps::List(EveListProps {
                title: "Topology".into(),
                role: Some("linear-fallback".into()),
                items: linear_items,
            }),
            state_bindings: vec![binding(
                "items",
                "epiphany.model-atlas.graph.linear",
                "graph.linear",
                "Linear projection of the same graph for compact and TUI lowerings.",
            )],
            children: Vec::new(),
        }],
    }
}

fn selected_entanglement<'a>(
    projection: &'a AtlasEntanglementProjection,
    presentation: &AtlasEvePresentationState,
) -> Option<&'a AtlasProjectedEntanglement> {
    presentation
        .selected_entanglement_id
        .as_deref()
        .and_then(|selected| {
            projection
                .entanglements
                .iter()
                .find(|edge| entanglement_id(edge) == selected)
        })
        .or_else(|| {
            projection.entanglements.iter().find(|edge| {
                presentation
                    .attention_filter
                    .accepts(entanglement_state(edge))
            })
        })
        .or_else(|| projection.entanglements.first())
}

fn entanglement_state(edge: &AtlasProjectedEntanglement) -> AtlasEveEntanglementState {
    if edge.compatibility == AtlasCompatibility::ClaimRetired
        || edge.claim_freshness == AtlasPublicationFreshness::Retired
    {
        AtlasEveEntanglementState::Retired
    } else if matches!(
        edge.compatibility,
        AtlasCompatibility::ContractIdMismatch
            | AtlasCompatibility::VersionSchemeMismatch
            | AtlasCompatibility::VersionMismatch
    ) || matches!(
        edge.verification,
        AtlasVerificationState::Failed | AtlasVerificationState::ExactEdgeMismatch
    ) {
        AtlasEveEntanglementState::Disputed
    } else if edge.claim_freshness == AtlasPublicationFreshness::LastKnownStale
        || edge.offer_freshness == Some(AtlasPublicationFreshness::LastKnownStale)
        || edge.verification == AtlasVerificationState::LastKnownStale
    {
        AtlasEveEntanglementState::Stale
    } else if matches!(
        edge.compatibility,
        AtlasCompatibility::Unresolved
            | AtlasCompatibility::OfferMissing
            | AtlasCompatibility::OfferWithdrawn
    ) || edge.verification == AtlasVerificationState::Missing
    {
        AtlasEveEntanglementState::Degraded
    } else {
        AtlasEveEntanglementState::Current
    }
}

fn entanglement_id(edge: &AtlasProjectedEntanglement) -> String {
    format!(
        "entanglement:{}:{}",
        edge.consumer.repository_uri, edge.claim_id
    )
}

fn repository_node_id(repository: &AtlasRepositoryIdentity) -> String {
    format!("repo:{}", repository.repository_uri)
}

fn entanglement_label(edge: &AtlasProjectedEntanglement) -> String {
    format!(
        "{}: {} -> {}",
        edge.claim_label,
        edge.consumer.workspace_id,
        edge.provider
            .as_ref()
            .map(|provider| provider.workspace_id.as_str())
            .unwrap_or("unresolved")
    )
}

fn entanglement_summary(
    projection: &AtlasEntanglementProjection,
    edge: &AtlasProjectedEntanglement,
) -> String {
    let publisher_age = entanglement_publisher_age_ms(projection, edge)
        .map(|age_ms| format!("{age_ms} ms"))
        .unwrap_or_else(|| "unknown".into());
    format!(
        "{} dependency; failure semantics {}; compatibility {}; verification {}; claim freshness {}; offer freshness {}; publisher age {}.",
        entanglement_kind_name(edge.entanglement_kind),
        failure_semantics_name(edge.failure_semantics),
        compatibility_name(edge.compatibility),
        verification_name(edge.verification),
        freshness_name(edge.claim_freshness),
        edge.offer_freshness
            .map(freshness_name)
            .unwrap_or("missing"),
        publisher_age,
    )
}

fn entanglement_item(
    projection: &AtlasEntanglementProjection,
    edge: &AtlasProjectedEntanglement,
) -> EveListItem {
    EveListItem {
        label: entanglement_label(edge),
        status: entanglement_state(edge).as_wire_name().into(),
        detail: entanglement_summary(projection, edge),
        badges: vec![
            entanglement_kind_name(edge.entanglement_kind).into(),
            failure_semantics_name(edge.failure_semantics).into(),
        ],
    }
}

fn claim_items(edge: &AtlasProjectedEntanglement) -> Vec<EveListItem> {
    let mut items = vec![EveListItem {
        label: format!("Consumer claim: {}", edge.claim_label),
        status: freshness_name(edge.claim_freshness).into(),
        detail: format!(
            "{} owns claim {} from publication {} with {} impact scope; requires {}; Body evidence {}.",
            edge.consumer.repository_uri,
            edge.claim_id,
            edge.claim_publication_id,
            impact_scope_name(&edge.impact_scope),
            contract_requirement_name(&edge.claim_requirement),
            body_evidence_name(&edge.claim_body_evidence),
        ),
        badges: vec![edge.consumer.workspace_id.clone()],
    }];

    if let Some(provider) = &edge.provider {
        items.push(EveListItem {
            label: format!(
                "Provider offer: {}",
                edge.offer_label.as_deref().unwrap_or("missing")
            ),
            status: edge
                .offer_freshness
                .map(freshness_name)
                .unwrap_or("missing")
                .into(),
            detail: match (&edge.surface_id, &edge.offer_publication_id) {
                (Some(surface_id), Some(publication_id)) => format!(
                    "{} owns surface {} from publication {}; contract {}; lifecycle {}; Body evidence {}.",
                    provider.repository_uri,
                    surface_id,
                    publication_id,
                    edge.offer_contract
                        .as_ref()
                        .map(contract_descriptor_name)
                        .unwrap_or_else(|| "missing".into()),
                    edge.offer_lifecycle
                        .as_ref()
                        .map(offer_lifecycle_name)
                        .unwrap_or("missing"),
                    body_evidence_name(&edge.offer_body_evidence),
                ),
                _ => format!(
                    "{} has no admitted offer for this claim.",
                    provider.repository_uri
                ),
            },
            badges: vec![provider.workspace_id.clone()],
        });
    }

    items
}

fn evidence_items(
    projection: &AtlasEntanglementProjection,
    edge: &AtlasProjectedEntanglement,
) -> Vec<EveListItem> {
    let mut items = vec![
        EveListItem {
            label: "Compatibility".into(),
            status: compatibility_name(edge.compatibility).into(),
            detail: format!(
                "Epiphany Modeling compared requirement {} with offer {} and derived {}.",
                contract_requirement_name(&edge.claim_requirement),
                edge.offer_contract
                    .as_ref()
                    .map(contract_descriptor_name)
                    .unwrap_or_else(|| "missing".into()),
                compatibility_name(edge.compatibility),
            ),
            badges: vec!["modeling".into()],
        },
        EveListItem {
            label: "Soul verification".into(),
            status: verification_name(edge.verification).into(),
            detail: match (
                &edge.verification_publication_id,
                &edge.verification_evidence_sha256,
            ) {
                (Some(publication), Some(evidence)) => format!(
                    "Exact claim/offer verification state is {}; publication {}; evidence {}.",
                    verification_name(edge.verification),
                    publication,
                    evidence,
                ),
                _ => format!(
                    "Exact claim/offer verification state is {}; no current exact Soul evidence applies.",
                    verification_name(edge.verification)
                ),
            },
            badges: vec!["soul".into()],
        },
    ];
    let cycle_memberships = projection
        .cycles
        .iter()
        .filter(|cycle| {
            cycle
                .repositories
                .iter()
                .any(|repository| repository == &edge.consumer)
                && edge.provider.as_ref().is_some_and(|provider| {
                    cycle
                        .repositories
                        .iter()
                        .any(|repository| repository == provider)
                })
                && cycle.entanglement_kinds.contains(&edge.entanglement_kind)
        })
        .map(|cycle| cycle_class_name(cycle.classification))
        .collect::<Vec<_>>();
    items.push(EveListItem {
        label: "Cycle membership".into(),
        status: if cycle_memberships.is_empty() {
            "none".into()
        } else {
            "present".into()
        },
        detail: if cycle_memberships.is_empty() {
            "This edge is not a member of a projected cycle.".into()
        } else {
            format!(
                "Projected cycle classes: {}.",
                cycle_memberships.join(" | ")
            )
        },
        badges: vec!["projector".into()],
    });
    let radius = edge.provider.as_ref().and_then(|provider| {
        edge.surface_id.and_then(|surface_id| {
            projection
                .blast_radii
                .iter()
                .find(|radius| radius.source == *provider && radius.source_surface_id == surface_id)
        })
    });
    items.push(EveListItem {
        label: "Blast radius".into(),
        status: radius.map(|_| "derived").unwrap_or("none").into(),
        detail: radius
            .map(|radius| {
                radius
                    .affected
                    .iter()
                    .map(|affected| {
                        format!(
                            "{}:{} hop(s)",
                            affected.repository.workspace_id, affected.minimum_hops
                        )
                    })
                    .collect::<Vec<_>>()
                    .join(" | ")
            })
            .filter(|detail| !detail.is_empty())
            .unwrap_or_else(|| "No downstream repository is projected from this surface.".into()),
        badges: vec!["projector".into()],
    });
    let watermarks = projection
        .publisher_status
        .iter()
        .filter(|status| {
            status.publisher == edge.consumer
                || edge
                    .provider
                    .as_ref()
                    .is_some_and(|provider| status.publisher == *provider)
        })
        .map(|status| {
            format!(
                "{} heartbeat={} age_ms={} documents={}",
                status.publisher.workspace_id,
                status.heartbeat_sequence,
                projection
                    .evaluated_at_unix_ms
                    .saturating_sub(status.heartbeat_at_unix_ms),
                status.watermarks.len()
            )
        })
        .collect::<Vec<_>>();
    items.push(EveListItem {
        label: "Publisher watermarks".into(),
        status: "bounded".into(),
        detail: watermarks.join(" | "),
        badges: vec!["odin".into()],
    });
    items
}

fn entanglement_publisher_age_ms(
    projection: &AtlasEntanglementProjection,
    edge: &AtlasProjectedEntanglement,
) -> Option<u64> {
    projection
        .publisher_status
        .iter()
        .filter(|status| {
            status.publisher == edge.consumer
                || edge
                    .provider
                    .as_ref()
                    .is_some_and(|provider| status.publisher == *provider)
        })
        .map(|status| {
            projection
                .evaluated_at_unix_ms
                .saturating_sub(status.heartbeat_at_unix_ms)
        })
        .max()
}

fn graph_values(
    projection: &AtlasEntanglementProjection,
) -> (Vec<EveGraphNode>, Vec<EveGraphEdge>) {
    let mut nodes = BTreeMap::<String, EveGraphNode>::new();
    let mut edges = Vec::with_capacity(projection.entanglements.len());

    for edge in &projection.entanglements {
        let consumer_id = repository_node_id(&edge.consumer);
        nodes
            .entry(consumer_id.clone())
            .or_insert_with(|| repository_graph_node(consumer_id.clone(), &edge.consumer));

        let provider_id = if let Some(provider) = &edge.provider {
            let provider_id = repository_node_id(provider);
            nodes
                .entry(provider_id.clone())
                .or_insert_with(|| repository_graph_node(provider_id.clone(), provider));
            provider_id
        } else {
            let unresolved_id = format!("repo:unresolved:{}", edge.claim_id);
            nodes
                .entry(unresolved_id.clone())
                .or_insert_with(|| EveGraphNode {
                    id: unresolved_id.clone(),
                    label: "Unresolved provider".into(),
                    owner: edge.consumer.repository_uri.clone(),
                });
            unresolved_id
        };

        edges.push(EveGraphEdge {
            id: entanglement_id(edge),
            from: consumer_id,
            to: provider_id,
            state: entanglement_state(edge).as_wire_name().into(),
        });
    }

    (nodes.into_values().collect(), edges)
}

fn repository_graph_node(id: String, repository: &AtlasRepositoryIdentity) -> EveGraphNode {
    EveGraphNode {
        id,
        label: repository.workspace_id.clone(),
        owner: repository.repository_uri.clone(),
    }
}

fn binding(
    target_prop: &str,
    pointer_id: &str,
    fragment: &str,
    route_description: &str,
) -> EveStateBinding {
    EveStateBinding {
        target_prop: target_prop.into(),
        pointer_id: pointer_id.into(),
        source_id: format!("{MODEL_ATLAS_PROJECTION_SOURCE}#{fragment}"),
        schema_id: ATLAS_PROJECTION_SCHEMA.into(),
        route_kind: "network".into(),
        route_description: route_description.into(),
    }
}

fn presentation_command_advertisements() -> Vec<EveAdvertisedCommand> {
    vec![
        EveAdvertisedCommand {
            command: MODEL_ATLAS_SELECT_COMMAND.into(),
            schema: EVE_COMMAND_SCHEMA.into(),
            transport: "cultmesh-command".into(),
            authority: MODEL_ATLAS_PRESENTATION_AUTHORITY.into(),
            presentation_only: true,
            domain_state_effects: "none".into(),
            summary: "Selects an entanglement for presentation without changing the modeled relationship."
                .into(),
        },
        EveAdvertisedCommand {
            command: MODEL_ATLAS_FILTER_COMMAND.into(),
            schema: EVE_COMMAND_SCHEMA.into(),
            transport: "cultmesh-command".into(),
            authority: MODEL_ATLAS_PRESENTATION_AUTHORITY.into(),
            presentation_only: true,
            domain_state_effects: "none".into(),
            summary: "Changes the attention view filter without changing the entanglement projection."
                .into(),
        },
    ]
}

fn presentation_command_descriptors() -> Vec<EveCommandDescriptor> {
    vec![
        EveCommandDescriptor {
            schema: EVE_COMMAND_SCHEMA.into(),
            command: MODEL_ATLAS_SELECT_COMMAND.into(),
            label: "Select entanglement".into(),
            surface_id: MODEL_ATLAS_SURFACE_ID.into(),
            transport: "cultmesh-command".into(),
            authority: MODEL_ATLAS_PRESENTATION_AUTHORITY.into(),
            result: "surface-republished-or-denied".into(),
            presentation_only: true,
            domain_state_effects: "none".into(),
        },
        EveCommandDescriptor {
            schema: EVE_COMMAND_SCHEMA.into(),
            command: MODEL_ATLAS_FILTER_COMMAND.into(),
            label: "Filter attention".into(),
            surface_id: MODEL_ATLAS_SURFACE_ID.into(),
            transport: "cultmesh-command".into(),
            authority: MODEL_ATLAS_PRESENTATION_AUTHORITY.into(),
            result: "surface-republished-or-denied".into(),
            presentation_only: true,
            domain_state_effects: "none".into(),
        },
    ]
}

fn model_atlas_styles() -> EveStyleSheet {
    EveStyleSheet {
        tokens: EveStyleTokens {
            color_background: "#0A0E12".into(),
            color_panel: "#151C23".into(),
            color_panel_alt: "#202A33".into(),
            color_text: "#F0F4F1".into(),
            color_muted: "#9DA9A2".into(),
            color_accent: "#78D6A3".into(),
            color_warning: "#D6B24A".into(),
            color_danger: "#C95D63".into(),
            font_body: "Ubuntu Sans, Ubuntu, sans-serif".into(),
            font_title: "Montserrat, Zen Kaku Gothic New, Ubuntu Sans, sans-serif".into(),
        },
    }
}

fn validate_projection(projection: &AtlasEntanglementProjection) -> Result<()> {
    ensure!(
        projection.schema_version == ATLAS_PROJECTION_SCHEMA,
        "Model Atlas Eve surface requires the canonical entanglement projection schema"
    );
    ensure!(
        projection.projection_sha256 == atlas_projection_digest(projection)?,
        "Model Atlas Eve surface refuses a projection whose canonical digest does not match"
    );
    Ok(())
}

fn validate_utc_timestamp(value: &str) -> Result<()> {
    let timestamp = DateTime::parse_from_rfc3339(value)
        .map_err(|_| anyhow::anyhow!("Eve surface timestamp must be RFC 3339"))?;
    ensure!(
        timestamp.offset().local_minus_utc() == 0,
        "Eve surface timestamp must be UTC"
    );
    Ok(())
}

fn entanglement_kind_name(kind: AtlasEntanglementKind) -> &'static str {
    match kind {
        AtlasEntanglementKind::Build => "build",
        AtlasEntanglementKind::Runtime => "runtime",
        AtlasEntanglementKind::Deployment => "deployment",
        AtlasEntanglementKind::SchemaProtocol => "schema_protocol",
        AtlasEntanglementKind::DataState => "data_state",
        AtlasEntanglementKind::InfrastructureControl => "infrastructure_control",
        AtlasEntanglementKind::Governance => "governance",
        AtlasEntanglementKind::LorePersona => "lore_persona",
    }
}

fn failure_semantics_name(semantics: AtlasFailureSemantics) -> &'static str {
    match semantics {
        AtlasFailureSemantics::FailClosed => "fail_closed",
        AtlasFailureSemantics::Degrade => "degrade",
        AtlasFailureSemantics::LastKnownSafe => "last_known_safe",
        AtlasFailureSemantics::HumanDecision => "human_decision",
    }
}

fn freshness_name(freshness: AtlasPublicationFreshness) -> &'static str {
    match freshness {
        AtlasPublicationFreshness::Current => "current",
        AtlasPublicationFreshness::LastKnownStale => "last_known_stale",
        AtlasPublicationFreshness::Retired => "retired",
    }
}

fn compatibility_name(compatibility: AtlasCompatibility) -> &'static str {
    match compatibility {
        AtlasCompatibility::Exact => "exact",
        AtlasCompatibility::Compatible => "compatible",
        AtlasCompatibility::Unresolved => "unresolved",
        AtlasCompatibility::OfferMissing => "offer_missing",
        AtlasCompatibility::OfferWithdrawn => "offer_withdrawn",
        AtlasCompatibility::ContractIdMismatch => "contract_id_mismatch",
        AtlasCompatibility::VersionSchemeMismatch => "version_scheme_mismatch",
        AtlasCompatibility::VersionMismatch => "version_mismatch",
        AtlasCompatibility::ClaimRetired => "claim_retired",
    }
}

fn verification_name(verification: AtlasVerificationState) -> &'static str {
    match verification {
        AtlasVerificationState::Passed => "passed",
        AtlasVerificationState::Failed => "failed",
        AtlasVerificationState::Missing => "missing",
        AtlasVerificationState::LastKnownStale => "last_known_stale",
        AtlasVerificationState::ExactEdgeMismatch => "exact_edge_mismatch",
    }
}

fn impact_scope_name(scope: &AtlasImpactScope) -> String {
    match scope {
        AtlasImpactScope::WholeRepository => "whole_repository".into(),
        AtlasImpactScope::LocalSurfaces { surface_ids } => {
            format!("{} local_surfaces", surface_ids.len())
        }
    }
}

fn contract_descriptor_name(contract: &AtlasContractDescriptor) -> String {
    match contract {
        AtlasContractDescriptor::Semver {
            contract_id,
            version,
        } => format!("semver:{contract_id}@{version}"),
        AtlasContractDescriptor::ExactSchema {
            contract_id,
            schema_id,
        } => format!("exact_schema:{contract_id}@{schema_id}"),
        AtlasContractDescriptor::ExactDigest {
            contract_id,
            sha256,
        } => format!("exact_digest:{contract_id}@{sha256}"),
    }
}

fn contract_requirement_name(requirement: &AtlasContractRequirement) -> String {
    match requirement {
        AtlasContractRequirement::Semver {
            contract_id,
            requirement,
        } => format!("semver:{contract_id}@{requirement}"),
        AtlasContractRequirement::ExactSchema {
            contract_id,
            schema_id,
        } => format!("exact_schema:{contract_id}@{schema_id}"),
        AtlasContractRequirement::ExactDigest {
            contract_id,
            sha256,
        } => format!("exact_digest:{contract_id}@{sha256}"),
    }
}

fn offer_lifecycle_name(lifecycle: &AtlasOfferLifecycle) -> &'static str {
    match lifecycle {
        AtlasOfferLifecycle::Active => "active",
        AtlasOfferLifecycle::Deprecated { .. } => "deprecated",
        AtlasOfferLifecycle::Withdrawn => "withdrawn",
    }
}

fn body_evidence_name(evidence: &[AtlasBodyEvidenceRef]) -> String {
    evidence
        .iter()
        .map(|source| format!("{}@{}", source.path, source.raw_sha256))
        .collect::<Vec<_>>()
        .join(" | ")
}

fn cycle_class_name(classification: AtlasCycleClass) -> &'static str {
    match classification {
        AtlasCycleClass::ForbiddenBuild => "forbidden_build",
        AtlasCycleClass::ForbiddenDeployment => "forbidden_deployment",
        AtlasCycleClass::ForbiddenInfrastructureControl => "forbidden_infrastructure_control",
        AtlasCycleClass::ReviewRequired => "review_required",
        AtlasCycleClass::Informational => "informational",
    }
}

fn is_false(value: &bool) -> bool {
    !*value
}

#[cfg(test)]
mod tests {
    use uuid::Uuid;

    use super::*;

    fn repository(workspace_id: &str) -> AtlasRepositoryIdentity {
        AtlasRepositoryIdentity::new("gamecult", workspace_id).unwrap()
    }

    fn projection_with(edges: Vec<AtlasProjectedEntanglement>) -> AtlasEntanglementProjection {
        let mut projection = AtlasEntanglementProjection {
            schema_version: ATLAS_PROJECTION_SCHEMA.into(),
            audience_id: "gamecult-operator".into(),
            evaluated_at_unix_ms: 1_786_665_600_000,
            source_publication_ids: Vec::new(),
            publisher_status: Vec::new(),
            entanglements: edges,
            cycles: Vec::new(),
            blast_radii: Vec::new(),
            projection_sha256: String::new(),
        };
        projection.projection_sha256 = atlas_projection_digest(&projection).unwrap();
        projection
    }

    fn edge(compatibility: AtlasCompatibility) -> AtlasProjectedEntanglement {
        AtlasProjectedEntanglement {
            claim_id: Uuid::from_u128(7),
            claim_label: "Epiphany consumes Eve".into(),
            claim_requirement: AtlasContractRequirement::ExactSchema {
                contract_id: "eve-surface".into(),
                schema_id: "gamecult.eve.surface.v1".into(),
            },
            claim_body_evidence: vec![AtlasBodyEvidenceRef {
                path: "Cargo.toml".into(),
                raw_sha256: "0".repeat(64),
            }],
            consumer: repository("epiphany"),
            provider: Some(repository("eve")),
            surface_id: Some(Uuid::from_u128(9)),
            offer_label: Some("Canonical Eve surface".into()),
            offer_contract: Some(AtlasContractDescriptor::ExactSchema {
                contract_id: "eve-surface".into(),
                schema_id: "gamecult.eve.surface.v1".into(),
            }),
            offer_lifecycle: Some(AtlasOfferLifecycle::Active),
            offer_body_evidence: vec![AtlasBodyEvidenceRef {
                path: "schemas/gamecult.eve.surface.v1.schema.json".into(),
                raw_sha256: "1".repeat(64),
            }],
            entanglement_kind: AtlasEntanglementKind::SchemaProtocol,
            failure_semantics: AtlasFailureSemantics::HumanDecision,
            impact_scope: AtlasImpactScope::WholeRepository,
            claim_freshness: AtlasPublicationFreshness::Current,
            offer_freshness: Some(AtlasPublicationFreshness::Current),
            compatibility,
            verification: AtlasVerificationState::Passed,
            claim_publication_id: format!("sha256-{}", "1".repeat(64)),
            offer_publication_id: Some(format!("sha256-{}", "2".repeat(64))),
            verification_publication_id: Some(format!("sha256-{}", "3".repeat(64))),
            verification_evidence_sha256: Some(format!("sha256-{}", "4".repeat(64))),
        }
    }

    #[test]
    fn publishes_eves_exact_provider_and_surface_identity() {
        let documents = project_model_atlas_eve_documents(
            &projection_with(Vec::new()),
            &AtlasEvePresentationState::default(),
            4,
            "2026-08-14T00:00:00Z",
        )
        .unwrap();

        assert_eq!(
            documents.advertisement.schema,
            EVE_PROVIDER_ADVERTISEMENT_SCHEMA
        );
        assert_eq!(documents.advertisement.provider_id, MODEL_ATLAS_PROVIDER_ID);
        assert_eq!(documents.advertisement.service_id, MODEL_ATLAS_SERVICE_ID);
        assert_eq!(documents.advertisement.verse_id, MODEL_ATLAS_VERSE_ID);
        assert_eq!(documents.surface.schema, EVE_SURFACE_SCHEMA);
        assert_eq!(documents.surface.surface.id, MODEL_ATLAS_SURFACE_ID);
        assert_eq!(documents.surface.version, 4);
        assert!(
            documents
                .advertisement
                .witnesses
                .iter()
                .any(|witness| { witness.reference == EVE_MODEL_ATLAS_CONFORMANCE_REF })
        );
        assert_eq!(
            documents.advertisement.surfaces[0].lowering_targets,
            vec!["gui".to_string(), "tui".to_string()]
        );
    }

    #[test]
    fn one_retained_tree_contains_attention_drilldown_and_secondary_graph() {
        let documents = project_model_atlas_eve_documents(
            &projection_with(vec![edge(AtlasCompatibility::VersionMismatch)]),
            &AtlasEvePresentationState::default(),
            1,
            "2026-08-14T00:00:00Z",
        )
        .unwrap();
        let root = &documents.surface.surface.root;

        assert_eq!(root.children.len(), 3);
        assert_eq!(root.children[0].id, MODEL_ATLAS_ATTENTION_ID);
        assert_eq!(root.children[1].id, MODEL_ATLAS_DRILLDOWN_ID);
        assert_eq!(root.children[2].id, MODEL_ATLAS_GRAPH_ID);
        assert!(matches!(
            &root.props,
            EveComponentProps::Root(EveRootProps { default_view, .. })
                if default_view == "attention"
        ));
        assert!(matches!(
            &root.children[2].props,
            EveComponentProps::Graph(EveGraphProps {
                role,
                presentation_fallback,
                ..
            }) if role == "secondary" && presentation_fallback == "children"
        ));
        assert_eq!(root.children[2].children[0].kind, "list");
    }

    #[test]
    fn attention_and_drilldown_report_the_oldest_endpoint_publisher_age() {
        let mut projection = projection_with(vec![edge(AtlasCompatibility::Compatible)]);
        projection.publisher_status = vec![
            crate::AtlasPublisherProjectionStatus {
                publisher: repository("epiphany"),
                runtime_id: "epiphany-runtime".into(),
                runtime_incarnation_id: "epiphany-incarnation".into(),
                heartbeat_sequence: 3,
                heartbeat_at_unix_ms: projection.evaluated_at_unix_ms - 250,
                freshness: AtlasPublicationFreshness::Current,
                watermarks: Vec::new(),
                status_publication_id: format!("sha256-{}", "5".repeat(64)),
            },
            crate::AtlasPublisherProjectionStatus {
                publisher: repository("eve"),
                runtime_id: "eve-runtime".into(),
                runtime_incarnation_id: "eve-incarnation".into(),
                heartbeat_sequence: 7,
                heartbeat_at_unix_ms: projection.evaluated_at_unix_ms - 900,
                freshness: AtlasPublicationFreshness::Current,
                watermarks: Vec::new(),
                status_publication_id: format!("sha256-{}", "6".repeat(64)),
            },
        ];

        let summary = entanglement_summary(&projection, &projection.entanglements[0]);
        assert!(summary.contains("publisher age 900 ms"));
        let evidence = evidence_items(&projection, &projection.entanglements[0]);
        assert!(
            evidence
                .iter()
                .find(|item| item.label == "Publisher watermarks")
                .is_some_and(|item| item.detail.contains("age_ms=900"))
        );
    }

    #[test]
    fn every_binding_reads_the_owned_projection_and_commands_are_presentation_only() {
        let documents = project_model_atlas_eve_documents(
            &projection_with(vec![edge(AtlasCompatibility::OfferMissing)]),
            &AtlasEvePresentationState::default(),
            1,
            "2026-08-14T00:00:00Z",
        )
        .unwrap();

        let mut bindings = Vec::new();
        collect_bindings(&documents.surface.surface.root, &mut bindings);
        assert!(!bindings.is_empty());
        assert!(bindings.iter().all(|binding| {
            binding.schema_id == ATLAS_PROJECTION_SCHEMA
                && binding.source_id.starts_with(MODEL_ATLAS_PROJECTION_SOURCE)
                && binding.route_kind == "network"
        }));
        assert!(documents.surface.commands.iter().all(|command| {
            command.presentation_only
                && command.domain_state_effects == "none"
                && command.authority == MODEL_ATLAS_PRESENTATION_AUTHORITY
        }));
        assert_eq!(
            documents.advertisement.surfaces[0].domain_mutation_route,
            MODEL_ATLAS_DOMAIN_MUTATION_ROUTE
        );
    }

    #[test]
    fn refuses_tampered_projection_or_non_utc_publication_time() {
        let mut projection = projection_with(Vec::new());
        projection.audience_id.push_str("-tampered");
        assert!(
            project_model_atlas_eve_documents(
                &projection,
                &AtlasEvePresentationState::default(),
                1,
                "2026-08-14T00:00:00Z",
            )
            .is_err()
        );

        let projection = projection_with(Vec::new());
        assert!(
            project_model_atlas_eve_documents(
                &projection,
                &AtlasEvePresentationState::default(),
                1,
                "2026-08-14T02:00:00+02:00",
            )
            .is_err()
        );
    }

    fn collect_bindings<'a>(component: &'a EveComponent, output: &mut Vec<&'a EveStateBinding>) {
        output.extend(component.state_bindings.iter());
        for child in &component.children {
            collect_bindings(child, output);
        }
    }
}
