use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum TransformAction {
    Add,
    Set,
    Remove,
    Replace,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeaderTransformation {
    pub action: TransformAction,
    pub name: String,
    pub value: Option<String>,
    pub pattern: Option<String>,
    pub replacement: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathTransformation {
    pub pattern: String,
    pub replacement: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryTransformation {
    pub action: TransformAction,
    pub name: String,
    pub value: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RequestTransformation {
    #[serde(default)]
    pub headers: Vec<HeaderTransformation>,
    pub path: Option<PathTransformation>,
    #[serde(default)]
    pub query_params: Vec<QueryTransformation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusCodeMapping {
    pub from: u16,
    pub to: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ResponseTransformation {
    #[serde(default)]
    pub headers: Vec<HeaderTransformation>,
    #[serde(default)]
    pub status_code_mappings: Vec<StatusCodeMapping>,
}
