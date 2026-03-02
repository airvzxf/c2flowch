use serde::{Deserialize, Serialize};

pub const EDGE_LABEL_BREAK: &str = "__Break__";
pub const EDGE_LABEL_DEPENDENCY: &str = "Dependency";
pub const EDGE_LABEL_TRUE: &str = "Verdadero";
pub const EDGE_LABEL_FALSE: &str = "Falso";

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub enum NodeType {
    /// Start/End of a function or program
    Terminator,
    /// Simple operation (assignment, function call)
    Process,
    /// Function call (Predefined Process)
    PredefinedProcess,
    /// Condition (if, switch, while)
    Decision,
    /// A single case within a switch statement
    SwitchCase,
    /// Input/Output operation
    InputOutput,
    /// Variable declaration/initialization
    Preparation,
    /// A group of nodes (e.g. a whole function)
    Subroutine,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FlowNode {
    pub id: String,
    pub node_type: NodeType,
    pub text: String,
    /// Optional metadata about the original source line
    pub source_line: Option<usize>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FlowEdge {
    pub from: String,
    pub to: String,
    pub label: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Flowchart {
    pub title: String,
    pub nodes: Vec<FlowNode>,
    pub edges: Vec<FlowEdge>,
    #[serde(default)]
    pub subroutines: Vec<Flowchart>,
}

impl Flowchart {
    pub fn add_node(&mut self, node: FlowNode) {
        self.nodes.push(node);
    }

    pub fn add_edge(&mut self, from: &str, to: &str, label: Option<String>) {
        self.edges.push(FlowEdge {
            from: from.to_string(),
            to: to.to_string(),
            label,
        });
    }

    pub fn to_yaml(&self) -> Result<String, serde_yaml::Error> {
        serde_yaml::to_string(self)
    }

    pub fn from_yaml(yaml: &str) -> Result<Self, serde_yaml::Error> {
        serde_yaml::from_str(yaml)
    }
}
