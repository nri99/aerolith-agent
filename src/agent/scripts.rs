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
        })();"#
    }

    /// JavaScript to traverse visible DOM and build a compact, token-efficient Semantic DOM snapshot.
    pub fn get_semantic_dom_script(request_id: u64) -> String {
        format!(
            r#"(function() {{
                try {{
                    window.__aerolith_elements = {{}};
                    let refId = 1;
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
            request_id, request_id
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
                        const opts = {{ bubbles: true, cancelable: true, view: window, clientX: rect.x + rect.width / 2, clientY: rect.y + rect.height / 2 }};

                        el.dispatchEvent(new MouseEvent('mouseover', opts));
                        el.dispatchEvent(new MouseEvent('mouseenter', opts));
                        el.dispatchEvent(new MouseEvent('mousedown', opts));
                        if (typeof el.focus === 'function') el.focus();
                        el.dispatchEvent(new MouseEvent('mouseup', opts));
                        el.dispatchEvent(new MouseEvent('click', opts));

                        if (window.ipc) {{
                            window.ipc.postMessage(JSON.stringify({{
                                type: 'agent_response',
                                request_id: {},
                                action: 'click',
                                payload: {{
                                    success: true,
                                    tag: el.tagName.toLowerCase(),
                                    text: (el.innerText || el.value || '').substring(0, 80)
                                }}
                            }}));
                        }}
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

                        if ({}) {{
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
                                action: 'type',
                                payload: {{
                                    success: true,
                                    value: el.value !== undefined ? el.value : el.textContent
                                }}
                            }}));
                        }}
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_script_generation() {
        let dom_script = AgentScripts::get_semantic_dom_script(42);
        assert!(dom_script.contains("request_id: 42"));
        assert!(dom_script.contains("data-aerolith-ref"));
        assert!(dom_script.contains("tree_text"));

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
    }
}
