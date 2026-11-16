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
    ScreenshotElement {
        target: String,
        #[serde(default)]
        padding: i32,
    },

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

    #[serde(rename = "get_forms")]
    GetForms {
        #[serde(default)]
        target: Option<String>,
    },

    #[serde(rename = "set_visibility")]
    SetVisibility {
        state: String,
    },

    #[serde(rename = "emulate_media")]
    EmulateMedia {
        #[serde(default)]
        media_type: Option<String>,
        #[serde(default)]
        color_scheme: Option<String>,
    },

    #[serde(rename = "get_element_bounds")]
    GetElementBounds {
        target: String,
    },

    #[serde(rename = "mock_route")]
    MockRoute {
        #[serde(default = "default_mock_action")]
        action: String,
        #[serde(default)]
        pattern: Option<String>,
        #[serde(default = "default_mock_status")]
        status: u16,
        #[serde(default = "default_mock_content_type")]
        content_type: String,
        #[serde(default)]
        body: Option<String>,
    },

    #[serde(rename = "get_viewport")]
    GetViewport,

    #[serde(rename = "set_element_attribute")]
    SetElementAttribute {
        target: String,
        attribute: String,
        #[serde(default)]
        value: Option<String>,
    },

    #[serde(rename = "search_page_text")]
    SearchPageText {
        query: String,
        #[serde(default = "default_max_search_matches")]
        max_matches: usize,
        #[serde(default)]
        case_sensitive: bool,
    },

    #[serde(rename = "dump_storage")]
    DumpStorage {
        #[serde(default = "default_dump_storage_type")]
        storage_type: String,
    },

    #[serde(rename = "human_jitter")]
    HumanJitter {
        #[serde(default = "default_jitter_duration")]
        duration_ms: u64,
        #[serde(default = "default_true")]
        move_mouse: bool,
        #[serde(default = "default_true")]
        micro_scroll: bool,
    },

    #[serde(rename = "type_keystrokes")]
    TypeKeystrokes {
        target: String,
        text: String,
        #[serde(default = "default_keystroke_delay")]
        delay_ms: u64,
        #[serde(default)]
        clear: bool,
        #[serde(default)]
        submit: bool,
    },

    #[serde(rename = "get_canvas_data")]
    GetCanvasData {
        target: String,
        #[serde(default = "default_mime_png")]
        mime_type: String,
        #[serde(default = "default_quality_high")]
        quality: f64,
    },

    #[serde(rename = "get_resource_timing")]
    GetResourceTiming {
        #[serde(default)]
        initiator_type: Option<String>,
        #[serde(default = "default_max_resources")]
        max_entries: usize,
        #[serde(default)]
        min_duration_ms: f64,
    },

    #[serde(rename = "get_dom_stats")]
    GetDomStats,

    #[serde(rename = "scroll_by")]
    ScrollBy {
        #[serde(default)]
        dx: i32,
        #[serde(default)]
        dy: i32,
        #[serde(default)]
        smooth: bool,
    },

    #[serde(rename = "scroll_container")]
    ScrollContainer {
        target: String,
        #[serde(default)]
        dx: i32,
        #[serde(default)]
        dy: i32,
        #[serde(default)]
        smooth: bool,
    },

    #[serde(rename = "select_element_text")]
    SelectElementText {
        target: String,
        #[serde(default)]
        start: Option<usize>,
        #[serde(default)]
        length: Option<usize>,
    },

    #[serde(rename = "extract_page_links")]
    ExtractPageLinks {
        #[serde(default = "default_max_page_links")]
        max_links: usize,
        #[serde(default)]
        external_only: bool,
        #[serde(default)]
        filter_path: Option<String>,
    },

    #[serde(rename = "get_element_state")]
    GetElementState {
        target: String,
    },

    #[serde(rename = "toggle_element_class")]
    ToggleElementClass {
        target: String,
        class_name: String,
        #[serde(default = "default_class_action")]
        action: String,
    },

    #[serde(rename = "set_element_text")]
    SetElementText {
        target: String,
        text: String,
    },

    #[serde(rename = "dispatch_custom_event")]
    DispatchCustomEvent {
        target: String,
        event_type: String,
        #[serde(default = "default_true")]
        bubbles: bool,
        #[serde(default = "default_true")]
        cancelable: bool,
        #[serde(default)]
        detail: Option<serde_json::Value>,
    },

    #[serde(rename = "get_active_element")]
    GetActiveElement,

    #[serde(rename = "check_element_clickable")]
    CheckElementClickable {
        target: String,
    },

    #[serde(rename = "dispatch_wheel_event")]
    DispatchWheelEvent {
        target: String,
        #[serde(default)]
        delta_x: f64,
        #[serde(default)]
        delta_y: f64,
        #[serde(default)]
        ctrl_key: bool,
    },

    #[serde(rename = "get_css_variables")]
    GetCssVariables {
        #[serde(default)]
        target: Option<String>,
        #[serde(default)]
        variable_names: Vec<String>,
    },

    #[serde(rename = "clear_highlights")]
    ClearHighlights,

    #[serde(rename = "simulate_pointer_click")]
    SimulatePointerClick {
        target: String,
        #[serde(default = "default_click_count")]
        click_count: usize,
        #[serde(default)]
        button: i32,
    },

    #[serde(rename = "extract_meta_tags")]
    ExtractMetaTags {
        #[serde(default = "default_true")]
        include_json_ld: bool,
    },

    #[serde(rename = "get_document_geometry")]
    GetDocumentGeometry,

    #[serde(rename = "set_device_orientation")]
    SetDeviceOrientation {
        #[serde(default)]
        alpha: f64,
        #[serde(default)]
        beta: f64,
        #[serde(default)]
        gamma: f64,
        #[serde(default)]
        absolute: bool,
    },

    #[serde(rename = "clear_browser_cache")]
    ClearBrowserCache {
        #[serde(default)]
        clear_storage: bool,
        #[serde(default)]
        clear_cookies: bool,
    },

    #[serde(rename = "get_element_spacing")]
    GetElementSpacing {
        target: String,
    },

    #[serde(rename = "wait_for_url")]
    WaitForUrl {
        pattern: String,
        #[serde(default = "default_wait_timeout")]
        timeout_ms: u64,
    },

    #[serde(rename = "dispatch_touch_event")]
    DispatchTouchEvent {
        event_type: String,
        #[serde(default)]
        touches: Vec<TouchPoint>,
    },

    #[serde(rename = "get_a11y_tree")]
    GetA11yTree {
        #[serde(default)]
        target: Option<String>,
        #[serde(default = "default_a11y_depth")]
        max_depth: usize,
    },

    #[serde(rename = "simulate_keyboard_chord")]
    SimulateKeyboardChord {
        chord: String,
        #[serde(default)]
        target: Option<String>,
    },

    #[serde(rename = "highlight_bounding_boxes")]
    HighlightBoundingBoxes {
        boxes: Vec<BoundingBoxMarker>,
    },

    #[serde(rename = "get_element_attributes")]
    GetElementAttributes {
        target: String,
    },

    #[serde(rename = "get_element_filter")]
    GetElementFilter {
        target: String,
    },

    #[serde(rename = "dismiss_all_modals")]
    DismissAllModals,

    #[serde(rename = "get_color_palette")]
    GetColorPalette {
        #[serde(default = "default_palette_limit")]
        limit: usize,
    },

    #[serde(rename = "drag_and_drop_element")]
    DragAndDropElement {
        source: String,
        target: String,
        #[serde(default = "default_drag_steps")]
        steps: u32,
        #[serde(default)]
        hold_ms: u64,
    },

    #[serde(rename = "audit_forms")]
    AuditForms {
        #[serde(default)]
        target: Option<String>,
    },

    #[serde(rename = "get_viewport_paging")]
    GetViewportPaging,

    #[serde(rename = "get_computed_pseudo_element")]
    GetComputedPseudoElement {
        target: String,
        #[serde(default = "default_pseudo_name")]
        pseudo: String,
        #[serde(default)]
        properties: Vec<String>,
    },

    #[serde(rename = "simulate_clipboard_paste")]
    SimulateClipboardPaste {
        #[serde(default)]
        target: Option<String>,
        text: String,
    },

    #[serde(rename = "smart_fill")]
    SmartFill {
        mapping: std::collections::HashMap<String, serde_json::Value>,
        #[serde(default)]
        submit: bool,
    },

    #[serde(rename = "observe_text")]
    ObserveText {
        target: String,
        #[serde(default = "default_observe_timeout")]
        timeout_ms: u64,
        #[serde(default = "default_settle_timeout")]
        settle_ms: u64,
    },

    #[serde(rename = "set_visibility_state")]
    SetVisibilityState {
        #[serde(default = "default_visibility_state")]
        state: String,
    },

    #[serde(rename = "get_computed_font")]
    GetComputedFont {
        target: String,
    },

    #[serde(rename = "get_action_map")]
    GetActionMap {
        #[serde(default = "default_action_map_viewport")]
        viewport_only: bool,
        #[serde(default = "default_action_map_limit")]
        max_elements: usize,
    },

    #[serde(rename = "get_motion_state")]
    GetMotionState {
        #[serde(default)]
        target: Option<String>,
    },

    #[serde(rename = "get_element_listeners")]
    GetElementListeners {
        target: String,
    },

    #[serde(rename = "get_web_vitals")]
    GetWebVitals,

    #[serde(rename = "get_select_options")]
    GetSelectOptions {
        target: String,
    },

    #[serde(rename = "fuzzy_find_elements")]
    FuzzyFindElements {
        query: String,
        #[serde(default = "default_fuzzy_limit")]
        limit: usize,
        #[serde(default)]
        container: Option<String>,
    },

    #[serde(rename = "get_heading_hierarchy")]
    GetHeadingHierarchy {
        #[serde(default)]
        container: Option<String>,
    },

    #[serde(rename = "get_element_stacking")]
    GetElementStacking {
        target: String,
    },

    #[serde(rename = "query_shadow_elements")]
    QueryShadowElements {
        selector: String,
        #[serde(default = "default_shadow_limit")]
        limit: usize,
        #[serde(default)]
        container: Option<String>,
    },

    #[serde(rename = "get_element_visibility_matrix")]
    GetElementVisibilityMatrix {
        target: String,
    },

    #[serde(rename = "get_error_digest")]
    GetErrorDigest {
        #[serde(default = "default_error_limit")]
        max_errors: usize,
        #[serde(default)]
        clear: bool,
    },

    #[serde(rename = "extract_grid_data")]
    ExtractGridData {
        #[serde(default)]
        target: Option<String>,
        #[serde(default = "default_grid_max_rows")]
        max_rows: usize,
    },

    #[serde(rename = "diff_form_inputs")]
    DiffFormInputs {
        #[serde(default)]
        target: Option<String>,
    },

    #[serde(rename = "unstick_scroll_locks")]
    UnstickScrollLocks,

    #[serde(rename = "diff_dom_text")]
    DiffDomText {
        #[serde(default)]
        baseline: Option<String>,
        #[serde(default)]
        target: Option<String>,
    },

    #[serde(rename = "get_frame_hierarchy")]
    GetFrameHierarchy,

    #[serde(rename = "get_role_elements")]
    GetRoleElements {
        #[serde(default)]
        roles: Vec<String>,
        #[serde(default = "default_true")]
        viewport_only: bool,
        #[serde(default = "default_role_limit")]
        limit: usize,
    },

    #[serde(rename = "harvest_resources")]
    HarvestResources {
        #[serde(default)]
        resource_types: Vec<String>,
    },

    #[serde(rename = "detect_bot_challenges")]
    DetectBotChallenges,

    #[serde(rename = "get_scroll_state")]
    GetScrollState {
        #[serde(default)]
        target: Option<String>,
    },

    #[serde(rename = "get_proximity_elements")]
    GetProximityElements {
        #[serde(default)]
        x: Option<f64>,
        #[serde(default)]
        y: Option<f64>,
        #[serde(default)]
        target: Option<String>,
        #[serde(default = "default_proximity_limit")]
        limit: usize,
        #[serde(default)]
        radius: Option<f64>,
        #[serde(default = "default_true")]
        interactive_only: bool,
    },

    #[serde(rename = "emulate_screen_metrics")]
    EmulateScreenMetrics {
        #[serde(default)]
        width: Option<u32>,
        #[serde(default)]
        height: Option<u32>,
        #[serde(default)]
        device_scale_factor: Option<f64>,
        #[serde(default)]
        mobile: bool,
        #[serde(default)]
        orientation: Option<String>,
        #[serde(default)]
        color_scheme: Option<String>,
    },

    #[serde(rename = "export_table_csv")]
    ExportTableCsv {
        #[serde(default)]
        target: Option<String>,
        #[serde(default = "default_true")]
        include_headers: bool,
        #[serde(default = "default_csv_delimiter")]
        delimiter: String,
    },

    #[serde(rename = "reset_form")]
    ResetForm {
        #[serde(default)]
        target: Option<String>,
        #[serde(default = "default_true")]
        trigger_events: bool,
        #[serde(default = "default_true")]
        clear_custom_validations: bool,
    },

    #[serde(rename = "get_datalist_options")]
    GetDatalistOptions {
        #[serde(default)]
        target: Option<String>,
    },

    #[serde(rename = "get_canvas_context")]
    GetCanvasContext {
        #[serde(default)]
        target: Option<String>,
    },

    #[serde(rename = "control_media_playback")]
    ControlMediaPlayback {
        #[serde(default = "default_playback_action")]
        action: String,
        #[serde(default)]
        target: Option<String>,
        #[serde(default)]
        value: Option<f64>,
    },

    #[serde(rename = "get_element_ancestry")]
    GetElementAncestry {
        target: String,
        #[serde(default = "default_ancestry_depth")]
        max_depth: usize,
    },

    #[serde(rename = "get_layout_metrics")]
    GetLayoutMetrics {
        #[serde(default)]
        target: Option<String>,
    },

    #[serde(rename = "audit_contrast_ratio")]
    AuditContrastRatio {
        target: String,
    },

    #[serde(rename = "dispatch_advanced_event")]
    DispatchAdvancedEvent {
        target: String,
        event_type: String,
        #[serde(default)]
        modifiers: Vec<String>,
        #[serde(default)]
        button: i32,
        #[serde(default = "default_pointer_pressure")]
        pressure: f64,
        #[serde(default = "default_click_detail")]
        detail: i32,
    },

    #[serde(rename = "get_network_har")]
    GetNetworkHar {
        #[serde(default)]
        clear: bool,
    },

    #[serde(rename = "get_caret_position")]
    GetCaretPosition {
        target: String,
        #[serde(default)]
        set_start: Option<usize>,
        #[serde(default)]
        set_end: Option<usize>,
    },

    #[serde(rename = "dismiss_cookie_banners")]
    DismissCookieBanners {
        #[serde(default = "default_true")]
        click_accept: bool,
    },

    #[serde(rename = "emulate_speech_api")]
    EmulateSpeechApi {
        #[serde(default)]
        action: Option<String>,
        #[serde(default)]
        mock_transcript: Option<String>,
    },

    #[serde(rename = "get_dom_mutations")]
    GetDomMutations {
        #[serde(default = "default_mutation_limit")]
        limit: usize,
        #[serde(default = "default_true")]
        clear: bool,
    },

    #[serde(rename = "get_active_routes")]
    GetActiveRoutes,

    #[serde(rename = "exec_editor_command")]
    ExecEditorCommand {
        command: String,
        #[serde(default)]
        value: Option<String>,
    },

    #[serde(rename = "preload_lazy_resources")]
    PreloadLazyResources {
        #[serde(default = "default_true")]
        force_all: bool,
        #[serde(default)]
        scroll_container: Option<String>,
    },

    #[serde(rename = "emulate_window_focus")]
    EmulateWindowFocus {
        #[serde(default = "default_true")]
        focused: bool,
        #[serde(default = "default_true")]
        emit_events: bool,
    },
}

fn default_mutation_limit() -> usize {
    50
}

fn default_pointer_pressure() -> f64 {
    0.5
}

fn default_click_detail() -> i32 {
    1
}

fn default_playback_action() -> String {
    "status".to_string()
}

fn default_ancestry_depth() -> usize {
    20
}

fn default_proximity_limit() -> usize {
    10
}

fn default_csv_delimiter() -> String {
    ",".to_string()
}

fn default_role_limit() -> usize {
    50
}

fn default_shadow_limit() -> usize {
    50
}

fn default_error_limit() -> usize {
    50
}

fn default_grid_max_rows() -> usize {
    100
}

fn default_fuzzy_limit() -> usize {
    10
}

fn default_action_map_viewport() -> bool {
    true
}

fn default_action_map_limit() -> usize {
    50
}

fn default_observe_timeout() -> u64 {
    10000
}

fn default_settle_timeout() -> u64 {
    1000
}

fn default_visibility_state() -> String {
    "visible".into()
}

fn default_pseudo_name() -> String {
    "::before".into()
}

fn default_palette_limit() -> usize {
    15
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoundingBoxMarker {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    #[serde(default)]
    pub label: Option<String>,
    #[serde(default)]
    pub color: Option<String>,
}

fn default_a11y_depth() -> usize {
    5
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TouchPoint {
    #[serde(default)]
    pub identifier: i32,
    pub target: String,
    pub x: f64,
    pub y: f64,
    #[serde(default = "default_touch_radius")]
    pub radius_x: f64,
    #[serde(default = "default_touch_radius")]
    pub radius_y: f64,
    #[serde(default)]
    pub rotation_angle: f64,
    #[serde(default = "default_touch_force")]
    pub force: f64,
}

fn default_touch_radius() -> f64 {
    1.0
}

fn default_touch_force() -> f64 {
    1.0
}

fn default_click_count() -> usize {
    1
}

fn default_class_action() -> String {
    "toggle".to_string()
}

fn default_max_page_links() -> usize {
    100
}

fn default_mime_png() -> String {
    "image/png".to_string()
}

fn default_quality_high() -> f64 {
    0.92
}

fn default_max_resources() -> usize {
    50
}

fn default_max_search_matches() -> usize {
    20
}

fn default_dump_storage_type() -> String {
    "all".to_string()
}

fn default_jitter_duration() -> u64 {
    800
}

fn default_true() -> bool {
    true
}

fn default_keystroke_delay() -> u64 {
    25
}

fn default_mock_action() -> String {
    "set".to_string()
}

fn default_mock_status() -> u16 {
    200
}

fn default_mock_content_type() -> String {
    "application/json".to_string()
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

        let search = AgentCommand::SearchPageText {
            query: "login".into(),
            max_matches: 10,
            case_sensitive: false,
        };
        let json_search = serde_json::to_string(&search).unwrap();
        assert!(json_search.contains("search_page_text"));
        assert!(json_search.contains("\"query\":\"login\""));

        let dump = AgentCommand::DumpStorage {
            storage_type: "session".into(),
        };
        let json_dump = serde_json::to_string(&dump).unwrap();
        assert!(json_dump.contains("dump_storage"));
        assert!(json_dump.contains("\"storage_type\":\"session\""));

        let jitter = AgentCommand::HumanJitter {
            duration_ms: 500,
            move_mouse: true,
            micro_scroll: true,
        };
        let json_jitter = serde_json::to_string(&jitter).unwrap();
        assert!(json_jitter.contains("human_jitter"));
        assert!(json_jitter.contains("\"duration_ms\":500"));

        let keystrokes = AgentCommand::TypeKeystrokes {
            target: "#user".into(),
            text: "admin".into(),
            delay_ms: 30,
            clear: true,
            submit: true,
        };
        let json_keystrokes = serde_json::to_string(&keystrokes).unwrap();
        assert!(json_keystrokes.contains("type_keystrokes"));
        assert!(json_keystrokes.contains("\"text\":\"admin\""));

        let canvas_cmd = AgentCommand::GetCanvasData {
            target: "#chart-canvas".into(),
            mime_type: "image/png".into(),
            quality: 0.95,
        };
        let json_canvas = serde_json::to_string(&canvas_cmd).unwrap();
        assert!(json_canvas.contains("get_canvas_data"));
        assert!(json_canvas.contains("#chart-canvas"));

        let res_cmd = AgentCommand::GetResourceTiming {
            initiator_type: Some("script".into()),
            max_entries: 25,
            min_duration_ms: 50.0,
        };
        let json_res = serde_json::to_string(&res_cmd).unwrap();
        assert!(json_res.contains("get_resource_timing"));
        assert!(json_res.contains("script"));

        let dom_stats = AgentCommand::GetDomStats;
        let json_stats = serde_json::to_string(&dom_stats).unwrap();
        assert!(json_stats.contains("get_dom_stats"));

        let scroll_by = AgentCommand::ScrollBy {
            dx: 50,
            dy: 300,
            smooth: false,
        };
        let json_scroll_by = serde_json::to_string(&scroll_by).unwrap();
        assert!(json_scroll_by.contains("scroll_by"));
        assert!(json_scroll_by.contains("\"dy\":300"));

        let scroll_cont = AgentCommand::ScrollContainer {
            target: "#chat-feed".into(),
            dx: 0,
            dy: 450,
            smooth: true,
        };
        let json_scroll_cont = serde_json::to_string(&scroll_cont).unwrap();
        assert!(json_scroll_cont.contains("scroll_container"));
        assert!(json_scroll_cont.contains("#chat-feed"));

        let select_txt = AgentCommand::SelectElementText {
            target: "#editor".into(),
            start: Some(0),
            length: Some(25),
        };
        let json_select = serde_json::to_string(&select_txt).unwrap();
        assert!(json_select.contains("select_element_text"));
        assert!(json_select.contains("#editor"));

        let links_cmd = AgentCommand::ExtractPageLinks {
            max_links: 50,
            external_only: true,
            filter_path: Some("/docs".into()),
        };
        let json_links = serde_json::to_string(&links_cmd).unwrap();
        assert!(json_links.contains("extract_page_links"));
        assert!(json_links.contains("/docs"));

        let el_state = AgentCommand::GetElementState {
            target: "#btn-submit".into(),
        };
        let json_state = serde_json::to_string(&el_state).unwrap();
        assert!(json_state.contains("get_element_state"));
        assert!(json_state.contains("#btn-submit"));

        let class_cmd = AgentCommand::ToggleElementClass {
            target: "#modal".into(),
            class_name: "hidden".into(),
            action: "toggle".into(),
        };
        let json_class = serde_json::to_string(&class_cmd).unwrap();
        assert!(json_class.contains("toggle_element_class"));
        assert!(json_class.contains("hidden"));

        let text_cmd = AgentCommand::SetElementText {
            target: "#heading".into(),
            text: "Updated Title".into(),
        };
        let json_text = serde_json::to_string(&text_cmd).unwrap();
        assert!(json_text.contains("set_element_text"));
        assert!(json_text.contains("Updated Title"));

        let evt_cmd = AgentCommand::DispatchCustomEvent {
            target: "#cart".into(),
            event_type: "cart:refresh".into(),
            bubbles: true,
            cancelable: true,
            detail: Some(serde_json::json!({ "items": 3 })),
        };
        let json_evt = serde_json::to_string(&evt_cmd).unwrap();
        assert!(json_evt.contains("dispatch_custom_event"));
        assert!(json_evt.contains("cart:refresh"));

        let active_cmd = AgentCommand::GetActiveElement;
        let json_active = serde_json::to_string(&active_cmd).unwrap();
        assert!(json_active.contains("get_active_element"));

        let clickable_cmd = AgentCommand::CheckElementClickable { target: "button#submit".into() };
        let json_clickable = serde_json::to_string(&clickable_cmd).unwrap();
        assert!(json_clickable.contains("check_element_clickable"));
        assert!(json_clickable.contains("button#submit"));

        let wheel_cmd = AgentCommand::DispatchWheelEvent {
            target: "#map".into(),
            delta_x: 0.0,
            delta_y: -120.0,
            ctrl_key: true,
        };
        let json_wheel = serde_json::to_string(&wheel_cmd).unwrap();
        assert!(json_wheel.contains("dispatch_wheel_event"));
        assert!(json_wheel.contains("-120"));

        let css_vars_cmd = AgentCommand::GetCssVariables {
            target: None,
            variable_names: vec!["--primary-color".into()],
        };
        let json_vars = serde_json::to_string(&css_vars_cmd).unwrap();
        assert!(json_vars.contains("get_css_variables"));
        assert!(json_vars.contains("--primary-color"));

        let clear_hl_cmd = AgentCommand::ClearHighlights;
        let json_clear = serde_json::to_string(&clear_hl_cmd).unwrap();
        assert!(json_clear.contains("clear_highlights"));
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

        let crop_cmd = AgentCommand::ScreenshotElement { target: "input#q".into(), padding: 10 };
        let crop_json = serde_json::to_string(&crop_cmd).unwrap();
        assert!(crop_json.contains("screenshot_element"));
        assert!(crop_json.contains("input#q"));
        assert!(crop_json.contains("10"));

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

        let forms_cmd = AgentCommand::GetForms {
            target: Some("#login-section".into()),
        };
        let forms_json = serde_json::to_string(&forms_cmd).unwrap();
        assert!(forms_json.contains("get_forms"));
        assert!(forms_json.contains("#login-section"));

        let vis_cmd = AgentCommand::SetVisibility {
            state: "visible".into(),
        };
        let vis_json = serde_json::to_string(&vis_cmd).unwrap();
        assert!(vis_json.contains("set_visibility"));
        assert!(vis_json.contains("visible"));

        let media_cmd = AgentCommand::EmulateMedia {
            media_type: Some("screen".into()),
            color_scheme: Some("dark".into()),
        };
        let media_json = serde_json::to_string(&media_cmd).unwrap();
        assert!(media_json.contains("emulate_media"));
        assert!(media_json.contains("dark"));

        let bounds_cmd = AgentCommand::GetElementBounds {
            target: "#cta-button".into(),
        };
        let bounds_json = serde_json::to_string(&bounds_cmd).unwrap();
        assert!(bounds_json.contains("get_element_bounds"));
        assert!(bounds_json.contains("#cta-button"));

        let mock_cmd = AgentCommand::MockRoute {
            action: "set".into(),
            pattern: Some("/api/user".into()),
            status: 200,
            content_type: "application/json".into(),
            body: Some(r#"{"id":1}"#.into()),
        };
        let mock_json = serde_json::to_string(&mock_cmd).unwrap();
        assert!(mock_json.contains("mock_route"));
        assert!(mock_json.contains("/api/user"));

        let vp_cmd = AgentCommand::GetViewport;
        let vp_json = serde_json::to_string(&vp_cmd).unwrap();
        assert!(vp_json.contains("get_viewport"));

        let attr_cmd = AgentCommand::SetElementAttribute {
            target: "#input1".into(),
            attribute: "disabled".into(),
            value: None,
        };
        let attr_json = serde_json::to_string(&attr_cmd).unwrap();
        assert!(attr_json.contains("set_element_attribute"));
        assert!(attr_json.contains("disabled"));

        let search_cmd = AgentCommand::SearchPageText {
            query: "Total: $".into(),
            max_matches: 10,
            case_sensitive: false,
        };
        let search_json = serde_json::to_string(&search_cmd).unwrap();
        assert!(search_json.contains("search_page_text"));
        assert!(search_json.contains("Total: $"));

        let dump_cmd = AgentCommand::DumpStorage {
            storage_type: "all".into(),
        };
        let dump_json = serde_json::to_string(&dump_cmd).unwrap();
        assert!(dump_json.contains("dump_storage"));

        let jitter_cmd = AgentCommand::HumanJitter {
            duration_ms: 1000,
            move_mouse: true,
            micro_scroll: true,
        };
        let jitter_json = serde_json::to_string(&jitter_cmd).unwrap();
        assert!(jitter_json.contains("human_jitter"));
        assert!(jitter_json.contains("1000"));

        let keys_cmd = AgentCommand::TypeKeystrokes {
            target: "#search-input".into(),
            text: "flights".into(),
            delay_ms: 30,
            clear: true,
            submit: false,
        };
        let keys_json = serde_json::to_string(&keys_cmd).unwrap();
        assert!(keys_json.contains("type_keystrokes"));
        assert!(keys_json.contains("flights"));

        let ptr_cmd = AgentCommand::SimulatePointerClick {
            target: "#canvas-target".into(),
            click_count: 1,
            button: 0,
        };
        let ptr_json = serde_json::to_string(&ptr_cmd).unwrap();
        assert!(ptr_json.contains("simulate_pointer_click"));
        assert!(ptr_json.contains("#canvas-target"));

        let meta_cmd = AgentCommand::ExtractMetaTags {
            include_json_ld: true,
        };
        let meta_json = serde_json::to_string(&meta_cmd).unwrap();
        assert!(meta_json.contains("extract_meta_tags"));

        let geom_cmd = AgentCommand::GetDocumentGeometry;
        let geom_json = serde_json::to_string(&geom_cmd).unwrap();
        assert!(geom_json.contains("get_document_geometry"));

        let orient_cmd = AgentCommand::SetDeviceOrientation {
            alpha: 45.0,
            beta: 15.0,
            gamma: -10.0,
            absolute: true,
        };
        let orient_json = serde_json::to_string(&orient_cmd).unwrap();
        assert!(orient_json.contains("set_device_orientation"));
        assert!(orient_json.contains("45"));

        let cache_cmd = AgentCommand::ClearBrowserCache {
            clear_storage: true,
            clear_cookies: true,
        };
        let cache_json = serde_json::to_string(&cache_cmd).unwrap();
        assert!(cache_json.contains("clear_browser_cache"));

        let spacing_cmd = AgentCommand::GetElementSpacing {
            target: "#card-view".into(),
        };
        let spacing_json = serde_json::to_string(&spacing_cmd).unwrap();
        assert!(spacing_json.contains("get_element_spacing"));
        assert!(spacing_json.contains("#card-view"));

        let wait_url_cmd = AgentCommand::WaitForUrl {
            pattern: "/dashboard".into(),
            timeout_ms: 6000,
        };
        let wait_url_json = serde_json::to_string(&wait_url_cmd).unwrap();
        assert!(wait_url_json.contains("wait_for_url"));
        assert!(wait_url_json.contains("/dashboard"));

        let touch_cmd = AgentCommand::DispatchTouchEvent {
            event_type: "touchstart".into(),
            touches: vec![TouchPoint {
                identifier: 0,
                target: "#canvas".into(),
                x: 100.0,
                y: 150.0,
                radius_x: 2.0,
                radius_y: 2.0,
                rotation_angle: 0.0,
                force: 1.0,
            }],
        };
        let touch_json = serde_json::to_string(&touch_cmd).unwrap();
        assert!(touch_json.contains("dispatch_touch_event"));
        assert!(touch_json.contains("touchstart"));

        let a11y_cmd = AgentCommand::GetA11yTree {
            target: Some("#main".into()),
            max_depth: 3,
        };
        let a11y_json = serde_json::to_string(&a11y_cmd).unwrap();
        assert!(a11y_json.contains("get_a11y_tree"));
        assert!(a11y_json.contains("#main"));

        let chord_cmd = AgentCommand::SimulateKeyboardChord {
            chord: "Control+Shift+P".into(),
            target: None,
        };
        let chord_json = serde_json::to_string(&chord_cmd).unwrap();
        assert!(chord_json.contains("simulate_keyboard_chord"));
        assert!(chord_json.contains("Control+Shift+P"));

        let boxes_cmd = AgentCommand::HighlightBoundingBoxes {
            boxes: vec![BoundingBoxMarker {
                x: 10.0,
                y: 20.0,
                width: 100.0,
                height: 50.0,
                label: Some("Button 1".into()),
                color: Some("#ff0000".into()),
            }],
        };
        let boxes_json = serde_json::to_string(&boxes_cmd).unwrap();
        assert!(boxes_json.contains("highlight_bounding_boxes"));
        assert!(boxes_json.contains("Button 1"));

        let attrs_cmd = AgentCommand::GetElementAttributes {
            target: "#btn-submit".into(),
        };
        let attrs_json = serde_json::to_string(&attrs_cmd).unwrap();
        assert!(attrs_json.contains("get_element_attributes"));
        assert!(attrs_json.contains("#btn-submit"));

        let filter_cmd = AgentCommand::GetElementFilter {
            target: "#overlay-modal".into(),
        };
        let filter_json = serde_json::to_string(&filter_cmd).unwrap();
        assert!(filter_json.contains("get_element_filter"));
        assert!(filter_json.contains("#overlay-modal"));

        let dismiss_cmd = AgentCommand::DismissAllModals;
        let dismiss_json = serde_json::to_string(&dismiss_cmd).unwrap();
        assert!(dismiss_json.contains("dismiss_all_modals"));

        let palette_cmd = AgentCommand::GetColorPalette { limit: 10 };
        let palette_json = serde_json::to_string(&palette_cmd).unwrap();
        assert!(palette_json.contains("get_color_palette"));
        assert!(palette_json.contains("10"));

        let dnd_cmd = AgentCommand::DragAndDropElement {
            source: "#item-1".into(),
            target: "#col-2".into(),
            steps: 8,
            hold_ms: 100,
        };
        let dnd_json = serde_json::to_string(&dnd_cmd).unwrap();
        assert!(dnd_json.contains("drag_and_drop_element"));
        assert!(dnd_json.contains("#item-1"));
        assert!(dnd_json.contains("#col-2"));

        let forms_cmd = AgentCommand::AuditForms { target: Some("#login-form".into()) };
        let forms_json = serde_json::to_string(&forms_cmd).unwrap();
        assert!(forms_json.contains("audit_forms"));
        assert!(forms_json.contains("#login-form"));

        let paging_cmd = AgentCommand::GetViewportPaging;
        let paging_json = serde_json::to_string(&paging_cmd).unwrap();
        assert!(paging_json.contains("get_viewport_paging"));

        let pseudo_cmd = AgentCommand::GetComputedPseudoElement {
            target: "#icon-check".into(),
            pseudo: "::before".into(),
            properties: vec!["content".into(), "color".into()],
        };
        let pseudo_json = serde_json::to_string(&pseudo_cmd).unwrap();
        assert!(pseudo_json.contains("get_computed_pseudo_element"));
        assert!(pseudo_json.contains("#icon-check"));
        assert!(pseudo_json.contains("::before"));
        assert!(pseudo_json.contains("content"));

        let paste_cmd = AgentCommand::SimulateClipboardPaste {
            target: Some("#editor".into()),
            text: "const x = 42;".into(),
        };
        let paste_json = serde_json::to_string(&paste_cmd).unwrap();
        assert!(paste_json.contains("simulate_clipboard_paste"));
        assert!(paste_json.contains("#editor"));
        assert!(paste_json.contains("const x = 42;"));

        let mut map = std::collections::HashMap::new();
        map.insert("First Name".into(), serde_json::json!("Alice"));
        let smart_cmd = AgentCommand::SmartFill { mapping: map, submit: true };
        let smart_json = serde_json::to_string(&smart_cmd).unwrap();
        assert!(smart_json.contains("smart_fill"));
        assert!(smart_json.contains("First Name"));
        assert!(smart_json.contains("Alice"));

        let obs_cmd = AgentCommand::ObserveText { target: "#stream".into(), timeout_ms: 5000, settle_ms: 500 };
        let obs_json = serde_json::to_string(&obs_cmd).unwrap();
        assert!(obs_json.contains("observe_text"));
        assert!(obs_json.contains("#stream"));

        let vis_cmd = AgentCommand::SetVisibilityState { state: "visible".into() };
        let vis_json = serde_json::to_string(&vis_cmd).unwrap();
        assert!(vis_json.contains("set_visibility_state"));
        assert!(vis_json.contains("visible"));

        let font_cmd = AgentCommand::GetComputedFont { target: "h1.title".into() };
        let font_json = serde_json::to_string(&font_cmd).unwrap();
        assert!(font_json.contains("get_computed_font"));
        assert!(font_json.contains("h1.title"));

        let action_cmd = AgentCommand::GetActionMap { viewport_only: true, max_elements: 30 };
        let action_json = serde_json::to_string(&action_cmd).unwrap();
        assert!(action_json.contains("get_action_map"));
        assert!(action_json.contains("30"));

        let motion_cmd = AgentCommand::GetMotionState { target: Some("#modal".into()) };
        let motion_json = serde_json::to_string(&motion_cmd).unwrap();
        assert!(motion_json.contains("get_motion_state"));
        assert!(motion_json.contains("#modal"));

        let listeners_cmd = AgentCommand::GetElementListeners { target: "#btn-click".into() };
        let listeners_json = serde_json::to_string(&listeners_cmd).unwrap();
        assert!(listeners_json.contains("get_element_listeners"));
        assert!(listeners_json.contains("#btn-click"));

        let vitals_cmd = AgentCommand::GetWebVitals;
        let vitals_json = serde_json::to_string(&vitals_cmd).unwrap();
        assert!(vitals_json.contains("get_web_vitals"));

        let select_cmd = AgentCommand::GetSelectOptions { target: "#country".into() };
        let select_json = serde_json::to_string(&select_cmd).unwrap();
        assert!(select_json.contains("get_select_options"));
        assert!(select_json.contains("#country"));

        let fuzzy_cmd = AgentCommand::FuzzyFindElements { query: "checkout".into(), limit: 5, container: None };
        let fuzzy_json = serde_json::to_string(&fuzzy_cmd).unwrap();
        assert!(fuzzy_json.contains("fuzzy_find_elements"));
        assert!(fuzzy_json.contains("checkout"));

        let heading_cmd = AgentCommand::GetHeadingHierarchy { container: Some("#article".into()) };
        let heading_json = serde_json::to_string(&heading_cmd).unwrap();
        assert!(heading_json.contains("get_heading_hierarchy"));
        assert!(heading_json.contains("#article"));

        let stack_cmd = AgentCommand::GetElementStacking { target: "#sticky-header".into() };
        let stack_json = serde_json::to_string(&stack_cmd).unwrap();
        assert!(stack_json.contains("get_element_stacking"));
        assert!(stack_json.contains("#sticky-header"));

        let shadow_cmd = AgentCommand::QueryShadowElements { selector: "button.submit".into(), limit: 10, container: Some("#host".into()) };
        let shadow_json = serde_json::to_string(&shadow_cmd).unwrap();
        assert!(shadow_json.contains("query_shadow_elements"));
        assert!(shadow_json.contains("button.submit"));
        assert!(shadow_json.contains("#host"));

        let vis_mat_cmd = AgentCommand::GetElementVisibilityMatrix { target: "#card-1".into() };
        let vis_mat_json = serde_json::to_string(&vis_mat_cmd).unwrap();
        assert!(vis_mat_json.contains("get_element_visibility_matrix"));
        assert!(vis_mat_json.contains("#card-1"));

        let err_cmd = AgentCommand::GetErrorDigest { max_errors: 25, clear: true };
        let err_json = serde_json::to_string(&err_cmd).unwrap();
        assert!(err_json.contains("get_error_digest"));
        assert!(err_json.contains("25"));

        let grid_cmd = AgentCommand::ExtractGridData { target: Some("#data-grid".into()), max_rows: 50 };
        let grid_json = serde_json::to_string(&grid_cmd).unwrap();
        assert!(grid_json.contains("extract_grid_data"));
        assert!(grid_json.contains("#data-grid"));
        assert!(grid_json.contains("50"));

        let diff_forms_cmd = AgentCommand::DiffFormInputs { target: Some("#checkout-form".into()) };
        let diff_forms_json = serde_json::to_string(&diff_forms_cmd).unwrap();
        assert!(diff_forms_json.contains("diff_form_inputs"));
        assert!(diff_forms_json.contains("#checkout-form"));

        let unstick_cmd = AgentCommand::UnstickScrollLocks;
        let unstick_json = serde_json::to_string(&unstick_cmd).unwrap();
        assert!(unstick_json.contains("unstick_scroll_locks"));

        let diff_dom_cmd = AgentCommand::DiffDomText { baseline: Some("old text".into()), target: None };
        let diff_dom_json = serde_json::to_string(&diff_dom_cmd).unwrap();
        assert!(diff_dom_json.contains("diff_dom_text"));
        assert!(diff_dom_json.contains("old text"));

        let frames_cmd = AgentCommand::GetFrameHierarchy;
        let frames_json = serde_json::to_string(&frames_cmd).unwrap();
        assert!(frames_json.contains("get_frame_hierarchy"));

        let roles_cmd = AgentCommand::GetRoleElements { roles: vec!["button".into(), "link".into()], viewport_only: true, limit: 25 };
        let roles_json = serde_json::to_string(&roles_cmd).unwrap();
        assert!(roles_json.contains("get_role_elements"));
        assert!(roles_json.contains("button"));
        assert!(roles_json.contains("link"));

        let harvest_cmd = AgentCommand::HarvestResources { resource_types: vec!["images".into(), "scripts".into()] };
        let harvest_json = serde_json::to_string(&harvest_cmd).unwrap();
        assert!(harvest_json.contains("harvest_resources"));
        assert!(harvest_json.contains("images"));

        let bot_cmd = AgentCommand::DetectBotChallenges;
        let bot_json = serde_json::to_string(&bot_cmd).unwrap();
        assert!(bot_json.contains("detect_bot_challenges"));

        let scroll_state_cmd = AgentCommand::GetScrollState { target: Some("#scrollable-div".into()) };
        let scroll_state_json = serde_json::to_string(&scroll_state_cmd).unwrap();
        assert!(scroll_state_json.contains("get_scroll_state"));
        assert!(scroll_state_json.contains("#scrollable-div"));

        let proximity_cmd = AgentCommand::GetProximityElements {
            x: Some(150.0),
            y: Some(250.0),
            target: Some("#search-box".into()),
            limit: 12,
            radius: Some(300.0),
            interactive_only: true,
        };
        let proximity_json = serde_json::to_string(&proximity_cmd).unwrap();
        assert!(proximity_json.contains("get_proximity_elements"));
        assert!(proximity_json.contains("#search-box"));
        assert!(proximity_json.contains("12"));

        let metrics_cmd = AgentCommand::EmulateScreenMetrics {
            width: Some(414),
            height: Some(896),
            device_scale_factor: Some(2.0),
            mobile: true,
            orientation: Some("portraitPrimary".into()),
            color_scheme: Some("dark".into()),
        };
        let metrics_json = serde_json::to_string(&metrics_cmd).unwrap();
        assert!(metrics_json.contains("emulate_screen_metrics"));
        assert!(metrics_json.contains("414"));
        assert!(metrics_json.contains("portraitPrimary"));

        let csv_cmd = AgentCommand::ExportTableCsv {
            target: Some("#results-table".into()),
            include_headers: true,
            delimiter: ";".into(),
        };
        let csv_json = serde_json::to_string(&csv_cmd).unwrap();
        assert!(csv_json.contains("export_table_csv"));
        assert!(csv_json.contains("#results-table"));
        assert!(csv_json.contains(";"));

        let reset_cmd = AgentCommand::ResetForm {
            target: Some("#reg-form".into()),
            trigger_events: true,
            clear_custom_validations: true,
        };
        let reset_json = serde_json::to_string(&reset_cmd).unwrap();
        assert!(reset_json.contains("reset_form"));
        assert!(reset_json.contains("#reg-form"));

        let datalist_cmd = AgentCommand::GetDatalistOptions { target: Some("#city-input".into()) };
        let datalist_json = serde_json::to_string(&datalist_cmd).unwrap();
        assert!(datalist_json.contains("get_datalist_options"));
        assert!(datalist_json.contains("#city-input"));

        let canvas_cmd = AgentCommand::GetCanvasContext { target: Some("#webgl-canvas".into()) };
        let canvas_json = serde_json::to_string(&canvas_cmd).unwrap();
        assert!(canvas_json.contains("get_canvas_context"));
        assert!(canvas_json.contains("#webgl-canvas"));

        let media_cmd = AgentCommand::ControlMediaPlayback {
            action: "seek".into(),
            target: Some("#promo-video".into()),
            value: Some(42.5),
        };
        let media_json = serde_json::to_string(&media_cmd).unwrap();
        assert!(media_json.contains("control_media_playback"));
        assert!(media_json.contains("seek"));
        assert!(media_json.contains("#promo-video"));

        let ancestry_cmd = AgentCommand::GetElementAncestry {
            target: "#submit-btn".into(),
            max_depth: 15,
        };
        let ancestry_json = serde_json::to_string(&ancestry_cmd).unwrap();
        assert!(ancestry_json.contains("get_element_ancestry"));
        assert!(ancestry_json.contains("#submit-btn"));
        assert!(ancestry_json.contains("15"));

        let layout_cmd = AgentCommand::GetLayoutMetrics { target: Some("#grid-container".into()) };
        let layout_json = serde_json::to_string(&layout_cmd).unwrap();
        assert!(layout_json.contains("get_layout_metrics"));
        assert!(layout_json.contains("#grid-container"));

        let contrast_cmd = AgentCommand::AuditContrastRatio { target: "#btn-cta".into() };
        let contrast_json = serde_json::to_string(&contrast_cmd).unwrap();
        assert!(contrast_json.contains("audit_contrast_ratio"));
        assert!(contrast_json.contains("#btn-cta"));

        let event_cmd = AgentCommand::DispatchAdvancedEvent {
            target: "#paint-area".into(),
            event_type: "pointerdown".into(),
            modifiers: vec!["shift".into(), "alt".into()],
            button: 0,
            pressure: 0.75,
            detail: 1,
        };
        let event_json = serde_json::to_string(&event_cmd).unwrap();
        assert!(event_json.contains("dispatch_advanced_event"));
        assert!(event_json.contains("pointerdown"));
        assert!(event_json.contains("shift"));

        let har_cmd = AgentCommand::GetNetworkHar { clear: true };
        let har_json = serde_json::to_string(&har_cmd).unwrap();
        assert!(har_json.contains("get_network_har"));

        let routes_cmd = AgentCommand::GetActiveRoutes;
        let routes_json = serde_json::to_string(&routes_cmd).unwrap();
        assert!(routes_json.contains("get_active_routes"));

        let exec_cmd = AgentCommand::ExecEditorCommand {
            command: "insertText".into(),
            value: Some("Hello Agent".into()),
        };
        let exec_json = serde_json::to_string(&exec_cmd).unwrap();
        assert!(exec_json.contains("exec_editor_command"));
        assert!(exec_json.contains("Hello Agent"));

        let preload_cmd = AgentCommand::PreloadLazyResources {
            force_all: true,
            scroll_container: Some("#gallery".into()),
        };
        let preload_json = serde_json::to_string(&preload_cmd).unwrap();
        assert!(preload_json.contains("preload_lazy_resources"));
        assert!(preload_json.contains("#gallery"));

        let focus_cmd = AgentCommand::EmulateWindowFocus {
            focused: true,
            emit_events: true,
        };
        let focus_json = serde_json::to_string(&focus_cmd).unwrap();
        assert!(focus_json.contains("emulate_window_focus"));
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
