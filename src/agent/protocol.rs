use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "action", content = "params")]
pub enum AgentCommand {
    #[serde(rename = "navigate")]
    Navigate { url: String },

    #[serde(rename = "snapshot")]
    Snapshot,

    #[serde(rename = "click")]
    Click { target: String },

    #[serde(rename = "type")]
    Type {
        target: String,
        text: String,
        #[serde(default)]
        clear: bool,
        #[serde(default)]
        submit: bool,
    },

    #[serde(rename = "scroll")]
    Scroll {
        direction: String,
        #[serde(default = "default_scroll_amount")]
        amount: i32,
    },

    #[serde(rename = "markdown")]
    Markdown,

    #[serde(rename = "eval")]
    Eval { expression: String },

    #[serde(rename = "screenshot")]
    Screenshot {
        #[serde(default)]
        som: bool,
    },

    #[serde(rename = "tabs_list")]
    TabsList,

    #[serde(rename = "tab_new")]
    TabNew {
        #[serde(default = "default_home_url")]
        url: String,
    },

    #[serde(rename = "tab_switch")]
    TabSwitch { index: usize },

    #[serde(rename = "tab_close")]
    TabClose { index: usize },

    #[serde(rename = "wait_for")]
    WaitFor {
        target: String,
        #[serde(default = "default_wait_timeout")]
        timeout_ms: u64,
    },
}

fn default_wait_timeout() -> u64 {
    5000
}

fn default_scroll_amount() -> i32 {
    500
}

fn default_home_url() -> String {
    "about:home".to_string()
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElementBounds {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomElement {
    #[serde(rename = "ref")]
    pub ref_id: u32,
    pub tag: String,
    pub role: String,
    pub text: String,
    pub placeholder: String,
    pub value: String,
    pub href: String,
    pub bounds: ElementBounds,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViewportInfo {
    pub width: u32,
    pub height: u32,
    pub scroll_x: i32,
    pub scroll_y: i32,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotResponse {
    pub url: String,
    pub title: String,
    pub tree_text: String,
    pub elements: Vec<DomElement>,
    pub viewport: ViewportInfo,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarkdownResponse {
    pub url: String,
    pub title: String,
    pub markdown: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentResultEnvelope {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl AgentResultEnvelope {
    pub fn ok(data: serde_json::Value) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
        }
    }

    pub fn err(message: impl Into<String>) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(message.into()),
        }
    }
}

// MCP (Model Context Protocol) JSON-RPC 2.0 Wire Types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpRequest {
    pub jsonrpc: String,
    pub id: Option<serde_json::Value>,
    pub method: String,
    #[serde(default)]
    pub params: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpResponse {
    pub jsonrpc: String,
    pub id: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<McpError>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpError {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpTool {
    pub name: String,
    pub description: String,
    pub input_schema: serde_json::Value,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_command_serialization() {
        let nav = AgentCommand::Navigate {
            url: "https://example.com".into(),
        };
        let json = serde_json::to_string(&nav).unwrap();
        assert!(json.contains("navigate"));
        assert!(json.contains("https://example.com"));

        let click = AgentCommand::Click { target: "4".into() };
        let json_click = serde_json::to_string(&click).unwrap();
        assert!(json_click.contains("click"));
        assert!(json_click.contains("\"target\":\"4\""));
    }

    #[test]
    fn test_result_envelope() {
        let ok = AgentResultEnvelope::ok(serde_json::json!({ "answer": 42 }));
        assert!(ok.success);
        assert_eq!(ok.data.unwrap()["answer"], 42);

        let err = AgentResultEnvelope::err("Element not found");
        assert!(!err.success);
        assert_eq!(err.error.unwrap(), "Element not found");
    }

    #[test]
    fn test_mcp_wire_serialization() {
        let req = McpRequest {
            jsonrpc: "2.0".into(),
            id: Some(serde_json::json!(1)),
            method: "tools/list".into(),
            params: None,
        };
        let s = serde_json::to_string(&req).unwrap();
        assert!(s.contains("tools/list"));

        let resp = McpResponse {
            jsonrpc: "2.0".into(),
            id: Some(serde_json::json!(1)),
            result: Some(serde_json::json!({ "status": "ok" })),
            error: None,
        };
        let rs = serde_json::to_string(&resp).unwrap();
        assert!(rs.contains("status"));
    }
}
