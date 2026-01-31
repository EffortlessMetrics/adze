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
        
        // Initial analysis
        this.analyze();
    }

    async parse(visualize = false) {
        const input = document.getElementById('input-code').value;
        this.setStatus('Parsing...');
        
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

    // Helper for creating elements with text content (XSS protection)
    createElement(tag, className, text) {
        const el = document.createElement(tag);
        if (className) el.className = className;
        if (text !== undefined && text !== null) el.textContent = text;
        return el;
    }

    displayTiming(timing) {
        const container = document.getElementById('timing-content');
        container.textContent = '';

        const createBar = (label, ms) => {
            const bar = this.createElement('div', 'timing-bar');

            const labelSpan = this.createElement('span', 'timing-label', label);
            bar.appendChild(labelSpan);

            const progress = this.createElement('div', 'timing-progress');
            const fill = this.createElement('div', 'timing-fill');
            fill.style.width = `${(ms / timing.total_ms) * 100}%`;
            progress.appendChild(fill);
            bar.appendChild(progress);

            const valueSpan = this.createElement('span', 'timing-value', `${ms.toFixed(2)}ms`);
            bar.appendChild(valueSpan);

            return bar;
        };
        
        const chart = this.createElement('div', 'timing-chart');
        chart.appendChild(createBar('Lexing', timing.lexing_ms));
        chart.appendChild(createBar('Parsing', timing.parsing_ms));

        // Total bar needs manual width 100%
        const totalBar = this.createElement('div', 'timing-bar');
        totalBar.appendChild(this.createElement('span', 'timing-label', 'Total'));
        const totalProgress = this.createElement('div', 'timing-progress');
        const totalFill = this.createElement('div', 'timing-fill');
        totalFill.style.width = '100%';
        totalProgress.appendChild(totalFill);
        totalBar.appendChild(totalProgress);
        totalBar.appendChild(this.createElement('span', 'timing-value', `${timing.total_ms.toFixed(2)}ms`));
        chart.appendChild(totalBar);

        container.appendChild(chart);
    }

    displayErrors(errors) {
        const container = document.getElementById('error-list');
        container.textContent = '';

        errors.forEach(err => {
            const div = this.createElement('div', 'error-item', `Line ${err.line}, Column ${err.column}: ${err.message}`);
            container.appendChild(div);
        });
        
        document.getElementById('errors').style.display = 'block';
    }

    hideErrors() {
        document.getElementById('errors').style.display = 'none';
    }

    displayAnalysis(analysis) {
        const container = document.getElementById('analysis-content');
        container.textContent = '';

        const stats = analysis.grammar_stats;
        
        const statGrid = this.createElement('div', 'stat-grid');
        const addStat = (title, value) => {
            const card = this.createElement('div', 'stat-card');
            card.appendChild(this.createElement('h4', '', title));
            card.appendChild(this.createElement('div', 'value', String(value)));
            statGrid.appendChild(card);
        };

        addStat('Rules', stats.rule_count);
        addStat('Terminals', stats.terminal_count);
        addStat('Non-terminals', stats.nonterminal_count);
        addStat('Avg Rule Length', stats.avg_rule_length.toFixed(1));

        container.appendChild(statGrid);

        if (analysis.conflicts.length > 0) {
            container.appendChild(this.createElement('h3', '', 'Conflicts'));
            const list = this.createElement('div', 'conflict-list');
            analysis.conflicts.forEach(c => {
                const item = this.createElement('div', 'conflict-item');
                const strong = this.createElement('strong', '', c.kind);
                item.appendChild(strong);
                item.appendChild(document.createTextNode(` in state ${c.state}: ${c.description}`));
                list.appendChild(item);
            });
            container.appendChild(list);
        }

        if (analysis.suggestions.length > 0) {
            container.appendChild(this.createElement('h3', '', 'Suggestions'));
            const list = this.createElement('div', 'suggestion-list');
            analysis.suggestions.forEach(s => {
                const item = this.createElement('div', `suggestion-item ${s.level.toLowerCase()}`, s.message);
                list.appendChild(item);
            });
            container.appendChild(list);
        }
    }

    displayVisualization(svg) {
        // SVG injection is intentionally left as innerHTML for visualization support.
        // This assumes the backend returns safe SVG.
        document.getElementById('tree-visualization').innerHTML = svg;
    }

    updateTestList() {
        const container = document.getElementById('test-list');
        container.textContent = '';
        
        this.tests.forEach(test => {
            const div = this.createElement('div', 'test-item');
            div.appendChild(this.createElement('span', '', test.name));

            const btn = this.createElement('button', '', 'Load');
            // Safe event listener attachment
            btn.addEventListener('click', () => this.loadTest(test.name));
            div.appendChild(btn);

            container.appendChild(div);
        });
    }

    displayTestResults(results) {
        const container = document.getElementById('test-list');
        container.textContent = '';
        
        results.forEach(([test, result]) => {
            const div = this.createElement('div', `test-item ${result.success ? 'pass' : 'fail'}`);
            div.appendChild(this.createElement('span', '', test.name));
            div.appendChild(this.createElement('span', '', result.success ? '✓ PASS' : '✗ FAIL'));
            container.appendChild(div);
        });
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
