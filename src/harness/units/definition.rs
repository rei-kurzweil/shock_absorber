use crate::harness::units::kinds::UnitKind;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UnitPortDefinition {
    pub name: String,
    pub description: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum UnitEdgeTarget {
    Node(String),
    Return,
    Fail,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UnitEdgeDefinition {
    pub from_node: String,
    pub port: String,
    pub target: UnitEdgeTarget,
    pub child_unit_id: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UnitGraphDefinition {
    pub entry_node: String,
    pub nodes: Vec<String>,
    pub ports: Vec<UnitPortDefinition>,
    pub edges: Vec<UnitEdgeDefinition>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum UnitLocalStateSchema {
    None,
    CollectionSummaryPagination,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UnitDefinition {
    pub id: String,
    pub label: String,
    pub kind: UnitKind,
    pub graph: Option<UnitGraphDefinition>,
    pub local_state_schema: UnitLocalStateSchema,
}
