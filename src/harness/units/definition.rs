use crate::harness::units::kinds::UnitKind;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct UnitColor(pub u8, pub u8, pub u8);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExecutionPortDirection {
    Input,
    Output,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExecutionPortSemantic {
    Input,
    Success,
    Failure,
    Branch,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExecutionPortDefinition {
    pub id: String,
    pub direction: ExecutionPortDirection,
    pub semantic: ExecutionPortSemantic,
    pub label: String,
    pub color: UnitColor,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExecutionUnitExecutor {
    Harness,
    Llm,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ExecutionEdgeTarget {
    UnitInput { unit_id: String, port_id: String },
    GraphOutput(String),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExecutionEdgeDefinition {
    pub from_unit_id: String,
    pub output_port_id: String,
    pub target: ExecutionEdgeTarget,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExecutionGraphDefinition {
    pub entry_unit_id: String,
    pub units: Vec<ExecutionUnitDefinition>,
    pub edges: Vec<ExecutionEdgeDefinition>,
    pub outputs: Vec<ExecutionPortDefinition>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExecutionUnitDefinition {
    pub id: String,
    pub label: String,
    pub kind: UnitKind,
    pub executor: ExecutionUnitExecutor,
    pub color: UnitColor,
    pub active_color: UnitColor,
    pub input: ExecutionPortDefinition,
    pub outputs: Vec<ExecutionPortDefinition>,
    pub child_graph: Option<ExecutionGraphDefinition>,
}

impl ExecutionGraphDefinition {
    pub fn validate(&self) -> Result<(), String> {
        use std::collections::BTreeSet;
        let ids = self
            .units
            .iter()
            .map(|u| u.id.as_str())
            .collect::<BTreeSet<_>>();
        if !ids.contains(self.entry_unit_id.as_str()) {
            return Err(format!("unknown entry unit `{}`", self.entry_unit_id));
        }
        if ids.len() != self.units.len() {
            return Err("duplicate unit id".into());
        }
        let graph_outputs = self
            .outputs
            .iter()
            .map(|p| p.id.as_str())
            .collect::<BTreeSet<_>>();
        for unit in &self.units {
            if unit.input.direction != ExecutionPortDirection::Input {
                return Err(format!(
                    "unit `{}` input port has output direction",
                    unit.id
                ));
            }
            if unit.color == unit.active_color {
                return Err(format!(
                    "unit `{}` normal and active colors must differ",
                    unit.id
                ));
            }
            if let Some(graph) = &unit.child_graph {
                graph.validate()?;
            }
        }
        for edge in &self.edges {
            let source = self
                .units
                .iter()
                .find(|u| u.id == edge.from_unit_id)
                .ok_or_else(|| format!("unknown edge source `{}`", edge.from_unit_id))?;
            if !source.outputs.iter().any(|p| {
                p.id == edge.output_port_id && p.direction == ExecutionPortDirection::Output
            }) {
                return Err(format!(
                    "unknown output `{}.{}`",
                    edge.from_unit_id, edge.output_port_id
                ));
            }
            match &edge.target {
                ExecutionEdgeTarget::UnitInput { unit_id, port_id } => {
                    let target = self
                        .units
                        .iter()
                        .find(|u| &u.id == unit_id)
                        .ok_or_else(|| format!("unknown edge target `{unit_id}`"))?;
                    if target.input.id != *port_id {
                        return Err(format!("unknown input `{unit_id}.{port_id}`"));
                    }
                }
                ExecutionEdgeTarget::GraphOutput(port)
                    if !graph_outputs.contains(port.as_str()) =>
                {
                    return Err(format!("unknown graph output `{port}`"));
                }
                ExecutionEdgeTarget::GraphOutput(_) => {}
            }
        }
        Ok(())
    }
}

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
    SearchPlanner,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UnitDefinition {
    pub id: String,
    pub label: String,
    pub kind: UnitKind,
    pub graph: Option<UnitGraphDefinition>,
    pub local_state_schema: UnitLocalStateSchema,
}
