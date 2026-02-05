from playwright.sync_api import Page, expect, sync_playwright
import time

def verify_xss(page: Page):
    # Navigate to the repro page
    page.goto("http://localhost:8080/repro_xss.html")

    # Wait for the JS to run
    time.sleep(1)

    # Check for XSS in test-list
    # If vulnerable, there will be an img tag inside .test-item span or directly in test-list depending on where it was injected
    # In my repro: { name: '<img src=x onerror=console.log("XSS_TRIGGERED_TEST_LIST")>', input: '1+1' }
    # Rendered: <span><img src=x ...></span>

    # We look for an img tag inside #test-list
    img_element = page.locator("#test-list img")

    if img_element.count() > 0:
        print("VULNERABILITY DETECTED: Found img tag in test-list")
    else:
        print("SECURE: No img tag found in test-list")

    # Check for XSS in error-list
    img_element_err = page.locator("#error-list img")
    if img_element_err.count() > 0:
        print("VULNERABILITY DETECTED: Found img tag in error-list")
    else:
        print("SECURE: No img tag found in error-list")

    # Check for XSS in analysis-content
    img_element_analysis = page.locator("#analysis-content img")
    if img_element_analysis.count() > 0:
        print("VULNERABILITY DETECTED: Found img tag in analysis-content")
    else:
        print("SECURE: No img tag found in analysis-content")

    # Take a screenshot
    page.screenshot(path="verification/xss_status.png")

if __name__ == "__main__":
    with sync_playwright() as p:
        browser = p.chromium.launch(headless=True)
        page = browser.new_page()
        try:
            verify_xss(page)
        finally:
            browser.close()
