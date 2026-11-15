"""
Aerolith Agent + Playwright Python Example.

Demonstrates:
  1. Connecting to Aerolith Agent over CDP
  2. Extracting token-efficient Semantic DOM snapshot
  3. Visual element highlighting before interaction
  4. Atomic form filling
  5. Capturing cropped element screenshot and SOM overlay
"""

from playwright.sync_api import sync_playwright
from aerolith_playwright import connect_aerolith


def main():
    print("Connecting Playwright to Aerolith Agent...")
    with sync_playwright() as p:
        browser, page = connect_aerolith(p, cdp_url="http://127.0.0.1:9222", agent_port=8765)

        print("Navigating to example page...")
        page.goto("https://duckduckgo.com")
        page.wait_for_load_state("networkidle")

        print("\n--- Semantic Accessibility Snapshot ---")
        snapshot = page.semantic_snapshot(viewport_only=True)
        print(snapshot[:500] + ("..." if len(snapshot) > 500 else ""))

        print("\nHighlighting search input...")
        page.highlight("input[name=q]")

        print("Filling search query atomically...")
        page.fill_form([{"target": "input[name=q]", "value": "Aerolith Agent browser"}], submit=True)

        page.wait_for_timeout(2000)

        print("Capturing Set-Of-Marks visual screenshot...")
        page.screenshot_som("duckduckgo_som.png")

        print("Done! Aerolith Playwright session completed successfully.")


if __name__ == "__main__":
    main()
