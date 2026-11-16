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

    #[serde(rename = "upload_file")]
    UploadFile {
        target: String,
        files: Vec<String>,
    },

    #[serde(rename = "location")]
    Location {
        latitude: f64,
        longitude: f64,
        #[serde(default)]
        accuracy: Option<f64>,
        #[serde(default)]
        timezone: Option<String>,
    },

    #[serde(rename = "media_control")]
    MediaControl {
        #[serde(default = "default_media_action")]
        action: String,
    },

    #[serde(rename = "inspect")]
    Inspect {
        target: String,
    },

    #[serde(rename = "route_rule")]
    RouteRule {
        #[serde(default = "default_route_action")]
        action: String,
        #[serde(default)]
        pattern: Option<String>,
        #[serde(default)]
        rule_type: Option<String>,
        #[serde(default)]
        status: Option<u16>,
        #[serde(default)]
        body: Option<String>,
        #[serde(default)]
        headers: Option<serde_json::Value>,
    },

    #[serde(rename = "double_click")]
    DoubleClick {
        target: String,
    },

    #[serde(rename = "right_click")]
    RightClick {
        target: String,
    },

    #[serde(rename = "extract_table")]
    ExtractTable {
        #[serde(default)]
        target: Option<String>,
    },

    #[serde(rename = "downloads")]
    Downloads {
        #[serde(default)]
        clear: bool,
    },

    #[serde(rename = "network_conditions")]
    NetworkConditions {
        #[serde(default = "default_net_profile")]
        profile: String,
        #[serde(default)]
        latency: Option<f64>,
        #[serde(default)]
        download_throughput: Option<f64>,
        #[serde(default)]
        upload_throughput: Option<f64>,
        #[serde(default)]
        offline: Option<bool>,
    },

    #[serde(rename = "inject_css")]
    InjectCss {
        css: String,
        #[serde(default)]
        remove: bool,
    },

    #[serde(rename = "query_elements")]
    QueryElements {
        selector: String,
        #[serde(default)]
        attribute: Option<String>,
        #[serde(default = "default_query_limit")]
        max_items: usize,
    },

    #[serde(rename = "focus")]
    Focus {
        target: String,
    },

    #[serde(rename = "blur")]
    Blur {
        target: String,
    },

    #[serde(rename = "page_info")]
    PageInfo,

    #[serde(rename = "get_html")]
    GetHtml {
        #[serde(default)]
        target: Option<String>,
    },

    #[serde(rename = "scroll_into_view")]
    ScrollIntoView {
        target: String,
        #[serde(default = "default_scroll_align")]
        align: String,
    },

    #[serde(rename = "set_zoom")]
    SetZoom {
        zoom_factor: f64,
    },

    #[serde(rename = "storage_item")]
    StorageItem {
        #[serde(default = "default_storage_type")]
        storage_type: String,
        #[serde(default = "default_storage_action")]
        action: String,
        #[serde(default)]
        key: Option<String>,
        #[serde(default)]
        value: Option<String>,
    },

    #[serde(rename = "assert_element")]
    AssertElement {
        target: String,
        #[serde(default)]
        assertions: ElementAssertion,
    },

    #[serde(rename = "wait_for_selector")]
    WaitForSelector {
        target: String,
        #[serde(default = "default_wait_state")]
        state: String,
        #[serde(default = "default_wait_timeout")]
        timeout_ms: u64,
    },

    #[serde(rename = "performance_metrics")]
    GetPerformanceMetrics,

    #[serde(rename = "scroll_to_coords")]
    ScrollToCoords {
        x: i32,
        y: i32,
        #[serde(default)]
        smooth: bool,
    },

    #[serde(rename = "page_outline")]
    GetPageOutline {
        #[serde(default = "default_outline_limit")]
        max_links: usize,
        #[serde(default = "default_outline_limit")]
        max_buttons: usize,
    },

    #[serde(rename = "wait_for_network_idle")]
    WaitForNetworkIdle {
        #[serde(default = "default_idle_time")]
        idle_time_ms: u64,
        #[serde(default = "default_idle_timeout")]
        timeout_ms: u64,
        #[serde(default = "default_max_inflight")]
        max_inflight: usize,
    },

    #[serde(rename = "watch_mutations")]
    WatchMutations {
        #[serde(default = "default_mutation_action")]
        action: String,
        #[serde(default)]
        target: Option<String>,
    },

    #[serde(rename = "tab_eval")]
    TabEval {
        tab_index: usize,
        script: String,
    },

    #[serde(rename = "response_body")]
    GetResponseBody {
        #[serde(default)]
        pattern: String,
        #[serde(default)]
        clear: bool,
    },

    #[serde(rename = "drop_file")]
    DropFile {
        target: String,
        file_name: String,
        #[serde(default = "default_mime_type")]
        mime_type: String,
        content: String,
    },

    #[serde(rename = "dom_tree")]
    GetDomTree {
        #[serde(default)]
        target: Option<String>,
        #[serde(default = "default_tree_depth")]
        max_depth: usize,
    },

    #[serde(rename = "set_offline")]
    SetOffline {
        offline: bool,
    },

    #[serde(rename = "wait_for_text")]
    WaitForText {
        #[serde(default)]
        target: Option<String>,
        text: String,
        #[serde(default = "default_wait_timeout")]
        timeout_ms: u64,
    },

    #[serde(rename = "computed_styles")]
    GetComputedStyles {
        target: String,
        #[serde(default = "default_style_properties")]
        properties: Vec<String>,
    },

    #[serde(rename = "wait_for_download")]
    WaitForDownload {
        #[serde(default)]
        file_pattern: Option<String>,
        #[serde(default = "default_download_timeout")]
        timeout_ms: u64,
    },

    #[serde(rename = "set_content")]
    SetContent {
        html: String,
    },
}

fn default_style_properties() -> Vec<String> {
    vec![
        "display".to_string(),
        "visibility".to_string(),
        "opacity".to_string(),
        "color".to_string(),
        "background-color".to_string(),
        "z-index".to_string(),
        "font-size".to_string(),
        "cursor".to_string(),
    ]
}

fn default_download_timeout() -> u64 {
    15000
}

fn default_mime_type() -> String {
    "application/octet-stream".to_string()
}

fn default_tree_depth() -> usize {
    4
}

fn default_outline_limit() -> usize {
    25
}

fn default_idle_time() -> u64 {
    500
}

fn default_idle_timeout() -> u64 {
    10000
}

fn default_max_inflight() -> usize {
    0
}

fn default_mutation_action() -> String {
    "get".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct ElementAssertion {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub visible: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hidden: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub disabled: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub checked: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text_contains: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub has_class: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub attr_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub attr_value: Option<String>,
}

fn default_wait_state() -> String {
    "visible".to_string()
}

fn default_scroll_align() -> String {
    "center".to_string()
}

fn default_storage_type() -> String {
    "local".to_string()
}

fn default_query_limit() -> usize {
    50
}

fn default_net_profile() -> String {
    "online".to_string()
}

fn default_route_action() -> String {
    "list".to_string()
}

fn default_media_action() -> String {
    "status".to_string()
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

        let up_cmd = AgentCommand::UploadFile {
            target: "input[type=file]".into(),
            files: vec!["test.png".into()],
        };
        let up_json = serde_json::to_string(&up_cmd).unwrap();
        assert!(up_json.contains("upload_file"));
        assert!(up_json.contains("test.png"));

        let loc_cmd = AgentCommand::Location {
            latitude: 40.7128,
            longitude: -74.0060,
            accuracy: Some(10.0),
            timezone: Some("America/New_York".into()),
        };
        let loc_json = serde_json::to_string(&loc_cmd).unwrap();
        assert!(loc_json.contains("location"));
        assert!(loc_json.contains("40.7128"));
        assert!(loc_json.contains("America/New_York"));

        let media_cmd = AgentCommand::MediaControl { action: "mute".into() };
        let media_json = serde_json::to_string(&media_cmd).unwrap();
        assert!(media_json.contains("media_control"));
        assert!(media_json.contains("mute"));

        let inspect_cmd = AgentCommand::Inspect { target: "#submit-btn".into() };
        let inspect_json = serde_json::to_string(&inspect_cmd).unwrap();
        assert!(inspect_json.contains("inspect"));
        assert!(inspect_json.contains("#submit-btn"));

        let route_cmd = AgentCommand::RouteRule {
            action: "add".into(),
            pattern: Some("*.png".into()),
            rule_type: Some("abort".into()),
            status: Some(404),
            body: None,
            headers: None,
        };
        let route_json = serde_json::to_string(&route_cmd).unwrap();
        assert!(route_json.contains("route_rule"));
        assert!(route_json.contains("*.png"));
        assert!(route_json.contains("abort"));

        let dbl_cmd = AgentCommand::DoubleClick { target: "tr#row-1".into() };
        let dbl_json = serde_json::to_string(&dbl_cmd).unwrap();
        assert!(dbl_json.contains("double_click"));
        assert!(dbl_json.contains("tr#row-1"));

        let rc_cmd = AgentCommand::RightClick { target: ".menu-item".into() };
        let rc_json = serde_json::to_string(&rc_cmd).unwrap();
        assert!(rc_json.contains("right_click"));
        assert!(rc_json.contains(".menu-item"));

        let tbl_cmd = AgentCommand::ExtractTable { target: Some("table.pricing".into()) };
        let tbl_json = serde_json::to_string(&tbl_cmd).unwrap();
        assert!(tbl_json.contains("extract_table"));
        assert!(tbl_json.contains("table.pricing"));

        let dl_cmd = AgentCommand::Downloads { clear: true };
        let dl_json = serde_json::to_string(&dl_cmd).unwrap();
        assert!(dl_json.contains("downloads"));
        assert!(dl_json.contains("true"));

        let net_cond_cmd = AgentCommand::NetworkConditions {
            profile: "slow-3g".into(),
            latency: Some(500.0),
            download_throughput: Some(50000.0),
            upload_throughput: Some(50000.0),
            offline: Some(false),
        };
        let net_cond_json = serde_json::to_string(&net_cond_cmd).unwrap();
        assert!(net_cond_json.contains("network_conditions"));
        assert!(net_cond_json.contains("slow-3g"));

        let css_cmd = AgentCommand::InjectCss { css: "header { display: none !important; }".into(), remove: false };
        let css_json = serde_json::to_string(&css_cmd).unwrap();
        assert!(css_json.contains("inject_css"));
        assert!(css_json.contains("header"));

        let q_cmd = AgentCommand::QueryElements { selector: "a.link".into(), attribute: Some("href".into()), max_items: 25 };
        let q_json = serde_json::to_string(&q_cmd).unwrap();
        assert!(q_json.contains("query_elements"));
        assert!(q_json.contains("a.link"));
        assert!(q_json.contains("href"));

        let focus_cmd = AgentCommand::Focus { target: "input#email".into() };
        let focus_json = serde_json::to_string(&focus_cmd).unwrap();
        assert!(focus_json.contains("focus"));
        assert!(focus_json.contains("input#email"));

        let blur_cmd = AgentCommand::Blur { target: "input#email".into() };
        let blur_json = serde_json::to_string(&blur_cmd).unwrap();
        assert!(blur_json.contains("blur"));

        let info_cmd = AgentCommand::PageInfo;
        let info_json = serde_json::to_string(&info_cmd).unwrap();
        assert!(info_json.contains("page_info"));

        let html_cmd = AgentCommand::GetHtml { target: Some("#content".into()) };
        let html_json = serde_json::to_string(&html_cmd).unwrap();
        assert!(html_json.contains("get_html"));
        assert!(html_json.contains("#content"));

        let scroll_cmd = AgentCommand::ScrollIntoView { target: "#footer".into(), align: "bottom".into() };
        let scroll_json = serde_json::to_string(&scroll_cmd).unwrap();
        assert!(scroll_json.contains("scroll_into_view"));
        assert!(scroll_json.contains("bottom"));

        let zoom_cmd = AgentCommand::SetZoom { zoom_factor: 1.5 };
        let zoom_json = serde_json::to_string(&zoom_cmd).unwrap();
        assert!(zoom_json.contains("set_zoom"));
        assert!(zoom_json.contains("1.5"));

        let store_cmd = AgentCommand::StorageItem {
            storage_type: "session".into(),
            action: "set".into(),
            key: Some("auth_token".into()),
            value: Some("xyz999".into()),
        };
        let store_json = serde_json::to_string(&store_cmd).unwrap();
        assert!(store_json.contains("storage_item"));
        assert!(store_json.contains("auth_token"));

        let assert_cmd = AgentCommand::AssertElement {
            target: "#btn".into(),
            assertions: ElementAssertion {
                visible: Some(true),
                disabled: Some(false),
                text_contains: Some("Submit".into()),
                ..Default::default()
            },
        };
        let assert_json = serde_json::to_string(&assert_cmd).unwrap();
        assert!(assert_json.contains("assert_element"));
        assert!(assert_json.contains("Submit"));

        let wait_sel_cmd = AgentCommand::WaitForSelector {
            target: ".modal".into(),
            state: "hidden".into(),
            timeout_ms: 3000,
        };
        let wait_sel_json = serde_json::to_string(&wait_sel_cmd).unwrap();
        assert!(wait_sel_json.contains("wait_for_selector"));
        assert!(wait_sel_json.contains("hidden"));

        let perf_cmd = AgentCommand::GetPerformanceMetrics;
        let perf_json = serde_json::to_string(&perf_cmd).unwrap();
        assert!(perf_json.contains("performance_metrics"));

        let scroll_coord_cmd = AgentCommand::ScrollToCoords { x: 0, y: 1200, smooth: true };
        let scroll_coord_json = serde_json::to_string(&scroll_coord_cmd).unwrap();
        assert!(scroll_coord_json.contains("scroll_to_coords"));
        assert!(scroll_coord_json.contains("1200"));

        let outline_cmd = AgentCommand::GetPageOutline { max_links: 30, max_buttons: 15 };
        let outline_json = serde_json::to_string(&outline_cmd).unwrap();
        assert!(outline_json.contains("page_outline"));
        assert!(outline_json.contains("30"));

        let idle_cmd = AgentCommand::WaitForNetworkIdle { idle_time_ms: 600, timeout_ms: 12000, max_inflight: 1 };
        let idle_json = serde_json::to_string(&idle_cmd).unwrap();
        assert!(idle_json.contains("wait_for_network_idle"));
        assert!(idle_json.contains("600"));

        let mut_cmd = AgentCommand::WatchMutations { action: "start".into(), target: Some("#app".into()) };
        let mut_json = serde_json::to_string(&mut_cmd).unwrap();
        assert!(mut_json.contains("watch_mutations"));
        assert!(mut_json.contains("start"));

        let tab_eval_cmd = AgentCommand::TabEval { tab_index: 2, script: "document.title".into() };
        let tab_eval_json = serde_json::to_string(&tab_eval_cmd).unwrap();
        assert!(tab_eval_json.contains("tab_eval"));
        assert!(tab_eval_json.contains("document.title"));

        let resp_body_cmd = AgentCommand::GetResponseBody { pattern: "api/items".into(), clear: true };
        let resp_body_json = serde_json::to_string(&resp_body_cmd).unwrap();
        assert!(resp_body_json.contains("response_body"));
        assert!(resp_body_json.contains("api/items"));

        let drop_file_cmd = AgentCommand::DropFile {
            target: "#dropzone".into(),
            file_name: "test.csv".into(),
            mime_type: "text/csv".into(),
            content: "a,b,c".into(),
        };
        let drop_file_json = serde_json::to_string(&drop_file_cmd).unwrap();
        assert!(drop_file_json.contains("drop_file"));
        assert!(drop_file_json.contains("test.csv"));

        let dom_tree_cmd = AgentCommand::GetDomTree { target: Some("#app".into()), max_depth: 3 };
        let dom_tree_json = serde_json::to_string(&dom_tree_cmd).unwrap();
        assert!(dom_tree_json.contains("dom_tree"));
        assert!(dom_tree_json.contains("3"));

        let offline_cmd = AgentCommand::SetOffline { offline: true };
        let offline_json = serde_json::to_string(&offline_cmd).unwrap();
        assert!(offline_json.contains("set_offline"));
        assert!(offline_json.contains("true"));

        let wait_txt_cmd = AgentCommand::WaitForText {
            target: Some("#msg".into()),
            text: "Success".into(),
            timeout_ms: 6000,
        };
        let wait_txt_json = serde_json::to_string(&wait_txt_cmd).unwrap();
        assert!(wait_txt_json.contains("wait_for_text"));
        assert!(wait_txt_json.contains("Success"));

        let style_cmd = AgentCommand::GetComputedStyles {
            target: "#header".into(),
            properties: vec!["color".into(), "z-index".into()],
        };
        let style_json = serde_json::to_string(&style_cmd).unwrap();
        assert!(style_json.contains("computed_styles"));
        assert!(style_json.contains("z-index"));

        let wait_dl_cmd = AgentCommand::WaitForDownload {
            file_pattern: Some("invoice".into()),
            timeout_ms: 10000,
        };
        let wait_dl_json = serde_json::to_string(&wait_dl_cmd).unwrap();
        assert!(wait_dl_json.contains("wait_for_download"));
        assert!(wait_dl_json.contains("invoice"));

        let content_cmd = AgentCommand::SetContent {
            html: "<h1>Hello Test</h1>".into(),
        };
        let content_json = serde_json::to_string(&content_cmd).unwrap();
        assert!(content_json.contains("set_content"));
        assert!(content_json.contains("Hello Test"));
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
