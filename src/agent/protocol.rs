use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "action", content = "params")]
pub enum AgentCommand {
    #[serde(rename = "navigate")]
    Navigate { url: String },

    #[serde(rename = "back")]
    Back,

    #[serde(rename = "forward")]
    Forward,

    #[serde(rename = "reload")]
    Reload,

    #[serde(rename = "get_text")]
    GetText {
        target: String,
        #[serde(default)]
        attribute: Option<String>,
    },

    #[serde(rename = "snapshot")]
    Snapshot {
        #[serde(default)]
        viewport_only: bool,
        #[serde(default)]
        filter_role: Option<String>,
    },

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
        #[serde(default)]
        full_page: bool,
    },

    #[serde(rename = "highlight")]
    Highlight { target: String },

    #[serde(rename = "screenshot_element")]
    ScreenshotElement { target: String },

    #[serde(rename = "fill_form")]
    FillForm {
        fields: Vec<FormField>,
        #[serde(default)]
        submit: bool,
    },

    #[serde(rename = "clipboard")]
    Clipboard {
        #[serde(default = "default_clipboard_action")]
        action: String,
        #[serde(default)]
        text: Option<String>,
    },

    #[serde(rename = "emulate")]
    Emulate {
        width: u32,
        height: u32,
        #[serde(default)]
        dpr: Option<f64>,
        #[serde(default)]
        mobile: bool,
    },

    #[serde(rename = "console_logs")]
    ConsoleLogs {
        #[serde(default)]
        clear: bool,
    },

    #[serde(rename = "pdf")]
    PrintPdf {
        #[serde(default)]
        landscape: bool,
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

    #[serde(rename = "storage_state")]
    StorageState {
        #[serde(default = "default_storage_action")]
        action: String,
        #[serde(default)]
        state: Option<serde_json::Value>,
    },

    #[serde(rename = "find")]
    Find { query: String },

    #[serde(rename = "dialog_policy")]
    DialogPolicy {
        #[serde(default)]
        alert: Option<String>,
        #[serde(default)]
        confirm: Option<String>,
        #[serde(default)]
        prompt_value: Option<String>,
    },

    #[serde(rename = "hover")]
    Hover { target: String },

    #[serde(rename = "key")]
    PressKey {
        key: String,
        #[serde(default)]
        target: Option<String>,
    },

    #[serde(rename = "select_option")]
    SelectOption {
        target: String,
        value: String,
    },

    #[serde(rename = "network_activity")]
    NetworkActivity {
        #[serde(default)]
        clear: bool,
    },

    #[serde(rename = "drag")]
    Drag {
        source: String,
        #[serde(default)]
        target: Option<String>,
        #[serde(default)]
        offset_x: Option<i32>,
        #[serde(default)]
        offset_y: Option<i32>,
        #[serde(default = "default_drag_steps")]
        steps: u32,
    },

    #[serde(rename = "cookies")]
    Cookies {
        #[serde(default = "default_cookies_action")]
        action: String,
        #[serde(default)]
        cookie: Option<serde_json::Value>,
        #[serde(default)]
        name: Option<String>,
        #[serde(default)]
        domain: Option<String>,
    },

    #[serde(rename = "extra_headers")]
    ExtraHeaders {
        headers: serde_json::Value,
    },

    #[serde(rename = "user_agent_override")]
    UserAgentOverride {
        user_agent: String,
        #[serde(default)]
        accept_language: Option<String>,
        #[serde(default)]
        platform: Option<String>,
    },
}

fn default_drag_steps() -> u32 {
    10
}

fn default_cookies_action() -> String {
    "get".to_string()
}

fn default_storage_action() -> String {
    "get".to_string()
}

fn default_clipboard_action() -> String {
    "read".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormField {
    pub target: String,
    pub value: String,
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
fn default_dpr() -> f64 {
    1.0
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElementBounds {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
    #[serde(default)]
    pub center_x: i32,
    #[serde(default)]
    pub center_y: i32,
    #[serde(default)]
    pub in_viewport: bool,
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
    #[serde(default)]
    pub matched_on: Option<String>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViewportInfo {
    pub width: u32,
    pub height: u32,
    pub scroll_x: i32,
    pub scroll_y: i32,
    #[serde(default = "default_dpr")]
    pub dpr: f64,
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

    #[test]
    fn test_storage_state_command_and_element_bounds() {
        let cmd = AgentCommand::StorageState {
            action: "get".into(),
            state: None,
        };
        let json = serde_json::to_string(&cmd).unwrap();
        assert!(json.contains("storage_state"));

        let bounds: ElementBounds = serde_json::from_str(
            r#"{"x":10,"y":20,"width":100,"height":50,"center_x":60,"center_y":45,"in_viewport":true}"#
        ).unwrap();
        assert_eq!(bounds.center_x, 60);
        assert_eq!(bounds.center_y, 45);
        assert!(bounds.in_viewport);

        let find_cmd = AgentCommand::Find { query: "Submit".into() };
        let find_json = serde_json::to_string(&find_cmd).unwrap();
        assert!(find_json.contains("find"));
        assert!(find_json.contains("Submit"));

        let snap_cmd = AgentCommand::Snapshot {
            viewport_only: true,
            filter_role: Some("button".into()),
        };
        let snap_json = serde_json::to_string(&snap_cmd).unwrap();
        assert!(snap_json.contains("viewport_only"));
        assert!(snap_json.contains("button"));

        let highlight_cmd = AgentCommand::Highlight { target: "1".into() };
        let hl_json = serde_json::to_string(&highlight_cmd).unwrap();
        assert!(hl_json.contains("highlight"));
        assert!(hl_json.contains("\"target\":\"1\""));

        let shot_cmd = AgentCommand::Screenshot { som: true, full_page: true };
        let shot_json = serde_json::to_string(&shot_cmd).unwrap();
        assert!(shot_json.contains("full_page"));

        let crop_cmd = AgentCommand::ScreenshotElement { target: "input#q".into() };
        let crop_json = serde_json::to_string(&crop_cmd).unwrap();
        assert!(crop_json.contains("screenshot_element"));
        assert!(crop_json.contains("input#q"));

        let fill_cmd = AgentCommand::FillForm {
            fields: vec![
                FormField { target: "user".into(), value: "admin".into() },
                FormField { target: "pass".into(), value: "secret".into() },
            ],
            submit: true,
        };
        let fill_json = serde_json::to_string(&fill_cmd).unwrap();
        assert!(fill_json.contains("fill_form"));
        assert!(fill_json.contains("admin"));
        assert!(fill_json.contains("secret"));

        let clip_cmd = AgentCommand::Clipboard {
            action: "set".into(),
            text: Some("copied".into()),
        };
        let clip_json = serde_json::to_string(&clip_cmd).unwrap();
        assert!(clip_json.contains("clipboard"));
        assert!(clip_json.contains("copied"));

        let back_cmd = AgentCommand::Back;
        assert!(serde_json::to_string(&back_cmd).unwrap().contains("back"));

        let fwd_cmd = AgentCommand::Forward;
        assert!(serde_json::to_string(&fwd_cmd).unwrap().contains("forward"));

        let reload_cmd = AgentCommand::Reload;
        assert!(serde_json::to_string(&reload_cmd).unwrap().contains("reload"));

        let text_cmd = AgentCommand::GetText { target: "h1".into(), attribute: Some("title".into()) };
        let text_json = serde_json::to_string(&text_cmd).unwrap();
        assert!(text_json.contains("get_text"));
        assert!(text_json.contains("h1"));
        assert!(text_json.contains("title"));

        let emu_cmd = AgentCommand::Emulate { width: 375, height: 667, dpr: Some(2.0), mobile: true };
        let emu_json = serde_json::to_string(&emu_cmd).unwrap();
        assert!(emu_json.contains("emulate"));
        assert!(emu_json.contains("375"));

        let log_cmd = AgentCommand::ConsoleLogs { clear: true };
        let log_json = serde_json::to_string(&log_cmd).unwrap();
        assert!(log_json.contains("console_logs"));
        assert!(log_json.contains("\"clear\":true"));

        let pdf_cmd = AgentCommand::PrintPdf { landscape: true };
        let pdf_json = serde_json::to_string(&pdf_cmd).unwrap();
        assert!(pdf_json.contains("pdf"));
        assert!(pdf_json.contains("\"landscape\":true"));

        let hover_cmd = AgentCommand::Hover { target: "nav.menu".into() };
        let hover_json = serde_json::to_string(&hover_cmd).unwrap();
        assert!(hover_json.contains("hover"));
        assert!(hover_json.contains("nav.menu"));

        let key_cmd = AgentCommand::PressKey { key: "Escape".into(), target: Some("#modal".into()) };
        let key_json = serde_json::to_string(&key_cmd).unwrap();
        assert!(key_json.contains("key"));
        assert!(key_json.contains("Escape"));
        assert!(key_json.contains("#modal"));

        let select_cmd = AgentCommand::SelectOption { target: "select#country".into(), value: "US".into() };
        let select_json = serde_json::to_string(&select_cmd).unwrap();
        assert!(select_json.contains("select_option"));
        assert!(select_json.contains("select#country"));
        assert!(select_json.contains("US"));

        let net_cmd = AgentCommand::NetworkActivity { clear: true };
        let net_json = serde_json::to_string(&net_cmd).unwrap();
        assert!(net_json.contains("network_activity"));
        assert!(net_json.contains("\"clear\":true"));

        let drag_cmd = AgentCommand::Drag {
            source: "#item1".into(),
            target: Some("#dropzone".into()),
            offset_x: Some(50),
            offset_y: Some(100),
            steps: 5,
        };
        let drag_json = serde_json::to_string(&drag_cmd).unwrap();
        assert!(drag_json.contains("drag"));
        assert!(drag_json.contains("#item1"));
        assert!(drag_json.contains("#dropzone"));

        let cookie_cmd = AgentCommand::Cookies {
            action: "set".into(),
            cookie: Some(serde_json::json!({ "name": "session", "value": "xyz" })),
            name: Some("session".into()),
            domain: Some("example.com".into()),
        };
        let cookie_json = serde_json::to_string(&cookie_cmd).unwrap();
        assert!(cookie_json.contains("cookies"));
        assert!(cookie_json.contains("session"));

        let headers_cmd = AgentCommand::ExtraHeaders {
            headers: serde_json::json!({ "X-Test": "123" }),
        };
        let headers_json = serde_json::to_string(&headers_cmd).unwrap();
        assert!(headers_json.contains("extra_headers"));
        assert!(headers_json.contains("X-Test"));

        let ua_cmd = AgentCommand::UserAgentOverride {
            user_agent: "Mozilla/5.0 Agent".into(),
            accept_language: Some("en-US".into()),
            platform: Some("Windows".into()),
        };
        let ua_json = serde_json::to_string(&ua_cmd).unwrap();
        assert!(ua_json.contains("user_agent_override"));
        assert!(ua_json.contains("Mozilla/5.0 Agent"));
    }

    #[test]
    fn test_form_field_deserialization() {
        let raw = r##"[{"target":"#user","value":"alice"},{"target":"#pass","value":"1234"}]"##;
        let fields: Vec<FormField> = serde_json::from_str(raw).unwrap();
        assert_eq!(fields.len(), 2);
        assert_eq!(fields[0].target, "#user");
        assert_eq!(fields[0].value, "alice");
        assert_eq!(fields[1].target, "#pass");
        assert_eq!(fields[1].value, "1234");
    }
}
