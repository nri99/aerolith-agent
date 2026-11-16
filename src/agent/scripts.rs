pub struct AgentScripts;

impl AgentScripts {
    /// JavaScript that initializes the persistent agent registry on the page.
    pub fn get_init_script() -> &'static str {
        r#"(function() {
            if (!window.__aerolith_elements) {
                window.__aerolith_elements = {};
                window.__aerolith_next_ref = 1;
            }
            if (window.__aerolith_routes === undefined) {
                window.__aerolith_routes = [];
            }
            if (window.__aerolith_inflight === undefined) {
                window.__aerolith_inflight = 0;
                window.__aerolith_network = [];
                try {
                    const origFetch = window.fetch;
                    if (origFetch) {
                        window.fetch = function() {
                            const rawUrl = typeof arguments[0] === 'string' ? arguments[0] : (arguments[0] && arguments[0].url || '');
                            const method = (arguments[1] && arguments[1].method || 'GET').toUpperCase();

                            if (window.__aerolith_routes && window.__aerolith_routes.length > 0) {
                                for (const r of window.__aerolith_routes) {
                                    const pat = r.pattern || '';
                                    const isMatch = pat === '*' || rawUrl.includes(pat) || (new RegExp(pat.replace(/\*/g, '.*')).test(rawUrl));
                                    if (isMatch) {
                                        if (r.rule_type === 'abort' || r.action === 'abort') {
                                            return Promise.reject(new TypeError('Failed to fetch (Aborted by Aerolith route rule: ' + pat + ')'));
                                        } else if (r.rule_type === 'mock' || r.action === 'mock') {
                                            const mockStatus = r.status || 200;
                                            const mockBody = r.body || '';
                                            const mockHeaders = r.headers || { 'Content-Type': 'application/json' };
                                            return Promise.resolve(new Response(mockBody, {
                                                status: mockStatus,
                                                headers: mockHeaders
                                            }));
                                        }
                                    }
                                }
                            }

                            window.__aerolith_inflight++;
                            const start = Date.now();
                            return origFetch.apply(this, arguments).then((res) => {
                                try {
                                    const entry = {
                                        url: String(rawUrl).substring(0, 300),
                                        method: method,
                                        status: res.status,
                                        ok: res.ok,
                                        duration_ms: Date.now() - start,
                                        timestamp: Date.now()
                                    };
                                    try {
                                        res.clone().text().then(txt => {
                                            if (txt) {
                                                entry.body = txt.length > 32768 ? txt.substring(0, 32768) + '...[truncated]' : txt;
                                            }
                                        }).catch(() => {});
                                    } catch (_) {}
                                    window.__aerolith_network.push(entry);
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
                            const url = String(this.__aerolith_url || '');
                            const method = String(this.__aerolith_method || 'GET').toUpperCase();

                            if (window.__aerolith_routes && window.__aerolith_routes.length > 0) {
                                for (const r of window.__aerolith_routes) {
                                    const pat = r.pattern || '';
                                    const isMatch = pat === '*' || url.includes(pat) || (new RegExp(pat.replace(/\*/g, '.*')).test(url));
                                    if (isMatch) {
                                        if (r.rule_type === 'abort' || r.action === 'abort') {
                                            setTimeout(() => {
                                                this.dispatchEvent(new Event('error'));
                                                this.dispatchEvent(new Event('abort'));
                                            }, 0);
                                            return;
                                        } else if (r.rule_type === 'mock' || r.action === 'mock') {
                                            setTimeout(() => {
                                                try {
                                                    Object.defineProperty(this, 'status', { value: r.status || 200, writable: true });
                                                    Object.defineProperty(this, 'responseText', { value: r.body || '', writable: true });
                                                    Object.defineProperty(this, 'readyState', { value: 4, writable: true });
                                                    this.dispatchEvent(new Event('readystatechange'));
                                                    this.dispatchEvent(new Event('load'));
                                                    this.dispatchEvent(new Event('loadend'));
                                                } catch (e) {}
                                            }, 0);
                                            return;
                                        }
                                    }
                                }
                            }

                            window.__aerolith_inflight++;
                            const start = Date.now();
                            this.addEventListener('loadend', () => {
                                window.__aerolith_inflight = Math.max(0, window.__aerolith_inflight - 1);
                                try {
                                    let respBody = null;
                                    try {
                                        if (this.responseText) {
                                            respBody = this.responseText.length > 32768 ? this.responseText.substring(0, 32768) + '...[truncated]' : this.responseText;
                                        }
                                    } catch (_) {}
                                    window.__aerolith_network.push({
                                        url: url.substring(0, 300),
                                        method: method,
                                        status: this.status,
                                        ok: this.status >= 200 && this.status < 300,
                                        body: respBody,
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
            if (window.__aerolith_downloads === undefined) {
                window.__aerolith_downloads = [];
                try {
                    window.addEventListener('click', (ev) => {
                        let el = ev.target;
                        while (el && el !== document) {
                            if (el.tagName === 'A' && (el.hasAttribute('download') || (el.href && (el.href.startsWith('blob:') || el.href.startsWith('data:'))))) {
                                try {
                                    window.__aerolith_downloads.push({
                                        url: el.href ? el.href.substring(0, 500) : '',
                                        filename: el.getAttribute('download') || '',
                                        text: (el.innerText || '').trim().substring(0, 100),
                                        timestamp: Date.now()
                                    });
                                    if (window.__aerolith_downloads.length > 50) window.__aerolith_downloads.shift();
                                } catch (e) {}
                                break;
                            }
                            el = el.parentElement;
                        }
                    }, true);
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

    /// JavaScript to wait for an element, networkidle, URL pattern, or JS condition with polling and timeout.
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
                const target = {};
                let idleConsecutive = 0;

                function check() {{
                    try {{
                        if (target === 'networkidle' || target.startsWith('networkidle')) {{
                            const inflight = window.__aerolith_inflight || 0;
                            if (inflight === 0) {{
                                idleConsecutive += 50;
                                if (idleConsecutive >= 250) {{
                                    if (window.ipc) {{
                                        window.ipc.postMessage(JSON.stringify({{
                                            type: 'agent_response',
                                            request_id: {},
                                            action: 'wait_for',
                                            payload: {{
                                                success: true,
                                                condition: 'networkidle',
                                                elapsed_ms: Date.now() - startTime,
                                                inflight_requests: 0
                                            }}
                                        }}));
                                    }}
                                    return;
                                }}
                            }} else {{
                                idleConsecutive = 0;
                            }}
                        }} else if (target.startsWith('url:')) {{
                            const expected = target.substring(4);
                            if (window.location.href.includes(expected)) {{
                                if (window.ipc) {{
                                    window.ipc.postMessage(JSON.stringify({{
                                        type: 'agent_response',
                                        request_id: {},
                                        action: 'wait_for',
                                        payload: {{
                                            success: true,
                                            condition: 'url',
                                            url: window.location.href,
                                            elapsed_ms: Date.now() - startTime
                                        }}
                                    }}));
                                }}
                                return;
                            }}
                        }} else if (target.startsWith('js:')) {{
                            const code = target.substring(3);
                            let res = null;
                            try {{
                                res = eval(code);
                            }} catch (e) {{}}
                            if (res) {{
                                if (window.ipc) {{
                                    window.ipc.postMessage(JSON.stringify({{
                                        type: 'agent_response',
                                        request_id: {},
                                        action: 'wait_for',
                                        payload: {{
                                            success: true,
                                            condition: 'js',
                                            result: res,
                                            elapsed_ms: Date.now() - startTime
                                        }}
                                    }}));
                                }}
                                return;
                            }}
                        }} else {{
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
                        }}

                        if (Date.now() - startTime >= timeout) {{
                            if (window.ipc) {{
                                window.ipc.postMessage(JSON.stringify({{
                                    type: 'agent_response',
                                    request_id: {},
                                    action: 'wait_for',
                                    error: 'Timeout waiting for: ' + target
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
            timeout_ms,
            serde_json::to_string(target).unwrap_or_default(),
            request_id,
            request_id,
            request_id,
            target_expr,
            request_id,
            request_id,
            request_id
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

    /// JavaScript to inspect all properties, computed styles, attributes, and bounds of a target element.
    pub fn get_inspect_script(request_id: u64, target: &str) -> String {
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
                    if (!el) {{
                        if (window.ipc) {{
                            window.ipc.postMessage(JSON.stringify({{
                                type: 'agent_response',
                                request_id: {},
                                action: 'inspect',
                                error: 'Element not found for target: ' + {}
                            }}));
                        }}
                        return;
                    }}

                    const rect = el.getBoundingClientRect();
                    const style = window.getComputedStyle(el);
                    const attrs = {{}};
                    if (el.attributes) {{
                        for (let i = 0; i < el.attributes.length; i++) {{
                            const a = el.attributes[i];
                            attrs[a.name] = a.value;
                        }}
                    }}

                    const vpWidth = window.innerWidth || document.documentElement.clientWidth || 1920;
                    const vpHeight = window.innerHeight || document.documentElement.clientHeight || 1080;
                    const inViewport = (
                        rect.top >= 0 &&
                        rect.left >= 0 &&
                        rect.bottom <= vpHeight &&
                        rect.right <= vpWidth
                    );

                    const info = {{
                        target: {},
                        tag_name: (el.tagName || '').toLowerCase(),
                        text: (el.innerText || el.textContent || '').trim().substring(0, 1000),
                        value: el.value !== undefined ? String(el.value) : null,
                        checked: el.checked !== undefined ? Boolean(el.checked) : null,
                        disabled: Boolean(el.disabled),
                        readonly: Boolean(el.readOnly),
                        child_count: el.children ? el.children.length : 0,
                        attributes: attrs,
                        computed_style: {{
                            display: style.display,
                            visibility: style.visibility,
                            opacity: style.opacity,
                            pointer_events: style.pointerEvents,
                            z_index: style.zIndex,
                            position: style.position,
                            cursor: style.cursor
                        }},
                        rect: {{
                            x: Math.round(rect.x),
                            y: Math.round(rect.y),
                            width: Math.round(rect.width),
                            height: Math.round(rect.height),
                            top: Math.round(rect.top),
                            left: Math.round(rect.left)
                        }},
                        state: {{
                            is_visible: style.display !== 'none' && style.visibility !== 'hidden' && parseFloat(style.opacity || '1') > 0 && rect.width > 0 && rect.height > 0,
                            is_in_viewport: inViewport,
                            is_focused: document.activeElement === el,
                            is_clickable: style.pointerEvents !== 'none' && !el.disabled
                        }}
                    }};

                    if (window.ipc) {{
                        window.ipc.postMessage(JSON.stringify({{
                            type: 'agent_response',
                            request_id: {},
                            action: 'inspect',
                            payload: info
                        }}));
                    }}
                }} catch (err) {{
                    if (window.ipc) {{
                        window.ipc.postMessage(JSON.stringify({{
                            type: 'agent_response',
                            request_id: {},
                            action: 'inspect',
                            error: String(err)
                        }}));
                    }}
                }}
            }})();"#,
            target_expr, request_id, serde_json::to_string(target).unwrap_or_default(), serde_json::to_string(target).unwrap_or_default(), request_id, request_id
        )
    }

    /// JavaScript to inspect, add, or clear dynamic network route interception rules.
    pub fn get_route_rule_script(
        request_id: u64,
        action: &str,
        pattern: Option<&str>,
        rule_type: Option<&str>,
        status: Option<u16>,
        body: Option<&str>,
        headers: Option<&serde_json::Value>,
    ) -> String {
        let pattern_json = serde_json::to_string(&pattern.unwrap_or("")).unwrap_or_default();
        let rule_type_json = serde_json::to_string(&rule_type.unwrap_or("abort")).unwrap_or_default();
        let status_val = status.unwrap_or(200);
        let body_json = serde_json::to_string(&body.unwrap_or("")).unwrap_or_default();
        let headers_json = serde_json::to_string(&headers.unwrap_or(&serde_json::json!({}))).unwrap_or_default();

        format!(
            r#"(function() {{
                try {{
                    if (!window.__aerolith_routes) {{
                        window.__aerolith_routes = [];
                    }}

                    const act = {};
                    const pat = {};
                    const rtype = {};
                    const st = {};
                    const bd = {};
                    const hd = {};

                    if (act === 'clear') {{
                        window.__aerolith_routes = [];
                    }} else if (act === 'remove') {{
                        if (pat) {{
                            window.__aerolith_routes = window.__aerolith_routes.filter(r => r.pattern !== pat);
                        }}
                    }} else if (act === 'add' || act === 'mock' || act === 'abort') {{
                        const effectiveType = (act === 'mock' || act === 'abort') ? act : rtype;
                        if (pat) {{
                            window.__aerolith_routes = window.__aerolith_routes.filter(r => r.pattern !== pat);
                            window.__aerolith_routes.push({{
                                pattern: pat,
                                rule_type: effectiveType,
                                status: st,
                                body: bd,
                                headers: hd,
                                created_at: Date.now()
                            }});
                        }}
                    }}

                    if (window.ipc) {{
                        window.ipc.postMessage(JSON.stringify({{
                            type: 'agent_response',
                            request_id: {},
                            action: 'route_rule',
                            payload: {{
                                success: true,
                                action: act,
                                count: window.__aerolith_routes.length,
                                routes: window.__aerolith_routes
                            }}
                        }}));
                    }}
                }} catch (err) {{
                    if (window.ipc) {{
                        window.ipc.postMessage(JSON.stringify({{
                            type: 'agent_response',
                            request_id: {},
                            action: 'route_rule',
                            error: String(err)
                        }}));
                    }}
                }}
            }})();"#,
            serde_json::to_string(action).unwrap_or_default(),
            pattern_json,
            rule_type_json,
            status_val,
            body_json,
            headers_json,
            request_id,
            request_id
        )
    }

    /// JavaScript to simulate a human-like double click with sticky-header avoidance and DOM mutation diffing.
    pub fn get_double_click_script(request_id: u64, target: &str) -> String {
        let is_numeric = target.chars().all(|c| c.is_ascii_digit());
        let target_expr = if is_numeric {
            format!("window.__aerolith_elements && window.__aerolith_elements[{}] || document.querySelector('[data-aerolith-ref=\"{}\"]')", target, target)
        } else {
            format!("document.querySelector({})", serde_json::to_string(target).unwrap_or_default())
        };

        format!(
            r#"(function() {{
                let attempts = 0;
                function executeDblClick() {{
                    try {{
                        const el = {};
                        if (!el) {{
                            if (++attempts < 10) {{
                                return setTimeout(executeDblClick, 50);
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

                        const opts1 = {{ bubbles: true, cancelable: true, view: window, clientX: cx, clientY: cy, detail: 1 }};
                        const opts2 = {{ bubbles: true, cancelable: true, view: window, clientX: cx, clientY: cy, detail: 2 }};
                        const initialUrl = window.location.href;
                        let mutationCount = 0;
                        let observer = null;
                        try {{
                            observer = new MutationObserver((mutations) => {{
                                mutationCount += mutations.length;
                            }});
                            observer.observe(document.body, {{ childList: true, subtree: true, attributes: true }});
                        }} catch (e) {{}}

                        el.dispatchEvent(new MouseEvent('mouseover', opts1));
                        el.dispatchEvent(new MouseEvent('mouseenter', opts1));
                        el.dispatchEvent(new MouseEvent('mousedown', opts1));
                        if (typeof el.focus === 'function') el.focus();
                        el.dispatchEvent(new MouseEvent('mouseup', opts1));
                        el.dispatchEvent(new MouseEvent('click', opts1));

                        el.dispatchEvent(new MouseEvent('mousedown', opts2));
                        el.dispatchEvent(new MouseEvent('mouseup', opts2));
                        el.dispatchEvent(new MouseEvent('click', opts2));
                        el.dispatchEvent(new MouseEvent('dblclick', opts2));

                        setTimeout(() => {{
                            if (observer) observer.disconnect();
                            const urlChanged = window.location.href !== initialUrl;
                            if (window.ipc) {{
                                window.ipc.postMessage(JSON.stringify({{
                                    type: 'agent_response',
                                    request_id: {},
                                    action: 'double_click',
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
                                action: 'double_click',
                                error: String(err)
                            }}));
                        }}
                    }}
                }}
                executeDblClick();
            }})();"#,
            target_expr, target, request_id, request_id
        )
    }

    /// JavaScript to simulate a right-click / contextmenu event with sticky-header avoidance and DOM mutation diffing.
    pub fn get_right_click_script(request_id: u64, target: &str) -> String {
        let is_numeric = target.chars().all(|c| c.is_ascii_digit());
        let target_expr = if is_numeric {
            format!("window.__aerolith_elements && window.__aerolith_elements[{}] || document.querySelector('[data-aerolith-ref=\"{}\"]')", target, target)
        } else {
            format!("document.querySelector({})", serde_json::to_string(target).unwrap_or_default())
        };

        format!(
            r#"(function() {{
                let attempts = 0;
                function executeRightClick() {{
                    try {{
                        const el = {};
                        if (!el) {{
                            if (++attempts < 10) {{
                                return setTimeout(executeRightClick, 50);
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

                        const initialUrl = window.location.href;
                        let mutationCount = 0;
                        let observer = null;
                        try {{
                            observer = new MutationObserver((mutations) => {{
                                mutationCount += mutations.length;
                            }});
                            observer.observe(document.body, {{ childList: true, subtree: true, attributes: true }});
                        }} catch (e) {{}}

                        const mouseOpts = {{ bubbles: true, cancelable: true, view: window, clientX: cx, clientY: cy, button: 2, buttons: 2 }};
                        el.dispatchEvent(new MouseEvent('mouseover', mouseOpts));
                        el.dispatchEvent(new MouseEvent('mouseenter', mouseOpts));
                        el.dispatchEvent(new MouseEvent('mousedown', mouseOpts));
                        if (typeof el.focus === 'function') el.focus();
                        el.dispatchEvent(new MouseEvent('mouseup', {{ ...mouseOpts, buttons: 0 }}));
                        el.dispatchEvent(new MouseEvent('contextmenu', {{ bubbles: true, cancelable: true, view: window, clientX: cx, clientY: cy, button: 2 }}));

                        setTimeout(() => {{
                            if (observer) observer.disconnect();
                            const urlChanged = window.location.href !== initialUrl;
                            if (window.ipc) {{
                                window.ipc.postMessage(JSON.stringify({{
                                    type: 'agent_response',
                                    request_id: {},
                                    action: 'right_click',
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
                                action: 'right_click',
                                error: String(err)
                            }}));
                        }}
                    }}
                }}
                executeRightClick();
            }})();"#,
            target_expr, target, request_id, request_id
        )
    }

    /// JavaScript to parse and extract structured table datasets into JSON row records.
    pub fn get_extract_table_script(request_id: u64, target: Option<&str>) -> String {
        let target_json = serde_json::to_string(&target.unwrap_or("")).unwrap_or_default();
        format!(
            r#"(function() {{
                try {{
                    let table = null;
                    const targetStr = {};
                    if (targetStr) {{
                        const isNum = /^\d+$/.test(targetStr);
                        table = isNum
                            ? (window.__aerolith_elements && window.__aerolith_elements[targetStr] || document.querySelector('[data-aerolith-ref="' + targetStr + '"]'))
                            : document.querySelector(targetStr);
                    }}
                    if (!table) {{
                        const tables = Array.from(document.querySelectorAll('table'));
                        if (tables.length > 0) {{
                            tables.sort((a, b) => b.querySelectorAll('td, th').length - a.querySelectorAll('td, th').length);
                            table = tables[0];
                        }}
                    }}
                    if (!table) {{
                        if (window.ipc) {{
                            window.ipc.postMessage(JSON.stringify({{
                                type: 'agent_response',
                                request_id: {},
                                action: 'extract_table',
                                error: 'No <table> element found on page'
                            }}));
                        }}
                        return;
                    }}

                    let title = '';
                    const caption = table.querySelector('caption');
                    if (caption) {{
                        title = (caption.innerText || '').trim();
                    }} else {{
                        let prev = table.previousElementSibling;
                        while (prev && !/^H[1-6]$/.test(prev.tagName)) {{
                            prev = prev.previousElementSibling;
                        }}
                        if (prev) title = (prev.innerText || '').trim();
                    }}

                    let headers = [];
                    const headerCells = Array.from(table.querySelectorAll('thead th, thead td, tr:first-child th'));
                    if (headerCells.length > 0) {{
                        headers = headerCells.map(c => (c.innerText || '').trim());
                    }}

                    const rows = [];
                    const allTrs = Array.from(table.querySelectorAll('tr'));
                    const trs = Array.from(table.querySelectorAll('tbody tr')).length > 0
                        ? Array.from(table.querySelectorAll('tbody tr'))
                        : (headers.length > 0 ? allTrs.slice(1) : allTrs);

                    for (const tr of trs) {{
                        const cells = Array.from(tr.querySelectorAll('td, th')).map(c => (c.innerText || '').trim());
                        if (cells.length === 0) continue;
                        if (headers.length > 0 && headers.length === cells.length) {{
                            const rowObj = {{}};
                            for (let i = 0; i < headers.length; i++) {{
                                rowObj[headers[i] || ('col_' + (i + 1))] = cells[i];
                            }}
                            rows.push(rowObj);
                        }} else {{
                            rows.push(cells);
                        }}
                    }}

                    const colCount = headers.length > 0 ? headers.length : (rows[0] ? (Array.isArray(rows[0]) ? rows[0].length : Object.keys(rows[0]).length) : 0);

                    if (window.ipc) {{
                        window.ipc.postMessage(JSON.stringify({{
                            type: 'agent_response',
                            request_id: {},
                            action: 'extract_table',
                            payload: {{
                                success: true,
                                title: title,
                                headers: headers,
                                row_count: rows.length,
                                col_count: colCount,
                                rows: rows
                            }}
                        }}));
                    }}
                }} catch (err) {{
                    if (window.ipc) {{
                        window.ipc.postMessage(JSON.stringify({{
                            type: 'agent_response',
                            request_id: {},
                            action: 'extract_table',
                            error: String(err)
                        }}));
                    }}
                }}
            }})();"#,
            target_json, request_id, request_id, request_id
        )
    }

    /// JavaScript to query or clear recorded browser file downloads.
    pub fn get_downloads_script(request_id: u64, clear: bool) -> String {
        format!(
            r#"(function() {{
                try {{
                    if (!window.__aerolith_downloads) {{
                        window.__aerolith_downloads = [];
                    }}
                    const dls = window.__aerolith_downloads.slice();
                    if ({}) {{
                        window.__aerolith_downloads = [];
                    }}
                    if (window.ipc) {{
                        window.ipc.postMessage(JSON.stringify({{
                            type: 'agent_response',
                            request_id: {},
                            action: 'downloads',
                            payload: {{
                                success: true,
                                count: dls.length,
                                downloads: dls
                            }}
                        }}));
                    }}
                }} catch (err) {{
                    if (window.ipc) {{
                        window.ipc.postMessage(JSON.stringify({{
                            type: 'agent_response',
                            request_id: {},
                            action: 'downloads',
                            error: String(err)
                        }}));
                    }}
                }}
            }})();"#,
            clear, request_id, request_id
        )
    }

    /// JavaScript to inject or remove custom CSS styles into the active page context.
    pub fn get_inject_css_script(request_id: u64, css: &str, remove: bool) -> String {
        let css_json = serde_json::to_string(css).unwrap_or_default();
        format!(
            r#"(function() {{
                try {{
                    const remove = {};
                    const cssText = {};
                    const STYLE_ID = '__aerolith_injected_css';
                    let el = document.getElementById(STYLE_ID);

                    if (remove) {{
                        if (el) {{
                            el.remove();
                        }}
                    }} else {{
                        if (!el) {{
                            el = document.createElement('style');
                            el.id = STYLE_ID;
                            (document.head || document.documentElement).appendChild(el);
                        }}
                        el.textContent = cssText;
                    }}

                    if (window.ipc) {{
                        window.ipc.postMessage(JSON.stringify({{
                            type: 'agent_response',
                            request_id: {},
                            action: 'inject_css',
                            payload: {{
                                success: true,
                                removed: remove,
                                css_length: cssText.length
                            }}
                        }}));
                    }}
                }} catch (err) {{
                    if (window.ipc) {{
                        window.ipc.postMessage(JSON.stringify({{
                            type: 'agent_response',
                            request_id: {},
                            action: 'inject_css',
                            error: String(err)
                        }}));
                    }}
                }}
            }})();"#,
            remove, css_json, request_id, request_id
        )
    }

    /// JavaScript to batch query all elements matching a CSS selector.
    pub fn get_query_elements_script(request_id: u64, selector: &str, attribute: Option<&str>, max_items: usize) -> String {
        let selector_json = serde_json::to_string(selector).unwrap_or_default();
        let attr_json = serde_json::to_string(&attribute.unwrap_or("")).unwrap_or_default();
        format!(
            r#"(function() {{
                try {{
                    if (!window.__aerolith_elements) {{
                        window.__aerolith_elements = {{}};
                        window.__aerolith_next_ref = 1;
                    }}

                    const sel = {};
                    const targetAttr = {};
                    const maxItems = {};
                    const allEls = Array.from(document.querySelectorAll(sel));
                    const totalMatched = allEls.length;
                    const sliceEls = allEls.slice(0, maxItems);

                    const items = sliceEls.map((el, i) => {{
                        let refId = el.getAttribute('data-aerolith-ref');
                        if (!refId) {{
                            refId = String(window.__aerolith_next_ref++);
                            el.setAttribute('data-aerolith-ref', refId);
                            window.__aerolith_elements[refId] = el;
                        }}

                        const rect = el.getBoundingClientRect();
                        let attrVal = null;
                        if (targetAttr) {{
                            attrVal = el.getAttribute(targetAttr);
                        }}

                        return {{
                            index: i,
                            ref: refId,
                            tag_name: el.tagName.toLowerCase(),
                            text: (el.innerText || el.textContent || '').trim().substring(0, 300),
                            value: el.value !== undefined ? String(el.value) : null,
                            attribute: attrVal,
                            rect: {{
                                x: Math.round(rect.x),
                                y: Math.round(rect.y),
                                width: Math.round(rect.width),
                                height: Math.round(rect.height)
                            }}
                        }};
                    }});

                    if (window.ipc) {{
                        window.ipc.postMessage(JSON.stringify({{
                            type: 'agent_response',
                            request_id: {},
                            action: 'query_elements',
                            payload: {{
                                success: true,
                                selector: sel,
                                count: items.length,
                                total_matched: totalMatched,
                                items: items
                            }}
                        }}));
                    }}
                }} catch (err) {{
                    if (window.ipc) {{
                        window.ipc.postMessage(JSON.stringify({{
                            type: 'agent_response',
                            request_id: {},
                            action: 'query_elements',
                            error: String(err)
                        }}));
                    }}
                }}
            }})();"#,
            selector_json, attr_json, max_items, request_id, request_id
        )
    }

    /// JavaScript to explicitly focus an element.
    pub fn get_focus_script(request_id: u64, target: &str) -> String {
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
                    if (!el) {{
                        if (window.ipc) {{
                            window.ipc.postMessage(JSON.stringify({{
                                type: 'agent_response',
                                request_id: {},
                                action: 'focus',
                                error: 'Element not found: ' + {}
                            }}));
                        }}
                        return;
                    }}

                    el.scrollIntoView({{ behavior: 'instant', block: 'nearest' }});
                    if (typeof el.focus === 'function') {{
                        el.focus();
                    }}
                    el.dispatchEvent(new FocusEvent('focusin', {{ bubbles: true }}));
                    el.dispatchEvent(new FocusEvent('focus', {{ bubbles: false }}));

                    if (window.ipc) {{
                        window.ipc.postMessage(JSON.stringify({{
                            type: 'agent_response',
                            request_id: {},
                            action: 'focus',
                            payload: {{
                                success: true,
                                tag: el.tagName.toLowerCase(),
                                is_focused: document.activeElement === el
                            }}
                        }}));
                    }}
                }} catch (err) {{
                    if (window.ipc) {{
                        window.ipc.postMessage(JSON.stringify({{
                            type: 'agent_response',
                            request_id: {},
                            action: 'focus',
                            error: String(err)
                        }}));
                    }}
                }}
            }})();"#,
            target_expr, request_id, serde_json::to_string(target).unwrap_or_default(), request_id, request_id
        )
    }

    /// JavaScript to explicitly blur an element.
    pub fn get_blur_script(request_id: u64, target: &str) -> String {
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
                    if (!el) {{
                        if (window.ipc) {{
                            window.ipc.postMessage(JSON.stringify({{
                                type: 'agent_response',
                                request_id: {},
                                action: 'blur',
                                error: 'Element not found: ' + {}
                            }}));
                        }}
                        return;
                    }}

                    if (typeof el.blur === 'function') {{
                        el.blur();
                    }}
                    el.dispatchEvent(new FocusEvent('focusout', {{ bubbles: true }}));
                    el.dispatchEvent(new FocusEvent('blur', {{ bubbles: false }}));

                    if (window.ipc) {{
                        window.ipc.postMessage(JSON.stringify({{
                            type: 'agent_response',
                            request_id: {},
                            action: 'blur',
                            payload: {{
                                success: true,
                                tag: el.tagName.toLowerCase()
                            }}
                        }}));
                    }}
                }} catch (err) {{
                    if (window.ipc) {{
                        window.ipc.postMessage(JSON.stringify({{
                            type: 'agent_response',
                            request_id: {},
                            action: 'blur',
                            error: String(err)
                        }}));
                    }}
                }}
            }})();"#,
            target_expr, request_id, serde_json::to_string(target).unwrap_or_default(), request_id, request_id
        )
    }

    /// JavaScript to extract detailed page layout, scroll metrics, and document metadata.
    pub fn get_page_info_script(request_id: u64) -> String {
        format!(
            r#"(function() {{
                try {{
                    const de = document.documentElement || {{}};
                    const bd = document.body || {{}};
                    const scrollX = window.scrollX || window.pageXOffset || 0;
                    const scrollY = window.scrollY || window.pageYOffset || 0;
                    const vpWidth = window.innerWidth || de.clientWidth || 1920;
                    const vpHeight = window.innerHeight || de.clientHeight || 1080;
                    const docWidth = Math.max(de.scrollWidth || 0, bd.scrollWidth || 0, vpWidth);
                    const docHeight = Math.max(de.scrollHeight || 0, bd.scrollHeight || 0, vpHeight);
                    const maxScrollY = Math.max(0, docHeight - vpHeight);
                    const scrollProgress = maxScrollY > 0 ? Math.min(1.0, Math.round((scrollY / maxScrollY) * 1000) / 1000) : 1.0;

                    const info = {{
                        url: window.location.href,
                        title: document.title || '',
                        ready_state: document.readyState || 'complete',
                        scroll_x: Math.round(scrollX),
                        scroll_y: Math.round(scrollY),
                        viewport_width: Math.round(vpWidth),
                        viewport_height: Math.round(vpHeight),
                        document_width: Math.round(docWidth),
                        document_height: Math.round(docHeight),
                        scroll_progress: scrollProgress
                    }};

                    if (window.ipc) {{
                        window.ipc.postMessage(JSON.stringify({{
                            type: 'agent_response',
                            request_id: {},
                            action: 'page_info',
                            payload: info
                        }}));
                    }}
                }} catch (err) {{
                    if (window.ipc) {{
                        window.ipc.postMessage(JSON.stringify({{
                            type: 'agent_response',
                            request_id: {},
                            action: 'page_info',
                            error: String(err)
                        }}));
                    }}
                }}
            }})();"#,
            request_id, request_id
        )
    }

    /// JavaScript to retrieve full page outerHTML or scoped element HTML.
    pub fn get_html_script(request_id: u64, target: Option<&str>) -> String {
        let target_json = serde_json::to_string(&target.unwrap_or("")).unwrap_or_default();
        format!(
            r#"(function() {{
                try {{
                    const targetStr = {};
                    let html = '';
                    if (targetStr) {{
                        const isNum = /^\d+$/.test(targetStr);
                        const el = isNum
                            ? (window.__aerolith_elements && window.__aerolith_elements[targetStr] || document.querySelector('[data-aerolith-ref="' + targetStr + '"]'))
                            : document.querySelector(targetStr);
                        if (!el) {{
                            if (window.ipc) {{
                                window.ipc.postMessage(JSON.stringify({{
                                    type: 'agent_response',
                                    request_id: {},
                                    action: 'get_html',
                                    error: 'Element not found for target: ' + targetStr
                                }}));
                            }}
                            return;
                        }}
                        html = el.outerHTML;
                    }} else {{
                        html = document.documentElement.outerHTML;
                    }}

                    if (window.ipc) {{
                        window.ipc.postMessage(JSON.stringify({{
                            type: 'agent_response',
                            request_id: {},
                            action: 'get_html',
                            payload: {{
                                success: true,
                                target: targetStr || 'document',
                                length: html.length,
                                html: html
                            }}
                        }}));
                    }}
                }} catch (err) {{
                    if (window.ipc) {{
                        window.ipc.postMessage(JSON.stringify({{
                            type: 'agent_response',
                            request_id: {},
                            action: 'get_html',
                            error: String(err)
                        }}));
                    }}
                }}
            }})();"#,
            target_json, request_id, request_id, request_id
        )
    }

    /// JavaScript to scroll a specific element into view with block alignment and sticky-header compensation.
    pub fn get_scroll_into_view_script(request_id: u64, target: &str, align: &str) -> String {
        let is_numeric = target.chars().all(|c| c.is_ascii_digit());
        let target_expr = if is_numeric {
            format!("window.__aerolith_elements && window.__aerolith_elements[{}] || document.querySelector('[data-aerolith-ref=\"{}\"]')", target, target)
        } else {
            format!("document.querySelector({})", serde_json::to_string(target).unwrap_or_default())
        };
        let align_json = serde_json::to_string(align).unwrap_or_else(|_| "\"center\"".into());

        format!(
            r#"(function() {{
                try {{
                    const el = {};
                    if (!el) {{
                        if (window.ipc) {{
                            window.ipc.postMessage(JSON.stringify({{
                                type: 'agent_response',
                                request_id: {},
                                action: 'scroll_into_view',
                                error: 'Element not found: ' + {}
                            }}));
                        }}
                        return;
                    }}

                    const alignBlock = {};
                    el.scrollIntoView({{ behavior: 'instant', block: alignBlock, inline: 'nearest' }});

                    const rect = el.getBoundingClientRect();
                    const cx = rect.x + rect.width / 2;
                    const cy = rect.y + rect.height / 2;
                    let occluded = false;
                    const topEl = document.elementFromPoint(cx, cy);
                    if (topEl && topEl !== el && !el.contains(topEl) && !topEl.contains(el)) {{
                        occluded = true;
                        window.scrollBy(0, -75);
                    }}

                    const updatedRect = el.getBoundingClientRect();
                    const vpHeight = window.innerHeight || document.documentElement.clientHeight || 1080;
                    const vpWidth = window.innerWidth || document.documentElement.clientWidth || 1920;
                    const inViewport = (
                        updatedRect.top >= 0 &&
                        updatedRect.left >= 0 &&
                        updatedRect.bottom <= vpHeight &&
                        updatedRect.right <= vpWidth
                    );

                    if (window.ipc) {{
                        window.ipc.postMessage(JSON.stringify({{
                            type: 'agent_response',
                            request_id: {},
                            action: 'scroll_into_view',
                            payload: {{
                                success: true,
                                tag: el.tagName.toLowerCase(),
                                align: alignBlock,
                                in_viewport: inViewport,
                                occluded: occluded,
                                rect: {{
                                    x: Math.round(updatedRect.x),
                                    y: Math.round(updatedRect.y),
                                    width: Math.round(updatedRect.width),
                                    height: Math.round(updatedRect.height)
                                }}
                            }}
                        }}));
                    }}
                }} catch (err) {{
                    if (window.ipc) {{
                        window.ipc.postMessage(JSON.stringify({{
                            type: 'agent_response',
                            request_id: {},
                            action: 'scroll_into_view',
                            error: String(err)
                        }}));
                    }}
                }}
            }})();"#,
            target_expr, request_id, serde_json::to_string(target).unwrap_or_default(), align_json, request_id, request_id
        )
    }

    /// JavaScript to inspect or mutate granular localStorage / sessionStorage keys.
    pub fn get_storage_item_script(
        request_id: u64,
        storage_type: &str,
        action: &str,
        key: Option<&str>,
        value: Option<&str>,
    ) -> String {
        let is_session = storage_type.eq_ignore_ascii_case("session");
        let act_json = serde_json::to_string(action).unwrap_or_default();
        let key_json = serde_json::to_string(&key.unwrap_or("")).unwrap_or_default();
        let val_json = serde_json::to_string(&value.unwrap_or("")).unwrap_or_default();

        format!(
            r#"(function() {{
                try {{
                    const store = {} ? window.sessionStorage : window.localStorage;
                    const act = {};
                    const targetKey = {};
                    const targetVal = {};
                    let result = null;

                    if (act === 'set') {{
                        if (targetKey) {{
                            store.setItem(targetKey, targetVal);
                            result = {{ key: targetKey, value: targetVal, status: 'saved' }};
                        }}
                    }} else if (act === 'delete' || act === 'remove') {{
                        if (targetKey) {{
                            store.removeItem(targetKey);
                            result = {{ key: targetKey, status: 'deleted' }};
                        }}
                    }} else if (act === 'clear') {{
                        store.clear();
                        result = {{ status: 'cleared' }};
                    }} else if (act === 'keys' || act === 'list') {{
                        const keys = [];
                        for (let i = 0; i < store.length; i++) {{
                            keys.push(store.key(i));
                        }}
                        result = {{ keys: keys, count: keys.length }};
                    }} else {{
                        // 'get'
                        const val = targetKey ? store.getItem(targetKey) : null;
                        result = {{ key: targetKey, value: val, exists: val !== null }};
                    }}

                    if (window.ipc) {{
                        window.ipc.postMessage(JSON.stringify({{
                            type: 'agent_response',
                            request_id: {},
                            action: 'storage_item',
                            payload: {{
                                success: true,
                                storage_type: {} ? 'session' : 'local',
                                action: act,
                                result: result
                            }}
                        }}));
                    }}
                }} catch (err) {{
                    if (window.ipc) {{
                        window.ipc.postMessage(JSON.stringify({{
                            type: 'agent_response',
                            request_id: {},
                            action: 'storage_item',
                            error: String(err)
                        }}));
                    }}
                }}
            }})();"#,
            is_session, act_json, key_json, val_json, request_id, is_session, request_id
        )
    }

    /// JavaScript to verify element state against specified assertions.
    pub fn get_assert_element_script(
        request_id: u64,
        target: &str,
        assertions: &crate::agent::protocol::ElementAssertion,
    ) -> String {
        let is_numeric = target.chars().all(|c| c.is_ascii_digit());
        let target_expr = if is_numeric {
            format!("window.__aerolith_elements && window.__aerolith_elements[{}] || document.querySelector('[data-aerolith-ref=\"{}\"]')", target, target)
        } else {
            format!("document.querySelector({})", serde_json::to_string(target).unwrap_or_default())
        };

        let assertions_json = serde_json::to_string(assertions).unwrap_or_else(|_| "{}".to_string());

        format!(
            r#"(function() {{
                try {{
                    const el = {};
                    const expected = {};
                    const failures = [];
                    const targetStr = {};

                    if (!el) {{
                        if (expected.hidden === true) {{
                            if (window.ipc) {{
                                window.ipc.postMessage(JSON.stringify({{
                                    type: 'agent_response',
                                    request_id: {},
                                    action: 'assert_element',
                                    payload: {{
                                        success: true,
                                        passed: true,
                                        target: targetStr,
                                        failures: [],
                                        actual: {{ present: false, hidden: true, visible: false }}
                                    }}
                                }}));
                            }}
                            return;
                        }}
                        if (window.ipc) {{
                            window.ipc.postMessage(JSON.stringify({{
                                type: 'agent_response',
                                request_id: {},
                                action: 'assert_element',
                                error: 'Element not found for target: ' + targetStr
                            }}));
                        }}
                        return;
                    }}

                    const style = window.getComputedStyle(el);
                    const rect = el.getBoundingClientRect();
                    const isVisible = Boolean(
                        (el.offsetWidth || el.offsetHeight || rect.width || rect.height) &&
                        style.visibility !== 'hidden' &&
                        style.display !== 'none'
                    );
                    const isHidden = !isVisible;
                    const isDisabled = Boolean(el.disabled || el.getAttribute('aria-disabled') === 'true');
                    const isChecked = Boolean(el.checked || el.getAttribute('aria-checked') === 'true');
                    const text = (el.innerText || el.textContent || '').trim();
                    const val = el.value !== undefined ? String(el.value) : (el.getAttribute('value') || '');
                    const classes = Array.from(el.classList || []);

                    if (expected.visible !== undefined && expected.visible !== null) {{
                        if (isVisible !== expected.visible) {{
                            failures.push('visible: expected ' + expected.visible + ', got ' + isVisible);
                        }}
                    }}
                    if (expected.hidden !== undefined && expected.hidden !== null) {{
                        if (isHidden !== expected.hidden) {{
                            failures.push('hidden: expected ' + expected.hidden + ', got ' + isHidden);
                        }}
                    }}
                    if (expected.disabled !== undefined && expected.disabled !== null) {{
                        if (isDisabled !== expected.disabled) {{
                            failures.push('disabled: expected ' + expected.disabled + ', got ' + isDisabled);
                        }}
                    }}
                    if (expected.checked !== undefined && expected.checked !== null) {{
                        if (isChecked !== expected.checked) {{
                            failures.push('checked: expected ' + expected.checked + ', got ' + isChecked);
                        }}
                    }}
                    if (expected.text_contains) {{
                        if (!text.toLowerCase().includes(expected.text_contains.toLowerCase())) {{
                            failures.push('text_contains: expected substring "' + expected.text_contains + '", got "' + text + '"');
                        }}
                    }}
                    if (expected.value !== undefined && expected.value !== null) {{
                        if (val !== expected.value) {{
                            failures.push('value: expected "' + expected.value + '", got "' + val + '"');
                        }}
                    }}
                    if (expected.has_class) {{
                        if (!classes.includes(expected.has_class)) {{
                            failures.push('has_class: expected class "' + expected.has_class + '", classes: [' + classes.join(', ') + ']');
                        }}
                    }}
                    if (expected.attr_name) {{
                        const actualAttr = el.getAttribute(expected.attr_name);
                        if (expected.attr_value !== undefined && expected.attr_value !== null) {{
                            if (actualAttr !== expected.attr_value) {{
                                failures.push('attribute ' + expected.attr_name + ': expected "' + expected.attr_value + '", got ' + JSON.stringify(actualAttr));
                            }}
                        }} else if (actualAttr === null) {{
                            failures.push('attribute ' + expected.attr_name + ': expected attribute to exist');
                        }}
                    }}

                    const passed = failures.length === 0;
                    if (window.ipc) {{
                        window.ipc.postMessage(JSON.stringify({{
                            type: 'agent_response',
                            request_id: {},
                            action: 'assert_element',
                            payload: {{
                                success: passed,
                                passed: passed,
                                target: targetStr,
                                failures: failures,
                                actual: {{
                                    present: true,
                                    visible: isVisible,
                                    hidden: isHidden,
                                    disabled: isDisabled,
                                    checked: isChecked,
                                    text: text,
                                    value: val,
                                    classes: classes
                                }}
                            }}
                        }}));
                    }}
                }} catch (err) {{
                    if (window.ipc) {{
                        window.ipc.postMessage(JSON.stringify({{
                            type: 'agent_response',
                            request_id: {},
                            action: 'assert_element',
                            error: String(err)
                        }}));
                    }}
                }}
            }})();"#,
            target_expr, assertions_json, serde_json::to_string(target).unwrap_or_default(),
            request_id, request_id, request_id, request_id
        )
    }

    /// JavaScript to wait for a selector to reach a specific state ('visible', 'hidden', 'attached', 'detached').
    pub fn get_wait_for_selector_script(
        request_id: u64,
        target: &str,
        state: &str,
        timeout_ms: u64,
    ) -> String {
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
                const targetState = {};
                const targetStr = {};

                function isConditionMet() {{
                    const el = {};
                    if (targetState === 'attached') {{
                        return Boolean(el);
                    }}
                    if (targetState === 'detached') {{
                        return !el;
                    }}
                    if (!el) {{
                        return targetState === 'hidden';
                    }}
                    const style = window.getComputedStyle(el);
                    const rect = el.getBoundingClientRect();
                    const isVis = Boolean(
                        (el.offsetWidth || el.offsetHeight || rect.width || rect.height) &&
                        style.visibility !== 'hidden' &&
                        style.display !== 'none'
                    );
                    if (targetState === 'visible') {{
                        return isVis;
                    }}
                    if (targetState === 'hidden') {{
                        return !isVis;
                    }}
                    return false;
                }}

                function check() {{
                    try {{
                        if (isConditionMet()) {{
                            if (window.ipc) {{
                                window.ipc.postMessage(JSON.stringify({{
                                    type: 'agent_response',
                                    request_id: {},
                                    action: 'wait_for_selector',
                                    payload: {{
                                        success: true,
                                        target: targetStr,
                                        state: targetState,
                                        elapsed_ms: Date.now() - startTime
                                    }}
                                }}));
                            }}
                            return;
                        }}

                        if (Date.now() - startTime >= timeout) {{
                            if (window.ipc) {{
                                window.ipc.postMessage(JSON.stringify({{
                                    type: 'agent_response',
                                    request_id: {},
                                    action: 'wait_for_selector',
                                    error: 'Timeout of ' + timeout + 'ms exceeded waiting for ' + targetStr + ' to reach state: ' + targetState
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
                                action: 'wait_for_selector',
                                error: String(err)
                            }}));
                        }}
                    }}
                }}

                check();
            }})();"#,
            timeout_ms, serde_json::to_string(state).unwrap_or_else(|_| "\"visible\"".into()),
            serde_json::to_string(target).unwrap_or_default(),
            target_expr, request_id, request_id, request_id
        )
    }

    /// JavaScript to capture performance metrics from Navigation and Resource Timing APIs.
    pub fn get_performance_metrics_script(request_id: u64) -> String {
        format!(
            r#"(function() {{
                try {{
                    const perf = window.performance;
                    if (!perf) {{
                        if (window.ipc) {{
                            window.ipc.postMessage(JSON.stringify({{
                                type: 'agent_response',
                                request_id: {},
                                action: 'performance_metrics',
                                error: 'window.performance is not supported'
                            }}));
                        }}
                        return;
                    }}

                    const navEntries = perf.getEntriesByType('navigation');
                    const nav = navEntries && navEntries.length > 0 ? navEntries[0] : null;
                    const resources = perf.getEntriesByType('resource') || [];

                    let scriptsCount = 0;
                    let stylesCount = 0;
                    let imagesCount = 0;
                    let fetchCount = 0;
                    let otherCount = 0;
                    let totalTransferBytes = 0;

                    for (let i = 0; i < resources.length; i++) {{
                        const r = resources[i];
                        const type = r.initiatorType || '';
                        if (type === 'script') scriptsCount++;
                        else if (type === 'link' || type === 'css') stylesCount++;
                        else if (type === 'img' || type === 'image') imagesCount++;
                        else if (type === 'fetch' || type === 'xmlhttprequest') fetchCount++;
                        else otherCount++;

                        totalTransferBytes += (r.transferSize || 0);
                    }}

                    if (nav && nav.transferSize) {{
                        totalTransferBytes += nav.transferSize;
                    }}

                    const metrics = {{
                        url: window.location.href,
                        navigation_type: nav ? nav.type : 'navigate',
                        ttfb_ms: nav ? Math.round(nav.responseStart - nav.requestStart) : 0,
                        dns_ms: nav ? Math.round(nav.domainLookupEnd - nav.domainLookupStart) : 0,
                        connect_ms: nav ? Math.round(nav.connectEnd - nav.connectStart) : 0,
                        dom_content_loaded_ms: nav ? Math.round(nav.domContentLoadedEventEnd - nav.startTime) : 0,
                        load_ms: nav ? Math.round(nav.loadEventEnd - nav.startTime) : 0,
                        duration_ms: nav ? Math.round(nav.duration) : 0,
                        total_resources: resources.length,
                        resource_breakdown: {{
                            scripts: scriptsCount,
                            stylesheets: stylesCount,
                            images: imagesCount,
                            fetch_xhr: fetchCount,
                            other: otherCount
                        }},
                        total_transfer_bytes: totalTransferBytes,
                        js_heap_used_mb: perf.memory ? +(perf.memory.usedJSHeapSize / (1024 * 1024)).toFixed(2) : null,
                        js_heap_total_mb: perf.memory ? +(perf.memory.totalJSHeapSize / (1024 * 1024)).toFixed(2) : null
                    }};

                    if (window.ipc) {{
                        window.ipc.postMessage(JSON.stringify({{
                            type: 'agent_response',
                            request_id: {},
                            action: 'performance_metrics',
                            payload: {{
                                success: true,
                                metrics: metrics
                            }}
                        }}));
                    }}
                }} catch (err) {{
                    if (window.ipc) {{
                        window.ipc.postMessage(JSON.stringify({{
                            type: 'agent_response',
                            request_id: {},
                            action: 'performance_metrics',
                            error: String(err)
                        }}));
                    }}
                }}
            }})();"#,
            request_id, request_id, request_id
        )
    }

    /// JavaScript to scroll the window directly to specific coordinates.
    pub fn get_scroll_to_coords_script(request_id: u64, x: i32, y: i32, smooth: bool) -> String {
        format!(
            r#"(function() {{
                try {{
                    const targetX = {};
                    const targetY = {};
                    const smooth = {};

                    window.scrollTo({{
                        left: targetX,
                        top: targetY,
                        behavior: smooth ? 'smooth' : 'instant'
                    }});

                    const doc = document.documentElement;
                    const maxScrollX = Math.max(0, (doc.scrollWidth || 0) - (window.innerWidth || 0));
                    const maxScrollY = Math.max(0, (doc.scrollHeight || 0) - (window.innerHeight || 0));

                    if (window.ipc) {{
                        window.ipc.postMessage(JSON.stringify({{
                            type: 'agent_response',
                            request_id: {},
                            action: 'scroll_to_coords',
                            payload: {{
                                success: true,
                                scroll_x: Math.round(window.scrollX || window.pageXOffset || 0),
                                scroll_y: Math.round(window.scrollY || window.pageYOffset || 0),
                                target_x: targetX,
                                target_y: targetY,
                                max_scroll_x: maxScrollX,
                                max_scroll_y: maxScrollY
                            }}
                        }}));
                    }}
                }} catch (err) {{
                    if (window.ipc) {{
                        window.ipc.postMessage(JSON.stringify({{
                            type: 'agent_response',
                            request_id: {},
                            action: 'scroll_to_coords',
                            error: String(err)
                        }}));
                    }}
                }}
            }})();"#,
            x, y, smooth, request_id, request_id
        )
    }

    /// JavaScript to generate a compact semantic outline of the page: headings, forms, landmark containers, buttons, and links.
    pub fn get_page_outline_script(request_id: u64, max_links: usize, max_buttons: usize) -> String {
        format!(
            r#"(function() {{
                try {{
                    const headings = [];
                    document.querySelectorAll('h1, h2, h3, h4, h5, h6').forEach(h => {{
                        const text = (h.innerText || h.textContent || '').trim();
                        if (text) {{
                            headings.push({{
                                tag: h.tagName.toLowerCase(),
                                level: parseInt(h.tagName.substring(1), 10),
                                text: text.length > 100 ? text.substring(0, 100) + '...' : text,
                                id: h.id || null,
                                ref: h.getAttribute('data-aerolith-ref') || null
                            }});
                        }}
                    }});

                    const forms = [];
                    document.querySelectorAll('form').forEach(f => {{
                        const fields = [];
                        f.querySelectorAll('input, select, textarea').forEach(inp => {{
                            const t = inp.type || inp.tagName.toLowerCase();
                            if (t !== 'hidden') {{
                                fields.push({{
                                    tag: inp.tagName.toLowerCase(),
                                    type: t,
                                    name: inp.name || null,
                                    id: inp.id || null,
                                    placeholder: inp.placeholder || null,
                                    value: inp.value ? (inp.type === 'password' ? '***' : inp.value) : null,
                                    required: inp.required || false,
                                    ref: inp.getAttribute('data-aerolith-ref') || null
                                }});
                            }}
                        }});
                        forms.push({{
                            id: f.id || null,
                            name: f.name || null,
                            action: f.action || null,
                            method: (f.method || 'GET').toUpperCase(),
                            fields_count: fields.length,
                            fields: fields
                        }});
                    }});

                    const landmarks = [];
                    document.querySelectorAll('main, nav, header, footer, aside, section[aria-label], [role="main"], [role="navigation"], [role="banner"]').forEach(lm => {{
                        const role = lm.getAttribute('role') || lm.tagName.toLowerCase();
                        const label = lm.getAttribute('aria-label') || lm.id || '';
                        landmarks.push({{
                            tag: lm.tagName.toLowerCase(),
                            role: role,
                            label: label || null,
                            ref: lm.getAttribute('data-aerolith-ref') || null
                        }});
                    }});

                    const buttons = [];
                    const btnEls = Array.from(document.querySelectorAll('button, input[type="submit"], input[type="button"], [role="button"]'));
                    for (let i = 0; i < Math.min(btnEls.length, {}); i++) {{
                        const b = btnEls[i];
                        const text = (b.innerText || b.value || b.getAttribute('aria-label') || '').trim();
                        if (text) {{
                            buttons.push({{
                                text: text.length > 50 ? text.substring(0, 50) + '...' : text,
                                disabled: b.disabled || b.getAttribute('aria-disabled') === 'true',
                                ref: b.getAttribute('data-aerolith-ref') || null
                            }});
                        }}
                    }}

                    const links = [];
                    const linkEls = Array.from(document.querySelectorAll('a[href]'));
                    for (let i = 0; i < Math.min(linkEls.length, {}); i++) {{
                        const a = linkEls[i];
                        const text = (a.innerText || a.getAttribute('aria-label') || '').trim();
                        const href = a.getAttribute('href') || '';
                        if (text && !href.startsWith('javascript:')) {{
                            links.push({{
                                text: text.length > 60 ? text.substring(0, 60) + '...' : text,
                                href: href.length > 120 ? href.substring(0, 120) + '...' : href,
                                ref: a.getAttribute('data-aerolith-ref') || null
                            }});
                        }}
                    }}

                    if (window.ipc) {{
                        window.ipc.postMessage(JSON.stringify({{
                            type: 'agent_response',
                            request_id: {},
                            action: 'page_outline',
                            payload: {{
                                success: true,
                                title: document.title || '',
                                url: window.location.href,
                                headings: headings,
                                forms: forms,
                                landmarks: landmarks,
                                buttons: buttons,
                                links: links
                            }}
                        }}));
                    }}
                }} catch (err) {{
                    if (window.ipc) {{
                        window.ipc.postMessage(JSON.stringify({{
                            type: 'agent_response',
                            request_id: {},
                            action: 'page_outline',
                            error: String(err)
                        }}));
                    }}
                }}
            }})();"#,
            max_buttons, max_links, request_id, request_id
        )
    }

    /// JavaScript to wait until active network requests remain at or below max_inflight for consecutive idle_time_ms.
    pub fn get_wait_for_network_idle_script(
        request_id: u64,
        idle_time_ms: u64,
        timeout_ms: u64,
        max_inflight: usize,
    ) -> String {
        format!(
            r#"(function() {{
                const startTime = Date.now();
                const idleRequired = {};
                const timeout = {};
                const maxInflight = {};
                let consecutiveIdle = 0;

                function check() {{
                    try {{
                        const inflight = window.__aerolith_inflight || 0;
                        if (inflight <= maxInflight) {{
                            consecutiveIdle += 50;
                            if (consecutiveIdle >= idleRequired) {{
                                if (window.ipc) {{
                                    window.ipc.postMessage(JSON.stringify({{
                                        type: 'agent_response',
                                        request_id: {},
                                        action: 'wait_for_network_idle',
                                        payload: {{
                                            success: true,
                                            elapsed_ms: Date.now() - startTime,
                                            idle_time_ms: consecutiveIdle,
                                            inflight_requests: inflight
                                        }}
                                    }}));
                                }}
                                return;
                            }}
                        }} else {{
                            consecutiveIdle = 0;
                        }}

                        if (Date.now() - startTime >= timeout) {{
                            if (window.ipc) {{
                                window.ipc.postMessage(JSON.stringify({{
                                    type: 'agent_response',
                                    request_id: {},
                                    action: 'wait_for_network_idle',
                                    error: 'Timeout of ' + timeout + 'ms exceeded waiting for network idle (inflight: ' + inflight + ')'
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
                                action: 'wait_for_network_idle',
                                error: String(err)
                            }}));
                        }}
                    }}
                }}

                check();
            }})();"#,
            idle_time_ms, timeout_ms, max_inflight, request_id, request_id, request_id
        )
    }

    /// JavaScript to control and query DOM mutation records (start, stop, clear, get).
    pub fn get_watch_mutations_script(
        request_id: u64,
        action: &str,
        target: Option<&str>,
    ) -> String {
        let is_numeric = target.map(|t| t.chars().all(|c| c.is_ascii_digit())).unwrap_or(false);
        let target_expr = if let Some(t) = target {
            if is_numeric {
                format!("window.__aerolith_elements && window.__aerolith_elements[{}] || document.querySelector('[data-aerolith-ref=\"{}\"]')", t, t)
            } else {
                format!("document.querySelector({})", serde_json::to_string(t).unwrap_or_default())
            }
        } else {
            "document.body || document.documentElement".to_string()
        };

        let action_json = serde_json::to_string(action).unwrap_or_default();

        format!(
            r#"(function() {{
                try {{
                    const act = {};
                    window.__aerolith_mutation_buffer = window.__aerolith_mutation_buffer || [];

                    if (act === 'start') {{
                        if (window.__aerolith_mutation_observer) {{
                            window.__aerolith_mutation_observer.disconnect();
                        }}
                        window.__aerolith_mutation_buffer = [];
                        const root = {} || document.body || document.documentElement;
                        window.__aerolith_mutation_observer = new MutationObserver((mutations) => {{
                            for (let i = 0; i < mutations.length; i++) {{
                                if (window.__aerolith_mutation_buffer.length >= 100) break;
                                const m = mutations[i];
                                const record = {{
                                    type: m.type,
                                    target_tag: m.target ? m.target.nodeName.toLowerCase() : '',
                                    target_id: m.target && m.target.id ? m.target.id : null,
                                    added_count: m.addedNodes ? m.addedNodes.length : 0,
                                    removed_count: m.removedNodes ? m.removedNodes.length : 0,
                                    attribute_name: m.attributeName || null,
                                    old_value: m.oldValue || null,
                                    timestamp: Date.now()
                                }};
                                window.__aerolith_mutation_buffer.push(record);
                            }}
                        }});
                        window.__aerolith_mutation_observer.observe(root, {{
                            childList: true,
                            subtree: true,
                            attributes: true,
                            attributeOldValue: true,
                            characterData: true
                        }});
                    }} else if (act === 'stop') {{
                        if (window.__aerolith_mutation_observer) {{
                            window.__aerolith_mutation_observer.disconnect();
                            window.__aerolith_mutation_observer = null;
                        }}
                    }} else if (act === 'clear') {{
                        window.__aerolith_mutation_buffer = [];
                    }}

                    const records = window.__aerolith_mutation_buffer || [];
                    const isWatching = Boolean(window.__aerolith_mutation_observer);

                    if (window.ipc) {{
                        window.ipc.postMessage(JSON.stringify({{
                            type: 'agent_response',
                            request_id: {},
                            action: 'watch_mutations',
                            payload: {{
                                success: true,
                                action: act,
                                is_watching: isWatching,
                                count: records.length,
                                mutations: records
                            }}
                        }}));
                    }}
                }} catch (err) {{
                    if (window.ipc) {{
                        window.ipc.postMessage(JSON.stringify({{
                            type: 'agent_response',
                            request_id: {},
                            action: 'watch_mutations',
                            error: String(err)
                        }}));
                    }}
                }}
            }})();"#,
            action_json, target_expr, request_id, request_id
        )
    }

    /// JavaScript to search recorded network activity for requests matching a URL pattern and return the response body.
    pub fn get_response_body_script(request_id: u64, pattern: &str, clear: bool) -> String {
        let pattern_json = serde_json::to_string(pattern).unwrap_or_default();
        format!(
            r#"(function() {{
                try {{
                    const net = window.__aerolith_network || [];
                    const pat = ({}).toLowerCase();
                    const matches = [];

                    for (let i = net.length - 1; i >= 0; i--) {{
                        const entry = net[i];
                        const url = (entry.url || '').toLowerCase();
                        if (!pat || url.includes(pat)) {{
                            matches.push(entry);
                        }}
                    }}

                    if ({}) {{
                        window.__aerolith_network = [];
                    }}

                    const lastMatch = matches.length > 0 ? matches[0] : null;

                    if (window.ipc) {{
                        window.ipc.postMessage(JSON.stringify({{
                            type: 'agent_response',
                            request_id: {},
                            action: 'response_body',
                            payload: {{
                                success: true,
                                pattern: {},
                                matched_count: matches.length,
                                response: lastMatch,
                                all_matches: matches.slice(0, 10)
                            }}
                        }}));
                    }}
                }} catch (err) {{
                    if (window.ipc) {{
                        window.ipc.postMessage(JSON.stringify({{
                            type: 'agent_response',
                            request_id: {},
                            action: 'response_body',
                            error: String(err)
                        }}));
                    }}
                }}
            }})();"#,
            pattern_json, clear, request_id, pattern_json, request_id
        )
    }

    /// JavaScript to dispatch a synthetic file drag-and-drop or file input upload.
    pub fn get_drop_file_script(
        request_id: u64,
        target: &str,
        file_name: &str,
        mime_type: &str,
        content: &str,
    ) -> String {
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
                    const targetStr = {};
                    if (!el) {{
                        if (window.ipc) {{
                            window.ipc.postMessage(JSON.stringify({{
                                type: 'agent_response',
                                request_id: {},
                                action: 'drop_file',
                                error: 'Element not found for target: ' + targetStr
                            }}));
                        }}
                        return;
                    }}

                    const fileName = {};
                    const mimeType = {};
                    const contentStr = {};

                    let bytes;
                    try {{
                        const binStr = atob(contentStr);
                        bytes = new Uint8Array(binStr.length);
                        for (let i = 0; i < binStr.length; i++) {{
                            bytes[i] = binStr.charCodeAt(i);
                        }}
                    }} catch (_) {{
                        bytes = new TextEncoder().encode(contentStr);
                    }}

                    const file = new File([bytes], fileName, {{ type: mimeType }});
                    const dt = new DataTransfer();
                    dt.items.add(file);

                    if (el.tagName === 'INPUT' && el.type === 'file') {{
                        el.files = dt.files;
                        el.dispatchEvent(new Event('change', {{ bubbles: true }}));
                        el.dispatchEvent(new Event('input', {{ bubbles: true }}));
                    }} else {{
                        const rect = el.getBoundingClientRect();
                        const cx = rect.x + rect.width / 2;
                        const cy = rect.y + rect.height / 2;
                        const eventInit = {{
                            bubbles: true,
                            cancelable: true,
                            dataTransfer: dt,
                            clientX: cx,
                            clientY: cy
                        }};
                        el.dispatchEvent(new DragEvent('dragenter', eventInit));
                        el.dispatchEvent(new DragEvent('dragover', eventInit));
                        el.dispatchEvent(new DragEvent('drop', eventInit));
                    }}

                    if (window.ipc) {{
                        window.ipc.postMessage(JSON.stringify({{
                            type: 'agent_response',
                            request_id: {},
                            action: 'drop_file',
                            payload: {{
                                success: true,
                                target: targetStr,
                                file_name: fileName,
                                file_size: bytes.length,
                                mime_type: mimeType
                            }}
                        }}));
                    }}
                }} catch (err) {{
                    if (window.ipc) {{
                        window.ipc.postMessage(JSON.stringify({{
                            type: 'agent_response',
                            request_id: {},
                            action: 'drop_file',
                            error: String(err)
                        }}));
                    }}
                }}
            }})();"#,
            target_expr, serde_json::to_string(target).unwrap_or_default(),
            request_id,
            serde_json::to_string(file_name).unwrap_or_default(),
            serde_json::to_string(mime_type).unwrap_or_default(),
            serde_json::to_string(content).unwrap_or_default(),
            request_id, request_id
        )
    }

    /// JavaScript to serialize a nested hierarchical JSON tree of DOM nodes up to max_depth.
    pub fn get_dom_tree_script(request_id: u64, target: Option<&str>, max_depth: usize) -> String {
        let is_numeric = target.map(|t| t.chars().all(|c| c.is_ascii_digit())).unwrap_or(false);
        let target_expr = if let Some(t) = target {
            if is_numeric {
                format!("window.__aerolith_elements && window.__aerolith_elements[{}] || document.querySelector('[data-aerolith-ref=\"{}\"]')", t, t)
            } else {
                format!("document.querySelector({})", serde_json::to_string(t).unwrap_or_default())
            }
        } else {
            "document.body || document.documentElement".to_string()
        };

        format!(
            r#"(function() {{
                try {{
                    const root = {};
                    const maxDepth = {};
                    const targetStr = {};

                    if (!root) {{
                        if (window.ipc) {{
                            window.ipc.postMessage(JSON.stringify({{
                                type: 'agent_response',
                                request_id: {},
                                action: 'dom_tree',
                                error: 'Root element not found for target: ' + targetStr
                            }}));
                        }}
                        return;
                    }}

                    function serializeNode(el, depth) {{
                        if (!el || el.nodeType !== 1) return null;
                        const style = window.getComputedStyle(el);
                        if (style.display === 'none') return null;

                        const rect = el.getBoundingClientRect();
                        const node = {{
                            tag: el.tagName.toLowerCase(),
                            id: el.id || null,
                            class: el.className ? String(el.className).trim() : null,
                            ref: el.getAttribute('data-aerolith-ref') || null,
                            rect: {{
                                x: Math.round(rect.x),
                                y: Math.round(rect.y),
                                width: Math.round(rect.width),
                                height: Math.round(rect.height)
                            }}
                        }};

                        const directText = Array.from(el.childNodes)
                            .filter(n => n.nodeType === 3)
                            .map(n => (n.textContent || '').trim())
                            .filter(Boolean)
                            .join(' ');
                        if (directText) {{
                            node.text = directText.length > 80 ? directText.substring(0, 80) + '...' : directText;
                        }}

                        if (depth < maxDepth && el.children && el.children.length > 0) {{
                            const children = [];
                            for (let i = 0; i < el.children.length; i++) {{
                                const childNode = serializeNode(el.children[i], depth + 1);
                                if (childNode) children.push(childNode);
                            }}
                            if (children.length > 0) {{
                                node.children = children;
                            }}
                        }}

                        return node;
                    }}

                    const tree = serializeNode(root, 0);

                    if (window.ipc) {{
                        window.ipc.postMessage(JSON.stringify({{
                            type: 'agent_response',
                            request_id: {},
                            action: 'dom_tree',
                            payload: {{
                                success: true,
                                root: targetStr || 'body',
                                depth_limit: maxDepth,
                                tree: tree
                            }}
                        }}));
                    }}
                }} catch (err) {{
                    if (window.ipc) {{
                        window.ipc.postMessage(JSON.stringify({{
                            type: 'agent_response',
                            request_id: {},
                            action: 'dom_tree',
                            error: String(err)
                        }}));
                    }}
                }}
            }})();"#,
            target_expr, max_depth, serde_json::to_string(&target.unwrap_or("")).unwrap_or_default(),
            request_id, request_id, request_id
        )
    }

    /// JavaScript to wait until a specific text substring appears in an element or document body.
    pub fn get_wait_for_text_script(
        request_id: u64,
        target: Option<&str>,
        text: &str,
        timeout_ms: u64,
    ) -> String {
        let text_json = serde_json::to_string(text).unwrap_or_default();
        let target_json = serde_json::to_string(&target.unwrap_or("")).unwrap_or_default();
        format!(
            r#"(function() {{
                const startTime = Date.now();
                const timeout = {};
                const expected = ({}).toLowerCase();
                const target = {};

                function check() {{
                    try {{
                        let el = document.body || document.documentElement;
                        if (target) {{
                            const isNum = /^\d+$/.test(target);
                            const found = isNum
                                ? (window.__aerolith_elements && window.__aerolith_elements[target] || document.querySelector('[data-aerolith-ref="' + target + '"]'))
                                : document.querySelector(target);
                            if (found) el = found;
                        }}

                        const currentText = (el.innerText || el.textContent || '').toLowerCase();
                        if (currentText.includes(expected)) {{
                            if (window.ipc) {{
                                window.ipc.postMessage(JSON.stringify({{
                                    type: 'agent_response',
                                    request_id: {},
                                    action: 'wait_for_text',
                                    payload: {{
                                        success: true,
                                        target: target || 'body',
                                        matched_text: expected,
                                        elapsed_ms: Date.now() - startTime
                                    }}
                                }}));
                            }}
                            return;
                        }}

                        if (Date.now() - startTime >= timeout) {{
                            if (window.ipc) {{
                                window.ipc.postMessage(JSON.stringify({{
                                    type: 'agent_response',
                                    request_id: {},
                                    action: 'wait_for_text',
                                    error: 'Timeout of ' + timeout + 'ms exceeded waiting for text: "' + expected + '"'
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
                                action: 'wait_for_text',
                                error: String(err)
                            }}));
                        }}
                    }}
                }}

                check();
            }})();"#,
            timeout_ms, text_json, target_json, request_id, request_id, request_id
        )
    }

    /// JavaScript to inspect computed CSS styles on a target element for a list of property names.
    pub fn get_computed_styles_script(
        request_id: u64,
        target: &str,
        properties: &[String],
    ) -> String {
        let is_numeric = target.chars().all(|c| c.is_ascii_digit());
        let target_expr = if is_numeric {
            format!("window.__aerolith_elements && window.__aerolith_elements[{}] || document.querySelector('[data-aerolith-ref=\"{}\"]')", target, target)
        } else {
            format!("document.querySelector({})", serde_json::to_string(target).unwrap_or_default())
        };
        let props_json = serde_json::to_string(properties).unwrap_or_else(|_| "[]".into());

        format!(
            r#"(function() {{
                try {{
                    const el = {};
                    if (!el) {{
                        if (window.ipc) {{
                            window.ipc.postMessage(JSON.stringify({{
                                type: 'agent_response',
                                request_id: {},
                                action: 'computed_styles',
                                error: 'Element not found'
                            }}));
                        }}
                        return;
                    }}

                    const props = {};
                    const computed = window.getComputedStyle(el);
                    const styles = {{}};
                    for (let i = 0; i < props.length; i++) {{
                        const p = props[i];
                        styles[p] = computed.getPropertyValue(p) || computed[p] || '';
                    }}

                    if (window.ipc) {{
                        window.ipc.postMessage(JSON.stringify({{
                            type: 'agent_response',
                            request_id: {},
                            action: 'computed_styles',
                            payload: {{
                                success: true,
                                styles: styles
                            }}
                        }}));
                    }}
                }} catch (err) {{
                    if (window.ipc) {{
                        window.ipc.postMessage(JSON.stringify({{
                            type: 'agent_response',
                            request_id: {},
                            action: 'computed_styles',
                            error: String(err)
                        }}));
                    }}
                }}
            }})();"#,
            target_expr, request_id, props_json, request_id, request_id
        )
    }

    /// JavaScript to wait until a new file download appears in the browser download history.
    pub fn get_wait_for_download_script(
        request_id: u64,
        file_pattern: Option<&str>,
        timeout_ms: u64,
    ) -> String {
        let pat_json = serde_json::to_string(&file_pattern.unwrap_or("")).unwrap_or_default();
        format!(
            r#"(function() {{
                const startTime = Date.now();
                const timeout = {};
                const pat = ({}).toLowerCase();
                const initialCount = window.__aerolith_downloads ? window.__aerolith_downloads.length : 0;

                function check() {{
                    try {{
                        const dls = window.__aerolith_downloads || [];
                        if (dls.length > initialCount) {{
                            for (let i = initialCount; i < dls.length; i++) {{
                                const dl = dls[i];
                                const name = (dl.file_name || dl.url || '').toLowerCase();
                                if (!pat || name.includes(pat)) {{
                                    if (window.ipc) {{
                                        window.ipc.postMessage(JSON.stringify({{
                                            type: 'agent_response',
                                            request_id: {},
                                            action: 'wait_for_download',
                                            payload: {{
                                                success: true,
                                                download: dl,
                                                elapsed_ms: Date.now() - startTime
                                            }}
                                        }}));
                                    }}
                                    return;
                                }}
                            }}
                        }}

                        if (Date.now() - startTime >= timeout) {{
                            if (window.ipc) {{
                                window.ipc.postMessage(JSON.stringify({{
                                    type: 'agent_response',
                                    request_id: {},
                                    action: 'wait_for_download',
                                    error: 'Timeout of ' + timeout + 'ms exceeded waiting for download' + (pat ? ' matching: ' + pat : '')
                                }}));
                            }}
                            return;
                        }}

                        setTimeout(check, 100);
                    }} catch (err) {{
                        if (window.ipc) {{
                            window.ipc.postMessage(JSON.stringify({{
                                type: 'agent_response',
                                request_id: {},
                                action: 'wait_for_download',
                                error: String(err)
                            }}));
                        }}
                    }}
                }}

                check();
            }})();"#,
            timeout_ms, pat_json, request_id, request_id, request_id
        )
    }

    /// JavaScript to replace the entire document content with raw HTML.
    pub fn get_set_content_script(request_id: u64, html: &str) -> String {
        let html_json = serde_json::to_string(html).unwrap_or_default();
        format!(
            r#"(function() {{
                try {{
                    const html = {};
                    document.open();
                    document.write(html);
                    document.close();

                    if (window.ipc) {{
                        window.ipc.postMessage(JSON.stringify({{
                            type: 'agent_response',
                            request_id: {},
                            action: 'set_content',
                            payload: {{
                                success: true,
                                length: html.length,
                                title: document.title || ''
                            }}
                        }}));
                    }}
                }} catch (err) {{
                    if (window.ipc) {{
                        window.ipc.postMessage(JSON.stringify({{
                            type: 'agent_response',
                            request_id: {},
                            action: 'set_content',
                            error: String(err)
                        }}));
                    }}
                }}
            }})();"#,
            html_json, request_id, request_id
        )
    }

    /// JavaScript to inspect and structure all HTML forms and their controls on the page.
    pub fn get_forms_script(request_id: u64, target: Option<&str>) -> String {
        let target_json = serde_json::to_string(&target.unwrap_or("")).unwrap_or_default();
        format!(
            r#"(function() {{
                try {{
                    const target = {};
                    let root = document;
                    if (target) {{
                        const isNum = /^\d+$/.test(target);
                        const el = isNum
                            ? (window.__aerolith_elements && window.__aerolith_elements[target] || document.querySelector('[data-aerolith-ref="' + target + '"]'))
                            : document.querySelector(target);
                        if (el) root = el;
                    }}

                    function getElementRef(el) {{
                        if (el.hasAttribute('data-aerolith-ref')) {{
                            return el.getAttribute('data-aerolith-ref');
                        }}
                        if (!window.__aerolith_elements) window.__aerolith_elements = {{}};
                        if (!window.__aerolith_next_ref) window.__aerolith_next_ref = 1;
                        const ref = String(window.__aerolith_next_ref++);
                        el.setAttribute('data-aerolith-ref', ref);
                        window.__aerolith_elements[ref] = el;
                        return ref;
                    }}

                    const forms = [];
                    const formNodes = root.tagName === 'FORM' ? [root] : Array.from(root.querySelectorAll('form'));

                    formNodes.forEach((form, fIdx) => {{
                        const formInfo = {{
                            index: fIdx,
                            id: form.id || null,
                            name: form.name || null,
                            action: form.action || '',
                            method: (form.method || 'GET').toUpperCase(),
                            ref: getElementRef(form),
                            fields: []
                        }};

                        const controls = form.querySelectorAll('input, select, textarea, button');
                        controls.forEach(ctrl => {{
                            const tag = ctrl.tagName.toLowerCase();
                            const type = (ctrl.type || (tag === 'textarea' ? 'textarea' : tag === 'select' ? 'select' : 'text')).toLowerCase();
                            const field = {{
                                ref: getElementRef(ctrl),
                                tag: tag,
                                type: type,
                                name: ctrl.name || null,
                                id: ctrl.id || null,
                                value: ctrl.value || '',
                                placeholder: ctrl.placeholder || null,
                                required: Boolean(ctrl.required),
                                disabled: Boolean(ctrl.disabled),
                                checked: ctrl.checked !== undefined ? Boolean(ctrl.checked) : null
                            }};

                            if (tag === 'select') {{
                                field.options = Array.from(ctrl.options).map(opt => ({{
                                    value: opt.value,
                                    text: opt.text,
                                    selected: opt.selected
                                }}));
                            }}

                            formInfo.fields.push(field);
                        }});

                        forms.push(formInfo);
                    }});

                    if (window.ipc) {{
                        window.ipc.postMessage(JSON.stringify({{
                            type: 'agent_response',
                            request_id: {},
                            action: 'get_forms',
                            payload: {{
                                success: true,
                                count: forms.length,
                                forms: forms
                            }}
                        }}));
                    }}
                }} catch (err) {{
                    if (window.ipc) {{
                        window.ipc.postMessage(JSON.stringify({{
                            type: 'agent_response',
                            request_id: {},
                            action: 'get_forms',
                            error: String(err)
                        }}));
                    }}
                }}
            }})();"#,
            target_json, request_id, request_id
        )
    }

    /// JavaScript to force-emulate document.visibilityState ('visible' or 'hidden') and window focus.
    pub fn get_set_visibility_script(request_id: u64, state: &str) -> String {
        let state_json = serde_json::to_string(state).unwrap_or_default();
        format!(
            r#"(function() {{
                try {{
                    const state = {};
                    const isHidden = state === 'hidden';

                    try {{
                        Object.defineProperty(document, 'visibilityState', {{
                            get: function() {{ return state; }},
                            configurable: true
                        }});
                        Object.defineProperty(document, 'hidden', {{
                            get: function() {{ return isHidden; }},
                            configurable: true
                        }});
                    }} catch (_) {{}}

                    document.dispatchEvent(new Event('visibilitychange', {{ bubbles: true }}));
                    if (!isHidden) {{
                        window.dispatchEvent(new Event('focus'));
                    }} else {{
                        window.dispatchEvent(new Event('blur'));
                    }}

                    if (window.ipc) {{
                        window.ipc.postMessage(JSON.stringify({{
                            type: 'agent_response',
                            request_id: {},
                            action: 'set_visibility',
                            payload: {{
                                success: true,
                                visibility_state: state,
                                hidden: isHidden
                            }}
                        }}));
                    }}
                }} catch (err) {{
                    if (window.ipc) {{
                        window.ipc.postMessage(JSON.stringify({{
                            type: 'agent_response',
                            request_id: {},
                            action: 'set_visibility',
                            error: String(err)
                        }}));
                    }}
                }}
            }})();"#,
            state_json, request_id, request_id
        )
    }

    /// JavaScript to extract exact element bounding rect and visibility metrics.
    pub fn get_element_bounds_script(request_id: u64, target: &str) -> String {
        let target_json = serde_json::to_string(target).unwrap_or_default();
        format!(
            r#"(function() {{
                try {{
                    const target = {};
                    const isNum = /^\d+$/.test(target);
                    const el = isNum
                        ? (window.__aerolith_elements && window.__aerolith_elements[target] || document.querySelector('[data-aerolith-ref="' + target + '"]'))
                        : document.querySelector(target);
                    if (!el) {{
                        if (window.ipc) {{
                            window.ipc.postMessage(JSON.stringify({{
                                type: 'agent_response',
                                request_id: {},
                                action: 'get_element_bounds',
                                error: 'Element not found for target: ' + target
                            }}));
                        }}
                        return;
                    }}

                    const r = el.getBoundingClientRect();
                    const vw = window.innerWidth || document.documentElement.clientWidth;
                    const vh = window.innerHeight || document.documentElement.clientHeight;
                    const inViewport = r.bottom > 0 && r.right > 0 && r.top < vh && r.left < vw;
                    const style = window.getComputedStyle(el);
                    const visible = style.display !== 'none' && style.visibility !== 'hidden' && style.opacity !== '0' && r.width > 0 && r.height > 0;

                    const bounds = {{
                        x: Math.round(r.x),
                        y: Math.round(r.y),
                        width: Math.round(r.width),
                        height: Math.round(r.height),
                        top: Math.round(r.top),
                        right: Math.round(r.right),
                        bottom: Math.round(r.bottom),
                        left: Math.round(r.left),
                        center_x: Math.round(r.x + r.width / 2),
                        center_y: Math.round(r.y + r.height / 2),
                        in_viewport: inViewport,
                        visible: visible
                    }};

                    if (window.ipc) {{
                        window.ipc.postMessage(JSON.stringify({{
                            type: 'agent_response',
                            request_id: {},
                            action: 'get_element_bounds',
                            payload: {{
                                success: true,
                                target: target,
                                bounds: bounds
                            }}
                        }}));
                    }}
                }} catch (err) {{
                    if (window.ipc) {{
                        window.ipc.postMessage(JSON.stringify({{
                            type: 'agent_response',
                            request_id: {},
                            action: 'get_element_bounds',
                            error: String(err)
                        }}));
                    }}
                }}
            }})();"#,
            target_json, request_id, request_id, request_id
        )
    }

    /// JavaScript to intercept fetch and XHR requests matching a URL pattern and fulfill with mocked responses.
    pub fn get_mock_route_script(
        request_id: u64,
        action: &str,
        pattern: Option<&str>,
        status: u16,
        content_type: &str,
        body: Option<&str>,
    ) -> String {
        let action_json = serde_json::to_string(action).unwrap_or_default();
        let pattern_json = serde_json::to_string(&pattern.unwrap_or("")).unwrap_or_default();
        let ct_json = serde_json::to_string(content_type).unwrap_or_default();
        let body_json = serde_json::to_string(&body.unwrap_or("")).unwrap_or_default();

        format!(
            r#"(function() {{
                try {{
                    if (!window.__aerolith_mocks) {{
                        window.__aerolith_mocks = [];
                    }}

                    if (!window.__aerolith_mock_installed) {{
                        window.__aerolith_mock_installed = true;
                        const originalFetch = window.fetch;
                        window.fetch = async function(input, init) {{
                            const url = typeof input === 'string' ? input : (input && input.url ? input.url : '');
                            if (window.__aerolith_mocks && window.__aerolith_mocks.length > 0) {{
                                for (const m of window.__aerolith_mocks) {{
                                    if (url.includes(m.pattern)) {{
                                        return new Response(m.body, {{
                                            status: m.status,
                                            headers: {{ 'Content-Type': m.contentType }}
                                        }});
                                    }}
                                }}
                            }}
                            return originalFetch.apply(this, arguments);
                        }};

                        const originalXhrOpen = XMLHttpRequest.prototype.open;
                        const originalXhrSend = XMLHttpRequest.prototype.send;
                        XMLHttpRequest.prototype.open = function(method, url) {{
                            this.__aerolith_url = String(url);
                            return originalXhrOpen.apply(this, arguments);
                        }};
                        XMLHttpRequest.prototype.send = function(data) {{
                            const url = this.__aerolith_url || '';
                            if (window.__aerolith_mocks && window.__aerolith_mocks.length > 0) {{
                                for (const m of window.__aerolith_mocks) {{
                                    if (url.includes(m.pattern)) {{
                                        Object.defineProperty(this, 'status', {{ value: m.status, writable: true }});
                                        Object.defineProperty(this, 'responseText', {{ value: m.body, writable: true }});
                                        Object.defineProperty(this, 'response', {{ value: m.body, writable: true }});
                                        Object.defineProperty(this, 'readyState', {{ value: 4, writable: true }});
                                        setTimeout(() => {{
                                            if (typeof this.onreadystatechange === 'function') this.onreadystatechange();
                                            if (typeof this.onload === 'function') this.onload();
                                        }}, 10);
                                        return;
                                    }}
                                }}
                            }}
                            return originalXhrSend.apply(this, arguments);
                        }};
                    }}

                    const action = {};
                    if (action === 'clear') {{
                        window.__aerolith_mocks = [];
                    }} else if (action === 'set') {{
                        const pat = {};
                        if (pat) {{
                            window.__aerolith_mocks = window.__aerolith_mocks.filter(m => m.pattern !== pat);
                            window.__aerolith_mocks.push({{
                                pattern: pat,
                                status: {},
                                contentType: {},
                                body: {}
                            }});
                        }}
                    }}

                    const currentMocks = window.__aerolith_mocks || [];

                    if (window.ipc) {{
                        window.ipc.postMessage(JSON.stringify({{
                            type: 'agent_response',
                            request_id: {},
                            action: 'mock_route',
                            payload: {{
                                success: true,
                                action: action,
                                active_mocks: currentMocks
                            }}
                        }}));
                    }}
                }} catch (err) {{
                    if (window.ipc) {{
                        window.ipc.postMessage(JSON.stringify({{
                            type: 'agent_response',
                            request_id: {},
                            action: 'mock_route',
                            error: String(err)
                        }}));
                    }}
                }}
            }})();"#,
            action_json, pattern_json, status, ct_json, body_json, request_id, request_id
        )
    }

    /// JavaScript to retrieve viewport dimensions, scroll offsets, screen size, and orientation.
    pub fn get_viewport_script(request_id: u64) -> String {
        format!(
            r#"(function() {{
                try {{
                    const vp = {{
                        inner_width: window.innerWidth,
                        inner_height: window.innerHeight,
                        outer_width: window.outerWidth,
                        outer_height: window.outerHeight,
                        device_pixel_ratio: window.devicePixelRatio || 1,
                        scroll_x: window.scrollX || window.pageXOffset || 0,
                        scroll_y: window.scrollY || window.pageYOffset || 0,
                        screen_width: window.screen ? window.screen.width : window.innerWidth,
                        screen_height: window.screen ? window.screen.height : window.innerHeight,
                        orientation: (window.screen && window.screen.orientation ? window.screen.orientation.type : 'landscape-primary')
                    }};

                    if (window.ipc) {{
                        window.ipc.postMessage(JSON.stringify({{
                            type: 'agent_response',
                            request_id: {},
                            action: 'get_viewport',
                            payload: {{
                                success: true,
                                viewport: vp
                            }}
                        }}));
                    }}
                }} catch (err) {{
                    if (window.ipc) {{
                        window.ipc.postMessage(JSON.stringify({{
                            type: 'agent_response',
                            request_id: {},
                            action: 'get_viewport',
                            error: String(err)
                        }}));
                    }}
                }}
            }})();"#,
            request_id, request_id
        )
    }

    /// JavaScript to set or remove an element HTML attribute for self-healing DOM mutation.
    pub fn get_set_element_attribute_script(
        request_id: u64,
        target: &str,
        attribute: &str,
        value: Option<&str>,
    ) -> String {
        let target_json = serde_json::to_string(target).unwrap_or_default();
        let attr_json = serde_json::to_string(attribute).unwrap_or_default();
        let val_json = match value {
            Some(v) => serde_json::to_string(v).unwrap_or_default(),
            None => "null".to_string(),
        };

        format!(
            r#"(function() {{
                try {{
                    const target = {};
                    const attr = {};
                    const val = {};
                    const isNum = /^\d+$/.test(target);
                    const el = isNum
                        ? (window.__aerolith_elements && window.__aerolith_elements[target] || document.querySelector('[data-aerolith-ref="' + target + '"]'))
                        : document.querySelector(target);
                    if (!el) {{
                        if (window.ipc) {{
                            window.ipc.postMessage(JSON.stringify({{
                                type: 'agent_response',
                                request_id: {},
                                action: 'set_element_attribute',
                                error: 'Element not found for target: ' + target
                            }}));
                        }}
                        return;
                    }}

                    const prevValue = el.getAttribute(attr);
                    if (val !== null) {{
                        el.setAttribute(attr, val);
                        if (attr === 'disabled') {{
                            el.disabled = true;
                        }} else if (attr === 'checked') {{
                            el.checked = true;
                        }}
                    }} else {{
                        el.removeAttribute(attr);
                        if (attr === 'disabled') {{
                            el.disabled = false;
                        }} else if (attr === 'checked') {{
                            el.checked = false;
                        }}
                    }}

                    if (window.ipc) {{
                        window.ipc.postMessage(JSON.stringify({{
                            type: 'agent_response',
                            request_id: {},
                            action: 'set_element_attribute',
                            payload: {{
                                success: true,
                                target: target,
                                attribute: attr,
                                previous_value: prevValue,
                                new_value: val,
                                removed: val === null
                            }}
                        }}));
                    }}
                }} catch (err) {{
                    if (window.ipc) {{
                        window.ipc.postMessage(JSON.stringify({{
                            type: 'agent_response',
                            request_id: {},
                            action: 'set_element_attribute',
                            error: String(err)
                        }}));
                    }}
                }}
            }})();"#,
            target_json, attr_json, val_json, request_id, request_id, request_id
        )
    }

    /// JavaScript to search all visible page text and return matched snippets, element refs, and bounding boxes.
    pub fn get_search_page_text_script(
        request_id: u64,
        query: &str,
        max_matches: usize,
        case_sensitive: bool,
    ) -> String {
        let query_json = serde_json::to_string(query).unwrap_or_default();
        format!(
            r#"(function() {{
                try {{
                    const rawQuery = {};
                    const maxMatches = {};
                    const caseSensitive = {};
                    const query = caseSensitive ? rawQuery : rawQuery.toLowerCase();

                    function getElementRef(el) {{
                        if (el.hasAttribute('data-aerolith-ref')) {{
                            return el.getAttribute('data-aerolith-ref');
                        }}
                        if (!window.__aerolith_elements) window.__aerolith_elements = {{}};
                        if (!window.__aerolith_next_ref) window.__aerolith_next_ref = 1;
                        const ref = String(window.__aerolith_next_ref++);
                        el.setAttribute('data-aerolith-ref', ref);
                        window.__aerolith_elements[ref] = el;
                        return ref;
                    }}

                    const matches = [];
                    const walker = document.createTreeWalker(
                        document.body || document.documentElement,
                        NodeFilter.SHOW_TEXT,
                        {{
                            acceptNode: function(node) {{
                                if (!node || !node.textContent) return NodeFilter.FILTER_REJECT;
                                const parent = node.parentElement;
                                if (!parent) return NodeFilter.FILTER_REJECT;
                                const tag = parent.tagName;
                                if (tag === 'SCRIPT' || tag === 'STYLE' || tag === 'NOSCRIPT') return NodeFilter.FILTER_REJECT;
                                const style = window.getComputedStyle(parent);
                                if (style.display === 'none' || style.visibility === 'hidden' || style.opacity === '0') {{
                                    return NodeFilter.FILTER_REJECT;
                                }}
                                return NodeFilter.FILTER_ACCEPT;
                            }}
                        }}
                    );

                    let currentNode;
                    while ((currentNode = walker.nextNode()) && matches.length < maxMatches) {{
                        const text = currentNode.textContent || '';
                        const compareText = caseSensitive ? text : text.toLowerCase();
                        let idx = compareText.indexOf(query);
                        if (idx !== -1) {{
                            const parent = currentNode.parentElement;
                            const r = parent.getBoundingClientRect();
                            const start = Math.max(0, idx - 40);
                            const end = Math.min(text.length, idx + query.length + 40);
                            const snippet = (start > 0 ? '...' : '') + text.substring(start, end).trim() + (end < text.length ? '...' : '');

                            matches.push({{
                                ref: getElementRef(parent),
                                tag: parent.tagName.toLowerCase(),
                                snippet: snippet,
                                exact_text: text.substring(idx, idx + query.length),
                                bounds: {{
                                    x: Math.round(r.x),
                                    y: Math.round(r.y),
                                    width: Math.round(r.width),
                                    height: Math.round(r.height)
                                }}
                            }});
                        }}
                    }}

                    if (window.ipc) {{
                        window.ipc.postMessage(JSON.stringify({{
                            type: 'agent_response',
                            request_id: {},
                            action: 'search_page_text',
                            payload: {{
                                success: true,
                                query: rawQuery,
                                count: matches.length,
                                matches: matches
                            }}
                        }}));
                    }}
                }} catch (err) {{
                    if (window.ipc) {{
                        window.ipc.postMessage(JSON.stringify({{
                            type: 'agent_response',
                            request_id: {},
                            action: 'search_page_text',
                            error: String(err)
                        }}));
                    }}
                }}
            }})();"#,
            query_json, max_matches, case_sensitive, request_id, request_id
        )
    }

    /// JavaScript to dump all localStorage and sessionStorage key-value pairs into structured maps.
    pub fn get_dump_storage_script(request_id: u64, storage_type: &str) -> String {
        let type_json = serde_json::to_string(storage_type).unwrap_or_default();
        format!(
            r#"(function() {{
                try {{
                    const type = ({}).toLowerCase();
                    const result = {{}};

                    if (type === 'local' || type === 'all') {{
                        const local = {{}};
                        try {{
                            for (let i = 0; i < localStorage.length; i++) {{
                                const k = localStorage.key(i);
                                if (k !== null) local[k] = localStorage.getItem(k);
                            }}
                        }} catch (_) {{}}
                        result.local = local;
                    }}

                    if (type === 'session' || type === 'all') {{
                        const session = {{}};
                        try {{
                            for (let i = 0; i < sessionStorage.length; i++) {{
                                const k = sessionStorage.key(i);
                                if (k !== null) session[k] = sessionStorage.getItem(k);
                            }}
                        }} catch (_) {{}}
                        result.session = session;
                    }}

                    if (window.ipc) {{
                        window.ipc.postMessage(JSON.stringify({{
                            type: 'agent_response',
                            request_id: {},
                            action: 'dump_storage',
                            payload: {{
                                success: true,
                                storage: result
                            }}
                        }}));
                    }}
                }} catch (err) {{
                    if (window.ipc) {{
                        window.ipc.postMessage(JSON.stringify({{
                            type: 'agent_response',
                            request_id: {},
                            action: 'dump_storage',
                            error: String(err)
                        }}));
                    }}
                }}
            }})();"#,
            type_json, request_id, request_id
        )
    }

    /// JavaScript to simulate human-like mouse micro-curves and micro-scrolls for anti-bot evasion.
    pub fn get_human_jitter_script(
        request_id: u64,
        duration_ms: u64,
        move_mouse: bool,
        micro_scroll: bool,
    ) -> String {
        format!(
            r#"(function() {{
                try {{
                    const duration = {};
                    const moveMouse = {};
                    const microScroll = {};
                    const startTime = Date.now();
                    const startX = window.innerWidth / 2;
                    const startY = window.innerHeight / 2;

                    let curX = startX;
                    let curY = startY;

                    function step() {{
                        const elapsed = Date.now() - startTime;
                        if (elapsed >= duration) {{
                            if (window.ipc) {{
                                window.ipc.postMessage(JSON.stringify({{
                                    type: 'agent_response',
                                    request_id: {},
                                    action: 'human_jitter',
                                    payload: {{
                                        success: true,
                                        duration_ms: elapsed,
                                        mouse_moved: moveMouse,
                                        micro_scrolled: microScroll
                                    }}
                                }}));
                            }}
                            return;
                        }}

                        if (moveMouse) {{
                            curX += (Math.random() - 0.5) * 8;
                            curY += (Math.random() - 0.5) * 6;
                            const evt = new MouseEvent('mousemove', {{
                                clientX: Math.round(curX),
                                clientY: Math.round(curY),
                                bubbles: true,
                                cancelable: true
                            }});
                            document.dispatchEvent(evt);
                        }}

                        if (microScroll && Math.random() < 0.2) {{
                            const dy = (Math.random() - 0.5) * 4;
                            window.scrollBy({{ top: dy, behavior: 'instant' }});
                        }}

                        setTimeout(step, 40);
                    }}

                    step();
                }} catch (err) {{
                    if (window.ipc) {{
                        window.ipc.postMessage(JSON.stringify({{
                            type: 'agent_response',
                            request_id: {},
                            action: 'human_jitter',
                            error: String(err)
                        }}));
                    }}
                }}
            }})();"#,
            duration_ms, move_mouse, micro_scroll, request_id, request_id
        )
    }

    /// JavaScript to type text character-by-character into target element with simulated keystroke events.
    pub fn get_type_keystrokes_script(
        request_id: u64,
        target: &str,
        text: &str,
        delay_ms: u64,
        clear: bool,
        submit: bool,
    ) -> String {
        let target_json = serde_json::to_string(target).unwrap_or_default();
        let text_json = serde_json::to_string(text).unwrap_or_default();

        format!(
            r#"(function() {{
                try {{
                    const target = {};
                    const text = {};
                    const delay = {};
                    const clearFirst = {};
                    const submitRequested = {};

                    const isNum = /^\d+$/.test(target);
                    const el = isNum
                        ? (window.__aerolith_elements && window.__aerolith_elements[target] || document.querySelector('[data-aerolith-ref="' + target + '"]'))
                        : document.querySelector(target);
                    if (!el) {{
                        if (window.ipc) {{
                            window.ipc.postMessage(JSON.stringify({{
                                type: 'agent_response',
                                request_id: {},
                                action: 'type_keystrokes',
                                error: 'Element not found for target: ' + target
                            }}));
                        }}
                        return;
                    }}

                    el.scrollIntoView({{ behavior: 'instant', block: 'center' }});
                    if (typeof el.focus === 'function') el.focus();

                    if (clearFirst) {{
                        if (el.value !== undefined) el.value = '';
                        else if (el.isContentEditable) el.textContent = '';
                        el.dispatchEvent(new Event('input', {{ bubbles: true }}));
                    }}

                    let idx = 0;
                    function typeChar() {{
                        if (idx >= text.length) {{
                            el.dispatchEvent(new Event('change', {{ bubbles: true }}));
                            if (submitRequested) {{
                                const enterOpts = {{ key: 'Enter', code: 'Enter', keyCode: 13, which: 13, bubbles: true, cancelable: true }};
                                el.dispatchEvent(new KeyboardEvent('keydown', enterOpts));
                                el.dispatchEvent(new KeyboardEvent('keypress', enterOpts));
                                el.dispatchEvent(new KeyboardEvent('keyup', enterOpts));
                                if (el.form && typeof el.form.requestSubmit === 'function') {{
                                    el.form.requestSubmit();
                                }}
                            }}

                            if (window.ipc) {{
                                window.ipc.postMessage(JSON.stringify({{
                                    type: 'agent_response',
                                    request_id: {},
                                    action: 'type_keystrokes',
                                    payload: {{
                                        success: true,
                                        target: target,
                                        chars_typed: text.length,
                                        value: el.value !== undefined ? el.value : el.textContent,
                                        submitted: submitRequested
                                    }}
                                }}));
                            }}
                            return;
                        }}

                        const ch = text[idx++];
                        const opts = {{ key: ch, bubbles: true, cancelable: true }};
                        el.dispatchEvent(new KeyboardEvent('keydown', opts));
                        el.dispatchEvent(new KeyboardEvent('keypress', opts));

                        if (el.value !== undefined) {{
                            el.value += ch;
                        }} else if (el.isContentEditable) {{
                            el.textContent += ch;
                        }}
                        el.dispatchEvent(new Event('input', {{ bubbles: true }}));
                        el.dispatchEvent(new KeyboardEvent('keyup', opts));

                        setTimeout(typeChar, delay);
                    }}

                    typeChar();
                }} catch (err) {{
                    if (window.ipc) {{
                        window.ipc.postMessage(JSON.stringify({{
                            type: 'agent_response',
                            request_id: {},
                            action: 'type_keystrokes',
                            error: String(err)
                        }}));
                    }}
                }}
            }})();"#,
            target_json, text_json, delay_ms, clear, submit, request_id, request_id, request_id
        )
    }

    /// JavaScript to extract base64 image data URL from <canvas>, <img>, or arbitrary element via offscreen rasterization.
    pub fn get_canvas_data_script(
        request_id: u64,
        target: &str,
        mime_type: &str,
        quality: f64,
    ) -> String {
        let target_json = serde_json::to_string(target).unwrap_or_default();
        let mime_json = serde_json::to_string(mime_type).unwrap_or_default();
        format!(
            r#"(function() {{
                try {{
                    const target = {};
                    const mime = {} || 'image/png';
                    const quality = {};

                    const isNum = /^\d+$/.test(target);
                    const el = isNum
                        ? (window.__aerolith_elements && window.__aerolith_elements[target] || document.querySelector('[data-aerolith-ref="' + target + '"]'))
                        : document.querySelector(target);

                    if (!el) {{
                        if (window.ipc) {{
                            window.ipc.postMessage(JSON.stringify({{
                                type: 'agent_response',
                                request_id: {},
                                action: 'get_canvas_data',
                                error: "Element not found for target: " + target
                            }}));
                        }}
                        return;
                    }}

                    let dataUrl = '';
                    let width = 0;
                    let height = 0;

                    if (el.tagName && el.tagName.toLowerCase() === 'canvas') {{
                        width = el.width || el.clientWidth;
                        height = el.height || el.clientHeight;
                        dataUrl = el.toDataURL(mime, quality);
                    }} else if (el.tagName && el.tagName.toLowerCase() === 'img') {{
                        width = el.naturalWidth || el.width || el.clientWidth;
                        height = el.naturalHeight || el.height || el.clientHeight;
                        const offscreen = document.createElement('canvas');
                        offscreen.width = width || 100;
                        offscreen.height = height || 100;
                        const ctx = offscreen.getContext('2d');
                        if (ctx) {{
                            ctx.drawImage(el, 0, 0, width, height);
                            dataUrl = offscreen.toDataURL(mime, quality);
                        }}
                    }} else {{
                        const rect = el.getBoundingClientRect();
                        width = Math.round(rect.width);
                        height = Math.round(rect.height);
                        const offscreen = document.createElement('canvas');
                        offscreen.width = width || 100;
                        offscreen.height = height || 100;
                        const ctx = offscreen.getContext('2d');
                        if (ctx) {{
                            ctx.fillStyle = '#ffffff';
                            ctx.fillRect(0, 0, width, height);
                            dataUrl = offscreen.toDataURL(mime, quality);
                        }}
                    }}

                    if (window.ipc) {{
                        window.ipc.postMessage(JSON.stringify({{
                            type: 'agent_response',
                            request_id: {},
                            action: 'get_canvas_data',
                            payload: {{
                                success: true,
                                target: target,
                                width: width,
                                height: height,
                                mime_type: mime,
                                data_url: dataUrl
                            }}
                        }}));
                    }}
                }} catch (err) {{
                    if (window.ipc) {{
                        window.ipc.postMessage(JSON.stringify({{
                            type: 'agent_response',
                            request_id: {},
                            action: 'get_canvas_data',
                            error: String(err)
                        }}));
                    }}
                }}
            }})();"#,
            target_json, mime_json, quality, request_id, request_id, request_id
        )
    }

    /// JavaScript to extract structured resource timing performance metrics from window.performance.
    pub fn get_resource_timing_script(
        request_id: u64,
        initiator: Option<&str>,
        max_entries: usize,
        min_duration: f64,
    ) -> String {
        let initiator_json = match initiator {
            Some(i) => serde_json::to_string(i).unwrap_or_else(|_| "null".to_string()),
            None => "null".to_string(),
        };

        format!(
            r#"(function() {{
                try {{
                    const initiatorFilter = {};
                    const maxEntries = {};
                    const minDuration = {};

                    const all = (window.performance && typeof window.performance.getEntriesByType === 'function')
                        ? window.performance.getEntriesByType('resource')
                        : [];

                    const filtered = [];
                    for (let i = 0; i < all.length; i++) {{
                        const entry = all[i];
                        if (initiatorFilter && entry.initiatorType !== initiatorFilter) continue;
                        if (entry.duration < minDuration) continue;

                        filtered.push({{
                            name: entry.name,
                            initiator_type: entry.initiatorType || 'other',
                            start_time_ms: Math.round(entry.startTime * 100) / 100,
                            duration_ms: Math.round(entry.duration * 100) / 100,
                            transfer_size_bytes: entry.transferSize || 0,
                            decoded_body_size_bytes: entry.decodedBodySize || 0
                        }});

                        if (filtered.length >= maxEntries) break;
                    }}

                    if (window.ipc) {{
                        window.ipc.postMessage(JSON.stringify({{
                            type: 'agent_response',
                            request_id: {},
                            action: 'get_resource_timing',
                            payload: {{
                                total_recorded: all.length,
                                returned_count: filtered.length,
                                resources: filtered
                            }}
                        }}));
                    }}
                }} catch (err) {{
                    if (window.ipc) {{
                        window.ipc.postMessage(JSON.stringify({{
                            type: 'agent_response',
                            request_id: {},
                            action: 'get_resource_timing',
                            error: String(err)
                        }}));
                    }}
                }}
            }})();"#,
            initiator_json, max_entries, min_duration, request_id, request_id
        )
    }

    /// JavaScript to analyze DOM complexity and return structural telemetry.
    pub fn get_dom_stats_script(request_id: u64) -> String {
        format!(
            r#"(function() {{
                try {{
                    const all = document.querySelectorAll('*');
                    let maxDepth = 0;
                    function measureDepth(node, current) {{
                        if (current > maxDepth) maxDepth = current;
                        for (let child of node.children) {{
                            measureDepth(child, current + 1);
                        }}
                    }}
                    if (document.documentElement) measureDepth(document.documentElement, 1);

                    const countTag = (tag) => document.querySelectorAll(tag).length;

                    const stats = {{
                        total_elements: all.length,
                        max_depth: maxDepth,
                        forms_count: countTag('form'),
                        inputs_count: countTag('input'),
                        buttons_count: countTag('button'),
                        links_count: countTag('a'),
                        images_count: countTag('img'),
                        tables_count: countTag('table'),
                        iframes_count: countTag('iframe'),
                        canvases_count: countTag('canvas'),
                        svgs_count: countTag('svg'),
                        scripts_count: countTag('script'),
                        stylesheets_count: countTag('link[rel="stylesheet"], style'),
                        document_height: Math.max(
                            document.body ? document.body.scrollHeight : 0,
                            document.documentElement ? document.documentElement.scrollHeight : 0
                        ),
                        document_width: Math.max(
                            document.body ? document.body.scrollWidth : 0,
                            document.documentElement ? document.documentElement.scrollWidth : 0
                        )
                    }};

                    if (window.ipc) {{
                        window.ipc.postMessage(JSON.stringify({{
                            type: 'agent_response',
                            request_id: {},
                            action: 'get_dom_stats',
                            payload: stats
                        }}));
                    }}
                }} catch (err) {{
                    if (window.ipc) {{
                        window.ipc.postMessage(JSON.stringify({{
                            type: 'agent_response',
                            request_id: {},
                            action: 'get_dom_stats',
                            error: String(err)
                        }}));
                    }}
                }}
            }})();"#,
            request_id, request_id
        )
    }

    /// JavaScript to perform relative window scroll by dx, dy deltas.
    pub fn get_scroll_by_script(request_id: u64, dx: i32, dy: i32, smooth: bool) -> String {
        format!(
            r#"(function() {{
                try {{
                    const dx = {};
                    const dy = {};
                    const smooth = {};
                    window.scrollBy({{
                        left: dx,
                        top: dy,
                        behavior: smooth ? 'smooth' : 'instant'
                    }});

                    setTimeout(() => {{
                        const sx = Math.round(window.scrollX || window.pageXOffset || 0);
                        const sy = Math.round(window.scrollY || window.pageYOffset || 0);
                        if (window.ipc) {{
                            window.ipc.postMessage(JSON.stringify({{
                                type: 'agent_response',
                                request_id: {},
                                action: 'scroll_by',
                                payload: {{
                                    success: true,
                                    dx: dx,
                                    dy: dy,
                                    scroll_x: sx,
                                    scroll_y: sy
                                }}
                            }}));
                        }}
                    }}, smooth ? 150 : 20);
                }} catch (err) {{
                    if (window.ipc) {{
                        window.ipc.postMessage(JSON.stringify({{
                            type: 'agent_response',
                            request_id: {},
                            action: 'scroll_by',
                            error: String(err)
                        }}));
                    }}
                }}
            }})();"#,
            dx, dy, smooth, request_id, request_id
        )
    }

    /// JavaScript to scroll a specific inner scrollable container (e.g. div, modal, textarea, section).
    pub fn get_scroll_container_script(
        request_id: u64,
        target: &str,
        dx: i32,
        dy: i32,
        smooth: bool,
    ) -> String {
        let target_json = serde_json::to_string(target).unwrap_or_default();
        format!(
            r#"(function() {{
                try {{
                    const target = {};
                    const dx = {};
                    const dy = {};
                    const smooth = {};

                    const isNum = /^\d+$/.test(target);
                    const el = isNum
                        ? (window.__aerolith_elements && window.__aerolith_elements[target] || document.querySelector('[data-aerolith-ref="' + target + '"]'))
                        : document.querySelector(target);

                    if (!el) {{
                        if (window.ipc) {{
                            window.ipc.postMessage(JSON.stringify({{
                                type: 'agent_response',
                                request_id: {},
                                action: 'scroll_container',
                                error: "Container element not found for target: " + target
                            }}));
                        }}
                        return;
                    }}

                    if (typeof el.scrollBy === 'function') {{
                        el.scrollBy({{
                            left: dx,
                            top: dy,
                            behavior: smooth ? 'smooth' : 'instant'
                        }});
                    }} else {{
                        el.scrollLeft += dx;
                        el.scrollTop += dy;
                    }}

                    setTimeout(() => {{
                        const sl = Math.round(el.scrollLeft || 0);
                        const st = Math.round(el.scrollTop || 0);
                        const sw = Math.round(el.scrollWidth || 0);
                        const sh = Math.round(el.scrollHeight || 0);
                        const cw = Math.round(el.clientWidth || 0);
                        const ch = Math.round(el.clientHeight || 0);
                        const atBottom = (st + ch) >= (sh - 2);
                        const atTop = st <= 2;

                        if (window.ipc) {{
                            window.ipc.postMessage(JSON.stringify({{
                                type: 'agent_response',
                                request_id: {},
                                action: 'scroll_container',
                                payload: {{
                                    success: true,
                                    target: target,
                                    dx: dx,
                                    dy: dy,
                                    scroll_left: sl,
                                    scroll_top: st,
                                    scroll_width: sw,
                                    scroll_height: sh,
                                    client_width: cw,
                                    client_height: ch,
                                    at_top: atTop,
                                    at_bottom: atBottom
                                }}
                            }}));
                        }}
                    }}, smooth ? 150 : 20);
                }} catch (err) {{
                    if (window.ipc) {{
                        window.ipc.postMessage(JSON.stringify({{
                            type: 'agent_response',
                            request_id: {},
                            action: 'scroll_container',
                            error: String(err)
                        }}));
                    }}
                }}
            }})();"#,
            target_json, dx, dy, smooth, request_id, request_id, request_id
        )
    }

    /// JavaScript to select text range inside input, textarea, or contentEditable element.
    pub fn get_select_element_text_script(
        request_id: u64,
        target: &str,
        start: Option<usize>,
        length: Option<usize>,
    ) -> String {
        let target_json = serde_json::to_string(target).unwrap_or_default();
        let start_json = match start {
            Some(s) => s.to_string(),
            None => "null".to_string(),
        };
        let len_json = match length {
            Some(l) => l.to_string(),
            None => "null".to_string(),
        };

        format!(
            r#"(function() {{
                try {{
                    const target = {};
                    const startPos = {};
                    const len = {};

                    const isNum = /^\d+$/.test(target);
                    const el = isNum
                        ? (window.__aerolith_elements && window.__aerolith_elements[target] || document.querySelector('[data-aerolith-ref="' + target + '"]'))
                        : document.querySelector(target);

                    if (!el) {{
                        if (window.ipc) {{
                            window.ipc.postMessage(JSON.stringify({{
                                type: 'agent_response',
                                request_id: {},
                                action: 'select_element_text',
                                error: "Element not found for target: " + target
                            }}));
                        }}
                        return;
                    }}

                    if (typeof el.focus === 'function') el.focus();
                    let selectedText = '';
                    let actualStart = startPos !== null ? startPos : 0;
                    let actualEnd = 0;

                    if (el.tagName && (el.tagName.toLowerCase() === 'input' || el.tagName.toLowerCase() === 'textarea')) {{
                        const val = el.value || '';
                        actualEnd = len !== null ? Math.min(val.length, actualStart + len) : val.length;
                        if (typeof el.setSelectionRange === 'function') {{
                            el.setSelectionRange(actualStart, actualEnd);
                        }}
                        selectedText = val.substring(actualStart, actualEnd);
                    }} else {{
                        const sel = window.getSelection();
                        if (sel) {{
                            sel.removeAllRanges();
                            const range = document.createRange();
                            range.selectNodeContents(el);
                            sel.addRange(range);
                            selectedText = sel.toString();
                            actualEnd = selectedText.length;
                        }}
                    }}

                    if (window.ipc) {{
                        window.ipc.postMessage(JSON.stringify({{
                            type: 'agent_response',
                            request_id: {},
                            action: 'select_element_text',
                            payload: {{
                                success: true,
                                target: target,
                                selected_text: selectedText,
                                start: actualStart,
                                end: actualEnd
                            }}
                        }}));
                    }}
                }} catch (err) {{
                    if (window.ipc) {{
                        window.ipc.postMessage(JSON.stringify({{
                            type: 'agent_response',
                            request_id: {},
                            action: 'select_element_text',
                            error: String(err)
                        }}));
                    }}
                }}
            }})();"#,
            target_json, start_json, len_json, request_id, request_id, request_id
        )
    }

    /// JavaScript to extract structured, deduplicated hyperlink graph from the active document.
    pub fn get_extract_page_links_script(
        request_id: u64,
        max_links: usize,
        external_only: bool,
        filter_path: Option<&str>,
    ) -> String {
        let filter_json = match filter_path {
            Some(f) => serde_json::to_string(f).unwrap_or_else(|_| "null".to_string()),
            None => "null".to_string(),
        };

        format!(
            r#"(function() {{
                try {{
                    const maxLinks = {};
                    const extOnly = {};
                    const filter = {};

                    const anchors = document.querySelectorAll('a[href]');
                    const currentOrigin = window.location.origin;
                    const seen = new Set();
                    const links = [];

                    for (let i = 0; i < anchors.length; i++) {{
                        const a = anchors[i];
                        let fullUrl = '';
                        try {{
                            fullUrl = new URL(a.getAttribute('href') || a.href, window.location.href).href;
                        }} catch (_) {{
                            continue;
                        }}

                        if (!fullUrl.startsWith('http://') && !fullUrl.startsWith('https://')) continue;
                        if (seen.has(fullUrl)) continue;

                        const isExternal = !fullUrl.startsWith(currentOrigin);
                        if (extOnly && !isExternal) continue;
                        if (filter && !fullUrl.includes(filter)) continue;

                        seen.add(fullUrl);
                        const text = (a.innerText || a.textContent || a.getAttribute('aria-label') || a.getAttribute('title') || '').trim();

                        links.push({{
                            url: fullUrl,
                            text: text.length > 120 ? text.substring(0, 120) + '...' : text,
                            is_external: isExternal,
                            target: a.getAttribute('target') || '_self',
                            rel: a.getAttribute('rel') || ''
                        }});

                        if (links.length >= maxLinks) break;
                    }}

                    if (window.ipc) {{
                        window.ipc.postMessage(JSON.stringify({{
                            type: 'agent_response',
                            request_id: {},
                            action: 'extract_page_links',
                            payload: {{
                                total_found: anchors.length,
                                returned_count: links.length,
                                current_origin: currentOrigin,
                                links: links
                            }}
                        }}));
                    }}
                }} catch (err) {{
                    if (window.ipc) {{
                        window.ipc.postMessage(JSON.stringify({{
                            type: 'agent_response',
                            request_id: {},
                            action: 'extract_page_links',
                            error: String(err)
                        }}));
                    }}
                }}
            }})();"#,
            max_links, external_only, filter_json, request_id, request_id
        )
    }

    /// JavaScript to inspect complete interactive and visual state of a DOM element in one atomic query.
    pub fn get_element_state_script(request_id: u64, target: &str) -> String {
        let target_json = serde_json::to_string(target).unwrap_or_default();
        format!(
            r#"(function() {{
                try {{
                    const target = {};
                    const isNum = /^\d+$/.test(target);
                    const el = isNum
                        ? (window.__aerolith_elements && window.__aerolith_elements[target] || document.querySelector('[data-aerolith-ref="' + target + '"]'))
                        : document.querySelector(target);

                    if (!el) {{
                        if (window.ipc) {{
                            window.ipc.postMessage(JSON.stringify({{
                                type: 'agent_response',
                                request_id: {},
                                action: 'get_element_state',
                                error: "Element not found for target: " + target
                            }}));
                        }}
                        return;
                    }}

                    const rect = el.getBoundingClientRect();
                    const style = window.getComputedStyle ? window.getComputedStyle(el) : null;
                    const isVisible = style
                        ? (style.display !== 'none' && style.visibility !== 'hidden' && style.opacity !== '0' && rect.width > 0 && rect.height > 0)
                        : (rect.width > 0 && rect.height > 0);
                    const inViewport = (
                        rect.bottom > 0 &&
                        rect.right > 0 &&
                        rect.top < (window.innerHeight || document.documentElement.clientHeight) &&
                        rect.left < (window.innerWidth || document.documentElement.clientWidth)
                    );

                    const classes = [];
                    if (el.classList) {{
                        for (let i = 0; i < el.classList.length; i++) {{
                            classes.push(el.classList[i]);
                        }}
                    }}

                    const state = {{
                        success: true,
                        target: target,
                        tag_name: el.tagName ? el.tagName.toLowerCase() : '',
                        id: el.id || '',
                        classes: classes,
                        is_visible: isVisible,
                        is_in_viewport: inViewport,
                        is_disabled: !!el.disabled || el.getAttribute('aria-disabled') === 'true',
                        is_focused: document.activeElement === el,
                        is_checked: !!el.checked || el.getAttribute('aria-checked') === 'true',
                        is_readonly: !!el.readOnly || el.getAttribute('aria-readonly') === 'true',
                        text: (el.innerText || el.textContent || '').trim().substring(0, 300),
                        value: el.value !== undefined ? String(el.value) : null,
                        bounds: {{
                            x: Math.round(rect.left),
                            y: Math.round(rect.top),
                            width: Math.round(rect.width),
                            height: Math.round(rect.height),
                            center_x: Math.round(rect.left + rect.width / 2),
                            center_y: Math.round(rect.top + rect.height / 2)
                        }}
                    }};

                    if (window.ipc) {{
                        window.ipc.postMessage(JSON.stringify({{
                            type: 'agent_response',
                            request_id: {},
                            action: 'get_element_state',
                            payload: state
                        }}));
                    }}
                }} catch (err) {{
                    if (window.ipc) {{
                        window.ipc.postMessage(JSON.stringify({{
                            type: 'agent_response',
                            request_id: {},
                            action: 'get_element_state',
                            error: String(err)
                        }}));
                    }}
                }}
            }})();"#,
            target_json, request_id, request_id, request_id
        )
    }

    /// JavaScript to add, remove, toggle, or check CSS class on target DOM element.
    pub fn get_toggle_element_class_script(
        request_id: u64,
        target: &str,
        class_name: &str,
        action: &str,
    ) -> String {
        let target_json = serde_json::to_string(target).unwrap_or_default();
        let class_json = serde_json::to_string(class_name).unwrap_or_default();
        let action_json = serde_json::to_string(action).unwrap_or_default();

        format!(
            r#"(function() {{
                try {{
                    const target = {};
                    const className = {};
                    const action = ({} || 'toggle').toLowerCase();

                    const isNum = /^\d+$/.test(target);
                    const el = isNum
                        ? (window.__aerolith_elements && window.__aerolith_elements[target] || document.querySelector('[data-aerolith-ref="' + target + '"]'))
                        : document.querySelector(target);

                    if (!el) {{
                        if (window.ipc) {{
                            window.ipc.postMessage(JSON.stringify({{
                                type: 'agent_response',
                                request_id: {},
                                action: 'toggle_element_class',
                                error: "Element not found for target: " + target
                            }}));
                        }}
                        return;
                    }}

                    if (!el.classList) {{
                        if (window.ipc) {{
                            window.ipc.postMessage(JSON.stringify({{
                                type: 'agent_response',
                                request_id: {},
                                action: 'toggle_element_class',
                                error: "Target element does not support classList"
                            }}));
                        }}
                        return;
                    }}

                    let present = false;
                    if (action === 'add') {{
                        el.classList.add(className);
                        present = true;
                    }} else if (action === 'remove') {{
                        el.classList.remove(className);
                        present = false;
                    }} else if (action === 'contains' || action === 'check') {{
                        present = el.classList.contains(className);
                    }} else {{
                        present = el.classList.toggle(className);
                    }}

                    const classes = [];
                    for (let i = 0; i < el.classList.length; i++) {{
                        classes.push(el.classList[i]);
                    }}

                    if (window.ipc) {{
                        window.ipc.postMessage(JSON.stringify({{
                            type: 'agent_response',
                            request_id: {},
                            action: 'toggle_element_class',
                            payload: {{
                                success: true,
                                target: target,
                                class_name: className,
                                action: action,
                                present: present,
                                classes: classes
                            }}
                        }}));
                    }}
                }} catch (err) {{
                    if (window.ipc) {{
                        window.ipc.postMessage(JSON.stringify({{
                            type: 'agent_response',
                            request_id: {},
                            action: 'toggle_element_class',
                            error: String(err)
                        }}));
                    }}
                }}
            }})();"#,
            target_json, class_json, action_json, request_id, request_id, request_id, request_id
        )
    }

    /// JavaScript to directly set textContent on target element and trigger input/change events.
    pub fn get_set_element_text_script(
        request_id: u64,
        target: &str,
        text: &str,
    ) -> String {
        let target_json = serde_json::to_string(target).unwrap_or_default();
        let text_json = serde_json::to_string(text).unwrap_or_default();

        format!(
            r#"(function() {{
                try {{
                    const target = {};
                    const text = {};

                    const isNum = /^\d+$/.test(target);
                    const el = isNum
                        ? (window.__aerolith_elements && window.__aerolith_elements[target] || document.querySelector('[data-aerolith-ref="' + target + '"]'))
                        : document.querySelector(target);

                    if (!el) {{
                        if (window.ipc) {{
                            window.ipc.postMessage(JSON.stringify({{
                                type: 'agent_response',
                                request_id: {},
                                action: 'set_element_text',
                                error: "Element not found for target: " + target
                            }}));
                        }}
                        return;
                    }}

                    if (el.tagName && (el.tagName.toLowerCase() === 'input' || el.tagName.toLowerCase() === 'textarea')) {{
                        el.value = text;
                    }} else {{
                        el.textContent = text;
                    }}

                    el.dispatchEvent(new Event('input', {{ bubbles: true }}));
                    el.dispatchEvent(new Event('change', {{ bubbles: true }}));

                    if (window.ipc) {{
                        window.ipc.postMessage(JSON.stringify({{
                            type: 'agent_response',
                            request_id: {},
                            action: 'set_element_text',
                            payload: {{
                                success: true,
                                target: target,
                                text: text,
                                tag_name: el.tagName ? el.tagName.toLowerCase() : ''
                            }}
                        }}));
                    }}
                }} catch (err) {{
                    if (window.ipc) {{
                        window.ipc.postMessage(JSON.stringify({{
                            type: 'agent_response',
                            request_id: {},
                            action: 'set_element_text',
                            error: String(err)
                        }}));
                    }}
                }}
            }})();"#,
            target_json, text_json, request_id, request_id, request_id
        )
    }

    /// JavaScript to dispatch a CustomEvent on target element with optional detail payload.
    pub fn get_dispatch_custom_event_script(
        request_id: u64,
        target: &str,
        event_type: &str,
        bubbles: bool,
        cancelable: bool,
        detail: Option<&serde_json::Value>,
    ) -> String {
        let target_json = serde_json::to_string(target).unwrap_or_default();
        let type_json = serde_json::to_string(event_type).unwrap_or_default();
        let detail_json = match detail {
            Some(d) => serde_json::to_string(d).unwrap_or_else(|_| "null".to_string()),
            None => "null".to_string(),
        };

        format!(
            r#"(function() {{
                try {{
                    const target = {};
                    const eventType = {};
                    const bubbles = {};
                    const cancelable = {};
                    const detail = {};

                    const isNum = /^\d+$/.test(target);
                    const el = isNum
                        ? (window.__aerolith_elements && window.__aerolith_elements[target] || document.querySelector('[data-aerolith-ref="' + target + '"]'))
                        : document.querySelector(target);

                    if (!el) {{
                        if (window.ipc) {{
                            window.ipc.postMessage(JSON.stringify({{
                                type: 'agent_response',
                                request_id: {},
                                action: 'dispatch_custom_event',
                                error: "Element not found for target: " + target
                            }}));
                        }}
                        return;
                    }}

                    const evt = new CustomEvent(eventType, {{
                        bubbles: bubbles,
                        cancelable: cancelable,
                        detail: detail
                    }});

                    const notCancelled = el.dispatchEvent(evt);

                    if (window.ipc) {{
                        window.ipc.postMessage(JSON.stringify({{
                            type: 'agent_response',
                            request_id: {},
                            action: 'dispatch_custom_event',
                            payload: {{
                                success: true,
                                target: target,
                                event_type: eventType,
                                default_prevented: !notCancelled
                            }}
                        }}));
                    }}
                }} catch (err) {{
                    if (window.ipc) {{
                        window.ipc.postMessage(JSON.stringify({{
                            type: 'agent_response',
                            request_id: {},
                            action: 'dispatch_custom_event',
                            error: String(err)
                        }}));
                    }}
                }}
            }})();"#,
            target_json, type_json, bubbles, cancelable, detail_json, request_id, request_id, request_id
        )
    }

    /// JavaScript to inspect what DOM element currently holds keyboard focus.
    pub fn get_active_element_script(request_id: u64) -> String {
        format!(
            r#"(function() {{
                try {{
                    const el = document.activeElement;
                    if (!el || el === document.body || el === document.documentElement) {{
                        if (window.ipc) {{
                            window.ipc.postMessage(JSON.stringify({{
                                type: 'agent_response',
                                request_id: {},
                                action: 'get_active_element',
                                payload: {{
                                    success: true,
                                    is_body: true,
                                    tag_name: el ? el.tagName.toLowerCase() : 'body',
                                    id: '',
                                    classes: []
                                }}
                            }}));
                        }}
                        return;
                    }}

                    const rect = el.getBoundingClientRect();
                    const classes = [];
                    if (el.classList) {{
                        for (let i = 0; i < el.classList.length; i++) {{
                            classes.push(el.classList[i]);
                        }}
                    }}

                    const refId = el.getAttribute('data-aerolith-ref') || null;

                    if (window.ipc) {{
                        window.ipc.postMessage(JSON.stringify({{
                            type: 'agent_response',
                            request_id: {},
                            action: 'get_active_element',
                            payload: {{
                                success: true,
                                is_body: false,
                                tag_name: el.tagName ? el.tagName.toLowerCase() : '',
                                id: el.id || '',
                                ref_id: refId,
                                classes: classes,
                                text: (el.innerText || el.textContent || '').trim().substring(0, 200),
                                value: el.value !== undefined ? String(el.value) : null,
                                bounds: {{
                                    x: Math.round(rect.left),
                                    y: Math.round(rect.top),
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
                            action: 'get_active_element',
                            error: String(err)
                        }}));
                    }}
                }}
            }})();"#,
            request_id, request_id, request_id
        )
    }

    /// JavaScript to verify if an element is clickable and identify any occluding DOM element.
    pub fn get_check_element_clickable_script(request_id: u64, target: &str) -> String {
        let target_json = serde_json::to_string(target).unwrap_or_default();
        format!(
            r#"(function() {{
                try {{
                    const target = {};
                    const isNum = /^\d+$/.test(target);
                    const el = isNum
                        ? (window.__aerolith_elements && window.__aerolith_elements[target] || document.querySelector('[data-aerolith-ref="' + target + '"]'))
                        : document.querySelector(target);

                    if (!el) {{
                        if (window.ipc) {{
                            window.ipc.postMessage(JSON.stringify({{
                                type: 'agent_response',
                                request_id: {},
                                action: 'check_element_clickable',
                                error: "Element not found for target: " + target
                            }}));
                        }}
                        return;
                    }}

                    const rect = el.getBoundingClientRect();
                    const inViewport = (
                        rect.bottom > 0 &&
                        rect.right > 0 &&
                        rect.top < (window.innerHeight || document.documentElement.clientHeight) &&
                        rect.left < (window.innerWidth || document.documentElement.clientWidth)
                    );

                    const cx = Math.round(rect.left + rect.width / 2);
                    const cy = Math.round(rect.top + rect.height / 2);

                    let clickable = false;
                    let occluded = false;
                    let occludingElement = null;

                    if (inViewport && cx >= 0 && cy >= 0 && cx <= window.innerWidth && cy <= window.innerHeight) {{
                        const topEl = document.elementFromPoint(cx, cy);
                        if (topEl === el || el.contains(topEl)) {{
                            clickable = true;
                        }} else if (topEl) {{
                            occluded = true;
                            const occClasses = [];
                            if (topEl.classList) {{
                                for (let i = 0; i < topEl.classList.length; i++) {{
                                    occClasses.push(topEl.classList[i]);
                                }}
                            }}
                            occludingElement = {{
                                tag_name: topEl.tagName ? topEl.tagName.toLowerCase() : '',
                                id: topEl.id || '',
                                classes: occClasses,
                                ref_id: topEl.getAttribute('data-aerolith-ref') || null
                            }};
                        }}
                    }}

                    if (window.ipc) {{
                        window.ipc.postMessage(JSON.stringify({{
                            type: 'agent_response',
                            request_id: {},
                            action: 'check_element_clickable',
                            payload: {{
                                success: true,
                                target: target,
                                clickable: clickable,
                                occluded: occluded,
                                in_viewport: inViewport,
                                center_x: cx,
                                center_y: cy,
                                occluding_element: occludingElement
                            }}
                        }}));
                    }}
                }} catch (err) {{
                    if (window.ipc) {{
                        window.ipc.postMessage(JSON.stringify({{
                            type: 'agent_response',
                            request_id: {},
                            action: 'check_element_clickable',
                            error: String(err)
                        }}));
                    }}
                }}
            }})();"#,
            target_json, request_id, request_id, request_id
        )
    }

    /// JavaScript to dispatch a WheelEvent on target element.
    pub fn get_dispatch_wheel_event_script(
        request_id: u64,
        target: &str,
        delta_x: f64,
        delta_y: f64,
        ctrl_key: bool,
    ) -> String {
        let target_json = serde_json::to_string(target).unwrap_or_default();
        format!(
            r#"(function() {{
                try {{
                    const target = {};
                    const deltaX = {};
                    const deltaY = {};
                    const ctrlKey = {};

                    const isNum = /^\d+$/.test(target);
                    const el = isNum
                        ? (window.__aerolith_elements && window.__aerolith_elements[target] || document.querySelector('[data-aerolith-ref="' + target + '"]'))
                        : document.querySelector(target);

                    if (!el) {{
                        if (window.ipc) {{
                            window.ipc.postMessage(JSON.stringify({{
                                type: 'agent_response',
                                request_id: {},
                                action: 'dispatch_wheel_event',
                                error: "Element not found for target: " + target
                            }}));
                        }}
                        return;
                    }}

                    const rect = el.getBoundingClientRect();
                    const cx = Math.round(rect.left + rect.width / 2);
                    const cy = Math.round(rect.top + rect.height / 2);

                    const evt = new WheelEvent('wheel', {{
                        bubbles: true,
                        cancelable: true,
                        clientX: cx,
                        clientY: cy,
                        deltaX: deltaX,
                        deltaY: deltaY,
                        ctrlKey: ctrlKey
                    }});

                    const notCancelled = el.dispatchEvent(evt);

                    if (window.ipc) {{
                        window.ipc.postMessage(JSON.stringify({{
                            type: 'agent_response',
                            request_id: {},
                            action: 'dispatch_wheel_event',
                            payload: {{
                                success: true,
                                target: target,
                                delta_x: deltaX,
                                delta_y: deltaY,
                                ctrl_key: ctrlKey,
                                default_prevented: !notCancelled
                            }}
                        }}));
                    }}
                }} catch (err) {{
                    if (window.ipc) {{
                        window.ipc.postMessage(JSON.stringify({{
                            type: 'agent_response',
                            request_id: {},
                            action: 'dispatch_wheel_event',
                            error: String(err)
                        }}));
                    }}
                }}
            }})();"#,
            target_json, delta_x, delta_y, ctrl_key, request_id, request_id, request_id
        )
    }

    /// JavaScript to inspect computed CSS custom properties (variables) on target or documentElement.
    pub fn get_css_variables_script(
        request_id: u64,
        target: Option<&str>,
        variable_names: &[String],
    ) -> String {
        let target_json = match target {
            Some(t) => serde_json::to_string(t).unwrap_or_else(|_| "null".to_string()),
            None => "null".to_string(),
        };
        let vars_json = serde_json::to_string(variable_names).unwrap_or_else(|_| "[]".to_string());

        format!(
            r#"(function() {{
                try {{
                    const target = {};
                    const varNames = {};

                    let el = document.documentElement;
                    if (target) {{
                        const isNum = /^\d+$/.test(target);
                        const found = isNum
                            ? (window.__aerolith_elements && window.__aerolith_elements[target] || document.querySelector('[data-aerolith-ref="' + target + '"]'))
                            : document.querySelector(target);
                        if (found) el = found;
                    }}

                    const computed = window.getComputedStyle ? window.getComputedStyle(el) : null;
                    const results = {{}};

                    if (computed && Array.isArray(varNames)) {{
                        for (let i = 0; i < varNames.length; i++) {{
                            const name = varNames[i];
                            const val = computed.getPropertyValue(name).trim();
                            results[name] = val;
                        }}
                    }}

                    if (window.ipc) {{
                        window.ipc.postMessage(JSON.stringify({{
                            type: 'agent_response',
                            request_id: {},
                            action: 'get_css_variables',
                            payload: {{
                                success: true,
                                target: target,
                                variables: results
                            }}
                        }}));
                    }}
                }} catch (err) {{
                    if (window.ipc) {{
                        window.ipc.postMessage(JSON.stringify({{
                            type: 'agent_response',
                            request_id: {},
                            action: 'get_css_variables',
                            error: String(err)
                        }}));
                    }}
                }}
            }})();"#,
            target_json, vars_json, request_id, request_id
        )
    }

    /// JavaScript to remove all active focus rings, Set-Of-Marks overlays, and visual badges.
    pub fn get_clear_highlights_script(request_id: u64) -> String {
        format!(
            r#"(function() {{
                try {{
                    let cleared = 0;
                    const rings = document.querySelectorAll('.__aerolith_highlight_ring, #__aerolith_som_overlay, [id^="__aerolith_som"]');
                    rings.forEach(el => {{
                        el.remove();
                        cleared++;
                    }});

                    if (window.ipc) {{
                        window.ipc.postMessage(JSON.stringify({{
                            type: 'agent_response',
                            request_id: {},
                            action: 'clear_highlights',
                            payload: {{
                                success: true,
                                cleared_count: cleared
                            }}
                        }}));
                    }}
                }} catch (err) {{
                    if (window.ipc) {{
                        window.ipc.postMessage(JSON.stringify({{
                            type: 'agent_response',
                            request_id: {},
                            action: 'clear_highlights',
                            error: String(err)
                        }}));
                    }}
                }}
            }})();"#,
            request_id, request_id
        )
    }

    /// JavaScript to dispatch a complete sequence of PointerEvents at target element center.
    pub fn get_simulate_pointer_click_script(
        request_id: u64,
        target: &str,
        click_count: usize,
        button: i32,
    ) -> String {
        let is_numeric = target.chars().all(|c| c.is_ascii_digit());
        let target_expr = if is_numeric {
            format!("(window.__aerolith_elements && window.__aerolith_elements[{}]) || document.querySelector('[data-aerolith-ref=\"{}\"]')", target, target)
        } else {
            format!("document.querySelector({})", serde_json::to_string(target).unwrap_or_default())
        };
        let target_json = serde_json::to_string(target).unwrap_or_default();

        format!(
            r#"(function() {{
                try {{
                    const el = {};
                    if (!el) throw new Error("Element not found for target: " + {});
                    const rect = el.getBoundingClientRect();
                    const clientX = rect.left + rect.width / 2;
                    const clientY = rect.top + rect.height / 2;
                    const screenX = (window.screenX || 0) + clientX;
                    const screenY = (window.screenY || 0) + clientY;
                    const btn = {};
                    const count = Math.max(1, {});

                    for (let i = 1; i <= count; i++) {{
                        const pointerInit = {{
                            bubbles: true,
                            cancelable: true,
                            composed: true,
                            clientX: clientX,
                            clientY: clientY,
                            screenX: screenX,
                            screenY: screenY,
                            button: btn,
                            buttons: 1,
                            pointerId: 1,
                            pointerType: 'mouse',
                            isPrimary: true,
                            pressure: 0.5,
                            detail: i
                        }};

                        el.dispatchEvent(new PointerEvent('pointerover', pointerInit));
                        el.dispatchEvent(new PointerEvent('pointerenter', pointerInit));
                        el.dispatchEvent(new PointerEvent('pointerdown', pointerInit));
                        el.dispatchEvent(new MouseEvent('mousedown', pointerInit));

                        if (typeof el.focus === 'function') {{
                            try {{ el.focus(); }} catch(e) {{}}
                        }}

                        const upInit = Object.assign({{}}, pointerInit, {{ buttons: 0, pressure: 0 }});
                        el.dispatchEvent(new PointerEvent('pointerup', upInit));
                        el.dispatchEvent(new MouseEvent('mouseup', upInit));
                        el.dispatchEvent(new MouseEvent('click', upInit));
                    }}

                    if (window.ipc) {{
                        window.ipc.postMessage(JSON.stringify({{
                            type: 'agent_response',
                            request_id: {},
                            action: 'simulate_pointer_click',
                            payload: {{
                                success: true,
                                target: {},
                                client_x: clientX,
                                client_y: clientY,
                                click_count: count,
                                button: btn
                            }}
                        }}));
                    }}
                }} catch (err) {{
                    if (window.ipc) {{
                        window.ipc.postMessage(JSON.stringify({{
                            type: 'agent_response',
                            request_id: {},
                            action: 'simulate_pointer_click',
                            error: String(err)
                        }}));
                    }}
                }}
            }})();"#,
            target_expr, target_json, button, click_count, request_id, target_json, request_id
        )
    }

    /// JavaScript to extract structured metadata, OpenGraph, Twitter cards, and JSON-LD schemas.
    pub fn get_extract_meta_tags_script(request_id: u64, include_json_ld: bool) -> String {
        format!(
            r#"(function() {{
                try {{
                    const standard = {{}};
                    const opengraph = {{}};
                    const twitter = {{}};
                    const other_meta = [];
                    const links = [];
                    const json_ld = [];

                    document.querySelectorAll('meta').forEach(meta => {{
                        const name = meta.getAttribute('name');
                        const prop = meta.getAttribute('property');
                        const httpEquiv = meta.getAttribute('http-equiv');
                        const content = meta.getAttribute('content') || '';
                        const charset = meta.getAttribute('charset');

                        if (charset) {{
                            standard['charset'] = charset;
                            return;
                        }}
                        const key = name || prop || httpEquiv;
                        if (!key) return;

                        if (key.startsWith('og:')) {{
                            opengraph[key.substring(3)] = content;
                        }} else if (key.startsWith('twitter:')) {{
                            twitter[key.substring(8)] = content;
                        }} else if (name) {{
                            standard[name.toLowerCase()] = content;
                        }} else {{
                            other_meta.push({{ key: key, content: content }});
                        }}
                    }});

                    document.querySelectorAll('link[rel]').forEach(link => {{
                        const rel = link.getAttribute('rel');
                        const href = link.getAttribute('href');
                        if (rel && href) {{
                            links.push({{
                                rel: rel,
                                href: href,
                                type: link.getAttribute('type') || null
                            }});
                        }}
                    }});

                    if ({}) {{
                        document.querySelectorAll('script[type="application/ld+json"]').forEach(s => {{
                            try {{
                                const parsed = JSON.parse(s.textContent || '');
                                json_ld.push(parsed);
                            }} catch (e) {{
                                json_ld.push({{ raw: (s.textContent || '').trim(), error: String(e) }});
                            }}
                        }});
                    }}

                    if (window.ipc) {{
                        window.ipc.postMessage(JSON.stringify({{
                            type: 'agent_response',
                            request_id: {},
                            action: 'extract_meta_tags',
                            payload: {{
                                success: true,
                                title: document.title || '',
                                url: window.location.href,
                                standard: standard,
                                opengraph: opengraph,
                                twitter: twitter,
                                links: links,
                                json_ld: json_ld
                            }}
                        }}));
                    }}
                }} catch (err) {{
                    if (window.ipc) {{
                        window.ipc.postMessage(JSON.stringify({{
                            type: 'agent_response',
                            request_id: {},
                            action: 'extract_meta_tags',
                            error: String(err)
                        }}));
                    }}
                }}
            }})();"#,
            include_json_ld, request_id, request_id
        )
    }

    /// JavaScript to extract complete document layout, viewport dimensions, and scroll metrics.
    pub fn get_document_geometry_script(request_id: u64) -> String {
        format!(
            r#"(function() {{
                try {{
                    const docEl = document.documentElement || document.body;
                    const vpWidth = window.innerWidth || 0;
                    const vpHeight = window.innerHeight || 0;
                    const dpr = window.devicePixelRatio || 1;

                    let visualVp = null;
                    if (window.visualViewport) {{
                        visualVp = {{
                            width: window.visualViewport.width,
                            height: window.visualViewport.height,
                            scale: window.visualViewport.scale,
                            page_left: window.visualViewport.pageLeft,
                            page_top: window.visualViewport.pageTop,
                            offset_left: window.visualViewport.offsetLeft,
                            offset_top: window.visualViewport.offsetTop
                        }};
                    }}

                    const scrollX = window.scrollX || window.pageXOffset || (docEl ? docEl.scrollLeft : 0) || 0;
                    const scrollY = window.scrollY || window.pageYOffset || (docEl ? docEl.scrollTop : 0) || 0;
                    const scrollWidth = docEl ? docEl.scrollWidth : 0;
                    const scrollHeight = docEl ? docEl.scrollHeight : 0;
                    const clientWidth = docEl ? docEl.clientWidth : 0;
                    const clientHeight = docEl ? docEl.clientHeight : 0;

                    const maxScrollX = Math.max(0, scrollWidth - vpWidth);
                    const maxScrollY = Math.max(0, scrollHeight - vpHeight);
                    const percentX = maxScrollX > 0 ? Math.min(100, Math.max(0, (scrollX / maxScrollX) * 100)) : 0;
                    const percentY = maxScrollY > 0 ? Math.min(100, Math.max(0, (scrollY / maxScrollY) * 100)) : 0;

                    let scrollableContainers = 0;
                    try {{
                        const allNodes = document.querySelectorAll('*');
                        for (let i = 0; i < allNodes.length; i++) {{
                            const node = allNodes[i];
                            if (node.scrollHeight > node.clientHeight + 10 || node.scrollWidth > node.clientWidth + 10) {{
                                const style = window.getComputedStyle(node);
                                const overflowY = style.overflowY;
                                const overflowX = style.overflowX;
                                if (overflowY === 'auto' || overflowY === 'scroll' || overflowX === 'auto' || overflowX === 'scroll') {{
                                    scrollableContainers++;
                                }}
                            }}
                        }}
                    }} catch (e) {{}}

                    if (window.ipc) {{
                        window.ipc.postMessage(JSON.stringify({{
                            type: 'agent_response',
                            request_id: {},
                            action: 'get_document_geometry',
                            payload: {{
                                success: true,
                                viewport: {{
                                    width: vpWidth,
                                    height: vpHeight,
                                    device_pixel_ratio: dpr
                                }},
                                visual_viewport: visualVp,
                                document: {{
                                    scroll_width: scrollWidth,
                                    scroll_height: scrollHeight,
                                    client_width: clientWidth,
                                    client_height: clientHeight
                                }},
                                scroll: {{
                                    scroll_x: scrollX,
                                    scroll_y: scrollY,
                                    max_scroll_x: maxScrollX,
                                    max_scroll_y: maxScrollY,
                                    percent_x: Math.round(percentX * 100) / 100,
                                    percent_y: Math.round(percentY * 100) / 100
                                }},
                                scrollable_containers_count: scrollableContainers
                            }}
                        }}));
                    }}
                }} catch (err) {{
                    if (window.ipc) {{
                        window.ipc.postMessage(JSON.stringify({{
                            type: 'agent_response',
                            request_id: {},
                            action: 'get_document_geometry',
                            error: String(err)
                        }}));
                    }}
                }}
            }})();"#,
            request_id, request_id
        )
    }

    /// JavaScript to emulate mobile device orientation (alpha, beta, gamma tilt angles).
    pub fn get_set_device_orientation_script(
        request_id: u64,
        alpha: f64,
        beta: f64,
        gamma: f64,
        absolute: bool,
    ) -> String {
        format!(
            r#"(function() {{
                try {{
                    const alpha = {};
                    const beta = {};
                    const gamma = {};
                    const absolute = {};

                    window.__aerolith_device_orientation = {{ alpha, beta, gamma, absolute }};

                    let dispatched = false;
                    try {{
                        const event = new DeviceOrientationEvent('deviceorientation', {{
                            alpha: alpha,
                            beta: beta,
                            gamma: gamma,
                            absolute: absolute,
                            bubbles: false,
                            cancelable: false
                        }});
                        window.dispatchEvent(event);
                        dispatched = true;
                    }} catch (e) {{
                        dispatched = false;
                    }}

                    if (window.ipc) {{
                        window.ipc.postMessage(JSON.stringify({{
                            type: 'agent_response',
                            request_id: {},
                            action: 'set_device_orientation',
                            payload: {{
                                success: true,
                                alpha: alpha,
                                beta: beta,
                                gamma: gamma,
                                absolute: absolute,
                                dispatched: dispatched
                            }}
                        }}));
                    }}
                }} catch (err) {{
                    if (window.ipc) {{
                        window.ipc.postMessage(JSON.stringify({{
                            type: 'agent_response',
                            request_id: {},
                            action: 'set_device_orientation',
                            error: String(err)
                        }}));
                    }}
                }}
            }})();"#,
            alpha, beta, gamma, absolute, request_id, request_id
        )
    }

    /// JavaScript to purge browser CacheStorage, localStorage, sessionStorage, and cookies.
    pub fn get_clear_browser_cache_script(
        request_id: u64,
        clear_storage: bool,
        clear_cookies: bool,
    ) -> String {
        format!(
            r#"(async function() {{
                try {{
                    let cachesCleared = 0;
                    if (window.caches && window.caches.keys) {{
                        try {{
                            const keys = await window.caches.keys();
                            for (const k of keys) {{
                                if (await window.caches.delete(k)) cachesCleared++;
                            }}
                        }} catch (e) {{}}
                    }}

                    let storageCleared = false;
                    if ({}) {{
                        try {{
                            if (window.localStorage) window.localStorage.clear();
                            if (window.sessionStorage) window.sessionStorage.clear();
                            storageCleared = true;
                        }} catch (e) {{}}
                    }}

                    let cookiesCleared = 0;
                    if ({}) {{
                        try {{
                            const cookies = document.cookie.split(';');
                            for (let i = 0; i < cookies.length; i++) {{
                                const cookie = cookies[i];
                                const eqPos = cookie.indexOf('=');
                                const name = eqPos > -1 ? cookie.substr(0, eqPos).trim() : cookie.trim();
                                if (name) {{
                                    document.cookie = name + '=;expires=Thu, 01 Jan 1970 00:00:00 GMT;path=/';
                                    cookiesCleared++;
                                }}
                            }}
                        }} catch (e) {{}}
                    }}

                    if (window.ipc) {{
                        window.ipc.postMessage(JSON.stringify({{
                            type: 'agent_response',
                            request_id: {},
                            action: 'clear_browser_cache',
                            payload: {{
                                success: true,
                                caches_cleared: cachesCleared,
                                storage_cleared: storageCleared,
                                cookies_cleared: cookiesCleared
                            }}
                        }}));
                    }}
                }} catch (err) {{
                    if (window.ipc) {{
                        window.ipc.postMessage(JSON.stringify({{
                            type: 'agent_response',
                            request_id: {},
                            action: 'clear_browser_cache',
                            error: String(err)
                        }}));
                    }}
                }}
            }})();"#,
            clear_storage, clear_cookies, request_id, request_id
        )
    }

    /// JavaScript to inspect box-model computed spacing (margin, padding, border, dimensions, z-index).
    pub fn get_element_spacing_script(request_id: u64, target: &str) -> String {
        let is_numeric = target.chars().all(|c| c.is_ascii_digit());
        let target_expr = if is_numeric {
            format!("(window.__aerolith_elements && window.__aerolith_elements[{}]) || document.querySelector('[data-aerolith-ref=\"{}\"]')", target, target)
        } else {
            format!("document.querySelector({})", serde_json::to_string(target).unwrap_or_default())
        };
        let target_json = serde_json::to_string(target).unwrap_or_default();

        format!(
            r#"(function() {{
                try {{
                    const el = {};
                    if (!el) throw new Error("Element not found for target: " + {});
                    const style = window.getComputedStyle(el);
                    const rect = el.getBoundingClientRect();

                    const parsePx = (val) => {{
                        const n = parseFloat(val);
                        return isNaN(n) ? 0 : n;
                    }};

                    const margin = {{
                        top: parsePx(style.marginTop),
                        right: parsePx(style.marginRight),
                        bottom: parsePx(style.marginBottom),
                        left: parsePx(style.marginLeft)
                    }};

                    const padding = {{
                        top: parsePx(style.paddingTop),
                        right: parsePx(style.paddingRight),
                        bottom: parsePx(style.paddingBottom),
                        left: parsePx(style.paddingLeft)
                    }};

                    const border = {{
                        top: parsePx(style.borderTopWidth),
                        right: parsePx(style.borderRightWidth),
                        bottom: parsePx(style.borderBottomWidth),
                        left: parsePx(style.borderLeftWidth)
                    }};

                    const dimensions = {{
                        width: rect.width,
                        height: rect.height,
                        client_width: el.clientWidth,
                        client_height: el.clientHeight,
                        content_width: Math.max(0, el.clientWidth - padding.left - padding.right),
                        content_height: Math.max(0, el.clientHeight - padding.top - padding.bottom)
                    }};

                    const layout = {{
                        box_sizing: style.boxSizing,
                        display: style.display,
                        position: style.position,
                        z_index: isNaN(parseInt(style.zIndex)) ? "auto" : parseInt(style.zIndex)
                    }};

                    if (window.ipc) {{
                        window.ipc.postMessage(JSON.stringify({{
                            type: 'agent_response',
                            request_id: {},
                            action: 'get_element_spacing',
                            payload: {{
                                success: true,
                                target: {},
                                margin: margin,
                                padding: padding,
                                border: border,
                                dimensions: dimensions,
                                layout: layout
                            }}
                        }}));
                    }}
                }} catch (err) {{
                    if (window.ipc) {{
                        window.ipc.postMessage(JSON.stringify({{
                            type: 'agent_response',
                            request_id: {},
                            action: 'get_element_spacing',
                            error: String(err)
                        }}));
                    }}
                }}
            }})();"#,
            target_expr, target_json, request_id, target_json, request_id
        )
    }

    /// JavaScript to wait until window.location.href matches a URL pattern.
    pub fn get_wait_for_url_script(request_id: u64, pattern: &str, timeout_ms: u64) -> String {
        let pattern_json = serde_json::to_string(pattern).unwrap_or_default();
        format!(
            r#"(function() {{
                try {{
                    const pattern = {};
                    const timeout = {};
                    const startTime = Date.now();

                    const isMatch = (url) => {{
                        if (url.includes(pattern)) return true;
                        try {{
                            const regex = new RegExp(pattern);
                            if (regex.test(url)) return true;
                        }} catch (e) {{}}
                        return false;
                    }};

                    if (isMatch(window.location.href)) {{
                        if (window.ipc) {{
                            window.ipc.postMessage(JSON.stringify({{
                                type: 'agent_response',
                                request_id: {},
                                action: 'wait_for_url',
                                payload: {{
                                    success: true,
                                    url: window.location.href,
                                    pattern: pattern,
                                    elapsed_ms: 0
                                }}
                            }}));
                        }}
                        return;
                    }}

                    const interval = setInterval(() => {{
                        const elapsed = Date.now() - startTime;
                        if (isMatch(window.location.href)) {{
                            clearInterval(interval);
                            if (window.ipc) {{
                                window.ipc.postMessage(JSON.stringify({{
                                    type: 'agent_response',
                                    request_id: {},
                                    action: 'wait_for_url',
                                    payload: {{
                                        success: true,
                                        url: window.location.href,
                                        pattern: pattern,
                                        elapsed_ms: elapsed
                                    }}
                                }}));
                            }}
                            return;
                        }}

                        if (elapsed >= timeout) {{
                            clearInterval(interval);
                            if (window.ipc) {{
                                window.ipc.postMessage(JSON.stringify({{
                                    type: 'agent_response',
                                    request_id: {},
                                    action: 'wait_for_url',
                                    error: "Timed out waiting for URL to match pattern: " + pattern + " (current URL: " + window.location.href + ")"
                                }}));
                            }}
                        }}
                    }}, 50);
                }} catch (err) {{
                    if (window.ipc) {{
                        window.ipc.postMessage(JSON.stringify({{
                            type: 'agent_response',
                            request_id: {},
                            action: 'wait_for_url',
                            error: String(err)
                        }}));
                    }}
                }}
            }})();"#,
            pattern_json, timeout_ms, request_id, request_id, request_id, request_id
        )
    }

    /// JavaScript to construct and dispatch synthetic TouchEvent with TouchList.
    pub fn get_dispatch_touch_event_script(
        request_id: u64,
        event_type: &str,
        touches: &[crate::agent::protocol::TouchPoint],
    ) -> String {
        let touches_json = serde_json::to_string(touches).unwrap_or_else(|_| "[]".into());
        let event_type_json = serde_json::to_string(event_type).unwrap_or_default();

        format!(
            r#"(function() {{
                try {{
                    const rawTouches = {};
                    const eventType = {};
                    const touchList = [];

                    for (let i = 0; i < rawTouches.length; i++) {{
                        const t = rawTouches[i];
                        const isNumeric = /^\d+$/.test(t.target);
                        const el = isNumeric
                            ? ((window.__aerolith_elements && window.__aerolith_elements[t.target]) || document.querySelector('[data-aerolith-ref="' + t.target + '"]'))
                            : document.querySelector(t.target);

                        const targetNode = el || document.body || document.documentElement;
                        const clientX = t.x;
                        const clientY = t.y;
                        const screenX = (window.screenX || 0) + clientX;
                        const screenY = (window.screenY || 0) + clientY;
                        const pageX = (window.scrollX || 0) + clientX;
                        const pageY = (window.scrollY || 0) + clientY;

                        let touchObj = null;
                        try {{
                            touchObj = new Touch({{
                                identifier: t.identifier || i,
                                target: targetNode,
                                clientX: clientX,
                                clientY: clientY,
                                screenX: screenX,
                                screenY: screenY,
                                pageX: pageX,
                                pageY: pageY,
                                radiusX: t.radius_x || 1.0,
                                radiusY: t.radius_y || 1.0,
                                rotationAngle: t.rotation_angle || 0.0,
                                force: t.force || 1.0
                            }});
                        }} catch (e) {{
                            touchObj = {{
                                identifier: t.identifier || i,
                                target: targetNode,
                                clientX: clientX,
                                clientY: clientY,
                                screenX: screenX,
                                screenY: screenY,
                                pageX: pageX,
                                pageY: pageY,
                                radiusX: t.radius_x || 1.0,
                                radiusY: t.radius_y || 1.0,
                                rotationAngle: t.rotation_angle || 0.0,
                                force: t.force || 1.0
                            }};
                        }}
                        touchList.push(touchObj);
                    }}

                    const primaryTarget = touchList.length > 0 && touchList[0].target ? touchList[0].target : (document.body || document.documentElement);
                    let dispatched = false;
                    try {{
                        const touchEv = new TouchEvent(eventType, {{
                            touches: touchList,
                            targetTouches: touchList,
                            changedTouches: touchList,
                            bubbles: true,
                            cancelable: true,
                            composed: true
                        }});
                        primaryTarget.dispatchEvent(touchEv);
                        dispatched = true;
                    }} catch (e) {{
                        const customEv = new CustomEvent(eventType, {{
                            bubbles: true,
                            cancelable: true,
                            detail: {{ touches: touchList }}
                        }});
                        primaryTarget.dispatchEvent(customEv);
                        dispatched = true;
                    }}

                    if (window.ipc) {{
                        window.ipc.postMessage(JSON.stringify({{
                            type: 'agent_response',
                            request_id: {},
                            action: 'dispatch_touch_event',
                            payload: {{
                                success: true,
                                event_type: eventType,
                                touches_count: touchList.length,
                                dispatched: dispatched
                            }}
                        }}));
                    }}
                }} catch (err) {{
                    if (window.ipc) {{
                        window.ipc.postMessage(JSON.stringify({{
                            type: 'agent_response',
                            request_id: {},
                            action: 'dispatch_touch_event',
                            error: String(err)
                        }}));
                    }}
                }}
            }})();"#,
            touches_json, event_type_json, request_id, request_id
        )
    }

    /// JavaScript to extract a hierarchical computed accessibility (a11y) tree.
    pub fn get_a11y_tree_script(request_id: u64, target: Option<&str>, max_depth: usize) -> String {
        let target_expr = match target {
            Some(t) => {
                let is_numeric = t.chars().all(|c| c.is_ascii_digit());
                if is_numeric {
                    format!("(window.__aerolith_elements && window.__aerolith_elements[{}]) || document.querySelector('[data-aerolith-ref=\"{}\"]')", t, t)
                } else {
                    format!("document.querySelector({})", serde_json::to_string(t).unwrap_or_default())
                }
            }
            None => "document.body || document.documentElement".to_string(),
        };

        format!(
            r#"(function() {{
                try {{
                    const rootEl = {};
                    if (!rootEl) throw new Error("A11y root element not found");
                    const maxDepth = {};

                    const getRole = (el) => {{
                        if (el.getAttribute('role')) return el.getAttribute('role');
                        const tag = el.tagName.toLowerCase();
                        if (tag === 'button') return 'button';
                        if (tag === 'a' && el.href) return 'link';
                        if (tag === 'input') {{
                            const type = (el.type || 'text').toLowerCase();
                            if (type === 'checkbox') return 'checkbox';
                            if (type === 'radio') return 'radio';
                            if (type === 'button' || type === 'submit' || type === 'reset') return 'button';
                            return 'textbox';
                        }}
                        if (tag === 'textarea') return 'textbox';
                        if (tag === 'select') return 'combobox';
                        if (/^h[1-6]$/.test(tag)) return 'heading';
                        if (tag === 'nav') return 'navigation';
                        if (tag === 'main') return 'main';
                        if (tag === 'header') return 'banner';
                        if (tag === 'footer') return 'contentinfo';
                        if (tag === 'article') return 'article';
                        if (tag === 'aside') return 'complementary';
                        if (tag === 'dialog') return 'dialog';
                        if (tag === 'form') return 'form';
                        if (tag === 'table') return 'table';
                        if (tag === 'ul' || tag === 'ol') return 'list';
                        if (tag === 'li') return 'listitem';
                        if (tag === 'img') return 'img';
                        return 'generic';
                    }};

                    const getName = (el) => {{
                        if (el.getAttribute('aria-label')) return el.getAttribute('aria-label').trim();
                        if (el.getAttribute('aria-labelledby')) {{
                            const id = el.getAttribute('aria-labelledby');
                            const labelEl = document.getElementById(id);
                            if (labelEl) return (labelEl.textContent || '').trim();
                        }}
                        if (el.getAttribute('alt')) return el.getAttribute('alt').trim();
                        if (el.getAttribute('placeholder')) return el.getAttribute('placeholder').trim();
                        if (el.getAttribute('title')) return el.getAttribute('title').trim();
                        const tag = el.tagName.toLowerCase();
                        if (tag === 'button' || tag === 'a' || /^h[1-6]$/.test(tag)) {{
                            return (el.textContent || '').trim().replace(/\s+/g, ' ');
                        }}
                        return '';
                    }};

                    const serializeA11y = (el, depth) => {{
                        if (!el || el.nodeType !== 1) return null;
                        if (el.getAttribute('aria-hidden') === 'true') return null;
                        const style = window.getComputedStyle(el);
                        if (style.display === 'none' || style.visibility === 'hidden') return null;

                        const role = getRole(el);
                        const name = getName(el);
                        const rect = el.getBoundingClientRect();

                        const states = {{
                            disabled: el.disabled || el.getAttribute('aria-disabled') === 'true',
                            expanded: el.getAttribute('aria-expanded') === 'true' ? true : (el.getAttribute('aria-expanded') === 'false' ? false : null),
                            selected: el.getAttribute('aria-selected') === 'true',
                            checked: el.checked || el.getAttribute('aria-checked') === 'true',
                            focused: document.activeElement === el,
                            hidden: false
                        }};

                        const node = {{
                            tag: el.tagName.toLowerCase(),
                            role: role,
                            name: name,
                            description: el.getAttribute('aria-description') || '',
                            value: el.value || el.getAttribute('aria-valuenow') || '',
                            states: states,
                            bounds: {{
                                x: Math.round(rect.left),
                                y: Math.round(rect.top),
                                width: Math.round(rect.width),
                                height: Math.round(rect.height)
                            }},
                            children: []
                        }};

                        if (depth < maxDepth) {{
                            for (let i = 0; i < el.children.length; i++) {{
                                const childNode = serializeA11y(el.children[i], depth + 1);
                                if (childNode) node.children.push(childNode);
                            }}
                        }}
                        return node;
                    }};

                    const tree = serializeA11y(rootEl, 0);

                    if (window.ipc) {{
                        window.ipc.postMessage(JSON.stringify({{
                            type: 'agent_response',
                            request_id: {},
                            action: 'get_a11y_tree',
                            payload: {{
                                success: true,
                                tree: tree
                            }}
                        }}));
                    }}
                }} catch (err) {{
                    if (window.ipc) {{
                        window.ipc.postMessage(JSON.stringify({{
                            type: 'agent_response',
                            request_id: {},
                            action: 'get_a11y_tree',
                            error: String(err)
                        }}));
                    }}
                }}
            }})();"#,
            target_expr, max_depth, request_id, request_id
        )
    }

    /// JavaScript to dispatch a keyboard hotkey chord (e.g. Control+Shift+P, Meta+K).
    pub fn get_simulate_keyboard_chord_script(request_id: u64, chord: &str, target: Option<&str>) -> String {
        let chord_json = serde_json::to_string(chord).unwrap_or_default();
        let target_expr = match target {
            Some(t) => {
                let is_numeric = t.chars().all(|c| c.is_ascii_digit());
                if is_numeric {
                    format!("(window.__aerolith_elements && window.__aerolith_elements[{}]) || document.querySelector('[data-aerolith-ref=\"{}\"]')", t, t)
                } else {
                    format!("document.querySelector({})", serde_json::to_string(t).unwrap_or_default())
                }
            }
            None => "document.activeElement || document.body || document.documentElement".to_string(),
        };

        format!(
            r#"(function() {{
                try {{
                    const chord = {};
                    const targetEl = {} || document.body;
                    const tokens = chord.split('+').map(t => t.trim()).filter(Boolean);

                    let ctrl = false;
                    let shift = false;
                    let alt = false;
                    let meta = false;
                    let key = ' ';

                    for (const token of tokens) {{
                        const lower = token.toLowerCase();
                        if (lower === 'control' || lower === 'ctrl') ctrl = true;
                        else if (lower === 'shift') shift = true;
                        else if (lower === 'alt' || lower === 'option') alt = true;
                        else if (lower === 'meta' || lower === 'command' || lower === 'cmd') meta = true;
                        else key = token;
                    }}

                    const modifiers = [
                        {{ active: ctrl, key: 'Control', code: 'ControlLeft' }},
                        {{ active: shift, key: 'Shift', code: 'ShiftLeft' }},
                        {{ active: alt, key: 'Alt', code: 'AltLeft' }},
                        {{ active: meta, key: 'Meta', code: 'MetaLeft' }}
                    ];

                    for (const mod of modifiers) {{
                        if (mod.active) {{
                            targetEl.dispatchEvent(new KeyboardEvent('keydown', {{
                                key: mod.key,
                                code: mod.code,
                                ctrlKey: ctrl,
                                shiftKey: shift,
                                altKey: alt,
                                metaKey: meta,
                                bubbles: true,
                                cancelable: true
                            }}));
                        }}
                    }}

                    targetEl.dispatchEvent(new KeyboardEvent('keydown', {{
                        key: key,
                        code: 'Key' + key.toUpperCase(),
                        ctrlKey: ctrl,
                        shiftKey: shift,
                        altKey: alt,
                        metaKey: meta,
                        bubbles: true,
                        cancelable: true
                    }}));

                    targetEl.dispatchEvent(new KeyboardEvent('keyup', {{
                        key: key,
                        code: 'Key' + key.toUpperCase(),
                        ctrlKey: ctrl,
                        shiftKey: shift,
                        altKey: alt,
                        metaKey: meta,
                        bubbles: true,
                        cancelable: true
                    }}));

                    for (let i = modifiers.length - 1; i >= 0; i--) {{
                        const mod = modifiers[i];
                        if (mod.active) {{
                            targetEl.dispatchEvent(new KeyboardEvent('keyup', {{
                                key: mod.key,
                                code: mod.code,
                                ctrlKey: ctrl,
                                shiftKey: shift,
                                altKey: alt,
                                metaKey: meta,
                                bubbles: true,
                                cancelable: true
                            }}));
                        }}
                    }}

                    if (window.ipc) {{
                        window.ipc.postMessage(JSON.stringify({{
                            type: 'agent_response',
                            request_id: {},
                            action: 'simulate_keyboard_chord',
                            payload: {{
                                success: true,
                                chord: chord,
                                key: key,
                                modifiers: {{ ctrl: ctrl, shift: shift, alt: alt, meta: meta }},
                                dispatched: true
                            }}
                        }}));
                    }}
                }} catch (err) {{
                    if (window.ipc) {{
                        window.ipc.postMessage(JSON.stringify({{
                            type: 'agent_response',
                            request_id: {},
                            action: 'simulate_keyboard_chord',
                            error: String(err)
                        }}));
                    }}
                }}
            }})();"#,
            chord_json, target_expr, request_id, request_id
        )
    }

    /// JavaScript to draw annotated bounding boxes with labels on the page.
    pub fn get_highlight_bounding_boxes_script(
        request_id: u64,
        boxes: &[crate::agent::protocol::BoundingBoxMarker],
    ) -> String {
        let boxes_json = serde_json::to_string(boxes).unwrap_or_else(|_| "[]".into());

        format!(
            r#"(function() {{
                try {{
                    const rawBoxes = {};
                    let container = document.getElementById('__aerolith_boxes_overlay');
                    if (!container) {{
                        container = document.createElement('div');
                        container.id = '__aerolith_boxes_overlay';
                        container.style.position = 'absolute';
                        container.style.top = '0';
                        container.style.left = '0';
                        container.style.width = '100%';
                        container.style.height = '100%';
                        container.style.pointerEvents = 'none';
                        container.style.zIndex = '2147483646';
                        document.body.appendChild(container);
                    }} else {{
                        container.innerHTML = '';
                    }}

                    for (let i = 0; i < rawBoxes.length; i++) {{
                        const b = rawBoxes[i];
                        const boxDiv = document.createElement('div');
                        boxDiv.style.position = 'absolute';
                        boxDiv.style.left = b.x + 'px';
                        boxDiv.style.top = b.y + 'px';
                        boxDiv.style.width = b.width + 'px';
                        boxDiv.style.height = b.height + 'px';
                        const color = b.color || '#0066ff';
                        boxDiv.style.border = '2px solid ' + color;
                        boxDiv.style.backgroundColor = 'rgba(0, 102, 255, 0.12)';
                        boxDiv.style.boxSizing = 'border-box';
                        boxDiv.style.pointerEvents = 'none';

                        if (b.label) {{
                            const labelSpan = document.createElement('span');
                            labelSpan.textContent = b.label;
                            labelSpan.style.position = 'absolute';
                            labelSpan.style.top = '-18px';
                            labelSpan.style.left = '-2px';
                            labelSpan.style.background = color;
                            labelSpan.style.color = '#ffffff';
                            labelSpan.style.fontSize = '10px';
                            labelSpan.style.fontFamily = 'monospace';
                            labelSpan.style.padding = '1px 4px';
                            labelSpan.style.borderRadius = '2px';
                            labelSpan.style.whiteSpace = 'nowrap';
                            boxDiv.appendChild(labelSpan);
                        }}
                        container.appendChild(boxDiv);
                    }}

                    if (window.ipc) {{
                        window.ipc.postMessage(JSON.stringify({{
                            type: 'agent_response',
                            request_id: {},
                            action: 'highlight_bounding_boxes',
                            payload: {{
                                success: true,
                                boxes_count: rawBoxes.length
                            }}
                        }}));
                    }}
                }} catch (err) {{
                    if (window.ipc) {{
                        window.ipc.postMessage(JSON.stringify({{
                            type: 'agent_response',
                            request_id: {},
                            action: 'highlight_bounding_boxes',
                            error: String(err)
                        }}));
                    }}
                }}
            }})();"#,
            boxes_json, request_id, request_id
        )
    }

    /// JavaScript to extract all HTML attributes of target element as a key-value dictionary.
    pub fn get_element_attributes_script(request_id: u64, target: &str) -> String {
        let is_numeric = target.chars().all(|c| c.is_ascii_digit());
        let target_expr = if is_numeric {
            format!("(window.__aerolith_elements && window.__aerolith_elements[{}]) || document.querySelector('[data-aerolith-ref=\"{}\"]')", target, target)
        } else {
            format!("document.querySelector({})", serde_json::to_string(target).unwrap_or_default())
        };
        let target_json = serde_json::to_string(target).unwrap_or_default();

        format!(
            r#"(function() {{
                try {{
                    const el = {};
                    if (!el) throw new Error("Element not found for target: " + {});
                    const attrs = {{}};
                    for (let i = 0; i < el.attributes.length; i++) {{
                        const a = el.attributes[i];
                        attrs[a.name] = a.value;
                    }}

                    if (window.ipc) {{
                        window.ipc.postMessage(JSON.stringify({{
                            type: 'agent_response',
                            request_id: {},
                            action: 'get_element_attributes',
                            payload: {{
                                success: true,
                                target: {},
                                attributes: attrs
                            }}
                        }}));
                    }}
                }} catch (err) {{
                    if (window.ipc) {{
                        window.ipc.postMessage(JSON.stringify({{
                            type: 'agent_response',
                            request_id: {},
                            action: 'get_element_attributes',
                            error: String(err)
                        }}));
                    }}
                }}
            }})();"#,
            target_expr, target_json, request_id, target_json, request_id
        )
    }

    /// JavaScript to inspect CSS filters, backdrop filters, opacity, pointer-events, and ancestor occlusion.
    pub fn get_element_filter_script(request_id: u64, target: &str) -> String {
        let is_numeric = target.chars().all(|c| c.is_ascii_digit());
        let target_expr = if is_numeric {
            format!("(window.__aerolith_elements && window.__aerolith_elements[{}]) || document.querySelector('[data-aerolith-ref=\"{}\"]')", target, target)
        } else {
            format!("document.querySelector({})", serde_json::to_string(target).unwrap_or_default())
        };
        let target_json = serde_json::to_string(target).unwrap_or_default();

        format!(
            r#"(function() {{
                try {{
                    const el = {};
                    if (!el) throw new Error("Element not found for target: " + {});
                    const style = window.getComputedStyle(el);

                    let hasOccludingAncestor = false;
                    let curr = el.parentElement;
                    while (curr && curr !== document.documentElement) {{
                        const pStyle = window.getComputedStyle(curr);
                        if (pStyle.display === 'none' || pStyle.visibility === 'hidden' || parseFloat(pStyle.opacity) < 0.05 || pStyle.pointerEvents === 'none') {{
                            hasOccludingAncestor = true;
                            break;
                        }}
                        curr = curr.parentElement;
                    }}

                    const opacityVal = parseFloat(style.opacity);
                    const opacity = isNaN(opacityVal) ? 1.0 : opacityVal;
                    const isVisible = style.display !== 'none' && style.visibility !== 'hidden' && opacity > 0.05 && !hasOccludingAncestor;

                    if (window.ipc) {{
                        window.ipc.postMessage(JSON.stringify({{
                            type: 'agent_response',
                            request_id: {},
                            action: 'get_element_filter',
                            payload: {{
                                success: true,
                                target: {},
                                filter: style.filter || 'none',
                                backdrop_filter: style.backdropFilter || style.webkitBackdropFilter || 'none',
                                opacity: opacity,
                                visibility: style.visibility,
                                display: style.display,
                                pointer_events: style.pointerEvents,
                                mix_blend_mode: style.mixBlendMode || 'normal',
                                transform: style.transform || 'none',
                                has_occluding_ancestor: hasOccludingAncestor,
                                is_effectively_visible: isVisible
                            }}
                        }}));
                    }}
                }} catch (err) {{
                    if (window.ipc) {{
                        window.ipc.postMessage(JSON.stringify({{
                            type: 'agent_response',
                            request_id: {},
                            action: 'get_element_filter',
                            error: String(err)
                        }}));
                    }}
                }}
            }})();"#,
            target_expr, target_json, request_id, target_json, request_id
        )
    }

    /// JavaScript to autonomously dismiss cookie banners, newsletter modals, and full-screen backdrops.
    pub fn get_dismiss_all_modals_script(request_id: u64) -> String {
        format!(
            r#"(function() {{
                try {{
                    let buttonsClicked = 0;
                    let overlaysRemoved = 0;

                    const closeSelectors = [
                        'button[aria-label*="close" i]',
                        'button[aria-label*="dismiss" i]',
                        '.modal-close',
                        '[data-dismiss="modal"]',
                        '[data-close]',
                        '.close-modal',
                        'button.close',
                        '#onetrust-accept-btn-handler',
                        '#onetrust-reject-all-handler',
                        '#accept-recommended-btn-handler',
                        'button[id*="cookie-accept" i]',
                        'button[id*="cookie-close" i]',
                        'button[class*="cookie-accept" i]',
                        'button[class*="cookie-close" i]',
                        '.privy-dismiss-content',
                        'a[aria-label*="close" i]'
                    ];

                    for (const sel of closeSelectors) {{
                        try {{
                            const btns = document.querySelectorAll(sel);
                            for (let i = 0; i < btns.length; i++) {{
                                const b = btns[i];
                                const rect = b.getBoundingClientRect();
                                if (rect.width > 0 && rect.height > 0) {{
                                    b.click();
                                    buttonsClicked++;
                                }}
                            }}
                        }} catch (e) {{}}
                    }}

                    // Remove fixed/absolute backdrops with high z-index
                    const allDivs = document.querySelectorAll('div, section, aside');
                    const vpW = window.innerWidth;
                    const vpH = window.innerHeight;
                    for (let i = 0; i < allDivs.length; i++) {{
                        const el = allDivs[i];
                        try {{
                            const st = window.getComputedStyle(el);
                            if (st.position === 'fixed' || st.position === 'absolute') {{
                                const z = parseInt(st.zIndex);
                                if (!isNaN(z) && z >= 999) {{
                                    const rect = el.getBoundingClientRect();
                                    if (rect.width >= vpW * 0.8 && rect.height >= vpH * 0.8) {{
                                        // Check if contains form inputs
                                        const hasInputs = el.querySelector('input:not([type="hidden"]), select, textarea');
                                        if (!hasInputs) {{
                                            el.remove();
                                            overlaysRemoved++;
                                        }}
                                    }}
                                }}
                            }}
                        }} catch (e) {{}}
                    }}

                    // Restore page scrollability
                    if (document.body) document.body.style.overflow = 'auto';
                    if (document.documentElement) document.documentElement.style.overflow = 'auto';

                    if (window.ipc) {{
                        window.ipc.postMessage(JSON.stringify({{
                            type: 'agent_response',
                            request_id: {},
                            action: 'dismiss_all_modals',
                            payload: {{
                                success: true,
                                buttons_clicked: buttonsClicked,
                                overlays_removed: overlaysRemoved,
                                scroll_restored: true
                            }}
                        }}));
                    }}
                }} catch (err) {{
                    if (window.ipc) {{
                        window.ipc.postMessage(JSON.stringify({{
                            type: 'agent_response',
                            request_id: {},
                            action: 'dismiss_all_modals',
                            error: String(err)
                        }}));
                    }}
                }}
            }})();"#,
            request_id, request_id
        )
    }

    /// JavaScript to extract ranked dominant color palette and theme variables from visible elements.
    pub fn get_color_palette_script(request_id: u64, limit: usize) -> String {
        format!(
            r#"(function() {{
                try {{
                    const colorCounts = {{}};
                    const colorTypes = {{}};

                    const rgbToHex = (rgb) => {{
                        const m = rgb.match(/rgba?\((\d+),\s*(\d+),\s*(\d+)/);
                        if (!m) return rgb;
                        const r = parseInt(m[1]).toString(16).padStart(2, '0');
                        const g = parseInt(m[2]).toString(16).padStart(2, '0');
                        const b = parseInt(m[3]).toString(16).padStart(2, '0');
                        return '#' + r + g + b;
                    }};

                    const addColor = (c, type) => {{
                        if (!c || c === 'transparent' || c === 'rgba(0, 0, 0, 0)') return;
                        colorCounts[c] = (colorCounts[c] || 0) + 1;
                        if (!colorTypes[c]) colorTypes[c] = new Set();
                        colorTypes[c].add(type);
                    }};

                    const elements = document.querySelectorAll('body, header, nav, main, footer, article, section, button, a, h1, h2, h3, p, div.card, div.btn');
                    for (let i = 0; i < elements.length; i++) {{
                        const el = elements[i];
                        try {{
                            const st = window.getComputedStyle(el);
                            addColor(st.backgroundColor, 'background');
                            addColor(st.color, 'text');
                            if (st.borderColor && st.borderWidth && parseFloat(st.borderWidth) > 0) {{
                                addColor(st.borderColor, 'border');
                            }}
                        }} catch (e) {{}}
                    }}

                    const sorted = Object.keys(colorCounts).sort((a, b) => colorCounts[b] - colorCounts[a]);
                    const maxLimit = Math.max(1, {});
                    const palette = sorted.slice(0, maxLimit).map(c => ({{
                        color: c,
                        hex: rgbToHex(c),
                        count: colorCounts[c],
                        roles: Array.from(colorTypes[c] || [])
                    }}));

                    // Query common CSS custom property theme colors from root
                    const rootStyle = window.getComputedStyle(document.documentElement);
                    const themeVars = {{}};
                    const commonVarNames = ['--primary', '--secondary', '--accent', '--background', '--foreground', '--muted', '--border'];
                    for (const v of commonVarNames) {{
                        const val = rootStyle.getPropertyValue(v).trim();
                        if (val) themeVars[v] = val;
                    }}

                    if (window.ipc) {{
                        window.ipc.postMessage(JSON.stringify({{
                            type: 'agent_response',
                            request_id: {},
                            action: 'get_color_palette',
                            payload: {{
                                success: true,
                                palette: palette,
                                theme_variables: themeVars
                            }}
                        }}));
                    }}
                }} catch (err) {{
                    if (window.ipc) {{
                        window.ipc.postMessage(JSON.stringify({{
                            type: 'agent_response',
                            request_id: {},
                            action: 'get_color_palette',
                            error: String(err)
                        }}));
                    }}
                }}
            }})();"#,
            limit, request_id, request_id
        )
    }

    /// JavaScript to simulate a full HTML5/Pointer drag and drop sequence between two elements.
    pub fn get_drag_and_drop_element_script(
        request_id: u64,
        source: &str,
        target: &str,
        steps: u32,
        hold_ms: u64,
    ) -> String {
        let is_src_num = source.chars().all(|c| c.is_ascii_digit());
        let src_expr = if is_src_num {
            format!("(window.__aerolith_elements && window.__aerolith_elements[{}]) || document.querySelector('[data-aerolith-ref=\"{}\"]')", source, source)
        } else {
            format!("document.querySelector({})", serde_json::to_string(source).unwrap_or_default())
        };

        let is_tgt_num = target.chars().all(|c| c.is_ascii_digit());
        let tgt_expr = if is_tgt_num {
            format!("(window.__aerolith_elements && window.__aerolith_elements[{}]) || document.querySelector('[data-aerolith-ref=\"{}\"]')", target, target)
        } else {
            format!("document.querySelector({})", serde_json::to_string(target).unwrap_or_default())
        };

        let src_json = serde_json::to_string(source).unwrap_or_default();
        let tgt_json = serde_json::to_string(target).unwrap_or_default();

        format!(
            r#"(async function() {{
                try {{
                    const srcEl = {};
                    if (!srcEl) throw new Error("Source element not found: " + {});
                    const tgtEl = {};
                    if (!tgtEl) throw new Error("Target element not found: " + {});

                    const srcRect = srcEl.getBoundingClientRect();
                    const tgtRect = tgtEl.getBoundingClientRect();

                    const startX = srcRect.left + srcRect.width / 2;
                    const startY = srcRect.top + srcRect.height / 2;
                    const endX = tgtRect.left + tgtRect.width / 2;
                    const endY = tgtRect.top + tgtRect.height / 2;
                    const steps = Math.max(2, {});
                    const holdMs = {};

                    let dataTransfer = null;
                    try {{
                        dataTransfer = new DataTransfer();
                    }} catch (e) {{}}

                    const dispatchEvent = (el, type, x, y, extra = {{}}) => {{
                        const init = Object.assign({{
                            bubbles: true,
                            cancelable: true,
                            composed: true,
                            clientX: x,
                            clientY: y,
                            screenX: (window.screenX || 0) + x,
                            screenY: (window.screenY || 0) + y,
                            button: 0,
                            buttons: 1,
                            dataTransfer: dataTransfer
                        }}, extra);

                        try {{
                            el.dispatchEvent(new MouseEvent(type, init));
                        }} catch (e) {{}}
                        try {{
                            if (type.startsWith('drag') || type === 'drop') {{
                                el.dispatchEvent(new DragEvent(type, init));
                            }}
                        }} catch (e) {{}}
                    }};

                    // 1. Mouse down / Pointer down on source
                    dispatchEvent(srcEl, 'pointerdown', startX, startY);
                    dispatchEvent(srcEl, 'mousedown', startX, startY);
                    dispatchEvent(srcEl, 'dragstart', startX, startY);

                    if (holdMs > 0) {{
                        await new Promise(r => setTimeout(r, holdMs));
                    }}

                    // 2. Interpolate steps across path
                    for (let i = 1; i < steps; i++) {{
                        const curX = startX + (endX - startX) * (i / steps);
                        const curY = startY + (endY - startY) * (i / steps);
                        const elUnder = document.elementFromPoint(curX, curY) || srcEl;
                        dispatchEvent(elUnder, 'pointermove', curX, curY);
                        dispatchEvent(elUnder, 'mousemove', curX, curY);
                        dispatchEvent(elUnder, 'dragover', curX, curY);
                    }}

                    // 3. Drop on target
                    dispatchEvent(tgtEl, 'dragenter', endX, endY);
                    dispatchEvent(tgtEl, 'dragover', endX, endY);
                    dispatchEvent(tgtEl, 'drop', endX, endY);
                    dispatchEvent(srcEl, 'dragend', endX, endY);
                    dispatchEvent(tgtEl, 'pointerup', endX, endY);
                    dispatchEvent(tgtEl, 'mouseup', endX, endY);

                    if (window.ipc) {{
                        window.ipc.postMessage(JSON.stringify({{
                            type: 'agent_response',
                            request_id: {},
                            action: 'drag_and_drop_element',
                            payload: {{
                                success: true,
                                source: {},
                                target: {},
                                start: {{ x: startX, y: startY }},
                                end: {{ x: endX, y: endY }},
                                steps: steps,
                                dropped: true
                            }}
                        }}));
                    }}
                }} catch (err) {{
                    if (window.ipc) {{
                        window.ipc.postMessage(JSON.stringify({{
                            type: 'agent_response',
                            request_id: {},
                            action: 'drag_and_drop_element',
                            error: String(err)
                        }}));
                    }}
                }}
            }})();"#,
            src_expr, src_json, tgt_expr, tgt_json, steps, hold_ms, request_id, src_json, tgt_json, request_id
        )
    }

    /// JavaScript to audit forms, input validation rules, and validity states.
    pub fn get_audit_forms_script(request_id: u64, target: Option<&str>) -> String {
        let (root_expr, target_json) = match target {
            Some(t) => {
                let json = serde_json::to_string(t).unwrap_or_default();
                (format!("(window.__aerolith_elements && window.__aerolith_elements[{}]) || document.querySelector({})", json, json), json)
            }
            None => ("document".to_string(), "null".to_string()),
        };

        format!(
            r#"(function() {{
                try {{
                    const root = {};
                    if (!root) throw new Error("Target root element not found for: " + {});
                    const formElements = root.tagName === 'FORM' ? [root] : Array.from(root.querySelectorAll('form'));

                    const formsData = [];
                    for (let fIdx = 0; fIdx < formElements.length; fIdx++) {{
                        const form = formElements[fIdx];
                        const controls = Array.from(form.querySelectorAll('input, select, textarea, button'));
                        const fields = [];
                        let formIsValid = true;

                        for (const ctrl of controls) {{
                            const isButton = ctrl.tagName === 'BUTTON' || (ctrl.tagName === 'INPUT' && ['submit', 'button', 'reset'].includes(ctrl.type));
                            const validity = ctrl.validity ? {{
                                valid: ctrl.validity.valid,
                                value_missing: ctrl.validity.valueMissing,
                                type_mismatch: ctrl.validity.typeMismatch,
                                pattern_mismatch: ctrl.validity.patternMismatch,
                                too_short: ctrl.validity.tooShort,
                                too_long: ctrl.validity.tooLong
                            }} : null;

                            if (ctrl.validity && !ctrl.validity.valid) formIsValid = false;

                            let val = '';
                            if (ctrl.type === 'password') {{
                                val = ctrl.value ? '[PASSWORD ' + ctrl.value.length + ' chars]' : '';
                            }} else if (ctrl.type === 'checkbox' || ctrl.type === 'radio') {{
                                val = ctrl.checked ? 'true' : 'false';
                            }} else {{
                                val = (ctrl.value || '').slice(0, 100);
                            }}

                            const labels = [];
                            if (ctrl.labels) {{
                                for (let l = 0; l < ctrl.labels.length; l++) {{
                                    labels.push(ctrl.labels[l].textContent.trim());
                                }}
                            }}

                            fields.push({{
                                tag: ctrl.tagName.toLowerCase(),
                                type: ctrl.type || '',
                                name: ctrl.name || '',
                                id: ctrl.id || '',
                                value: val,
                                placeholder: ctrl.placeholder || '',
                                required: ctrl.required || false,
                                disabled: ctrl.disabled || false,
                                readonly: ctrl.readOnly || false,
                                pattern: ctrl.pattern || '',
                                is_button: isButton,
                                is_submit: ctrl.type === 'submit' || (ctrl.tagName === 'BUTTON' && !ctrl.type),
                                validity: validity,
                                validation_message: ctrl.validationMessage || '',
                                labels: labels
                            }});
                        }}

                        formsData.push({{
                            form_id: form.id || '',
                            form_name: form.name || '',
                            action: form.action || '',
                            method: (form.method || 'GET').toUpperCase(),
                            is_valid: formIsValid,
                            fields_count: fields.length,
                            fields: fields
                        }});
                    }}

                    if (window.ipc) {{
                        window.ipc.postMessage(JSON.stringify({{
                            type: 'agent_response',
                            request_id: {},
                            action: 'audit_forms',
                            payload: {{
                                success: true,
                                forms_count: formsData.length,
                                forms: formsData
                            }}
                        }}));
                    }}
                }} catch (err) {{
                    if (window.ipc) {{
                        window.ipc.postMessage(JSON.stringify({{
                            type: 'agent_response',
                            request_id: {},
                            action: 'audit_forms',
                            error: String(err)
                        }}));
                    }}
                }}
            }})();"#,
            root_expr, target_json, request_id, request_id
        )
    }

    /// JavaScript to compute viewport paging metrics and multi-page scroll coordinates.
    pub fn get_viewport_paging_script(request_id: u64) -> String {
        format!(
            r#"(function() {{
                try {{
                    const vh = window.innerHeight || 800;
                    const vw = window.innerWidth || 1280;
                    const scrollY = window.scrollY || window.pageYOffset || 0;
                    const scrollX = window.scrollX || window.pageXOffset || 0;
                    const docEl = document.documentElement || {{}};
                    const body = document.body || {{}};

                    const scrollHeight = Math.max(docEl.scrollHeight || 0, body.scrollHeight || 0, vh);
                    const scrollWidth = Math.max(docEl.scrollWidth || 0, body.scrollWidth || 0, vw);

                    const totalPages = Math.ceil(scrollHeight / vh);
                    const currentPage = Math.min(totalPages, Math.floor(scrollY / vh) + 1);
                    const canScrollDown = scrollY + vh < scrollHeight - 5;
                    const canScrollUp = scrollY > 5;

                    const nextPageY = Math.min(scrollHeight - vh, scrollY + vh);
                    const prevPageY = Math.max(0, scrollY - vh);

                    const maxScrollY = Math.max(1, scrollHeight - vh);
                    const scrollProgress = Math.min(100, Math.max(0, Math.round((scrollY / maxScrollY) * 100)));

                    if (window.ipc) {{
                        window.ipc.postMessage(JSON.stringify({{
                            type: 'agent_response',
                            request_id: {},
                            action: 'get_viewport_paging',
                            payload: {{
                                success: true,
                                viewport_width: vw,
                                viewport_height: vh,
                                scroll_x: scrollX,
                                scroll_y: scrollY,
                                scroll_width: scrollWidth,
                                scroll_height: scrollHeight,
                                total_pages: totalPages,
                                current_page: currentPage,
                                can_scroll_down: canScrollDown,
                                can_scroll_up: canScrollUp,
                                next_page_scroll_y: nextPageY,
                                prev_page_scroll_y: prevPageY,
                                scroll_progress_percent: scrollProgress
                            }}
                        }}));
                    }}
                }} catch (err) {{
                    if (window.ipc) {{
                        window.ipc.postMessage(JSON.stringify({{
                            type: 'agent_response',
                            request_id: {},
                            action: 'get_viewport_paging',
                            error: String(err)
                        }}));
                    }}
                }}
            }})();"#,
            request_id, request_id
        )
    }

    /// JavaScript to inspect computed styles of pseudo-elements (e.g. ::before, ::after).
    pub fn get_computed_pseudo_element_script(
        request_id: u64,
        target: &str,
        pseudo: &str,
        properties: &[String],
    ) -> String {
        let is_numeric = target.chars().all(|c| c.is_ascii_digit());
        let target_expr = if is_numeric {
            format!("(window.__aerolith_elements && window.__aerolith_elements[{}]) || document.querySelector('[data-aerolith-ref=\"{}\"]')", target, target)
        } else {
            format!("document.querySelector({})", serde_json::to_string(target).unwrap_or_default())
        };
        let target_json = serde_json::to_string(target).unwrap_or_default();
        let pseudo_json = serde_json::to_string(pseudo).unwrap_or_default();
        let props_json = serde_json::to_string(properties).unwrap_or_else(|_| "[]".into());

        format!(
            r#"(function() {{
                try {{
                    const el = {};
                    if (!el) throw new Error("Element not found for target: " + {});
                    const pseudo = {};
                    const props = {};
                    const style = window.getComputedStyle(el, pseudo);

                    const result = {{}};
                    if (props && props.length > 0) {{
                        for (const p of props) {{
                            result[p] = style.getPropertyValue(p) || style[p] || '';
                        }}
                    }} else {{
                        const defaultProps = ['content', 'display', 'visibility', 'color', 'background-color', 'opacity', 'width', 'height', 'font-size', 'font-family'];
                        for (const p of defaultProps) {{
                            result[p] = style.getPropertyValue(p) || style[p] || '';
                        }}
                    }}

                    if (window.ipc) {{
                        window.ipc.postMessage(JSON.stringify({{
                            type: 'agent_response',
                            request_id: {},
                            action: 'get_computed_pseudo_element',
                            payload: {{
                                success: true,
                                target: {},
                                pseudo: pseudo,
                                properties: result
                            }}
                        }}));
                    }}
                }} catch (err) {{
                    if (window.ipc) {{
                        window.ipc.postMessage(JSON.stringify({{
                            type: 'agent_response',
                            request_id: {},
                            action: 'get_computed_pseudo_element',
                            error: String(err)
                        }}));
                    }}
                }}
            }})();"#,
            target_expr, target_json, pseudo_json, props_json, request_id, target_json, request_id
        )
    }

    /// JavaScript to simulate a clipboard paste event into a target or active element with DataTransfer text.
    pub fn get_simulate_clipboard_paste_script(
        request_id: u64,
        target: Option<&str>,
        text: &str,
    ) -> String {
        let (target_expr, target_json) = match target {
            Some(t) => {
                let is_numeric = t.chars().all(|c| c.is_ascii_digit());
                let expr = if is_numeric {
                    format!("(window.__aerolith_elements && window.__aerolith_elements[{}]) || document.querySelector('[data-aerolith-ref=\"{}\"]')", t, t)
                } else {
                    format!("document.querySelector({})", serde_json::to_string(t).unwrap_or_default())
                };
                (expr, serde_json::to_string(t).unwrap_or_default())
            }
            None => ("document.activeElement || document.body".to_string(), "null".to_string()),
        };
        let text_json = serde_json::to_string(text).unwrap_or_default();

        format!(
            r#"(function() {{
                try {{
                    const el = {};
                    if (!el) throw new Error("Target element not found for paste: " + {});
                    const text = {};

                    if (el.focus && typeof el.focus === 'function') {{
                        el.focus();
                    }}

                    let dt = null;
                    try {{
                        dt = new DataTransfer();
                        dt.setData('text/plain', text);
                    }} catch (e) {{}}

                    let pasteEvt = null;
                    try {{
                        pasteEvt = new ClipboardEvent('paste', {{
                            bubbles: true,
                            cancelable: true,
                            composed: true,
                            clipboardData: dt
                        }});
                    }} catch (e) {{
                        try {{
                            pasteEvt = document.createEvent('Event');
                            pasteEvt.initEvent('paste', true, true);
                            if (dt) pasteEvt.clipboardData = dt;
                        }} catch (e2) {{}}
                    }}

                    let prevented = false;
                    if (pasteEvt) {{
                        prevented = !el.dispatchEvent(pasteEvt);
                    }}

                    let insertedFallback = false;
                    if (!prevented) {{
                        if (document.queryCommandSupported && document.queryCommandSupported('insertText')) {{
                            insertedFallback = document.execCommand('insertText', false, text);
                        }}
                        if (!insertedFallback && (el.tagName === 'INPUT' || el.tagName === 'TEXTAREA')) {{
                            const start = el.selectionStart || 0;
                            const end = el.selectionEnd || 0;
                            const current = el.value || '';
                            el.value = current.substring(0, start) + text + current.substring(end);
                            el.selectionStart = el.selectionEnd = start + text.length;
                            try {{
                                el.dispatchEvent(new Event('input', {{ bubbles: true }}));
                                el.dispatchEvent(new Event('change', {{ bubbles: true }}));
                            }} catch (e) {{}}
                            insertedFallback = true;
                        }}
                    }}

                    if (window.ipc) {{
                        window.ipc.postMessage(JSON.stringify({{
                            type: 'agent_response',
                            request_id: {},
                            action: 'simulate_clipboard_paste',
                            payload: {{
                                success: true,
                                target: {},
                                pasted_length: text.length,
                                default_prevented: prevented,
                                fallback_inserted: insertedFallback
                            }}
                        }}));
                    }}
                }} catch (err) {{
                    if (window.ipc) {{
                        window.ipc.postMessage(JSON.stringify({{
                            type: 'agent_response',
                            request_id: {},
                            action: 'simulate_clipboard_paste',
                            error: String(err)
                        }}));
                    }}
                }}
            }})();"#,
            target_expr, target_json, text_json, request_id, target_json, request_id
        )
    }

    /// JavaScript to intelligently map semantic keys to form controls and auto-fill them.
    pub fn get_smart_fill_script(
        request_id: u64,
        mapping: &std::collections::HashMap<String, serde_json::Value>,
        submit: bool,
    ) -> String {
        let mapping_json = serde_json::to_string(mapping).unwrap_or_else(|_| "{}".into());

        format!(
            r#"(function() {{
                try {{
                    const mapping = {};
                    const shouldSubmit = {};
                    const keys = Object.keys(mapping);
                    const filled = [];
                    const matchedKeys = new Set();

                    const normalize = (s) => String(s || '').toLowerCase().replace(/[^a-z0-9]/g, '');

                    const controls = Array.from(document.querySelectorAll('input, select, textarea'));
                    let targetForm = null;

                    for (const key of keys) {{
                        const val = mapping[key];
                        const normKey = normalize(key);
                        let matchedEl = null;

                        for (const ctrl of controls) {{
                            if (ctrl.type === 'hidden' || ctrl.disabled) continue;

                            let labelText = '';
                            if (ctrl.labels && ctrl.labels.length > 0) {{
                                labelText = Array.from(ctrl.labels).map(l => l.textContent).join(' ');
                            }} else if (ctrl.id) {{
                                const l = document.querySelector('label[for="' + ctrl.id + '"]');
                                if (l) labelText = l.textContent;
                            }}

                            const normLabel = normalize(labelText);
                            const normPlaceholder = normalize(ctrl.placeholder);
                            const normName = normalize(ctrl.name);
                            const normId = normalize(ctrl.id);
                            const normAria = normalize(ctrl.getAttribute('aria-label') || '');

                            if ((normLabel && (normLabel.includes(normKey) || normKey.includes(normLabel))) ||
                                (normPlaceholder && (normPlaceholder.includes(normKey) || normKey.includes(normPlaceholder))) ||
                                (normName && (normName.includes(normKey) || normKey.includes(normName))) ||
                                (normId && (normId.includes(normKey) || normKey.includes(normId))) ||
                                (normAria && (normAria.includes(normKey) || normKey.includes(normAria)))) {{
                                matchedEl = ctrl;
                                break;
                            }}
                        }}

                        if (matchedEl) {{
                            matchedKeys.add(key);
                            if (matchedEl.form) targetForm = matchedEl.form;

                            if (matchedEl.tagName === 'SELECT') {{
                                const strVal = String(val).toLowerCase();
                                let foundOpt = false;
                                for (let i = 0; i < matchedEl.options.length; i++) {{
                                    const opt = matchedEl.options[i];
                                    if (opt.value.toLowerCase() === strVal || opt.text.toLowerCase().includes(strVal)) {{
                                        matchedEl.selectedIndex = i;
                                        foundOpt = true;
                                        break;
                                    }}
                                }}
                                matchedEl.dispatchEvent(new Event('input', {{ bubbles: true }}));
                                matchedEl.dispatchEvent(new Event('change', {{ bubbles: true }}));
                                filled.push({{ key: key, tag: 'select', matched_by: matchedEl.name || matchedEl.id || 'select', success: foundOpt }});
                            }} else if (matchedEl.type === 'checkbox' || matchedEl.type === 'radio') {{
                                const boolVal = val === true || val === 'true' || val === 1 || val === '1';
                                matchedEl.checked = boolVal;
                                matchedEl.dispatchEvent(new Event('change', {{ bubbles: true }}));
                                filled.push({{ key: key, tag: matchedEl.type, matched_by: matchedEl.name || matchedEl.id || matchedEl.type, checked: boolVal }});
                            }} else {{
                                matchedEl.value = String(val === null || val === undefined ? '' : val);
                                matchedEl.dispatchEvent(new Event('input', {{ bubbles: true }}));
                                matchedEl.dispatchEvent(new Event('change', {{ bubbles: true }}));
                                matchedEl.dispatchEvent(new Event('blur', {{ bubbles: true }}));
                                filled.push({{ key: key, tag: matchedEl.tagName.toLowerCase(), matched_by: matchedEl.name || matchedEl.id || matchedEl.placeholder || 'input' }});
                            }}
                        }}
                    }}

                    let submitted = false;
                    if (shouldSubmit && targetForm) {{
                        const submitBtn = targetForm.querySelector('button[type="submit"], input[type="submit"], button:not([type])');
                        if (submitBtn) {{
                            submitBtn.click();
                            submitted = true;
                        }} else if (typeof targetForm.requestSubmit === 'function') {{
                            targetForm.requestSubmit();
                            submitted = true;
                        }}
                    }}

                    const unmatched = keys.filter(k => !matchedKeys.has(k));

                    if (window.ipc) {{
                        window.ipc.postMessage(JSON.stringify({{
                            type: 'agent_response',
                            request_id: {},
                            action: 'smart_fill',
                            payload: {{
                                success: true,
                                filled_count: filled.length,
                                total_keys: keys.length,
                                filled: filled,
                                unmatched_keys: unmatched,
                                form_submitted: submitted
                            }}
                        }}));
                    }}
                }} catch (err) {{
                    if (window.ipc) {{
                        window.ipc.postMessage(JSON.stringify({{
                            type: 'agent_response',
                            request_id: {},
                            action: 'smart_fill',
                            error: String(err)
                        }}));
                    }}
                }}
            }})();"#,
            mapping_json, submit, request_id, request_id
        )
    }

    /// JavaScript to observe streaming or asynchronous text updates on an element until it settles or times out.
    pub fn get_observe_text_script(
        request_id: u64,
        target: &str,
        timeout_ms: u64,
        settle_ms: u64,
    ) -> String {
        let is_numeric = target.chars().all(|c| c.is_ascii_digit());
        let target_expr = if is_numeric {
            format!("(window.__aerolith_elements && window.__aerolith_elements[{}]) || document.querySelector('[data-aerolith-ref=\"{}\"]')", target, target)
        } else {
            format!("document.querySelector({})", serde_json::to_string(target).unwrap_or_default())
        };
        let target_json = serde_json::to_string(target).unwrap_or_default();

        format!(
            r#"(function() {{
                try {{
                    const el = {};
                    if (!el) throw new Error("Target element not found to observe: " + {});
                    const maxTimeout = Math.max(500, {});
                    const settleDelay = Math.max(200, {});

                    const initialText = el.innerText || el.textContent || '';
                    let currentText = initialText;
                    let mutationCount = 0;
                    const startTime = Date.now();

                    let settleTimer = null;
                    let overallTimer = null;
                    let finished = false;

                    const finish = (settled, timedOut) => {{
                        if (finished) return;
                        finished = true;
                        if (settleTimer) clearTimeout(settleTimer);
                        if (overallTimer) clearTimeout(overallTimer);
                        try {{ observer.disconnect(); }} catch (e) {{}}

                        const finalText = el.innerText || el.textContent || '';
                        if (window.ipc) {{
                            window.ipc.postMessage(JSON.stringify({{
                                type: 'agent_response',
                                request_id: {},
                                action: 'observe_text',
                                payload: {{
                                    success: true,
                                    target: {},
                                    initial_text: initialText,
                                    final_text: finalText,
                                    text_length: finalText.length,
                                    mutations_count: mutationCount,
                                    settled: settled,
                                    timed_out: timedOut,
                                    elapsed_ms: Date.now() - startTime
                                }}
                            }}));
                        }}
                    }};

                    const observer = new MutationObserver(() => {{
                        mutationCount++;
                        currentText = el.innerText || el.textContent || '';
                        if (settleTimer) clearTimeout(settleTimer);
                        settleTimer = setTimeout(() => finish(true, false), settleDelay);
                    }});

                    observer.observe(el, {{
                        characterData: true,
                        childList: true,
                        subtree: true
                    }});

                    overallTimer = setTimeout(() => finish(false, true), maxTimeout);
                }} catch (err) {{
                    if (window.ipc) {{
                        window.ipc.postMessage(JSON.stringify({{
                            type: 'agent_response',
                            request_id: {},
                            action: 'observe_text',
                            error: String(err)
                        }}));
                    }}
                }}
            }})();"#,
            target_expr, target_json, timeout_ms, settle_ms, request_id, target_json, request_id
        )
    }

    /// JavaScript to override document.visibilityState, document.hidden, and dispatch visibilitychange events.
    pub fn get_set_visibility_state_script(request_id: u64, state: &str) -> String {
        let is_hidden = state == "hidden";
        let state_json = serde_json::to_string(state).unwrap_or_default();

        format!(
            r#"(function() {{
                try {{
                    const targetState = {};
                    const isHidden = {};

                    try {{
                        Object.defineProperty(Document.prototype, 'visibilityState', {{
                            get: function() {{ return targetState; }},
                            configurable: true
                        }});
                    }} catch (e) {{
                        try {{
                            Object.defineProperty(document, 'visibilityState', {{
                                get: function() {{ return targetState; }},
                                configurable: true
                            }});
                        }} catch (e2) {{}}
                    }}

                    try {{
                        Object.defineProperty(Document.prototype, 'hidden', {{
                            get: function() {{ return isHidden; }},
                            configurable: true
                        }});
                    }} catch (e) {{
                        try {{
                            Object.defineProperty(document, 'hidden', {{
                                get: function() {{ return isHidden; }},
                                configurable: true
                            }});
                        }} catch (e2) {{}}
                    }}

                    try {{
                        document.dispatchEvent(new Event('visibilitychange', {{ bubbles: true }}));
                        window.dispatchEvent(new Event(isHidden ? 'blur' : 'focus'));
                    }} catch (e) {{}}

                    if (window.ipc) {{
                        window.ipc.postMessage(JSON.stringify({{
                            type: 'agent_response',
                            request_id: {},
                            action: 'set_visibility_state',
                            payload: {{
                                success: true,
                                visibility_state: targetState,
                                hidden: isHidden
                            }}
                        }}));
                    }}
                }} catch (err) {{
                    if (window.ipc) {{
                        window.ipc.postMessage(JSON.stringify({{
                            type: 'agent_response',
                            request_id: {},
                            action: 'set_visibility_state',
                            error: String(err)
                        }}));
                    }}
                }}
            }})();"#,
            state_json, is_hidden, request_id, request_id
        )
    }

    /// JavaScript to extract typography, font family, weight, line-height, and canvas measurement metrics.
    pub fn get_computed_font_script(request_id: u64, target: &str) -> String {
        let is_numeric = target.chars().all(|c| c.is_ascii_digit());
        let target_expr = if is_numeric {
            format!("(window.__aerolith_elements && window.__aerolith_elements[{}]) || document.querySelector('[data-aerolith-ref=\"{}\"]')", target, target)
        } else {
            format!("document.querySelector({})", serde_json::to_string(target).unwrap_or_default())
        };
        let target_json = serde_json::to_string(target).unwrap_or_default();

        format!(
            r#"(function() {{
                try {{
                    const el = {};
                    if (!el) throw new Error("Target element not found for font inspection: " + {});
                    const style = window.getComputedStyle(el);

                    let fontLoaded = true;
                    if (document.fonts && typeof document.fonts.check === 'function') {{
                        try {{
                            fontLoaded = document.fonts.check((style.fontSize || '16px') + ' ' + (style.fontFamily || 'sans-serif'));
                        }} catch (e) {{}}
                    }}

                    let textMetrics = null;
                    try {{
                        const canvas = document.createElement('canvas');
                        const ctx = canvas.getContext('2d');
                        if (ctx) {{
                            ctx.font = (style.fontStyle || 'normal') + ' ' + (style.fontWeight || '400') + ' ' + (style.fontSize || '16px') + ' ' + (style.fontFamily || 'sans-serif');
                            const sampleText = (el.innerText || el.textContent || 'Sample Text').slice(0, 50);
                            const m = ctx.measureText(sampleText);
                            textMetrics = {{
                                measured_sample: sampleText,
                                width_px: Math.round(m.width * 100) / 100,
                                actual_bounding_box_ascent: m.actualBoundingBoxAscent || 0,
                                actual_bounding_box_descent: m.actualBoundingBoxDescent || 0
                            }};
                        }}
                    }} catch (e) {{}}

                    if (window.ipc) {{
                        window.ipc.postMessage(JSON.stringify({{
                            type: 'agent_response',
                            request_id: {},
                            action: 'get_computed_font',
                            payload: {{
                                success: true,
                                target: {},
                                font_family: style.fontFamily,
                                font_size: style.fontSize,
                                font_weight: style.fontWeight,
                                font_style: style.fontStyle,
                                line_height: style.lineHeight,
                                letter_spacing: style.letterSpacing,
                                text_align: style.textAlign,
                                text_transform: style.textTransform,
                                text_decoration: style.textDecorationLine || style.textDecoration,
                                color: style.color,
                                font_loaded: fontLoaded,
                                canvas_metrics: textMetrics
                            }}
                        }}));
                    }}
                }} catch (err) {{
                    if (window.ipc) {{
                        window.ipc.postMessage(JSON.stringify({{
                            type: 'agent_response',
                            request_id: {},
                            action: 'get_computed_font',
                            error: String(err)
                        }}));
                    }}
                }}
            }})();"#,
            target_expr, target_json, request_id, target_json, request_id
        )
    }

    /// JavaScript to extract dense interactive element click-map with centroids and ref bindings.
    pub fn get_action_map_script(request_id: u64, viewport_only: bool, max_elements: usize) -> String {
        format!(
            r#"(function() {{
                try {{
                    const viewportOnly = {};
                    const maxLimit = Math.max(1, {});
                    const vh = window.innerHeight || 800;
                    const vw = window.innerWidth || 1280;

                    if (!window.__aerolith_elements) window.__aerolith_elements = {{}};
                    if (!window.__aerolith_next_ref) window.__aerolith_next_ref = 1;

                    const candidates = Array.from(document.querySelectorAll(
                        'a[href], button, input:not([type="hidden"]), select, textarea, [role="button"], [role="link"], [role="checkbox"], [role="tab"], [role="menuitem"], [onclick], [tabindex]:not([tabindex="-1"])'
                    ));

                    const actionList = [];
                    let totalInteractive = 0;

                    for (let i = 0; i < candidates.length; i++) {{
                        const el = candidates[i];
                        const rect = el.getBoundingClientRect();
                        if (rect.width <= 2 || rect.height <= 2) continue;

                        const style = window.getComputedStyle(el);
                        if (style.display === 'none' || style.visibility === 'hidden' || parseFloat(style.opacity) < 0.05) continue;

                        const inView = rect.bottom > 0 && rect.right > 0 && rect.top < vh && rect.left < vw;
                        totalInteractive++;
                        if (viewportOnly && !inView) continue;

                        let ref = el.getAttribute('data-aerolith-ref');
                        if (!ref) {{
                            ref = String(window.__aerolith_next_ref++);
                            el.setAttribute('data-aerolith-ref', ref);
                        }}
                        window.__aerolith_elements[ref] = el;

                        let label = '';
                        if (el.getAttribute('aria-label')) {{
                            label = el.getAttribute('aria-label');
                        }} else if (el.labels && el.labels.length > 0) {{
                            label = Array.from(el.labels).map(l => l.textContent.trim()).join(' ');
                        }} else if (el.innerText && el.innerText.trim()) {{
                            label = el.innerText.trim().slice(0, 80);
                        }} else if (el.title) {{
                            label = el.title;
                        }} else if (el.placeholder) {{
                            label = el.placeholder;
                        }} else if (el.value) {{
                            label = String(el.value).slice(0, 50);
                        }}

                        let role = el.getAttribute('role') || el.tagName.toLowerCase();
                        if (el.tagName === 'INPUT') role = 'input-' + (el.type || 'text');

                        const cx = Math.round(rect.left + rect.width / 2);
                        const cy = Math.round(rect.top + rect.height / 2);
                        const isDisabled = Boolean(el.disabled || el.getAttribute('aria-disabled') === 'true');

                        actionList.push({{
                            ref: ref,
                            tag: el.tagName.toLowerCase(),
                            role: role,
                            label: label,
                            center_x: cx,
                            center_y: cy,
                            bounds: {{
                                x: Math.round(rect.left),
                                y: Math.round(rect.top),
                                width: Math.round(rect.width),
                                height: Math.round(rect.height)
                            }},
                            in_viewport: inView,
                            disabled: isDisabled
                        }});

                        if (actionList.length >= maxLimit) break;
                    }}

                    if (window.ipc) {{
                        window.ipc.postMessage(JSON.stringify({{
                            type: 'agent_response',
                            request_id: {},
                            action: 'get_action_map',
                            payload: {{
                                success: true,
                                total_interactive_count: totalInteractive,
                                returned_count: actionList.length,
                                viewport_only: viewportOnly,
                                elements: actionList
                            }}
                        }}));
                    }}
                }} catch (err) {{
                    if (window.ipc) {{
                        window.ipc.postMessage(JSON.stringify({{
                            type: 'agent_response',
                            request_id: {},
                            action: 'get_action_map',
                            error: String(err)
                        }}));
                    }}
                }}
            }})();"#,
            viewport_only, max_elements, request_id, request_id
        )
    }

    /// JavaScript to inspect active CSS animations and transitions on the page or target element.
    pub fn get_motion_state_script(request_id: u64, target: Option<&str>) -> String {
        let (root_expr, target_json) = match target {
            Some(t) => {
                let json = serde_json::to_string(t).unwrap_or_default();
                (format!("(window.__aerolith_elements && window.__aerolith_elements[{}]) || document.querySelector({})", json, json), json)
            }
            None => ("document".to_string(), "null".to_string()),
        };

        format!(
            r#"(function() {{
                try {{
                    const root = {};
                    if (!root) throw new Error("Target not found for motion state: " + {});

                    let anims = [];
                    if (typeof root.getAnimations === 'function') {{
                        anims = root.getAnimations();
                    }} else if (typeof document.getAnimations === 'function') {{
                        anims = document.getAnimations();
                    }}

                    let runningCount = 0;
                    const animDetails = [];

                    for (let i = 0; i < anims.length; i++) {{
                        const a = anims[i];
                        const playState = a.playState || 'unknown';
                        if (playState === 'running') runningCount++;

                        let duration = null;
                        if (a.effect && typeof a.effect.getTiming === 'function') {{
                            const timing = a.effect.getTiming();
                            duration = typeof timing.duration === 'number' ? timing.duration : null;
                        }}

                        let targetTag = 'unknown';
                        if (a.effect && a.effect.target) {{
                            targetTag = (a.effect.target.tagName || '').toLowerCase();
                        }}

                        animDetails.push({{
                            name: a.animationName || 'transition',
                            play_state: playState,
                            duration_ms: duration,
                            current_time_ms: typeof a.currentTime === 'number' ? Math.round(a.currentTime) : null,
                            target_tag: targetTag
                        }});
                    }}

                    if (window.ipc) {{
                        window.ipc.postMessage(JSON.stringify({{
                            type: 'agent_response',
                            request_id: {},
                            action: 'get_motion_state',
                            payload: {{
                                success: true,
                                is_animating: runningCount > 0,
                                running_count: runningCount,
                                total_animations: anims.length,
                                animations: animDetails
                            }}
                        }}));
                    }}
                }} catch (err) {{
                    if (window.ipc) {{
                        window.ipc.postMessage(JSON.stringify({{
                            type: 'agent_response',
                            request_id: {},
                            action: 'get_motion_state',
                            error: String(err)
                        }}));
                    }}
                }}
            }})();"#,
            root_expr, target_json, request_id, request_id
        )
    }

    /// JavaScript to inspect event listeners, cursor style, and interactive affordances of an element.
    pub fn get_element_listeners_script(request_id: u64, target: &str) -> String {
        let is_numeric = target.chars().all(|c| c.is_ascii_digit());
        let target_expr = if is_numeric {
            format!("(window.__aerolith_elements && window.__aerolith_elements[{}]) || document.querySelector('[data-aerolith-ref=\"{}\"]')", target, target)
        } else {
            format!("document.querySelector({})", serde_json::to_string(target).unwrap_or_default())
        };
        let target_json = serde_json::to_string(target).unwrap_or_default();

        format!(
            r#"(function() {{
                try {{
                    const el = {};
                    if (!el) throw new Error("Element not found for listeners inspection: " + {});
                    const style = window.getComputedStyle(el);

                    const knownHandlers = ['onclick', 'onmousedown', 'onmouseup', 'onkeydown', 'onkeyup', 'onchange', 'onsubmit', 'oninput', 'onfocus', 'onblur'];
                    const foundHandlers = [];
                    for (const h of knownHandlers) {{
                        if (el[h] !== null && el[h] !== undefined) foundHandlers.push(h);
                        else if (el.hasAttribute && el.hasAttribute(h)) foundHandlers.push(h);
                    }}

                    const cursor = style.cursor || 'auto';
                    const role = el.getAttribute('role') || '';
                    const tabIndex = el.tabIndex;
                    const isNativeInteractive = ['A', 'BUTTON', 'INPUT', 'SELECT', 'TEXTAREA'].includes(el.tagName);
                    const isRoleInteractive = ['button', 'link', 'checkbox', 'tab', 'menuitem'].includes(role.toLowerCase());
                    const isClickable = isNativeInteractive || isRoleInteractive || cursor === 'pointer' || foundHandlers.includes('onclick');

                    if (window.ipc) {{
                        window.ipc.postMessage(JSON.stringify({{
                            type: 'agent_response',
                            request_id: {},
                            action: 'get_element_listeners',
                            payload: {{
                                success: true,
                                target: {},
                                is_interactive: isClickable,
                                cursor: cursor,
                                aria_role: role,
                                tab_index: tabIndex,
                                inline_handlers: foundHandlers,
                                has_click_handler: foundHandlers.includes('onclick') || cursor === 'pointer'
                            }}
                        }}));
                    }}
                }} catch (err) {{
                    if (window.ipc) {{
                        window.ipc.postMessage(JSON.stringify({{
                            type: 'agent_response',
                            request_id: {},
                            action: 'get_element_listeners',
                            error: String(err)
                        }}));
                    }}
                }}
            }})();"#,
            target_expr, target_json, request_id, target_json, request_id
        )
    }

    /// JavaScript to extract Core Web Vitals and performance navigation timings.
    pub fn get_web_vitals_script(request_id: u64) -> String {
        format!(
            r#"(function() {{
                try {{
                    const navEntries = (window.performance && window.performance.getEntriesByType)
                        ? window.performance.getEntriesByType('navigation')
                        : [];
                    const nav = navEntries.length > 0 ? navEntries[0] : null;
                    const timing = window.performance ? window.performance.timing : null;

                    let ttfb = null;
                    let domContentLoaded = null;
                    let domInteractive = null;
                    let loadComplete = null;
                    let dnsLookup = null;
                    let tcpConnect = null;

                    if (nav) {{
                        ttfb = Math.round(nav.responseStart - nav.requestStart);
                        domContentLoaded = Math.round(nav.domContentLoadedEventEnd - nav.fetchStart);
                        domInteractive = Math.round(nav.domInteractive - nav.fetchStart);
                        loadComplete = Math.round(nav.loadEventEnd - nav.fetchStart);
                        dnsLookup = Math.round(nav.domainLookupEnd - nav.domainLookupStart);
                        tcpConnect = Math.round(nav.connectEnd - nav.connectStart);
                    }} else if (timing) {{
                        ttfb = Math.max(0, timing.responseStart - timing.requestStart);
                        domContentLoaded = Math.max(0, timing.domContentLoadedEventEnd - timing.navigationStart);
                        domInteractive = Math.max(0, timing.domInteractive - timing.navigationStart);
                        loadComplete = Math.max(0, timing.loadEventEnd - timing.navigationStart);
                        dnsLookup = Math.max(0, timing.domainLookupEnd - timing.domainLookupStart);
                        tcpConnect = Math.max(0, timing.connectEnd - timing.connectStart);
                    }}

                    let fcp = null;
                    try {{
                        const fcpEntries = window.performance ? window.performance.getEntriesByName('first-contentful-paint') : [];
                        if (fcpEntries.length > 0) fcp = Math.round(fcpEntries[0].startTime);
                    }} catch (e) {{}}

                    let jsHeapUsedMb = null;
                    let jsHeapTotalMb = null;
                    if (window.performance && window.performance.memory) {{
                        jsHeapUsedMb = Math.round(window.performance.memory.usedJSHeapSize / (1024 * 1024) * 100) / 100;
                        jsHeapTotalMb = Math.round(window.performance.memory.totalJSHeapSize / (1024 * 1024) * 100) / 100;
                    }}

                    if (window.ipc) {{
                        window.ipc.postMessage(JSON.stringify({{
                            type: 'agent_response',
                            request_id: {},
                            action: 'get_web_vitals',
                            payload: {{
                                success: true,
                                ttfb_ms: ttfb,
                                dom_content_loaded_ms: domContentLoaded,
                                dom_interactive_ms: domInteractive,
                                load_complete_ms: loadComplete,
                                dns_lookup_ms: dnsLookup,
                                tcp_connect_ms: tcpConnect,
                                first_contentful_paint_ms: fcp,
                                js_heap_used_mb: jsHeapUsedMb,
                                js_heap_total_mb: jsHeapTotalMb
                            }}
                        }}));
                    }}
                }} catch (err) {{
                    if (window.ipc) {{
                        window.ipc.postMessage(JSON.stringify({{
                            type: 'agent_response',
                            request_id: {},
                            action: 'get_web_vitals',
                            error: String(err)
                        }}));
                    }}
                }}
            }})();"#,
            request_id, request_id
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

        let inspect_script = AgentScripts::get_inspect_script(65, "button#checkout");
        assert!(inspect_script.contains("request_id: 65"));
        assert!(inspect_script.contains("action: 'inspect'"));
        assert!(inspect_script.contains("button#checkout"));
        assert!(inspect_script.contains("computed_style"));

        let route_script = AgentScripts::get_route_rule_script(
            66,
            "add",
            Some("*.png"),
            Some("abort"),
            Some(404),
            None,
            None,
        );
        assert!(route_script.contains("request_id: 66"));
        assert!(route_script.contains("action: 'route_rule'"));
        assert!(route_script.contains("*.png"));
        assert!(route_script.contains("abort"));

        let wait_idle_script = AgentScripts::get_wait_for_script(67, "networkidle", 5000);
        assert!(wait_idle_script.contains("networkidle"));
        assert!(wait_idle_script.contains("idleConsecutive"));

        let wait_url_script = AgentScripts::get_wait_for_script(68, "url:/dashboard", 5000);
        assert!(wait_url_script.contains("url:"));
        assert!(wait_url_script.contains("/dashboard"));

        let dbl_script = AgentScripts::get_double_click_script(69, "tr.row-5");
        assert!(dbl_script.contains("request_id: 69"));
        assert!(dbl_script.contains("action: 'double_click'"));
        assert!(dbl_script.contains("dblclick"));
        assert!(dbl_script.contains("tr.row-5"));

        let rc_script = AgentScripts::get_right_click_script(70, "div.item");
        assert!(rc_script.contains("request_id: 70"));
        assert!(rc_script.contains("action: 'right_click'"));
        assert!(rc_script.contains("contextmenu"));

        let tbl_script = AgentScripts::get_extract_table_script(71, Some("table.rates"));
        assert!(tbl_script.contains("request_id: 71"));
        assert!(tbl_script.contains("action: 'extract_table'"));
        assert!(tbl_script.contains("table.rates"));

        let dl_script = AgentScripts::get_downloads_script(72, true);
        assert!(dl_script.contains("request_id: 72"));
        assert!(dl_script.contains("action: 'downloads'"));
        assert!(dl_script.contains("__aerolith_downloads"));

        let css_script = AgentScripts::get_inject_css_script(73, "nav { display: none; }", false);
        assert!(css_script.contains("request_id: 73"));
        assert!(css_script.contains("action: 'inject_css'"));
        assert!(css_script.contains("__aerolith_injected_css"));

        let q_script = AgentScripts::get_query_elements_script(74, "a.result", Some("href"), 10);
        assert!(q_script.contains("request_id: 74"));
        assert!(q_script.contains("action: 'query_elements'"));
        assert!(q_script.contains("a.result"));
        assert!(q_script.contains("href"));

        let focus_script = AgentScripts::get_focus_script(75, "#search");
        assert!(focus_script.contains("request_id: 75"));
        assert!(focus_script.contains("action: 'focus'"));
        assert!(focus_script.contains("#search"));

        let blur_script = AgentScripts::get_blur_script(76, "#search");
        assert!(blur_script.contains("request_id: 76"));
        assert!(blur_script.contains("action: 'blur'"));

        let info_script = AgentScripts::get_page_info_script(77);
        assert!(info_script.contains("request_id: 77"));
        assert!(info_script.contains("action: 'page_info'"));
        assert!(info_script.contains("scroll_progress"));

        let html_script = AgentScripts::get_html_script(78, Some("#app"));
        assert!(html_script.contains("request_id: 78"));
        assert!(html_script.contains("action: 'get_html'"));
        assert!(html_script.contains("outerHTML"));
        assert!(html_script.contains("#app"));

        let scroll_view_script = AgentScripts::get_scroll_into_view_script(79, "#pricing", "start");
        assert!(scroll_view_script.contains("request_id: 79"));
        assert!(scroll_view_script.contains("action: 'scroll_into_view'"));
        assert!(scroll_view_script.contains("scrollIntoView"));
        assert!(scroll_view_script.contains("in_viewport"));

        let store_item_script = AgentScripts::get_storage_item_script(80, "local", "set", Some("theme"), Some("dark"));
        assert!(store_item_script.contains("request_id: 80"));
        assert!(store_item_script.contains("action: 'storage_item'"));
        assert!(store_item_script.contains("setItem"));

        let assertions = crate::agent::protocol::ElementAssertion {
            visible: Some(true),
            disabled: Some(false),
            text_contains: Some("Submit".into()),
            ..Default::default()
        };
        let assert_el_script = AgentScripts::get_assert_element_script(81, "button#save", &assertions);
        assert!(assert_el_script.contains("request_id: 81"));
        assert!(assert_el_script.contains("action: 'assert_element'"));
        assert!(assert_el_script.contains("button#save"));
        assert!(assert_el_script.contains("Submit"));

        let wait_sel_script = AgentScripts::get_wait_for_selector_script(82, ".spinner", "hidden", 4000);
        assert!(wait_sel_script.contains("request_id: 82"));
        assert!(wait_sel_script.contains("action: 'wait_for_selector'"));
        assert!(wait_sel_script.contains(".spinner"));
        assert!(wait_sel_script.contains("hidden"));

        let perf_script = AgentScripts::get_performance_metrics_script(83);
        assert!(perf_script.contains("request_id: 83"));
        assert!(perf_script.contains("action: 'performance_metrics'"));
        assert!(perf_script.contains("ttfb_ms"));
        assert!(perf_script.contains("dom_content_loaded_ms"));

        let scroll_coords_script = AgentScripts::get_scroll_to_coords_script(84, 0, 500, true);
        assert!(scroll_coords_script.contains("request_id: 84"));
        assert!(scroll_coords_script.contains("action: 'scroll_to_coords'"));
        assert!(scroll_coords_script.contains("scrollTo"));

        let outline_script = AgentScripts::get_page_outline_script(85, 20, 10);
        assert!(outline_script.contains("request_id: 85"));
        assert!(outline_script.contains("action: 'page_outline'"));
        assert!(outline_script.contains("headings"));
        assert!(outline_script.contains("forms"));

        let idle_script = AgentScripts::get_wait_for_network_idle_script(86, 500, 8000, 0);
        assert!(idle_script.contains("request_id: 86"));
        assert!(idle_script.contains("action: 'wait_for_network_idle'"));
        assert!(idle_script.contains("idleRequired = 500"));

        let mut_script = AgentScripts::get_watch_mutations_script(87, "start", Some("#container"));
        assert!(mut_script.contains("request_id: 87"));
        assert!(mut_script.contains("action: 'watch_mutations'"));
        assert!(mut_script.contains("MutationObserver"));

        let resp_body_script = AgentScripts::get_response_body_script(88, "api/data", true);
        assert!(resp_body_script.contains("request_id: 88"));
        assert!(resp_body_script.contains("action: 'response_body'"));
        assert!(resp_body_script.contains("api/data"));

        let drop_script = AgentScripts::get_drop_file_script(89, "#upload", "file.txt", "text/plain", "hello");
        assert!(drop_script.contains("request_id: 89"));
        assert!(drop_script.contains("action: 'drop_file'"));
        assert!(drop_script.contains("file.txt"));
        assert!(drop_script.contains("DataTransfer"));

        let tree_script = AgentScripts::get_dom_tree_script(90, Some("#main"), 3);
        assert!(tree_script.contains("request_id: 90"));
        assert!(tree_script.contains("action: 'dom_tree'"));
        assert!(tree_script.contains("serializeNode"));

        let wait_txt = AgentScripts::get_wait_for_text_script(91, Some("#banner"), "welcome", 4000);
        assert!(wait_txt.contains("request_id: 91"));
        assert!(wait_txt.contains("action: 'wait_for_text'"));
        assert!(wait_txt.contains("welcome"));
        assert!(wait_txt.contains("#banner"));

        let styles_script = AgentScripts::get_computed_styles_script(92, "#header", &["color".into(), "display".into()]);
        assert!(styles_script.contains("request_id: 92"));
        assert!(styles_script.contains("action: 'computed_styles'"));
        assert!(styles_script.contains("#header"));
        assert!(styles_script.contains("getComputedStyle"));

        let wait_dl = AgentScripts::get_wait_for_download_script(93, Some("receipt"), 12000);
        assert!(wait_dl.contains("request_id: 93"));
        assert!(wait_dl.contains("action: 'wait_for_download'"));
        assert!(wait_dl.contains("receipt"));

        let set_cnt = AgentScripts::get_set_content_script(94, "<h1>Welcome Home</h1>");
        assert!(set_cnt.contains("request_id: 94"));
        assert!(set_cnt.contains("action: 'set_content'"));
        assert!(set_cnt.contains("document.write"));
        assert!(set_cnt.contains("Welcome Home"));

        let forms_script = AgentScripts::get_forms_script(95, Some("#auth-modal"));
        assert!(forms_script.contains("request_id: 95"));
        assert!(forms_script.contains("action: 'get_forms'"));
        assert!(forms_script.contains("#auth-modal"));
        assert!(forms_script.contains("querySelectorAll('form')"));

        let vis_script = AgentScripts::get_set_visibility_script(96, "visible");
        assert!(vis_script.contains("request_id: 96"));
        assert!(vis_script.contains("action: 'set_visibility'"));
        assert!(vis_script.contains("visibilitychange"));
        assert!(vis_script.contains("visible"));

        let bounds_script = AgentScripts::get_element_bounds_script(97, "#login-btn");
        assert!(bounds_script.contains("request_id: 97"));
        assert!(bounds_script.contains("action: 'get_element_bounds'"));
        assert!(bounds_script.contains("#login-btn"));
        assert!(bounds_script.contains("getBoundingClientRect"));

        let mock_script = AgentScripts::get_mock_route_script(98, "set", Some("/api/auth"), 200, "application/json", Some(r#"{"status":"ok"}"#));
        assert!(mock_script.contains("request_id: 98"));
        assert!(mock_script.contains("action: 'mock_route'"));
        assert!(mock_script.contains("/api/auth"));
        assert!(mock_script.contains("__aerolith_mocks"));

        let vp_script = AgentScripts::get_viewport_script(99);
        assert!(vp_script.contains("request_id: 99"));
        assert!(vp_script.contains("action: 'get_viewport'"));
        assert!(vp_script.contains("innerWidth"));
        assert!(vp_script.contains("devicePixelRatio"));

        let attr_script = AgentScripts::get_set_element_attribute_script(100, "#btn", "disabled", None);
        assert!(attr_script.contains("request_id: 100"));
        assert!(attr_script.contains("action: 'set_element_attribute'"));
        assert!(attr_script.contains("removeAttribute"));
        assert!(attr_script.contains("disabled"));

        let search_text_script = AgentScripts::get_search_page_text_script(101, "Order Total", 5, false);
        assert!(search_text_script.contains("request_id: 101"));
        assert!(search_text_script.contains("action: 'search_page_text'"));
        assert!(search_text_script.contains("Order Total"));
        assert!(search_text_script.contains("TreeWalker"));

        let dump_script = AgentScripts::get_dump_storage_script(102, "all");
        assert!(dump_script.contains("request_id: 102"));
        assert!(dump_script.contains("action: 'dump_storage'"));
        assert!(dump_script.contains("localStorage"));
        assert!(dump_script.contains("sessionStorage"));

        let jitter_script = AgentScripts::get_human_jitter_script(103, 1200, true, true);
        assert!(jitter_script.contains("request_id: 103"));
        assert!(jitter_script.contains("action: 'human_jitter'"));
        assert!(jitter_script.contains("mousemove"));

        let type_keys_script = AgentScripts::get_type_keystrokes_script(104, "#query", "flights", 20, true, false);
        assert!(type_keys_script.contains("request_id: 104"));
        assert!(type_keys_script.contains("action: 'type_keystrokes'"));
        assert!(type_keys_script.contains("#query"));
        assert!(type_keys_script.contains("flights"));

        let canvas_data_script = AgentScripts::get_canvas_data_script(105, "#qr-canvas", "image/png", 0.9);
        assert!(canvas_data_script.contains("request_id: 105"));
        assert!(canvas_data_script.contains("action: 'get_canvas_data'"));
        assert!(canvas_data_script.contains("#qr-canvas"));
        assert!(canvas_data_script.contains("toDataURL"));

        let res_timing_script = AgentScripts::get_resource_timing_script(106, Some("img"), 30, 10.0);
        assert!(res_timing_script.contains("request_id: 106"));
        assert!(res_timing_script.contains("action: 'get_resource_timing'"));
        assert!(res_timing_script.contains("getEntriesByType('resource')"));
        assert!(res_timing_script.contains("img"));

        let dom_stats_script = AgentScripts::get_dom_stats_script(107);
        assert!(dom_stats_script.contains("request_id: 107"));
        assert!(dom_stats_script.contains("action: 'get_dom_stats'"));
        assert!(dom_stats_script.contains("total_elements"));
        assert!(dom_stats_script.contains("max_depth"));

        let scroll_by_script = AgentScripts::get_scroll_by_script(108, 0, 400, false);
        assert!(scroll_by_script.contains("request_id: 108"));
        assert!(scroll_by_script.contains("action: 'scroll_by'"));
        assert!(scroll_by_script.contains("window.scrollBy"));

        let scroll_cont_script = AgentScripts::get_scroll_container_script(109, "#feed", 0, 500, false);
        assert!(scroll_cont_script.contains("request_id: 109"));
        assert!(scroll_cont_script.contains("action: 'scroll_container'"));
        assert!(scroll_cont_script.contains("#feed"));
        assert!(scroll_cont_script.contains("scrollLeft"));

        let select_txt_script = AgentScripts::get_select_element_text_script(110, "#editor", Some(5), Some(15));
        assert!(select_txt_script.contains("request_id: 110"));
        assert!(select_txt_script.contains("action: 'select_element_text'"));
        assert!(select_txt_script.contains("#editor"));
        assert!(select_txt_script.contains("getSelection"));

        let links_script = AgentScripts::get_extract_page_links_script(111, 50, true, Some("/api"));
        assert!(links_script.contains("request_id: 111"));
        assert!(links_script.contains("action: 'extract_page_links'"));
        assert!(links_script.contains("querySelectorAll('a[href]')"));
        assert!(links_script.contains("/api"));

        let state_script = AgentScripts::get_element_state_script(112, "#submit-btn");
        assert!(state_script.contains("request_id: 112"));
        assert!(state_script.contains("action: 'get_element_state'"));
        assert!(state_script.contains("#submit-btn"));
        assert!(state_script.contains("is_in_viewport"));

        let class_script = AgentScripts::get_toggle_element_class_script(113, "#drawer", "open", "toggle");
        assert!(class_script.contains("request_id: 113"));
        assert!(class_script.contains("action: 'toggle_element_class'"));
        assert!(class_script.contains("#drawer"));
        assert!(class_script.contains("classList.toggle"));

        let text_script = AgentScripts::get_set_element_text_script(114, "#title", "New Value");
        assert!(text_script.contains("request_id: 114"));
        assert!(text_script.contains("action: 'set_element_text'"));
        assert!(text_script.contains("#title"));
        assert!(text_script.contains("New Value"));

        let evt_script = AgentScripts::get_dispatch_custom_event_script(115, "#widget", "change", true, true, None);
        assert!(evt_script.contains("request_id: 115"));
        assert!(evt_script.contains("action: 'dispatch_custom_event'"));
        assert!(evt_script.contains("new CustomEvent"));

        let active_script = AgentScripts::get_active_element_script(116);
        assert!(active_script.contains("request_id: 116"));
        assert!(active_script.contains("action: 'get_active_element'"));
        assert!(active_script.contains("document.activeElement"));

        let click_test_script = AgentScripts::get_check_element_clickable_script(117, "#submit");
        assert!(click_test_script.contains("request_id: 117"));
        assert!(click_test_script.contains("action: 'check_element_clickable'"));
        assert!(click_test_script.contains("elementFromPoint"));

        let wheel_script = AgentScripts::get_dispatch_wheel_event_script(118, "#map", 0.0, -100.0, true);
        assert!(wheel_script.contains("request_id: 118"));
        assert!(wheel_script.contains("action: 'dispatch_wheel_event'"));
        assert!(wheel_script.contains("new WheelEvent"));

        let css_vars_script = AgentScripts::get_css_variables_script(119, None, &["--main-bg".into()]);
        assert!(css_vars_script.contains("request_id: 119"));
        assert!(css_vars_script.contains("action: 'get_css_variables'"));
        assert!(css_vars_script.contains("--main-bg"));

        let clear_hl_script = AgentScripts::get_clear_highlights_script(120);
        assert!(clear_hl_script.contains("request_id: 120"));
        assert!(clear_hl_script.contains("action: 'clear_highlights'"));
        assert!(clear_hl_script.contains("__aerolith_highlight_ring"));

        let ptr_script = AgentScripts::get_simulate_pointer_click_script(121, "#canvas-btn", 2, 0);
        assert!(ptr_script.contains("request_id: 121"));
        assert!(ptr_script.contains("action: 'simulate_pointer_click'"));
        assert!(ptr_script.contains("new PointerEvent('pointerdown'"));
        assert!(ptr_script.contains("#canvas-btn"));

        let meta_script = AgentScripts::get_extract_meta_tags_script(122, true);
        assert!(meta_script.contains("request_id: 122"));
        assert!(meta_script.contains("action: 'extract_meta_tags'"));
        assert!(meta_script.contains("querySelectorAll('meta')"));
        assert!(meta_script.contains("application/ld+json"));

        let geom_script = AgentScripts::get_document_geometry_script(123);
        assert!(geom_script.contains("request_id: 123"));
        assert!(geom_script.contains("action: 'get_document_geometry'"));
        assert!(geom_script.contains("visual_viewport"));
        assert!(geom_script.contains("scrollable_containers_count"));

        let orient_script = AgentScripts::get_set_device_orientation_script(124, 90.0, 45.0, -15.0, true);
        assert!(orient_script.contains("request_id: 124"));
        assert!(orient_script.contains("action: 'set_device_orientation'"));
        assert!(orient_script.contains("new DeviceOrientationEvent"));
        assert!(orient_script.contains("const alpha = 90"));

        let cache_script = AgentScripts::get_clear_browser_cache_script(125, true, true);
        assert!(cache_script.contains("request_id: 125"));
        assert!(cache_script.contains("action: 'clear_browser_cache'"));
        assert!(cache_script.contains("caches.keys"));
        assert!(cache_script.contains("localStorage.clear"));

        let spacing_script = AgentScripts::get_element_spacing_script(126, "#card-body");
        assert!(spacing_script.contains("request_id: 126"));
        assert!(spacing_script.contains("action: 'get_element_spacing'"));
        assert!(spacing_script.contains("marginTop"));
        assert!(spacing_script.contains("paddingTop"));

        let wait_url_script = AgentScripts::get_wait_for_url_script(127, "/dashboard", 5000);
        assert!(wait_url_script.contains("request_id: 127"));
        assert!(wait_url_script.contains("action: 'wait_for_url'"));
        assert!(wait_url_script.contains("/dashboard"));

        let touches = vec![crate::agent::protocol::TouchPoint {
            identifier: 1,
            target: "#swipe-area".into(),
            x: 50.0,
            y: 75.0,
            radius_x: 1.0,
            radius_y: 1.0,
            rotation_angle: 0.0,
            force: 1.0,
        }];
        let touch_script = AgentScripts::get_dispatch_touch_event_script(128, "touchstart", &touches);
        assert!(touch_script.contains("request_id: 128"));
        assert!(touch_script.contains("action: 'dispatch_touch_event'"));
        assert!(touch_script.contains("touchstart"));
        assert!(touch_script.contains("#swipe-area"));

        let a11y_script = AgentScripts::get_a11y_tree_script(129, Some("#content"), 4);
        assert!(a11y_script.contains("request_id: 129"));
        assert!(a11y_script.contains("action: 'get_a11y_tree'"));
        assert!(a11y_script.contains("#content"));
        assert!(a11y_script.contains("maxDepth = 4"));

        let chord_script = AgentScripts::get_simulate_keyboard_chord_script(130, "Control+Shift+P", None);
        assert!(chord_script.contains("request_id: 130"));
        assert!(chord_script.contains("action: 'simulate_keyboard_chord'"));
        assert!(chord_script.contains("Control+Shift+P"));
        assert!(chord_script.contains("ctrlKey: ctrl"));

        let boxes = vec![crate::agent::protocol::BoundingBoxMarker {
            x: 100.0,
            y: 200.0,
            width: 300.0,
            height: 150.0,
            label: Some("Hero Card".into()),
            color: Some("#00ff00".into()),
        }];
        let boxes_script = AgentScripts::get_highlight_bounding_boxes_script(131, &boxes);
        assert!(boxes_script.contains("request_id: 131"));
        assert!(boxes_script.contains("action: 'highlight_bounding_boxes'"));
        assert!(boxes_script.contains("__aerolith_boxes_overlay"));
        assert!(boxes_script.contains("Hero Card"));

        let attrs_script = AgentScripts::get_element_attributes_script(132, "input#email");
        assert!(attrs_script.contains("request_id: 132"));
        assert!(attrs_script.contains("action: 'get_element_attributes'"));
        assert!(attrs_script.contains("input#email"));
        assert!(attrs_script.contains("el.attributes"));

        let filter_script = AgentScripts::get_element_filter_script(133, "#overlay-modal");
        assert!(filter_script.contains("request_id: 133"));
        assert!(filter_script.contains("action: 'get_element_filter'"));
        assert!(filter_script.contains("#overlay-modal"));
        assert!(filter_script.contains("hasOccludingAncestor"));

        let modals_script = AgentScripts::get_dismiss_all_modals_script(134);
        assert!(modals_script.contains("request_id: 134"));
        assert!(modals_script.contains("action: 'dismiss_all_modals'"));
        assert!(modals_script.contains("overlays_removed"));

        let palette_script = AgentScripts::get_color_palette_script(135, 12);
        assert!(palette_script.contains("request_id: 135"));
        assert!(palette_script.contains("action: 'get_color_palette'"));
        assert!(palette_script.contains("Math.max(1, 12)"));

        let dnd_script = AgentScripts::get_drag_and_drop_element_script(136, "#drag-item", "#drop-zone", 6, 50);
        assert!(dnd_script.contains("request_id: 136"));
        assert!(dnd_script.contains("action: 'drag_and_drop_element'"));
        assert!(dnd_script.contains("#drag-item"));
        assert!(dnd_script.contains("#drop-zone"));
        assert!(dnd_script.contains("dragstart"));
        assert!(dnd_script.contains("drop"));

        let forms_script = AgentScripts::get_audit_forms_script(137, Some("#signup"));
        assert!(forms_script.contains("request_id: 137"));
        assert!(forms_script.contains("action: 'audit_forms'"));
        assert!(forms_script.contains("#signup"));
        assert!(forms_script.contains("formElements"));

        let paging_script = AgentScripts::get_viewport_paging_script(138);
        assert!(paging_script.contains("request_id: 138"));
        assert!(paging_script.contains("action: 'get_viewport_paging'"));
        assert!(paging_script.contains("totalPages"));
        assert!(paging_script.contains("scrollProgress"));

        let pseudo_script = AgentScripts::get_computed_pseudo_element_script(139, "#btn", "::before", &["content".into()]);
        assert!(pseudo_script.contains("request_id: 139"));
        assert!(pseudo_script.contains("action: 'get_computed_pseudo_element'"));
        assert!(pseudo_script.contains("#btn"));
        assert!(pseudo_script.contains("::before"));
        assert!(pseudo_script.contains("content"));

        let paste_script = AgentScripts::get_simulate_clipboard_paste_script(140, Some("#code"), "hello world");
        assert!(paste_script.contains("request_id: 140"));
        assert!(paste_script.contains("action: 'simulate_clipboard_paste'"));
        assert!(paste_script.contains("#code"));
        assert!(paste_script.contains("hello world"));
        assert!(paste_script.contains("ClipboardEvent"));

        let mut fill_map = std::collections::HashMap::new();
        fill_map.insert("Full Name".into(), serde_json::json!("Alice Doe"));
        let smart_script = AgentScripts::get_smart_fill_script(141, &fill_map, true);
        assert!(smart_script.contains("request_id: 141"));
        assert!(smart_script.contains("action: 'smart_fill'"));
        assert!(smart_script.contains("Full Name"));
        assert!(smart_script.contains("Alice Doe"));

        let obs_script = AgentScripts::get_observe_text_script(142, "#stream", 5000, 500);
        assert!(obs_script.contains("request_id: 142"));
        assert!(obs_script.contains("action: 'observe_text'"));
        assert!(obs_script.contains("#stream"));
        assert!(obs_script.contains("MutationObserver"));

        let vis_script = AgentScripts::get_set_visibility_state_script(143, "visible");
        assert!(vis_script.contains("request_id: 143"));
        assert!(vis_script.contains("action: 'set_visibility_state'"));
        assert!(vis_script.contains("visibilitychange"));
        assert!(vis_script.contains("visible"));

        let font_script = AgentScripts::get_computed_font_script(144, "h1.title");
        assert!(font_script.contains("request_id: 144"));
        assert!(font_script.contains("action: 'get_computed_font'"));
        assert!(font_script.contains("h1.title"));
        assert!(font_script.contains("measureText"));

        let action_script = AgentScripts::get_action_map_script(145, true, 20);
        assert!(action_script.contains("request_id: 145"));
        assert!(action_script.contains("action: 'get_action_map'"));
        assert!(action_script.contains("center_x"));
        assert!(action_script.contains("data-aerolith-ref"));

        let motion_script = AgentScripts::get_motion_state_script(146, Some("#spinner"));
        assert!(motion_script.contains("request_id: 146"));
        assert!(motion_script.contains("action: 'get_motion_state'"));
        assert!(motion_script.contains("#spinner"));
        assert!(motion_script.contains("getAnimations"));

        let listeners_script = AgentScripts::get_element_listeners_script(147, "#btn-click");
        assert!(listeners_script.contains("request_id: 147"));
        assert!(listeners_script.contains("action: 'get_element_listeners'"));
        assert!(listeners_script.contains("#btn-click"));
        assert!(listeners_script.contains("onclick"));

        let vitals_script = AgentScripts::get_web_vitals_script(148);
        assert!(vitals_script.contains("request_id: 148"));
        assert!(vitals_script.contains("action: 'get_web_vitals'"));
        assert!(vitals_script.contains("ttfb_ms"));
        assert!(vitals_script.contains("dom_content_loaded_ms"));
    }
}
