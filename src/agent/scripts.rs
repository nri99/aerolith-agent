pub struct AgentScripts;

impl AgentScripts {
    /// JavaScript that initializes the persistent agent registry on the page.
    pub fn get_init_script() -> &'static str {
        r#"(function() {
            if (!window.__aerolith_elements) {
                window.__aerolith_elements = {};
                window.__aerolith_next_ref = 1;
            }
            if (window.__aerolith_inflight === undefined) {
                window.__aerolith_inflight = 0;
                window.__aerolith_network = [];
                try {
                    const origFetch = window.fetch;
                    if (origFetch) {
                        window.fetch = function() {
                            window.__aerolith_inflight++;
                            const start = Date.now();
                            const rawUrl = typeof arguments[0] === 'string' ? arguments[0] : (arguments[0] && arguments[0].url || '');
                            const method = (arguments[1] && arguments[1].method || 'GET').toUpperCase();
                            return origFetch.apply(this, arguments).then((res) => {
                                try {
                                    window.__aerolith_network.push({
                                        url: String(rawUrl).substring(0, 300),
                                        method: method,
                                        status: res.status,
                                        ok: res.ok,
                                        duration_ms: Date.now() - start,
                                        timestamp: Date.now()
                                    });
                                    if (window.__aerolith_network.length > 50) window.__aerolith_network.shift();
                                } catch (e) {}
                                return res;
                            }).catch((err) => {
                                try {
                                    window.__aerolith_network.push({
                                        url: String(rawUrl).substring(0, 300),
                                        method: method,
                                        status: 0,
                                        ok: false,
                                        error: String(err),
                                        duration_ms: Date.now() - start,
                                        timestamp: Date.now()
                                    });
                                    if (window.__aerolith_network.length > 50) window.__aerolith_network.shift();
                                } catch (e) {}
                                throw err;
                            }).finally(() => {
                                window.__aerolith_inflight = Math.max(0, window.__aerolith_inflight - 1);
                            });
                        };
                    }
                    const origSend = XMLHttpRequest.prototype.send;
                    if (origSend) {
                        const origOpen = XMLHttpRequest.prototype.open;
                        XMLHttpRequest.prototype.open = function(method, url) {
                            this.__aerolith_url = url;
                            this.__aerolith_method = method;
                            return origOpen.apply(this, arguments);
                        };
                        XMLHttpRequest.prototype.send = function() {
                            window.__aerolith_inflight++;
                            const start = Date.now();
                            const url = String(this.__aerolith_url || '');
                            const method = String(this.__aerolith_method || 'GET').toUpperCase();
                            this.addEventListener('loadend', () => {
                                window.__aerolith_inflight = Math.max(0, window.__aerolith_inflight - 1);
                                try {
                                    window.__aerolith_network.push({
                                        url: url.substring(0, 300),
                                        method: method,
                                        status: this.status,
                                        ok: this.status >= 200 && this.status < 300,
                                        duration_ms: Date.now() - start,
                                        timestamp: Date.now()
                                    });
                                    if (window.__aerolith_network.length > 50) window.__aerolith_network.shift();
                                } catch (e) {}
                            }, { once: true });
                            return origSend.apply(this, arguments);
                        };
                    }
                } catch (e) {}
            }
            if (window.__aerolith_dialogs === undefined) {
                window.__aerolith_dialogs = [];
                window.__aerolith_dialog_policy = {
                    alert: 'accept',
                    confirm: 'accept',
                    prompt: 'accept',
                    prompt_value: ''
                };
                try {
                    window.alert = function(msg) {
                        window.__aerolith_dialogs.push({ type: 'alert', message: String(msg), timestamp: Date.now() });
                        if (window.ipc) {
                            window.ipc.postMessage(JSON.stringify({
                                type: 'agent_dialog',
                                dialog_type: 'alert',
                                message: String(msg)
                            }));
                        }
                    };
                    window.confirm = function(msg) {
                        const action = window.__aerolith_dialog_policy.confirm === 'dismiss' ? false : true;
                        window.__aerolith_dialogs.push({ type: 'confirm', message: String(msg), response: action, timestamp: Date.now() });
                        if (window.ipc) {
                            window.ipc.postMessage(JSON.stringify({
                                type: 'agent_dialog',
                                dialog_type: 'confirm',
                                message: String(msg),
                                response: action
                            }));
                        }
                        return action;
                    };
                    window.prompt = function(msg, defaultVal) {
                        const val = window.__aerolith_dialog_policy.prompt === 'dismiss' ? null : (window.__aerolith_dialog_policy.prompt_value || defaultVal || '');
                        window.__aerolith_dialogs.push({ type: 'prompt', message: String(msg), response: val, timestamp: Date.now() });
                        if (window.ipc) {
                            window.ipc.postMessage(JSON.stringify({
                                type: 'agent_dialog',
                                dialog_type: 'prompt',
                                message: String(msg),
                                response: val
                            }));
                        }
                        return val;
                    };
                } catch (e) {}
            }
            if (window.__aerolith_console === undefined) {
                window.__aerolith_console = [];
                try {
                    const captureLog = (level, args) => {
                        try {
                            const msg = Array.from(args).map(a => typeof a === 'object' ? JSON.stringify(a) : String(a)).join(' ');
                            window.__aerolith_console.push({ level, message: msg.substring(0, 1000), timestamp: Date.now() });
                            if (window.__aerolith_console.length > 100) window.__aerolith_console.shift();
                        } catch (e) {}
                    };
                    const origLog = console.log;
                    console.log = function() { captureLog('log', arguments); if (origLog) origLog.apply(console, arguments); };
                    const origWarn = console.warn;
                    console.warn = function() { captureLog('warn', arguments); if (origWarn) origWarn.apply(console, arguments); };
                    const origErr = console.error;
                    console.error = function() { captureLog('error', arguments); if (origErr) origErr.apply(console, arguments); };
                    window.addEventListener('error', (ev) => {
                        captureLog('error', [ev.message || 'Window error', ev.filename || '', ev.lineno || '']);
                    });
                } catch (e) {}
            }
        })();"#
    }

    /// JavaScript to traverse visible DOM and build a compact, token-efficient Semantic DOM snapshot.
    pub fn get_semantic_dom_script(request_id: u64, viewport_only: bool, filter_role: Option<&str>) -> String {
        let filter_role_json = serde_json::to_string(&filter_role).unwrap_or_else(|_| "null".into());
        format!(
            r#"(function() {{
                try {{
                    window.__aerolith_elements = {{}};
                    let refId = 1;
                    const viewportOnly = {};
                    const filterRole = {};
                    const interactiveTags = new Set(['A', 'BUTTON', 'INPUT', 'SELECT', 'TEXTAREA', 'SUMMARY', 'DETAILS', 'LABEL']);
                    const interactiveRoles = new Set(['button', 'link', 'checkbox', 'menuitem', 'option', 'radio', 'switch', 'tab', 'combobox', 'textbox', 'searchbox']);

                    function isVisible(el) {{
                        if (!el || el.nodeType !== 1) return false;
                        const style = window.getComputedStyle(el);
                        if (style.display === 'none' || style.visibility === 'hidden' || style.opacity === '0') return false;
                        const rect = el.getBoundingClientRect();
                        return rect.width > 0 && rect.height > 0;
                    }}

                    function isInteractive(el) {{
                        const tag = el.tagName;
                        if (interactiveTags.has(tag)) return true;
                        const role = el.getAttribute('role');
                        if (role && interactiveRoles.has(role.toLowerCase())) return true;
                        if (el.hasAttribute('onclick') || el.hasAttribute('contenteditable')) return true;
                        if (el.getAttribute('tabindex') && parseInt(el.getAttribute('tabindex'), 10) >= 0) return true;
                        const style = window.getComputedStyle(el);
                        if (style.cursor === 'pointer') return true;
                        return false;
                    }}

                    const elementsList = [];
                    const lines = [];

                    function walk(node, depth) {{
                        if (!node || node.nodeType !== 1) return;
                        if (!isVisible(node)) return;

                        const tag = node.tagName.toLowerCase();
                        if (['script', 'style', 'noscript', 'svg', 'iframe'].includes(tag)) return;

                        const interactive = isInteractive(node);
                        const isHeading = /^h[1-6]$/.test(tag);
                        const isParagraph = tag === 'p';

                        let currentRef = null;
                        if (interactive) {{
                            const role = (node.getAttribute('role') || tag).toLowerCase();
                            if (filterRole && role !== filterRole.toLowerCase() && tag !== filterRole.toLowerCase()) return;

                            const rect = node.getBoundingClientRect();
                            const inViewport = rect.top < window.innerHeight && rect.bottom > 0 && rect.left < window.innerWidth && rect.right > 0;
                            if (viewportOnly && !inViewport) return;

                            currentRef = refId++;
                            node.setAttribute('data-aerolith-ref', String(currentRef));
                            window.__aerolith_elements[currentRef] = node;

                            const rect = node.getBoundingClientRect();
                            const text = (node.innerText || node.textContent || node.getAttribute('aria-label') || node.getAttribute('title') || node.getAttribute('alt') || '').trim().replace(/\s+/g, ' ');
                            const placeholder = node.getAttribute('placeholder') || '';
                            const value = node.value !== undefined ? String(node.value) : '';
                            const href = node.getAttribute('href') || '';
                            const role = node.getAttribute('role') || tag;
                            const inViewport = rect.top < window.innerHeight && rect.bottom > 0 && rect.left < window.innerWidth && rect.right > 0;
                            const centerX = Math.round(rect.x + rect.width / 2);
                            const centerY = Math.round(rect.y + rect.height / 2);

                            elementsList.push({{
                                ref: currentRef,
                                tag: tag,
                                role: role,
                                text: text.substring(0, 150),
                                placeholder: placeholder.substring(0, 100),
                                value: value.substring(0, 100),
                                href: href.substring(0, 200),
                                bounds: {{
                                    x: Math.round(rect.x),
                                    y: Math.round(rect.y),
                                    width: Math.round(rect.width),
                                    height: Math.round(rect.height),
                                    center_x: centerX,
                                    center_y: centerY,
                                    in_viewport: inViewport
                                }}
                            }});

                            let desc = `[${{currentRef}}] <${{tag}}`;
                            if (role && role !== tag) desc += ` role="${{role}}"`;
                            if (text) desc += ` "${{text.substring(0, 80)}}"`;
                            if (placeholder) desc += ` placeholder="${{placeholder}}"`;
                            if (value) desc += ` value="${{value.substring(0, 50)}}"`;
                            if (href) desc += ` href="${{href.substring(0, 80)}}"`;
                            desc += ` bounds=${{Math.round(rect.x)}},${{Math.round(rect.y)}},${{Math.round(rect.width)}},${{Math.round(rect.height)}} center=${{centerX}},${{centerY}}>`;

                            lines.push('  '.repeat(depth) + desc);
                        }} else if (isHeading || isParagraph) {{
                            const text = (node.innerText || '').trim().replace(/\s+/g, ' ');
                            if (text && text.length > 2) {{
                                lines.push('  '.repeat(depth) + `<${{tag}}> ${{text.substring(0, 150)}}`);
                            }}
                        }}

                        for (let child of node.children) {{
                            walk(child, depth + (interactive ? 1 : 0));
                        }}
                    }}

                    walk(document.body, 0);

                    const payload = {{
                        url: window.location.href,
                        title: document.title,
                        tree_text: lines.join('\n'),
                        elements: elementsList,
                        viewport: {{
                            width: window.innerWidth,
                            height: window.innerHeight,
                            scroll_x: window.scrollX,
                            scroll_y: window.scrollY,
                            dpr: window.devicePixelRatio || 1
                        }}
                    }};

                    if (window.ipc) {{
                        window.ipc.postMessage(JSON.stringify({{
                            type: 'agent_response',
                            request_id: {},
                            action: 'snapshot',
                            payload: payload
                        }}));
                    }}
                }} catch (err) {{
                    if (window.ipc) {{
                        window.ipc.postMessage(JSON.stringify({{
                            type: 'agent_response',
                            request_id: {},
                            action: 'snapshot',
                            error: String(err)
                        }}));
                    }}
                }}
            }})();"#,
            viewport_only, filter_role_json, request_id, request_id
        )
    }

    /// JavaScript to inject Set-Of-Marks (SOM) visual bounding labels over interactive elements.
    pub fn get_som_inject_script() -> &'static str {
        r#"(function() {
            try {
                let overlay = document.getElementById('__aerolith_som_overlay');
                if (overlay) overlay.remove();

                overlay = document.createElement('div');
                overlay.id = '__aerolith_som_overlay';
                overlay.style.cssText = 'position:fixed;top:0;left:0;width:100vw;height:100vh;pointer-events:none !important;z-index:2147483647 !important;overflow:hidden;user-select:none !important;';

                const colors = ['#e91e63', '#9c27b0', '#00bcd4', '#2e7d32', '#ef6c00', '#304ffe', '#c2185b', '#00838f'];

                if (window.__aerolith_elements) {
                    for (const [ref, el] of Object.entries(window.__aerolith_elements)) {
                        if (!el || !el.getBoundingClientRect) continue;
                        const rect = el.getBoundingClientRect();
                        if (rect.width <= 0 || rect.height <= 0) continue;
                        if (rect.bottom < 0 || rect.top > window.innerHeight || rect.right < 0 || rect.left > window.innerWidth) continue;

                        const num = parseInt(ref, 10);
                        const color = colors[num % colors.length];

                        // Outline box with subtle 10% translucent tint
                        const box = document.createElement('div');
                        box.style.cssText = `position:absolute;left:${rect.left}px;top:${rect.top}px;width:${rect.width}px;height:${rect.height}px;border:2px solid ${color};background-color:${color}18;box-sizing:border-box;pointer-events:none !important;z-index:2147483647;`;
                        overlay.appendChild(box);

                        // Numeric label badge with adaptive compact font and positioning
                        const isCompact = rect.width < 32 || rect.height < 22;
                        const fontSize = isCompact ? '9px' : '11px';
                        const padding = isCompact ? '0px 2px' : '1px 4px';
                        const lineHeight = isCompact ? '12px' : '14px';

                        let badgeTop = rect.top - 18;
                        if (badgeTop < 0) {
                            badgeTop = rect.top + 2;
                        }
                        const badgeLeft = Math.max(0, rect.left);

                        const badge = document.createElement('div');
                        badge.style.cssText = `position:absolute;left:${badgeLeft}px;top:${badgeTop}px;background:${color};color:#ffffff;font-size:${fontSize};font-weight:bold;font-family:ui-monospace,SFMono-Regular,Consolas,monospace;padding:${padding};border-radius:3px;box-shadow:0 1px 4px rgba(0,0,0,0.7),inset 0 0 0 1px rgba(255,255,255,0.3);text-shadow:0 1px 2px rgba(0,0,0,0.8);pointer-events:none !important;line-height:${lineHeight};z-index:2147483647;white-space:nowrap;user-select:none !important;`;
                        badge.textContent = `${ref}`;
                        overlay.appendChild(badge);
                    }
                }
                document.body.appendChild(overlay);
            } catch (e) {}
        })();"#
    }

    /// JavaScript to remove Set-Of-Marks overlay.
    #[allow(dead_code)]
    pub fn get_som_cleanup_script() -> &'static str {
        r#"(function() {
            const overlay = document.getElementById('__aerolith_som_overlay');
            if (overlay) overlay.remove();
        })();"#
    }

    /// JavaScript to export cookies, localStorage, and sessionStorage from the active webview.
    pub fn get_storage_export_script(request_id: u64) -> String {
        format!(
            r#"(function() {{
                try {{
                    const localData = {{}};
                    try {{
                        for (let i = 0; i < localStorage.length; i++) {{
                            const k = localStorage.key(i);
                            if (k !== null) localData[k] = localStorage.getItem(k);
                        }}
                    }} catch (e) {{}}

                    const sessionData = {{}};
                    try {{
                        for (let i = 0; i < sessionStorage.length; i++) {{
                            const k = sessionStorage.key(i);
                            if (k !== null) sessionData[k] = sessionStorage.getItem(k);
                        }}
                    }} catch (e) {{}}

                    const payload = {{
                        url: window.location.href,
                        cookies: document.cookie || "",
                        local_storage: localData,
                        session_storage: sessionData
                    }};

                    if (window.ipc) {{
                        window.ipc.postMessage(JSON.stringify({{
                            type: 'agent_response',
                            request_id: {},
                            action: 'storage_state',
                            payload: payload
                        }}));
                    }}
                }} catch (err) {{
                    if (window.ipc) {{
                        window.ipc.postMessage(JSON.stringify({{
                            type: 'agent_response',
                            request_id: {},
                            action: 'storage_state',
                            error: String(err)
                        }}));
                    }}
                }}
            }})();"#,
            request_id, request_id
        )
    }

    /// JavaScript to import cookies, localStorage, and sessionStorage into the active webview.
    pub fn get_storage_import_script(request_id: u64, state: &serde_json::Value) -> String {
        let state_json = serde_json::to_string(state).unwrap_or_else(|_| "{}".to_string());
        format!(
            r#"(function() {{
                try {{
                    const state = {};
                    let cookiesRestored = 0;
                    if (state.cookies && typeof state.cookies === 'string') {{
                        const pairs = state.cookies.split(';');
                        for (let p of pairs) {{
                            const trimmed = p.trim();
                            if (trimmed) {{
                                document.cookie = trimmed + '; path=/;';
                                cookiesRestored++;
                            }}
                        }}
                    }}

                    let localCount = 0;
                    if (state.local_storage && typeof state.local_storage === 'object') {{
                        for (let [k, v] of Object.entries(state.local_storage)) {{
                            try {{
                                localStorage.setItem(k, typeof v === 'string' ? v : JSON.stringify(v));
                                localCount++;
                            }} catch (e) {{}}
                        }}
                    }}

                    let sessionCount = 0;
                    if (state.session_storage && typeof state.session_storage === 'object') {{
                        for (let [k, v] of Object.entries(state.session_storage)) {{
                            try {{
                                sessionStorage.setItem(k, typeof v === 'string' ? v : JSON.stringify(v));
                                sessionCount++;
                            }} catch (e) {{}}
                        }}
                    }}

                    if (window.ipc) {{
                        window.ipc.postMessage(JSON.stringify({{
                            type: 'agent_response',
                            request_id: {},
                            action: 'storage_state',
                            payload: {{
                                success: true,
                                cookies_restored: cookiesRestored,
                                local_storage_keys: localCount,
                                session_storage_keys: sessionCount
                            }}
                        }}));
                    }}
                }} catch (err) {{
                    if (window.ipc) {{
                        window.ipc.postMessage(JSON.stringify({{
                            type: 'agent_response',
                            request_id: {},
                            action: 'storage_state',
                            error: String(err)
                        }}));
                    }}
                }}
            }})();"#,
            state_json, request_id, request_id
        )
    }

    /// JavaScript to extract clean article Markdown from the active page.
    pub fn get_markdown_script(request_id: u64) -> String {
        format!(
            r#"(function() {{
                try {{
                    function htmlToMarkdown(root) {{
                        const lines = [];

                        function processNode(node) {{
                            if (!node) return;
                            if (node.nodeType === 3) {{
                                const text = node.textContent.trim();
                                if (text) lines.push(text);
                                return;
                            }}
                            if (node.nodeType !== 1) return;

                            const tag = node.tagName.toLowerCase();
                            if (['script', 'style', 'noscript', 'svg', 'nav', 'footer', 'aside', 'iframe'].includes(tag)) return;

                            if (/^h[1-6]$/.test(tag)) {{
                                const level = parseInt(tag[1], 10);
                                const text = (node.innerText || '').trim();
                                if (text) lines.push('\n' + '#'.repeat(level) + ' ' + text + '\n');
                                return;
                            }}

                            if (tag === 'p') {{
                                const text = (node.innerText || '').trim();
                                if (text) lines.push('\n' + text + '\n');
                                return;
                            }}

                            if (tag === 'pre' || tag === 'code') {{
                                const text = (node.innerText || '').trim();
                                if (text) lines.push('\n```\n' + text + '\n```\n');
                                return;
                            }}

                            if (tag === 'blockquote') {{
                                const text = (node.innerText || '').trim();
                                if (text) lines.push('\n> ' + text.replace(/\n/g, '\n> ') + '\n');
                                return;
                            }}

                            if (tag === 'ul' || tag === 'ol') {{
                                let idx = 1;
                                for (let li of node.children) {{
                                    if (li.tagName.toLowerCase() === 'li') {{
                                        const prefix = tag === 'ul' ? '- ' : `${{idx++}}. `;
                                        lines.push(prefix + (li.innerText || '').trim());
                                    }}
                                }}
                                lines.push('');
                                return;
                            }}

                            if (tag === 'table') {{
                                const rows = Array.from(node.querySelectorAll('tr'));
                                if (rows.length > 0) {{
                                    lines.push('');
                                    rows.forEach((tr, rIdx) => {{
                                        const cells = Array.from(tr.querySelectorAll('th, td')).map(c => c.innerText.trim().replace(/\|/g, '\\|'));
                                        lines.push('| ' + cells.join(' | ') + ' |');
                                        if (rIdx === 0) {{
                                            lines.push('| ' + cells.map(() => '---').join(' | ') + ' |');
                                        }}
                                    }});
                                    lines.push('');
                                }}
                                return;
                            }}

                            if (tag === 'a') {{
                                const text = (node.innerText || '').trim();
                                const href = node.getAttribute('href');
                                if (text && href) {{
                                    lines.push(`[${{text}}](${{href}})`);
                                    return;
                                }}
                            }}

                            for (let child of node.childNodes) {{
                                processNode(child);
                            }}
                        }}

                        const target = root.querySelector('main, article, [role="main"], #content, .content') || root.body;
                        processNode(target);
                        return lines.join('\n').replace(/\n{{3,}}/g, '\n\n').trim();
                    }}

                    const md = htmlToMarkdown(document);
                    if (window.ipc) {{
                        window.ipc.postMessage(JSON.stringify({{
                            type: 'agent_response',
                            request_id: {},
                            action: 'markdown',
                            payload: {{
                                url: window.location.href,
                                title: document.title,
                                markdown: md
                            }}
                        }}));
                    }}
                }} catch (err) {{
                    if (window.ipc) {{
                        window.ipc.postMessage(JSON.stringify({{
                            type: 'agent_response',
                            request_id: {},
                            action: 'markdown',
                            error: String(err)
                        }}));
                    }}
                }}
            }})();"#,
            request_id, request_id
        )
    }

    /// JavaScript to click an element by ref number or CSS selector with automatic retry.
    pub fn get_click_script(request_id: u64, target: &str) -> String {
        let is_numeric = target.chars().all(|c| c.is_ascii_digit());
        let target_expr = if is_numeric {
            format!("window.__aerolith_elements && window.__aerolith_elements[{}] || document.querySelector('[data-aerolith-ref=\"{}\"]')", target, target)
        } else {
            format!("document.querySelector({})", serde_json::to_string(target).unwrap_or_default())
        };

        format!(
            r#"(function() {{
                let attempts = 0;
                function executeClick() {{
                    try {{
                        const el = {};
                        if (!el) {{
                            if (++attempts < 10) {{
                                return setTimeout(executeClick, 50);
                            }}
                            throw new Error("Element not found for target: {}");
                        }}
                        el.scrollIntoView({{ behavior: 'instant', block: 'center' }});

                        const rect = el.getBoundingClientRect();
                        const cx = rect.x + rect.width / 2;
                        const cy = rect.y + rect.height / 2;
                        let occluded = false;
                        const topEl = document.elementFromPoint(cx, cy);
                        if (topEl && topEl !== el && !el.contains(topEl) && !topEl.contains(el)) {{
                            occluded = true;
                            window.scrollBy(0, -75);
                        }}

                        const opts = {{ bubbles: true, cancelable: true, view: window, clientX: cx, clientY: cy }};
                        const initialUrl = window.location.href;
                        let mutationCount = 0;
                        let observer = null;
                        try {{
                            observer = new MutationObserver((mutations) => {{
                                mutationCount += mutations.length;
                            }});
                            observer.observe(document.body, {{ childList: true, subtree: true, attributes: true }});
                        }} catch (e) {{}}

                        el.dispatchEvent(new MouseEvent('mouseover', opts));
                        el.dispatchEvent(new MouseEvent('mouseenter', opts));
                        el.dispatchEvent(new MouseEvent('mousedown', opts));
                        if (typeof el.focus === 'function') el.focus();
                        el.dispatchEvent(new MouseEvent('mouseup', opts));
                        el.dispatchEvent(new MouseEvent('click', opts));
                        if (typeof el.click === 'function') {{
                            try {{ el.click(); }} catch (e) {{}}
                        }}

                        setTimeout(() => {{
                            if (observer) observer.disconnect();
                            const urlChanged = window.location.href !== initialUrl;
                            if (window.ipc) {{
                                window.ipc.postMessage(JSON.stringify({{
                                    type: 'agent_response',
                                    request_id: {},
                                    action: 'click',
                                    payload: {{
                                        success: true,
                                        tag: el.tagName.toLowerCase(),
                                        text: (el.innerText || el.value || '').substring(0, 80),
                                        url_changed: urlChanged,
                                        mutations_observed: mutationCount,
                                        occluded: occluded
                                    }}
                                }}));
                            }}
                        }}, 60);
                    }} catch (err) {{
                        if (window.ipc) {{
                            window.ipc.postMessage(JSON.stringify({{
                                type: 'agent_response',
                                request_id: {},
                                action: 'click',
                                error: String(err)
                            }}));
                        }}
                    }}
                }}
                executeClick();
            }})();"#,
            target_expr, target, request_id, request_id
        )
    }

    /// JavaScript to type text into an input element with automatic retry.
    pub fn get_type_script(request_id: u64, target: &str, text: &str, clear: bool, submit: bool) -> String {
        let is_numeric = target.chars().all(|c| c.is_ascii_digit());
        let target_expr = if is_numeric {
            format!("window.__aerolith_elements && window.__aerolith_elements[{}] || document.querySelector('[data-aerolith-ref=\"{}\"]')", target, target)
        } else {
            format!("document.querySelector({})", serde_json::to_string(target).unwrap_or_default())
        };

        let json_text = serde_json::to_string(text).unwrap_or_else(|_| "\"\"".into());

        format!(
            r#"(function() {{
                let attempts = 0;
                function executeType() {{
                    try {{
                        const el = {};
                        if (!el) {{
                            if (++attempts < 10) {{
                                return setTimeout(executeType, 50);
                            }}
                            throw new Error("Element not found for target: {}");
                        }}
                        el.scrollIntoView({{ behavior: 'instant', block: 'center' }});
                        if (typeof el.focus === 'function') el.focus();

                        const initialUrl = window.location.href;
                        if ({}) {{
                            el.value = '';
                            el.dispatchEvent(new Event('input', {{ bubbles: true }}));
                        }}

                        const textToType = {};
                        if (el.value !== undefined) {{
                            el.value += textToType;
                        }} else if (el.isContentEditable) {{
                            el.textContent += textToType;
                        }}
                        el.dispatchEvent(new Event('input', {{ bubbles: true }}));
                        el.dispatchEvent(new Event('change', {{ bubbles: true }}));

                        const submitRequested = {};
                        if (submitRequested) {{
                            const enterOpts = {{ key: 'Enter', code: 'Enter', keyCode: 13, which: 13, bubbles: true, cancelable: true }};
                            el.dispatchEvent(new KeyboardEvent('keydown', enterOpts));
                            el.dispatchEvent(new KeyboardEvent('keypress', enterOpts));
                            el.dispatchEvent(new KeyboardEvent('keyup', enterOpts));
                            if (el.form && typeof el.form.requestSubmit === 'function') {{
                                el.form.requestSubmit();
                            }}
                        }}

                        setTimeout(() => {{
                            const urlChanged = window.location.href !== initialUrl;
                            if (window.ipc) {{
                                window.ipc.postMessage(JSON.stringify({{
                                    type: 'agent_response',
                                    request_id: {},
                                    action: 'type',
                                    payload: {{
                                        success: true,
                                        value: el.value !== undefined ? el.value : el.textContent,
                                        submitted: submitRequested,
                                        url_changed: urlChanged
                                    }}
                                }}));
                            }}
                        }}, 60);
                    }} catch (err) {{
                        if (window.ipc) {{
                            window.ipc.postMessage(JSON.stringify({{
                                type: 'agent_response',
                                request_id: {},
                                action: 'type',
                                error: String(err)
                            }}));
                        }}
                    }}
                }}
                executeType();
            }})();"#,
            target_expr, target, clear, json_text, submit, request_id, request_id
        )
    }

    /// JavaScript to wait for an element or network idle state with polling and timeout.
    pub fn get_wait_for_script(request_id: u64, target: &str, timeout_ms: u64) -> String {
        let is_numeric = target.chars().all(|c| c.is_ascii_digit());
        let target_expr = if is_numeric {
            format!("window.__aerolith_elements && window.__aerolith_elements[{}] || document.querySelector('[data-aerolith-ref=\"{}\"]')", target, target)
        } else {
            format!("document.querySelector({})", serde_json::to_string(target).unwrap_or_default())
        };

        format!(
            r#"(function() {{
                const startTime = Date.now();
                const timeout = {};

                function check() {{
                    try {{
                        const el = {};
                        if (el) {{
                            const rect = el.getBoundingClientRect();
                            if (rect.width > 0 && rect.height > 0) {{
                                if (window.ipc) {{
                                    window.ipc.postMessage(JSON.stringify({{
                                        type: 'agent_response',
                                        request_id: {},
                                        action: 'wait_for',
                                        payload: {{
                                            success: true,
                                            found: true,
                                            elapsed_ms: Date.now() - startTime,
                                            inflight_requests: window.__aerolith_inflight || 0
                                        }}
                                    }}));
                                }}
                                return;
                            }}
                        }}

                        if (Date.now() - startTime >= timeout) {{
                            if (window.ipc) {{
                                window.ipc.postMessage(JSON.stringify({{
                                    type: 'agent_response',
                                    request_id: {},
                                    action: 'wait_for',
                                    error: 'Timeout waiting for element: ' + {}
                                }}));
                            }}
                            return;
                        }}

                        setTimeout(check, 50);
                    }} catch (err) {{
                        if (window.ipc) {{
                            window.ipc.postMessage(JSON.stringify({{
                                type: 'agent_response',
                                request_id: {},
                                action: 'wait_for',
                                error: String(err)
                            }}));
                        }}
                    }}
                }}

                check();
            }})();"#,
            timeout_ms, target_expr, request_id, request_id, serde_json::to_string(target).unwrap_or_default(), request_id
        )
    }

    /// JavaScript to scroll the page.
    pub fn get_scroll_script(request_id: u64, direction: &str, amount: i32) -> String {
        let (dx, dy) = match direction.to_lowercase().as_str() {
            "up" => (0, -amount),
            "down" => (0, amount),
            "left" => (-amount, 0),
            "right" => (amount, 0),
            "top" => (0, -1000000),
            "bottom" => (0, 1000000),
            _ => (0, amount),
        };

        format!(
            r#"(function() {{
                try {{
                    window.scrollBy({{ left: {}, top: {}, behavior: 'instant' }});
                    if (window.ipc) {{
                        window.ipc.postMessage(JSON.stringify({{
                            type: 'agent_response',
                            request_id: {},
                            action: 'scroll',
                            payload: {{
                                success: true,
                                scroll_x: window.scrollX,
                                scroll_y: window.scrollY
                            }}
                        }}));
                    }}
                }} catch (err) {{
                    if (window.ipc) {{
                        window.ipc.postMessage(JSON.stringify({{
                            type: 'agent_response',
                            request_id: {},
                            action: 'scroll',
                            error: String(err)
                        }}));
                    }}
                }}
            }})();"#,
            dx, dy, request_id, request_id
        )
    }

    /// JavaScript to evaluate arbitrary JS and return result.
    pub fn get_eval_script(request_id: u64, expression: &str) -> String {
        let json_expr = serde_json::to_string(expression).unwrap_or_else(|_| "\"\"".into());
        format!(
            r#"(function() {{
                try {{
                    const result = eval({});
                    let safeResult = result;
                    if (result instanceof Node) {{
                        safeResult = `<${{result.tagName ? result.tagName.toLowerCase() : 'node'}}>`;
                    }} else if (typeof result === 'function') {{
                        safeResult = result.toString();
                    }}
                    if (window.ipc) {{
                        window.ipc.postMessage(JSON.stringify({{
                            type: 'agent_response',
                            request_id: {},
                            action: 'eval',
                            payload: {{
                                result: safeResult
                            }}
                        }}));
                    }}
                }} catch (err) {{
                    if (window.ipc) {{
                        window.ipc.postMessage(JSON.stringify({{
                            type: 'agent_response',
                            request_id: {},
                            action: 'eval',
                            error: String(err)
                        }}));
                    }}
                }}
            }})();"#,
            json_expr, request_id, request_id
        )
    }

    /// JavaScript to find elements by text, placeholder, aria-label, value, or role.
    pub fn get_find_script(request_id: u64, query: &str) -> String {
        let json_query = serde_json::to_string(query).unwrap_or_else(|_| "\"\"".into());
        format!(
            r#"(function() {{
                try {{
                    if (!window.__aerolith_elements) {{
                        window.__aerolith_elements = {{}};
                        window.__aerolith_next_ref = 1;
                    }}
                    const query = ({}).toLowerCase().trim();
                    const results = [];
                    const interactiveTags = new Set(['A', 'BUTTON', 'INPUT', 'SELECT', 'TEXTAREA', 'SUMMARY', 'DETAILS', 'LABEL']);

                    function isVisible(el) {{
                        if (!el || el.nodeType !== 1) return false;
                        const style = window.getComputedStyle(el);
                        if (style.display === 'none' || style.visibility === 'hidden' || style.opacity === '0') return false;
                        const rect = el.getBoundingClientRect();
                        return rect.width > 0 && rect.height > 0;
                    }}

                    const allElements = document.querySelectorAll('*');
                    for (const el of allElements) {{
                        if (!isVisible(el)) continue;
                        const tag = el.tagName.toUpperCase();
                        const isInteractive = interactiveTags.has(tag) || el.hasAttribute('onclick') || el.getAttribute('role') || el.isContentEditable;
                        if (!isInteractive) continue;

                        const text = (el.innerText || el.textContent || '').trim().toLowerCase();
                        const aria = (el.getAttribute('aria-label') || '').toLowerCase();
                        const placeholder = (el.getAttribute('placeholder') || '').toLowerCase();
                        const value = el.value !== undefined ? String(el.value).toLowerCase() : '';
                        const role = (el.getAttribute('role') || '').toLowerCase();

                        let matchedOn = null;
                        if (text.includes(query)) matchedOn = 'text';
                        else if (aria.includes(query)) matchedOn = 'aria-label';
                        else if (placeholder.includes(query)) matchedOn = 'placeholder';
                        else if (value.includes(query)) matchedOn = 'value';
                        else if (role.includes(query)) matchedOn = 'role';

                        if (matchedOn) {{
                            let ref = el.getAttribute('data-aerolith-ref');
                            if (!ref) {{
                                ref = String(window.__aerolith_next_ref++);
                                el.setAttribute('data-aerolith-ref', ref);
                            }}
                            window.__aerolith_elements[ref] = el;
                            const rect = el.getBoundingClientRect();
                            results.push({{
                                ref: parseInt(ref, 10),
                                tag: el.tagName.toLowerCase(),
                                role: role || el.tagName.toLowerCase(),
                                text: (el.innerText || el.textContent || '').trim().substring(0, 100),
                                placeholder: el.getAttribute('placeholder') || '',
                                value: el.value !== undefined ? String(el.value) : '',
                                href: el.getAttribute('href') || '',
                                matched_on: matchedOn,
                                bounds: {{
                                    x: Math.round(rect.x),
                                    y: Math.round(rect.y),
                                    width: Math.round(rect.width),
                                    height: Math.round(rect.height),
                                    center_x: Math.round(rect.x + rect.width / 2),
                                    center_y: Math.round(rect.y + rect.height / 2),
                                    in_viewport: rect.top < window.innerHeight && rect.bottom > 0 && rect.left < window.innerWidth && rect.right > 0
                                }}
                            }});
                        }}
                    }}

                    if (window.ipc) {{
                        window.ipc.postMessage(JSON.stringify({{
                            type: 'agent_response',
                            request_id: {},
                            action: 'find',
                            payload: {{
                                query: {},
                                count: results.length,
                                matches: results
                            }}
                        }}));
                    }}
                }} catch (err) {{
                    if (window.ipc) {{
                        window.ipc.postMessage(JSON.stringify({{
                            type: 'agent_response',
                            request_id: {},
                            action: 'find',
                            error: String(err)
                        }}));
                    }}
                }}
            }})();"#,
            json_query, request_id, json_query, request_id
        )
    }

    /// JavaScript to update dialog handling policy or query recorded dialogs.
    pub fn get_dialog_policy_script(request_id: u64, alert: Option<&str>, confirm: Option<&str>, prompt_val: Option<&str>) -> String {
        let alert_json = serde_json::to_string(&alert).unwrap_or_else(|_| "null".into());
        let confirm_json = serde_json::to_string(&confirm).unwrap_or_else(|_| "null".into());
        let prompt_json = serde_json::to_string(&prompt_val).unwrap_or_else(|_| "null".into());

        format!(
            r#"(function() {{
                try {{
                    if (!window.__aerolith_dialog_policy) {{
                        window.__aerolith_dialog_policy = {{ alert: 'accept', confirm: 'accept', prompt: 'accept', prompt_value: '' }};
                    }}
                    if (!window.__aerolith_dialogs) {{
                        window.__aerolith_dialogs = [];
                    }}
                    const a = {};
                    const c = {};
                    const p = {};
                    if (a) window.__aerolith_dialog_policy.alert = a;
                    if (c) window.__aerolith_dialog_policy.confirm = c;
                    if (p !== null) {{
                        window.__aerolith_dialog_policy.prompt = 'accept';
                        window.__aerolith_dialog_policy.prompt_value = p;
                    }}

                    if (window.ipc) {{
                        window.ipc.postMessage(JSON.stringify({{
                            type: 'agent_response',
                            request_id: {},
                            action: 'dialog_policy',
                            payload: {{
                                policy: window.__aerolith_dialog_policy,
                                dialogs: window.__aerolith_dialogs
                            }}
                        }}));
                    }}
                }} catch (err) {{
                    if (window.ipc) {{
                        window.ipc.postMessage(JSON.stringify({{
                            type: 'agent_response',
                            request_id: {},
                            action: 'dialog_policy',
                            error: String(err)
                        }}));
                    }}
                }}
            }})();"#,
            alert_json, confirm_json, prompt_json, request_id, request_id
        )
    }

    /// JavaScript to visually highlight an element with an animated focus ring.
    pub fn get_highlight_script(request_id: u64, target: &str) -> String {
        let is_numeric = target.chars().all(|c| c.is_ascii_digit());
        let target_expr = if is_numeric {
            format!("window.__aerolith_elements && window.__aerolith_elements[{}] || document.querySelector('[data-aerolith-ref=\"{}\"]')", target, target)
        } else {
            format!("document.querySelector({})", serde_json::to_string(target).unwrap_or_default())
        };

        format!(
            r#"(function() {{
                try {{
                    const el = {};
                    if (!el) throw new Error("Element not found for target: {}");
                    el.scrollIntoView({{ behavior: 'instant', block: 'center' }});

                    let ring = document.getElementById('__aerolith_highlight_ring');
                    if (ring) ring.remove();

                    const rect = el.getBoundingClientRect();
                    ring = document.createElement('div');
                    ring.id = '__aerolith_highlight_ring';
                    ring.style.cssText = `position:fixed;left:${{rect.left - 4}}px;top:${{rect.top - 4}}px;width:${{rect.width + 8}}px;height:${{rect.height + 8}}px;border:3px solid #00e676;border-radius:4px;box-shadow:0 0 16px #00e676;pointer-events:none;z-index:2147483647;transition:opacity 0.5s ease;`;
                    document.body.appendChild(ring);

                    setTimeout(() => {{
                        if (ring) {{
                            ring.style.opacity = '0';
                            setTimeout(() => ring.remove(), 500);
                        }}
                    }}, 2500);

                    if (window.ipc) {{
                        window.ipc.postMessage(JSON.stringify({{
                            type: 'agent_response',
                            request_id: {},
                            action: 'highlight',
                            payload: {{
                                success: true,
                                tag: el.tagName.toLowerCase(),
                                bounds: {{
                                    x: Math.round(rect.x),
                                    y: Math.round(rect.y),
                                    width: Math.round(rect.width),
                                    height: Math.round(rect.height)
                                }}
                            }}
                        }}));
                    }}
                }} catch (err) {{
                    if (window.ipc) {{
                        window.ipc.postMessage(JSON.stringify({{
                            type: 'agent_response',
                            request_id: {},
                            action: 'highlight',
                            error: String(err)
                        }}));
                    }}
                }}
            }})();"#,
            target_expr, target, request_id, request_id
        )
    }

    /// JavaScript to atomically fill multiple form fields in one turn and optionally submit.
    pub fn get_fill_form_script(request_id: u64, fields: &[crate::agent::protocol::FormField], submit: bool) -> String {
        let fields_json = serde_json::to_string(fields).unwrap_or_else(|_| "[]".into());
        format!(
            r#"(function() {{
                try {{
                    const fields = {};
                    let filledCount = 0;
                    let lastForm = null;
                    for (const f of fields) {{
                        const isNum = /^\d+$/.test(f.target);
                        const el = isNum
                            ? (window.__aerolith_elements && window.__aerolith_elements[f.target] || document.querySelector('[data-aerolith-ref="' + f.target + '"]'))
                            : document.querySelector(f.target);
                        if (el) {{
                            if (el.form) lastForm = el.form;
                            el.scrollIntoView({{ behavior: 'instant', block: 'center' }});
                            if (typeof el.focus === 'function') el.focus();
                            if (el.value !== undefined) {{
                                el.value = f.value;
                            }} else if (el.isContentEditable) {{
                                el.textContent = f.value;
                            }}
                            el.dispatchEvent(new Event('input', {{ bubbles: true }}));
                            el.dispatchEvent(new Event('change', {{ bubbles: true }}));
                            filledCount++;
                        }}
                    }}

                    if ({}) {{
                        if (lastForm && typeof lastForm.requestSubmit === 'function') {{
                            lastForm.requestSubmit();
                        }}
                    }}

                    if (window.ipc) {{
                        window.ipc.postMessage(JSON.stringify({{
                            type: 'agent_response',
                            request_id: {},
                            action: 'fill_form',
                            payload: {{
                                success: true,
                                filled_count: filledCount,
                                total_fields: fields.length,
                                submitted: {}
                            }}
                        }}));
                    }}
                }} catch (err) {{
                    if (window.ipc) {{
                        window.ipc.postMessage(JSON.stringify({{
                            type: 'agent_response',
                            request_id: {},
                            action: 'fill_form',
                            error: String(err)
                        }}));
                    }}
                }}
            }})();"#,
            fields_json, submit, request_id, submit, request_id
        )
    }

    /// JavaScript to read or write the browser clipboard.
    pub fn get_clipboard_script(request_id: u64, action: &str, text: Option<&str>) -> String {
        let text_json = serde_json::to_string(&text.unwrap_or("")).unwrap_or_else(|_| "\"\"".into());
        let action_clean = action.to_lowercase();
        let is_write = action_clean == "set" || action_clean == "write";

        format!(
            r#"(async function() {{
                try {{
                    const isWrite = {};
                    let resultText = '';
                    if (isWrite) {{
                        const toWrite = {};
                        if (navigator.clipboard && navigator.clipboard.writeText) {{
                            await navigator.clipboard.writeText(toWrite);
                        }} else {{
                            const ta = document.createElement('textarea');
                            ta.value = toWrite;
                            ta.style.position = 'fixed';
                            ta.style.opacity = '0';
                            document.body.appendChild(ta);
                            ta.select();
                            document.execCommand('copy');
                            ta.remove();
                        }}
                        resultText = toWrite;
                    }} else {{
                        if (navigator.clipboard && navigator.clipboard.readText) {{
                            resultText = await navigator.clipboard.readText();
                        }} else {{
                            resultText = window.getSelection ? window.getSelection().toString() : '';
                        }}
                    }}

                    if (window.ipc) {{
                        window.ipc.postMessage(JSON.stringify({{
                            type: 'agent_response',
                            request_id: {},
                            action: 'clipboard',
                            payload: {{
                                success: true,
                                action: isWrite ? 'write' : 'read',
                                text: resultText
                            }}
                        }}));
                    }}
                }} catch (err) {{
                    if (window.ipc) {{
                        window.ipc.postMessage(JSON.stringify({{
                            type: 'agent_response',
                            request_id: {},
                            action: 'clipboard',
                            error: String(err)
                        }}));
                    }}
                }}
            }})();"#,
            is_write, text_json, request_id, request_id
        )
    }

    /// JavaScript to extract text or an attribute from a specific element.
    pub fn get_text_script(request_id: u64, target: &str, attribute: Option<&str>) -> String {
        let is_numeric = target.chars().all(|c| c.is_ascii_digit());
        let target_expr = if is_numeric {
            format!("window.__aerolith_elements && window.__aerolith_elements[{}] || document.querySelector('[data-aerolith-ref=\"{}\"]')", target, target)
        } else {
            format!("document.querySelector({})", serde_json::to_string(target).unwrap_or_default())
        };

        let attr_json = serde_json::to_string(&attribute).unwrap_or_else(|_| "null".into());

        format!(
            r#"(function() {{
                try {{
                    const el = {};
                    if (!el) throw new Error("Element not found for target: {}");
                    const attr = {};
                    let content = '';
                    if (attr) {{
                        content = el.getAttribute(attr) || '';
                    }} else if (el.value !== undefined) {{
                        content = el.value;
                    }} else {{
                        content = el.innerText || el.textContent || '';
                    }}

                    if (window.ipc) {{
                        window.ipc.postMessage(JSON.stringify({{
                            type: 'agent_response',
                            request_id: {},
                            action: 'get_text',
                            payload: {{
                                success: true,
                                target: {},
                                tag: el.tagName.toLowerCase(),
                                text: content.trim()
                            }}
                        }}));
                    }}
                }} catch (err) {{
                    if (window.ipc) {{
                        window.ipc.postMessage(JSON.stringify({{
                            type: 'agent_response',
                            request_id: {},
                            action: 'get_text',
                            error: String(err)
                        }}));
                    }}
                }}
            }})();"#,
            target_expr, target, attr_json, request_id, serde_json::to_string(target).unwrap_or_default(), request_id
        )
    }

    /// JavaScript to retrieve captured console logs and page errors.
    pub fn get_console_logs_script(request_id: u64, clear: bool) -> String {
        format!(
            r#"(function() {{
                try {{
                    const logs = window.__aerolith_console || [];
                    const resultLogs = logs.slice();
                    if ({}) {{
                        window.__aerolith_console = [];
                    }}
                    if (window.ipc) {{
                        window.ipc.postMessage(JSON.stringify({{
                            type: 'agent_response',
                            request_id: {},
                            action: 'console_logs',
                            payload: {{
                                success: true,
                                count: resultLogs.length,
                                logs: resultLogs
                            }}
                        }}));
                    }}
                }} catch (err) {{
                    if (window.ipc) {{
                        window.ipc.postMessage(JSON.stringify({{
                            type: 'agent_response',
                            request_id: {},
                            action: 'console_logs',
                            error: String(err)
                        }}));
                    }}
                }}
            }})();"#,
            clear, request_id, request_id
        )
    }

    /// JavaScript to hover over an element and trigger hover/mouseenter states.
    pub fn get_hover_script(request_id: u64, target: &str) -> String {
        let is_numeric = target.chars().all(|c| c.is_ascii_digit());
        let target_expr = if is_numeric {
            format!("window.__aerolith_elements && window.__aerolith_elements[{}] || document.querySelector('[data-aerolith-ref=\"{}\"]')", target, target)
        } else {
            format!("document.querySelector({})", serde_json::to_string(target).unwrap_or_default())
        };

        format!(
            r#"(function() {{
                try {{
                    const el = {};
                    if (!el) throw new Error("Element not found for target: {}");
                    el.scrollIntoView({{ behavior: 'instant', block: 'center' }});

                    const rect = el.getBoundingClientRect();
                    const cx = rect.x + rect.width / 2;
                    const cy = rect.y + rect.height / 2;
                    const topEl = document.elementFromPoint(cx, cy);
                    const occluded = topEl && topEl !== el && !el.contains(topEl) && !topEl.contains(el);

                    let mutationCount = 0;
                    let observer = null;
                    try {{
                        observer = new MutationObserver((mutations) => {{
                            mutationCount += mutations.length;
                        }});
                        observer.observe(document.body, {{ childList: true, subtree: true, attributes: true }});
                    }} catch (e) {{}}

                    const opts = {{ bubbles: true, cancelable: true, view: window, clientX: cx, clientY: cy }};
                    el.dispatchEvent(new MouseEvent('mousemove', opts));
                    el.dispatchEvent(new MouseEvent('mouseover', opts));
                    el.dispatchEvent(new MouseEvent('mouseenter', opts));

                    setTimeout(() => {{
                        if (observer) observer.disconnect();
                        if (window.ipc) {{
                            window.ipc.postMessage(JSON.stringify({{
                                type: 'agent_response',
                                request_id: {},
                                action: 'hover',
                                payload: {{
                                    success: true,
                                    target: {},
                                    occluded: Boolean(occluded),
                                    bounds: {{
                                        x: Math.round(rect.x),
                                        y: Math.round(rect.y),
                                        width: Math.round(rect.width),
                                        height: Math.round(rect.height)
                                    }},
                                    mutations_observed: mutationCount
                                }}
                            }}));
                        }}
                    }}, 40);
                }} catch (err) {{
                    if (window.ipc) {{
                        window.ipc.postMessage(JSON.stringify({{
                            type: 'agent_response',
                            request_id: {},
                            action: 'hover',
                            error: String(err)
                        }}));
                    }}
                }}
            }})();"#,
            target_expr, target, request_id, serde_json::to_string(target).unwrap_or_default(), request_id
        )
    }

    /// JavaScript to dispatch realistic keyboard events (keydown, keypress, keyup).
    pub fn get_press_key_script(request_id: u64, key: &str, target: Option<&str>) -> String {
        let key_json = serde_json::to_string(key).unwrap_or_else(|_| "\"\"".into());
        let target_clause = match target {
            Some(t) => {
                let is_numeric = t.chars().all(|c| c.is_ascii_digit());
                if is_numeric {
                    format!("window.__aerolith_elements && window.__aerolith_elements[{}] || document.querySelector('[data-aerolith-ref=\"{}\"]')", t, t)
                } else {
                    format!("document.querySelector({})", serde_json::to_string(t).unwrap_or_default())
                }
            }
            None => "document.activeElement || document.body".to_string(),
        };

        format!(
            r#"(function() {{
                try {{
                    const keyName = {};
                    let el = {};
                    if (!el) el = document.activeElement || document.body;
                    if (typeof el.focus === 'function') el.focus();

                    const keyMap = {{
                        'Escape': {{ code: 'Escape', keyCode: 27, which: 27 }},
                        'Enter': {{ code: 'Enter', keyCode: 13, which: 13 }},
                        'Tab': {{ code: 'Tab', keyCode: 9, which: 9 }},
                        'Backspace': {{ code: 'Backspace', keyCode: 8, which: 8 }},
                        'Space': {{ code: 'Space', keyCode: 32, which: 32 }},
                        'ArrowUp': {{ code: 'ArrowUp', keyCode: 38, which: 38 }},
                        'ArrowDown': {{ code: 'ArrowDown', keyCode: 40, which: 40 }},
                        'ArrowLeft': {{ code: 'ArrowLeft', keyCode: 37, which: 37 }},
                        'ArrowRight': {{ code: 'ArrowRight', keyCode: 39, which: 39 }},
                        'PageUp': {{ code: 'PageUp', keyCode: 33, which: 33 }},
                        'PageDown': {{ code: 'PageDown', keyCode: 34, which: 34 }},
                        'Home': {{ code: 'Home', keyCode: 36, which: 36 }},
                        'End': {{ code: 'End', keyCode: 35, which: 35 }},
                        'Delete': {{ code: 'Delete', keyCode: 46, which: 46 }}
                    }};

                    const info = keyMap[keyName] || {{
                        code: keyName.length === 1 ? 'Key' + keyName.toUpperCase() : keyName,
                        keyCode: keyName.charCodeAt(0),
                        which: keyName.charCodeAt(0)
                    }};

                    const evOpts = {{
                        key: keyName,
                        code: info.code,
                        keyCode: info.keyCode,
                        which: info.which,
                        bubbles: true,
                        cancelable: true,
                        view: window
                    }};

                    el.dispatchEvent(new KeyboardEvent('keydown', evOpts));
                    if (keyName.length === 1 || keyName === 'Enter') {{
                        el.dispatchEvent(new KeyboardEvent('keypress', evOpts));
                    }}
                    el.dispatchEvent(new KeyboardEvent('keyup', evOpts));

                    if (window.ipc) {{
                        window.ipc.postMessage(JSON.stringify({{
                            type: 'agent_response',
                            request_id: {},
                            action: 'key',
                            payload: {{
                                success: true,
                                key: keyName,
                                target_tag: el.tagName ? el.tagName.toLowerCase() : 'body'
                            }}
                        }}));
                    }}
                }} catch (err) {{
                    if (window.ipc) {{
                        window.ipc.postMessage(JSON.stringify({{
                            type: 'agent_response',
                            request_id: {},
                            action: 'key',
                            error: String(err)
                        }}));
                    }}
                }}
            }})();"#,
            key_json, target_clause, request_id, request_id
        )
    }

    /// JavaScript to select an option in a HTML <select> element and dispatch change events.
    pub fn get_select_option_script(request_id: u64, target: &str, value: &str) -> String {
        let is_numeric = target.chars().all(|c| c.is_ascii_digit());
        let target_expr = if is_numeric {
            format!("window.__aerolith_elements && window.__aerolith_elements[{}] || document.querySelector('[data-aerolith-ref=\"{}\"]')", target, target)
        } else {
            format!("document.querySelector({})", serde_json::to_string(target).unwrap_or_default())
        };
        let value_json = serde_json::to_string(value).unwrap_or_else(|_| "\"\"".into());

        format!(
            r#"(function() {{
                try {{
                    const el = {};
                    if (!el) throw new Error("Element not found for target: {}");
                    if (el.tagName.toUpperCase() !== 'SELECT') {{
                        throw new Error("Target element is not a <select> element: <" + el.tagName.toLowerCase() + ">");
                    }}
                    el.scrollIntoView({{ behavior: 'instant', block: 'center' }});

                    const val = {};
                    const options = Array.from(el.options);
                    let matchedOpt = options.find(o => o.value === val);
                    if (!matchedOpt) {{
                        matchedOpt = options.find(o => (o.text || '').trim().toLowerCase() === val.toLowerCase());
                    }}
                    if (!matchedOpt) {{
                        const idx = parseInt(val, 10);
                        if (!isNaN(idx) && idx >= 0 && idx < options.length) {{
                            matchedOpt = options[idx];
                        }}
                    }}

                    if (!matchedOpt) {{
                        const available = options.map(o => o.value || o.text).slice(0, 10).join(', ');
                        throw new Error("Option not found for value '" + val + "'. Available: [" + available + "]");
                    }}

                    const prevValue = el.value;
                    el.value = matchedOpt.value;
                    matchedOpt.selected = true;

                    el.dispatchEvent(new Event('input', {{ bubbles: true, cancelable: true }}));
                    el.dispatchEvent(new Event('change', {{ bubbles: true, cancelable: true }}));

                    if (window.ipc) {{
                        window.ipc.postMessage(JSON.stringify({{
                            type: 'agent_response',
                            request_id: {},
                            action: 'select_option',
                            payload: {{
                                success: true,
                                target: {},
                                value: matchedOpt.value,
                                text: matchedOpt.text,
                                previous_value: prevValue
                            }}
                        }}));
                    }}
                }} catch (err) {{
                    if (window.ipc) {{
                        window.ipc.postMessage(JSON.stringify({{
                            type: 'agent_response',
                            request_id: {},
                            action: 'select_option',
                            error: String(err)
                        }}));
                    }}
                }}
            }})();"#,
            target_expr, target, value_json, request_id, serde_json::to_string(target).unwrap_or_default(), request_id
        )
    }

    /// JavaScript to retrieve recorded network requests.
    pub fn get_network_activity_script(request_id: u64, clear: bool) -> String {
        format!(
            r#"(function() {{
                try {{
                    const requests = window.__aerolith_network ? Array.from(window.__aerolith_network) : [];
                    const inflight = window.__aerolith_inflight || 0;
                    if ({}) {{
                        window.__aerolith_network = [];
                    }}
                    if (window.ipc) {{
                        window.ipc.postMessage(JSON.stringify({{
                            type: 'agent_response',
                            request_id: {},
                            action: 'network_activity',
                            payload: {{
                                requests: requests,
                                count: requests.length,
                                inflight_count: inflight
                            }}
                        }}));
                    }}
                }} catch (err) {{
                    if (window.ipc) {{
                        window.ipc.postMessage(JSON.stringify({{
                            type: 'agent_response',
                            request_id: {},
                            action: 'network_activity',
                            error: String(err)
                        }}));
                    }}
                }}
            }})();"#,
            clear, request_id, request_id
        )
    }

    /// JavaScript to perform a drag-and-drop or slider drag gesture with intermediate steps.
    pub fn get_drag_script(
        request_id: u64,
        source: &str,
        target: Option<&str>,
        offset_x: Option<i32>,
        offset_y: Option<i32>,
        steps: u32,
    ) -> String {
        let is_source_num = source.chars().all(|c| c.is_ascii_digit());
        let source_expr = if is_source_num {
            format!("window.__aerolith_elements && window.__aerolith_elements[{}] || document.querySelector('[data-aerolith-ref=\"{}\"]')", source, source)
        } else {
            format!("document.querySelector({})", serde_json::to_string(source).unwrap_or_default())
        };

        let target_clause = match target {
            Some(t) => {
                let is_target_num = t.chars().all(|c| c.is_ascii_digit());
                if is_target_num {
                    format!("window.__aerolith_elements && window.__aerolith_elements[{}] || document.querySelector('[data-aerolith-ref=\"{}\"]')", t, t)
                } else {
                    format!("document.querySelector({})", serde_json::to_string(t).unwrap_or_default())
                }
            }
            None => "null".to_string(),
        };

        let ox = offset_x.unwrap_or(0);
        let oy = offset_y.unwrap_or(0);
        let num_steps = steps.max(1);

        format!(
            r#"(function() {{
                try {{
                    const src = {};
                    if (!src) throw new Error("Source element not found: " + {});
                    src.scrollIntoView({{ behavior: 'instant', block: 'center' }});

                    const srcRect = src.getBoundingClientRect();
                    const startX = Math.round(srcRect.x + srcRect.width / 2);
                    const startY = Math.round(srcRect.y + srcRect.height / 2);

                    const tgt = {};
                    let endX = startX + {};
                    let endY = startY + {};

                    if (tgt) {{
                        const tgtRect = tgt.getBoundingClientRect();
                        endX = Math.round(tgtRect.x + tgtRect.width / 2) + {};
                        endY = Math.round(tgtRect.y + tgtRect.height / 2) + {};
                    }}

                    let mutationCount = 0;
                    let observer = null;
                    try {{
                        observer = new MutationObserver((mutations) => {{
                            mutationCount += mutations.length;
                        }});
                        observer.observe(document.body, {{ childList: true, subtree: true, attributes: true }});
                    }} catch (e) {{}}

                    if (src.tagName.toUpperCase() === 'INPUT' && src.type === 'range') {{
                        const min = parseFloat(src.min) || 0;
                        const max = parseFloat(src.max) || 100;
                        const ratio = Math.max(0, Math.min(1, (endX - srcRect.left) / srcRect.width));
                        const newVal = min + ratio * (max - min);
                        src.value = newVal;
                        src.dispatchEvent(new Event('input', {{ bubbles: true, cancelable: true }}));
                        src.dispatchEvent(new Event('change', {{ bubbles: true, cancelable: true }}));
                    }}

                    src.dispatchEvent(new MouseEvent('mousedown', {{
                        bubbles: true, cancelable: true, view: window,
                        clientX: startX, clientY: startY, buttons: 1
                    }}));

                    const steps = {};
                    for (let i = 1; i <= steps; i++) {{
                        const curX = Math.round(startX + (endX - startX) * (i / steps));
                        const curY = Math.round(startY + (endY - startY) * (i / steps));
                        const moveTarget = document.elementFromPoint(curX, curY) || src;
                        moveTarget.dispatchEvent(new MouseEvent('mousemove', {{
                            bubbles: true, cancelable: true, view: window,
                            clientX: curX, clientY: curY, buttons: 1
                        }}));
                    }}

                    const dropTarget = document.elementFromPoint(endX, endY) || tgt || src;
                    dropTarget.dispatchEvent(new MouseEvent('mouseup', {{
                        bubbles: true, cancelable: true, view: window,
                        clientX: endX, clientY: endY, buttons: 0
                    }}));

                    setTimeout(() => {{
                        if (observer) observer.disconnect();
                        if (window.ipc) {{
                            window.ipc.postMessage(JSON.stringify({{
                                type: 'agent_response',
                                request_id: {},
                                action: 'drag',
                                payload: {{
                                    success: true,
                                    source: {},
                                    start: {{ x: startX, y: startY }},
                                    end: {{ x: endX, y: endY }},
                                    steps: steps,
                                    mutations_observed: mutationCount
                                }}
                            }}));
                        }}
                    }}, 40);
                }} catch (err) {{
                    if (window.ipc) {{
                        window.ipc.postMessage(JSON.stringify({{
                            type: 'agent_response',
                            request_id: {},
                            action: 'drag',
                            error: String(err)
                        }}));
                    }}
                }}
            }})();"#,
            source_expr, serde_json::to_string(source).unwrap_or_default(),
            target_clause, ox, oy, ox, oy,
            num_steps,
            request_id, serde_json::to_string(source).unwrap_or_default(),
            request_id
        )
    }

    /// JavaScript to assign files to an <input type="file"> element via synthetic DataTransfer.
    pub fn get_upload_file_script(request_id: u64, target: &str, files: &[String]) -> String {
        let is_numeric = target.chars().all(|c| c.is_ascii_digit());
        let target_expr = if is_numeric {
            format!("window.__aerolith_elements && window.__aerolith_elements[{}] || document.querySelector('[data-aerolith-ref=\"{}\"]')", target, target)
        } else {
            format!("document.querySelector({})", serde_json::to_string(target).unwrap_or_default())
        };
        let files_json = serde_json::to_string(files).unwrap_or_else(|_| "[]".into());

        format!(
            r#"(function() {{
                try {{
                    const el = {};
                    if (!el) throw new Error("Element not found for target: " + {});
                    if (el.tagName.toUpperCase() !== 'INPUT' || (el.type || '').toLowerCase() !== 'file') {{
                        throw new Error("Target element is not an <input type=\"file\">: <" + el.tagName.toLowerCase() + ">");
                    }}
                    el.scrollIntoView({{ behavior: 'instant', block: 'center' }});

                    const fileNames = {};
                    const dt = new DataTransfer();
                    for (const name of fileNames) {{
                        const cleanName = String(name).split(/[\/\\]/).pop() || 'upload.bin';
                        const file = new File([''], cleanName, {{ type: 'application/octet-stream' }});
                        dt.items.add(file);
                    }}

                    el.files = dt.files;
                    el.dispatchEvent(new Event('input', {{ bubbles: true, cancelable: true }}));
                    el.dispatchEvent(new Event('change', {{ bubbles: true, cancelable: true }}));

                    if (window.ipc) {{
                        window.ipc.postMessage(JSON.stringify({{
                            type: 'agent_response',
                            request_id: {},
                            action: 'upload_file',
                            payload: {{
                                success: true,
                                target: {},
                                file_count: dt.files.length,
                                files: Array.from(dt.files).map(f => f.name)
                            }}
                        }}));
                    }}
                }} catch (err) {{
                    if (window.ipc) {{
                        window.ipc.postMessage(JSON.stringify({{
                            type: 'agent_response',
                            request_id: {},
                            action: 'upload_file',
                            error: String(err)
                        }}));
                    }}
                }}
            }})();"#,
            target_expr, serde_json::to_string(target).unwrap_or_default(),
            files_json, request_id, serde_json::to_string(target).unwrap_or_default(),
            request_id
        )
    }

    /// JavaScript to override navigator.geolocation in the page context.
    pub fn get_location_script(request_id: u64, lat: f64, lng: f64, accuracy: f64) -> String {
        format!(
            r#"(function() {{
                try {{
                    const mockCoords = {{
                        latitude: {},
                        longitude: {},
                        accuracy: {},
                        altitude: null,
                        altitudeAccuracy: null,
                        heading: null,
                        speed: null
                    }};
                    const mockPosition = {{
                        coords: mockCoords,
                        timestamp: Date.now()
                    }};

                    if (navigator.geolocation) {{
                        navigator.geolocation.getCurrentPosition = function(success, error, opts) {{
                            if (typeof success === 'function') {{
                                setTimeout(() => success(mockPosition), 0);
                            }}
                        }};
                        navigator.geolocation.watchPosition = function(success, error, opts) {{
                            if (typeof success === 'function') {{
                                setTimeout(() => success(mockPosition), 0);
                            }}
                            return Math.floor(Math.random() * 100000);
                        }};
                    }}

                    if (window.ipc) {{
                        window.ipc.postMessage(JSON.stringify({{
                            type: 'agent_response',
                            request_id: {},
                            action: 'location',
                            payload: {{
                                success: true,
                                latitude: {},
                                longitude: {},
                                accuracy: {}
                            }}
                        }}));
                    }}
                }} catch (err) {{
                    if (window.ipc) {{
                        window.ipc.postMessage(JSON.stringify({{
                            type: 'agent_response',
                            request_id: {},
                            action: 'location',
                            error: String(err)
                        }}));
                    }}
                }}
            }})();"#,
            lat, lng, accuracy, request_id, lat, lng, accuracy, request_id
        )
    }

    /// JavaScript to inspect and control page media (audio/video).
    pub fn get_media_control_script(request_id: u64, action: &str) -> String {
        let action_clean = action.to_lowercase();
        let act_json = serde_json::to_string(&action_clean).unwrap_or_else(|_| "\"status\"".into());

        format!(
            r#"(function() {{
                try {{
                    const act = {};
                    const mediaEls = Array.from(document.querySelectorAll('video, audio'));

                    for (const m of mediaEls) {{
                        try {{
                            if (act === 'pause') {{
                                m.pause();
                            }} else if (act === 'play') {{
                                m.play().catch(() => {{}});
                            }} else if (act === 'mute') {{
                                m.muted = true;
                            }} else if (act === 'unmute') {{
                                m.muted = false;
                            }}
                        }} catch (e) {{}}
                    }}

                    const mediaStatus = mediaEls.map(m => ({{
                        tag: m.tagName.toLowerCase(),
                        src: m.src || m.currentSrc || '',
                        paused: Boolean(m.paused),
                        muted: Boolean(m.muted),
                        current_time: Math.round(m.currentTime || 0),
                        duration: Math.round(m.duration || 0)
                    }}));

                    if (window.ipc) {{
                        window.ipc.postMessage(JSON.stringify({{
                            type: 'agent_response',
                            request_id: {},
                            action: 'media_control',
                            payload: {{
                                success: true,
                                action: act,
                                media_count: mediaEls.length,
                                media: mediaStatus
                            }}
                        }}));
                    }}
                }} catch (err) {{
                    if (window.ipc) {{
                        window.ipc.postMessage(JSON.stringify({{
                            type: 'agent_response',
                            request_id: {},
                            action: 'media_control',
                            error: String(err)
                        }}));
                    }}
                }}
            }})();"#,
            act_json, request_id, request_id
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_script_generation() {
        let dom_script = AgentScripts::get_semantic_dom_script(42, true, Some("button"));
        assert!(dom_script.contains("request_id: 42"));
        assert!(dom_script.contains("data-aerolith-ref"));
        assert!(dom_script.contains("tree_text"));
        assert!(dom_script.contains("viewportOnly"));
        assert!(dom_script.contains("button"));

        let som_script = AgentScripts::get_som_inject_script();
        assert!(som_script.contains("__aerolith_som_overlay"));

        let md_script = AgentScripts::get_markdown_script(43);
        assert!(md_script.contains("request_id: 43"));
        assert!(md_script.contains("htmlToMarkdown"));

        let click_script = AgentScripts::get_click_script(44, "7");
        assert!(click_script.contains("window.__aerolith_elements[7]"));

        let type_script = AgentScripts::get_type_script(45, "12", "hello world", true, true);
        assert!(type_script.contains("hello world"));
        assert!(type_script.contains("Enter"));

        let scroll_script = AgentScripts::get_scroll_script(46, "down", 500);
        assert!(scroll_script.contains("top: 500"));

        let wait_script = AgentScripts::get_wait_for_script(47, "button#submit", 3000);
        assert!(wait_script.contains("wait_for"));
        assert!(wait_script.contains("button#submit"));
        assert!(wait_script.contains("timeout = 3000"));

        let export_script = AgentScripts::get_storage_export_script(48);
        assert!(export_script.contains("storage_state"));
        assert!(export_script.contains("local_storage"));
        assert!(export_script.contains("request_id: 48"));

        let import_script = AgentScripts::get_storage_import_script(49, &serde_json::json!({
            "cookies": "sid=123",
            "local_storage": { "theme": "dark" }
        }));
        assert!(import_script.contains("storage_state"));
        assert!(import_script.contains("cookiesRestored"));
        assert!(import_script.contains("request_id: 49"));

        let find_script = AgentScripts::get_find_script(50, "Submit");
        assert!(find_script.contains("request_id: 50"));
        assert!(find_script.contains("action: 'find'"));
        assert!(find_script.contains("Submit"));

        let dialog_script = AgentScripts::get_dialog_policy_script(51, Some("accept"), Some("dismiss"), Some("user_input"));
        assert!(dialog_script.contains("request_id: 51"));
        assert!(dialog_script.contains("__aerolith_dialog_policy"));
        assert!(dialog_script.contains("dismiss"));

        let hl_script = AgentScripts::get_highlight_script(52, "4");
        assert!(hl_script.contains("request_id: 52"));
        assert!(hl_script.contains("__aerolith_highlight_ring"));
        assert!(hl_script.contains("action: 'highlight'"));

        assert!(click_script.contains("MutationObserver"));
        assert!(click_script.contains("mutations_observed"));
        assert!(type_script.contains("url_changed"));

        let fill_script = AgentScripts::get_fill_form_script(53, &[
            crate::agent::protocol::FormField { target: "1".into(), value: "hello".into() },
        ], true);
        assert!(fill_script.contains("request_id: 53"));
        assert!(fill_script.contains("action: 'fill_form'"));
        assert!(fill_script.contains("filled_count"));

        let clip_script = AgentScripts::get_clipboard_script(54, "set", Some("foo_bar"));
        assert!(clip_script.contains("request_id: 54"));
        assert!(clip_script.contains("action: 'clipboard'"));
        assert!(clip_script.contains("foo_bar"));

        let text_script = AgentScripts::get_text_script(55, "button#save", Some("data-id"));
        assert!(text_script.contains("request_id: 55"));
        assert!(text_script.contains("action: 'get_text'"));
        assert!(text_script.contains("button#save"));
        assert!(text_script.contains("data-id"));

        assert!(click_script.contains("occluded"));
        assert!(click_script.contains("elementFromPoint"));

        let console_script = AgentScripts::get_console_logs_script(56, true);
        assert!(console_script.contains("request_id: 56"));
        assert!(console_script.contains("action: 'console_logs'"));
        assert!(console_script.contains("__aerolith_console"));
        assert!(console_script.contains("window.__aerolith_console = []"));

        let hover_script = AgentScripts::get_hover_script(57, "nav.dropdown");
        assert!(hover_script.contains("request_id: 57"));
        assert!(hover_script.contains("action: 'hover'"));
        assert!(hover_script.contains("nav.dropdown"));
        assert!(hover_script.contains("mouseenter"));

        let key_script = AgentScripts::get_press_key_script(58, "Escape", Some("#overlay"));
        assert!(key_script.contains("request_id: 58"));
        assert!(key_script.contains("action: 'key'"));
        assert!(key_script.contains("Escape"));
        assert!(key_script.contains("#overlay"));

        let select_script = AgentScripts::get_select_option_script(59, "select#year", "2026");
        assert!(select_script.contains("request_id: 59"));
        assert!(select_script.contains("action: 'select_option'"));
        assert!(select_script.contains("select#year"));
        assert!(select_script.contains("2026"));

        let net_script = AgentScripts::get_network_activity_script(60, true);
        assert!(net_script.contains("request_id: 60"));
        assert!(net_script.contains("action: 'network_activity'"));
        assert!(net_script.contains("__aerolith_network"));

        let drag_script = AgentScripts::get_drag_script(61, "slider#vol", None, Some(100), Some(0), 10);
        assert!(drag_script.contains("request_id: 61"));
        assert!(drag_script.contains("action: 'drag'"));
        assert!(drag_script.contains("slider#vol"));
        assert!(drag_script.contains("steps = 10"));

        let up_script = AgentScripts::get_upload_file_script(62, "input#file", &["image.png".into()]);
        assert!(up_script.contains("request_id: 62"));
        assert!(up_script.contains("action: 'upload_file'"));
        assert!(up_script.contains("image.png"));
        assert!(up_script.contains("DataTransfer"));

        let loc_script = AgentScripts::get_location_script(63, 51.5074, -0.1278, 5.0);
        assert!(loc_script.contains("request_id: 63"));
        assert!(loc_script.contains("action: 'location'"));
        assert!(loc_script.contains("51.5074"));
        assert!(loc_script.contains("-0.1278"));

        let media_script = AgentScripts::get_media_control_script(64, "mute");
        assert!(media_script.contains("request_id: 64"));
        assert!(media_script.contains("action: 'media_control'"));
        assert!(media_script.contains("mute"));
    }
}
