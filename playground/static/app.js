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
        const errorList = document.getElementById('error-list');
        errorList.textContent = '';

        errors.forEach(err => {
            const div = document.createElement('div');
            div.className = 'error-item';
            div.textContent = `Line ${err.line}, Column ${err.column}: ${err.message}`;
            errorList.appendChild(div);
        });
        
        document.getElementById('errors').style.display = 'block';
    }

    hideErrors() {
        document.getElementById('errors').style.display = 'none';
    }

    displayAnalysis(analysis) {
        const stats = analysis.grammar_stats;
        const container = document.getElementById('analysis-content');
        container.textContent = '';

        // Stat Grid
        const grid = document.createElement('div');
        grid.className = 'stat-grid';

        const createStatCard = (title, value) => {
            const card = document.createElement('div');
            card.className = 'stat-card';
            const h4 = document.createElement('h4');
            h4.textContent = title;
            const v = document.createElement('div');
            v.className = 'value';
            v.textContent = value;
            card.appendChild(h4);
            card.appendChild(v);
            return card;
        };

        grid.appendChild(createStatCard('Rules', stats.rule_count));
        grid.appendChild(createStatCard('Terminals', stats.terminal_count));
        grid.appendChild(createStatCard('Non-terminals', stats.nonterminal_count));
        grid.appendChild(createStatCard('Avg Rule Length', stats.avg_rule_length.toFixed(1)));

        container.appendChild(grid);

        // Conflicts
        if (analysis.conflicts.length > 0) {
            const h3 = document.createElement('h3');
            h3.textContent = 'Conflicts';
            container.appendChild(h3);
            
            const list = document.createElement('div');
            list.className = 'conflict-list';
            
            analysis.conflicts.forEach(c => {
                const item = document.createElement('div');
                item.className = 'conflict-item';

                const strong = document.createElement('strong');
                strong.textContent = c.kind;

                item.appendChild(strong);
                item.appendChild(document.createTextNode(` in state ${c.state}: ${c.description}`));
                list.appendChild(item);
            });
            container.appendChild(list);
        }
        
        // Suggestions
        if (analysis.suggestions.length > 0) {
            const h3 = document.createElement('h3');
            h3.textContent = 'Suggestions';
            container.appendChild(h3);

            const list = document.createElement('div');
            list.className = 'suggestion-list';

            analysis.suggestions.forEach(s => {
                const item = document.createElement('div');
                item.className = `suggestion-item ${s.level.toLowerCase()}`;
                item.textContent = s.message;
                list.appendChild(item);
            });
            container.appendChild(list);
        }
    }

    displayVisualization(svg) {
        document.getElementById('tree-visualization').innerHTML = svg;
    }

    updateTestList() {
        const list = document.getElementById('test-list');
        list.textContent = '';
        
        this.tests.forEach(test => {
            const item = document.createElement('div');
            item.className = 'test-item';

            const span = document.createElement('span');
            span.textContent = test.name;

            const button = document.createElement('button');
            button.textContent = 'Load';
            button.addEventListener('click', () => this.loadTest(test.name));

            item.appendChild(span);
            item.appendChild(button);
            list.appendChild(item);
        });
    }

    displayTestResults(results) {
        const list = document.getElementById('test-list');
        list.textContent = '';
        
        results.forEach(([test, result]) => {
            const item = document.createElement('div');
            item.className = `test-item ${result.success ? 'pass' : 'fail'}`;

            const nameSpan = document.createElement('span');
            nameSpan.textContent = test.name;

            const statusSpan = document.createElement('span');
            statusSpan.textContent = result.success ? '✓ PASS' : '✗ FAIL';

            item.appendChild(nameSpan);
            item.appendChild(statusSpan);
            list.appendChild(item);
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