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
    }
}
