// Populate the sidebar
//
// This is a script, and not included directly in the page, to control the total size of the book.
// The TOC contains an entry for each page, so if each page includes a copy of the TOC,
// the total size of the page becomes O(n**2).
class MDBookSidebarScrollbox extends HTMLElement {
    constructor() {
        super();
    }
    connectedCallback() {
        this.innerHTML = '<ol class="chapter"><li class="chapter-item affix "><a href="index.html">Introduction</a></li><li class="chapter-item affix "><li class="part-title">Getting Started</li><li class="chapter-item "><a href="getting-started.html"><strong aria-hidden="true">1.</strong> Overview</a></li><li class="chapter-item "><a href="getting-started/installation.html"><strong aria-hidden="true">2.</strong> Installation</a></li><li class="chapter-item "><a href="getting-started/quickstart.html"><strong aria-hidden="true">3.</strong> Quick Start</a></li><li class="chapter-item "><a href="getting-started/migration.html"><strong aria-hidden="true">4.</strong> Migration Guide</a></li><li class="chapter-item affix "><li class="part-title">User Guide</li><li class="chapter-item "><a href="guide/architecture.html"><strong aria-hidden="true">5.</strong> Architecture</a></li><li class="chapter-item "><a href="guide/grammar-definition.html"><strong aria-hidden="true">6.</strong> Grammar Definition</a></li><li class="chapter-item "><a href="guide/grammar_patterns.html"><strong aria-hidden="true">7.</strong> Grammar Design Patterns</a></li><li class="chapter-item "><a href="guide/parser-generation.html"><strong aria-hidden="true">8.</strong> Parser Generation</a></li><li class="chapter-item "><a href="guide/glr-precedence-resolution.html"><strong aria-hidden="true">9.</strong> GLR Precedence Resolution</a></li><li class="chapter-item "><a href="guide/incremental-parsing.html"><strong aria-hidden="true">10.</strong> Incremental Parsing</a></li><li class="chapter-item "><a href="guide/query-patterns.html"><strong aria-hidden="true">11.</strong> Query and Pattern Matching</a></li><li class="chapter-item "><a href="guide/error-recovery.html"><strong aria-hidden="true">12.</strong> Error Recovery</a></li><li class="chapter-item "><a href="guide/golden-tests-maintenance.html"><strong aria-hidden="true">13.</strong> Golden Tests Maintenance</a></li><li class="chapter-item "><a href="guide/lsp-generation.html"><strong aria-hidden="true">14.</strong> LSP Server Generation</a></li><li class="chapter-item "><a href="guide/performance.html"><strong aria-hidden="true">15.</strong> Performance Optimization</a></li><li class="chapter-item "><a href="guide/migration.html"><strong aria-hidden="true">16.</strong> Migration: rust-sitter to Adze</a></li><li class="chapter-item affix "><li class="part-title">Advanced Topics</li><li class="chapter-item "><a href="advanced/glr-parsing.html"><strong aria-hidden="true">17.</strong> GLR Parsing</a></li><li class="chapter-item "><a href="advanced/optimizer-usage.html"><strong aria-hidden="true">18.</strong> Grammar Optimization</a></li><li class="chapter-item "><a href="advanced/external-scanners.html"><strong aria-hidden="true">19.</strong> External Scanners</a></li><li class="chapter-item "><a href="advanced/predicate-evaluation.html"><strong aria-hidden="true">20.</strong> Predicate Evaluation</a></li><li class="chapter-item "><a href="advanced/visualization.html"><strong aria-hidden="true">21.</strong> Visualization Tools</a></li><li class="chapter-item affix "><li class="part-title">Reference</li><li class="chapter-item "><a href="reference/api.html"><strong aria-hidden="true">22.</strong> API Documentation</a></li><li class="chapter-item "><a href="reference/s-expression-format.html"><strong aria-hidden="true">23.</strong> S-Expression Format</a></li><li class="chapter-item "><a href="reference/grammar-examples.html"><strong aria-hidden="true">24.</strong> Grammar Examples</a></li><li class="chapter-item "><a href="reference/language-support.html"><strong aria-hidden="true">25.</strong> Language Support</a></li><li class="chapter-item "><a href="reference/known-limitations.html"><strong aria-hidden="true">26.</strong> Known Limitations</a></li><li class="chapter-item affix "><li class="part-title">Development</li><li class="chapter-item "><a href="development/contributing.html"><strong aria-hidden="true">27.</strong> Contributing</a></li><li class="chapter-item "><a href="development/architecture.html"><strong aria-hidden="true">28.</strong> Architecture</a></li><li class="chapter-item "><a href="development/testing.html"><strong aria-hidden="true">29.</strong> Testing</a></li><li class="chapter-item "><a href="development/golden-tests.html"><strong aria-hidden="true">30.</strong> Golden Tests</a></li><li class="chapter-item "><a href="development/release.html"><strong aria-hidden="true">31.</strong> Release Process</a></li><li class="chapter-item affix "><li class="part-title">Concepts</li><li class="chapter-item "><a href="architecture.html"><strong aria-hidden="true">32.</strong> Architecture Overview</a></li><li class="chapter-item "><a href="microcrates.html"><strong aria-hidden="true">33.</strong> Microcrate Guide</a></li><li class="chapter-item "><a href="testing.html"><strong aria-hidden="true">34.</strong> Testing Overview</a></li><li class="chapter-item affix "><li class="part-title">Appendix</li><li class="chapter-item "><a href="appendix/changelog.html"><strong aria-hidden="true">35.</strong> Changelog</a></li><li class="chapter-item "><a href="appendix/faq.html"><strong aria-hidden="true">36.</strong> FAQ</a></li></ol>';
        // Set the current, active page, and reveal it if it's hidden
        let current_page = document.location.href.toString().split("#")[0].split("?")[0];
        if (current_page.endsWith("/")) {
            current_page += "index.html";
        }
        var links = Array.prototype.slice.call(this.querySelectorAll("a"));
        var l = links.length;
        for (var i = 0; i < l; ++i) {
            var link = links[i];
            var href = link.getAttribute("href");
            if (href && !href.startsWith("#") && !/^(?:[a-z+]+:)?\/\//.test(href)) {
                link.href = path_to_root + href;
            }
            // The "index" page is supposed to alias the first chapter in the book.
            if (link.href === current_page || (i === 0 && path_to_root === "" && current_page.endsWith("/index.html"))) {
                link.classList.add("active");
                var parent = link.parentElement;
                if (parent && parent.classList.contains("chapter-item")) {
                    parent.classList.add("expanded");
                }
                while (parent) {
                    if (parent.tagName === "LI" && parent.previousElementSibling) {
                        if (parent.previousElementSibling.classList.contains("chapter-item")) {
                            parent.previousElementSibling.classList.add("expanded");
                        }
                    }
                    parent = parent.parentElement;
                }
            }
        }
        // Track and set sidebar scroll position
        this.addEventListener('click', function(e) {
            if (e.target.tagName === 'A') {
                sessionStorage.setItem('sidebar-scroll', this.scrollTop);
            }
        }, { passive: true });
        var sidebarScrollTop = sessionStorage.getItem('sidebar-scroll');
        sessionStorage.removeItem('sidebar-scroll');
        if (sidebarScrollTop) {
            // preserve sidebar scroll position when navigating via links within sidebar
            this.scrollTop = sidebarScrollTop;
        } else {
            // scroll sidebar to current active section when navigating via "next/previous chapter" buttons
            var activeSection = document.querySelector('#sidebar .active');
            if (activeSection) {
                activeSection.scrollIntoView({ block: 'center' });
            }
        }
        // Toggle buttons
        var sidebarAnchorToggles = document.querySelectorAll('#sidebar a.toggle');
        function toggleSection(ev) {
            ev.currentTarget.parentElement.classList.toggle('expanded');
        }
        Array.from(sidebarAnchorToggles).forEach(function (el) {
            el.addEventListener('click', toggleSection);
        });
    }
}
window.customElements.define("mdbook-sidebar-scrollbox", MDBookSidebarScrollbox);
