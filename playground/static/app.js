// Rust Sitter Playground JavaScript

class Playground {
    constructor() {
        this.tests = [];
        this.currentResult = null;
        this.init();
    }

    init() {
        // Parse button
        document.getElementById('parse-btn').addEventListener('click', () => this.parse(false));
        document.getElementById('visualize-btn').addEventListener('click', () => this.parse(true));
        
        // Test management
        document.getElementById('add-test-btn').addEventListener('click', () => this.addTest());
        document.getElementById('run-tests-btn').addEventListener('click', () => this.runTests());
        
        // Import/Export
        document.getElementById('export-btn').addEventListener('click', () => this.export());
        document.getElementById('import-btn').addEventListener('click', () => {
            document.getElementById('import-file').click();
        });
        document.getElementById('import-file').addEventListener('change', (e) => this.import(e));
        
        // Tabs
        document.querySelectorAll('.tab').forEach(tab => {
            tab.addEventListener('click', () => this.switchTab(tab.dataset.tab));
        });
        
        // Keyboard shortcuts
        document.getElementById('input-code').addEventListener('keydown', (e) => {
            if ((e.ctrlKey || e.metaKey) && e.key === 'Enter') {
                e.preventDefault();
                this.parse(false);
            }
        });

        // Initial analysis
        this.analyze();
    }

    async parse(visualize = false) {
        const input = document.getElementById('input-code').value;
        const parseBtn = document.getElementById('parse-btn');
        const visualizeBtn = document.getElementById('visualize-btn');
        const originalText = parseBtn.textContent;

        this.setStatus('Parsing...');
        parseBtn.disabled = true;
        visualizeBtn.disabled = true;
        parseBtn.textContent = 'Parsing...';
        
        try {
            const response = await fetch('/api/parse', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ input, visualize })
            });
            
            const result = await response.json();
            this.currentResult = result;
            
            if (result.success) {
                this.displayTree(result.tree);
                this.displayTiming(result.timing);
                this.hideErrors();
                this.setStatus('Parse successful', 'success');
                
                if (visualize && result.visualization) {
                    this.displayVisualization(result.visualization);
                    this.switchTab('visualization');
                }
            } else {
                this.displayErrors(result.errors);
                this.setStatus('Parse failed', 'error');
            }
        } catch (error) {
            this.setStatus('Error: ' + error.message, 'error');
        } finally {
            parseBtn.disabled = false;
            visualizeBtn.disabled = false;
            parseBtn.textContent = originalText;
        }
    }

    async addTest() {
        const input = document.getElementById('input-code').value;
        const name = prompt('Test name:');
        
        if (!name) return;
        
        try {
            const response = await fetch('/api/test', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({
                    name,
                    input,
                    expected: this.currentResult?.tree,
                    tags: []
                })
            });
            
            if (response.ok) {
                this.tests.push({ name, input });
                this.updateTestList();
                this.setStatus('Test added', 'success');
            }
        } catch (error) {
            this.setStatus('Error: ' + error.message, 'error');
        }
    }

    async runTests() {
        this.setStatus('Running tests...');
        
        try {
            const response = await fetch('/api/tests');
            const results = await response.json();
            
            this.displayTestResults(results);
            
            const passed = results.filter(([test, result]) => result.success).length;
            const total = results.length;
            
            this.setStatus(`Tests: ${passed}/${total} passed`, passed === total ? 'success' : 'warning');
        } catch (error) {
            this.setStatus('Error: ' + error.message, 'error');
        }
    }

    async analyze() {
        try {
            const response = await fetch('/api/analyze');
            const analysis = await response.json();
            
            this.displayAnalysis(analysis);
        } catch (error) {
            console.error('Analysis error:', error);
        }
    }

    displayTree(tree) {
        const element = document.getElementById('parse-tree');
        element.textContent = tree || '(empty)';
        
        // Syntax highlighting if Prism is available
        if (window.Prism) {
            element.innerHTML = Prism.highlight(tree || '', Prism.languages.javascript, 'javascript');
        }
    }

    displayTiming(timing) {
        const html = `
            <div class="timing-chart">
                <div class="timing-bar">
                    <span class="timing-label">Lexing</span>
                    <div class="timing-progress">
                        <div class="timing-fill" style="width: ${(timing.lexing_ms / timing.total_ms) * 100}%"></div>
                    </div>
                    <span class="timing-value">${timing.lexing_ms.toFixed(2)}ms</span>
                </div>
                <div class="timing-bar">
                    <span class="timing-label">Parsing</span>
                    <div class="timing-progress">
                        <div class="timing-fill" style="width: ${(timing.parsing_ms / timing.total_ms) * 100}%"></div>
                    </div>
                    <span class="timing-value">${timing.parsing_ms.toFixed(2)}ms</span>
                </div>
                <div class="timing-bar">
                    <span class="timing-label">Total</span>
                    <div class="timing-progress">
                        <div class="timing-fill" style="width: 100%"></div>
                    </div>
                    <span class="timing-value">${timing.total_ms.toFixed(2)}ms</span>
                </div>
            </div>
        `;
        
        document.getElementById('timing-content').innerHTML = html;
    }

    displayErrors(errors) {
        const errorList = errors.map(err => 
            `<div class="error-item">Line ${err.line}, Column ${err.column}: ${err.message}</div>`
        ).join('');
        
        document.getElementById('error-list').innerHTML = errorList;
        document.getElementById('errors').style.display = 'block';
    }

    hideErrors() {
        document.getElementById('errors').style.display = 'none';
    }

    displayAnalysis(analysis) {
        const stats = analysis.grammar_stats;
        const html = `
            <div class="stat-grid">
                <div class="stat-card">
                    <h4>Rules</h4>
                    <div class="value">${stats.rule_count}</div>
                </div>
                <div class="stat-card">
                    <h4>Terminals</h4>
                    <div class="value">${stats.terminal_count}</div>
                </div>
                <div class="stat-card">
                    <h4>Non-terminals</h4>
                    <div class="value">${stats.nonterminal_count}</div>
                </div>
                <div class="stat-card">
                    <h4>Avg Rule Length</h4>
                    <div class="value">${stats.avg_rule_length.toFixed(1)}</div>
                </div>
            </div>
            
            ${analysis.conflicts.length > 0 ? `
                <h3>Conflicts</h3>
                <div class="conflict-list">
                    ${analysis.conflicts.map(c => `
                        <div class="conflict-item">
                            <strong>${c.kind}</strong> in state ${c.state}: ${c.description}
                        </div>
                    `).join('')}
                </div>
            ` : ''}
            
            ${analysis.suggestions.length > 0 ? `
                <h3>Suggestions</h3>
                <div class="suggestion-list">
                    ${analysis.suggestions.map(s => `
                        <div class="suggestion-item ${s.level.toLowerCase()}">
                            ${s.message}
                        </div>
                    `).join('')}
                </div>
            ` : ''}
        `;
        
        document.getElementById('analysis-content').innerHTML = html;
    }

    displayVisualization(svg) {
        document.getElementById('tree-visualization').innerHTML = svg;
    }

    updateTestList() {
        const html = this.tests.map(test => `
            <div class="test-item">
                <span>${test.name}</span>
                <button onclick="playground.loadTest('${test.name}')">Load</button>
            </div>
        `).join('');
        
        document.getElementById('test-list').innerHTML = html;
    }

    displayTestResults(results) {
        const html = results.map(([test, result]) => `
            <div class="test-item ${result.success ? 'pass' : 'fail'}">
                <span>${test.name}</span>
                <span>${result.success ? '✓ PASS' : '✗ FAIL'}</span>
            </div>
        `).join('');
        
        document.getElementById('test-list').innerHTML = html;
    }

    loadTest(name) {
        const test = this.tests.find(t => t.name === name);
        if (test) {
            document.getElementById('input-code').value = test.input;
            this.setStatus(`Loaded test: ${name}`);
        }
    }

    switchTab(tabName) {
        // Update tab buttons
        document.querySelectorAll('.tab').forEach(tab => {
            tab.classList.toggle('active', tab.dataset.tab === tabName);
        });
        
        // Update tab content
        document.querySelectorAll('.tab-pane').forEach(pane => {
            pane.classList.toggle('active', pane.id === `${tabName}-tab`);
        });
    }

    async export() {
        try {
            const response = await fetch('/api/export?format=download');
            const blob = await response.blob();
            
            const url = window.URL.createObjectURL(blob);
            const a = document.createElement('a');
            a.href = url;
            a.download = 'playground-session.json';
            a.click();
            
            this.setStatus('Session exported', 'success');
        } catch (error) {
            this.setStatus('Export failed: ' + error.message, 'error');
        }
    }

    async import(event) {
        const file = event.target.files[0];
        if (!file) return;
        
        try {
            const text = await file.text();
            const response = await fetch('/api/import', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: text
            });
            
            if (response.ok) {
                this.setStatus('Session imported', 'success');
                location.reload(); // Refresh to load imported data
            } else {
                throw new Error('Import failed');
            }
        } catch (error) {
            this.setStatus('Import failed: ' + error.message, 'error');
        }
    }

    setStatus(message, type = 'info') {
        const status = document.getElementById('status');
        status.textContent = message;
        status.className = 'status ' + type;
        
        if (type !== 'error') {
            setTimeout(() => {
                status.textContent = '';
                status.className = 'status';
            }, 3000);
        }
    }
}

// Initialize playground when page loads
const playground = new Playground();