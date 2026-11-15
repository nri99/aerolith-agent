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
                try {
                    const origFetch = window.fetch;
                    if (origFetch) {
                        window.fetch = function() {
                            window.__aerolith_inflight++;
                            return origFetch.apply(this, arguments).finally(() => {
                                window.__aerolith_inflight = Math.max(0, window.__aerolith_inflight - 1);
                            });
                        };
                    }
                    const origSend = XMLHttpRequest.prototype.send;
                    if (origSend) {
                        XMLHttpRequest.prototype.send = function() {
                            window.__aerolith_inflight++;
                            this.addEventListener('loadend', () => {
                                window.__aerolith_inflight = Math.max(0, window.__aerolith_inflight - 1);
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
    }
}
