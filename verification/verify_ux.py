from playwright.sync_api import sync_playwright, expect

def run(playwright):
    browser = playwright.chromium.launch(headless=True)
    page = browser.new_page()

    # Navigate to the static page served by python http.server
    page.goto("http://localhost:8081/static/index.html")

    # 1. Verify Accessibility Attributes
    print("Verifying attributes...")
    input_area = page.locator("#input-code")
    parse_btn = page.locator("#parse-btn")

    expect(input_area).to_have_attribute("aria-label", "Code input")
    expect(parse_btn).to_have_attribute("title", "Run parser (Ctrl+Enter)")
    print("Attributes verified.")

    # 2. Verify Interaction (Shortcut)
    print("Testing shortcut...")
    input_area.fill("test code")
    input_area.focus()

    # Trigger shortcut
    page.keyboard.press("Control+Enter")

    # Since backend is down, we expect an error in the status bar
    # The app catches fetch errors and sets status
    status = page.locator("#status")
    expect(status).to_be_visible()
    # It might say "Error: Failed to fetch" or similar depending on browser/network
    # We just wait for it to have class 'error'
    expect(status).to_have_class("status error", timeout=5000)

    print("Shortcut triggered fetch (confirmed by error message).")

    # 3. Screenshot
    page.screenshot(path="verification/verification.png")
    print("Screenshot saved.")

    browser.close()

with sync_playwright() as playwright:
    run(playwright)
