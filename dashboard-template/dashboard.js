// Dashboard JavaScript

const SUPPORT_ICONS = {
    full: '✅',
    partial: '⚠️',
    none: '❌',
    in_progress: '⏳'
};

async function loadDashboardData() {
    try {
        const response = await fetch('data.json');
        const data = await response.json();
        updateDashboard(data);
    } catch (error) {
        console.error('Failed to load dashboard data:', error);
        showError('Failed to load dashboard data');
    }
}

function updateDashboard(data) {
    // Update last updated time
    document.getElementById('last-updated').textContent = new Date(data.last_updated).toLocaleString();
    
    // Update overview metrics
    updateOverviewMetrics(data);
    
    // Update grammar table
    updateGrammarTable(data.grammar_status);
    
    // Update performance metrics
    updatePerformanceMetrics(data.performance);
    
    // Update corpus results
    updateCorpusResults(data.corpus_results);
    
    // Update adoption metrics
    updateAdoptionMetrics(data.adoption);
}

function updateOverviewMetrics(data) {
    // Calculate overall grammar support percentage
    const totalGrammars = data.grammar_status.length;
    const supportedGrammars = data.grammar_status.filter(g => g.parse_support === 'full').length;
    const supportPercentage = totalGrammars > 0 ? Math.round((supportedGrammars / totalGrammars) * 100) : 0;
    
    document.getElementById('grammar-support').textContent = `${supportPercentage}%`;
    document.getElementById('parse-speed').textContent = `${data.performance.parse_speed_mb_per_sec} MB/s`;
    document.getElementById('memory-usage').textContent = `${data.performance.memory_bytes_per_node} B/node`;
    document.getElementById('wasm-size').textContent = `${data.performance.wasm_size_kb} KB`;
}

function updateGrammarTable(grammarStatus) {
    const tbody = document.getElementById('grammar-tbody');
    tbody.innerHTML = '';
    
    grammarStatus.forEach(grammar => {
        const row = document.createElement('tr');
        
        // Grammar name
        const nameCell = document.createElement('td');
        nameCell.textContent = grammar.name;
        row.appendChild(nameCell);
        
        // Parse support
        const parseCell = document.createElement('td');
        parseCell.className = 'status-icon';
        parseCell.textContent = SUPPORT_ICONS[grammar.parse_support];
        row.appendChild(parseCell);
        
        // Query support
        const queryCell = document.createElement('td');
        queryCell.className = 'status-icon';
        queryCell.textContent = SUPPORT_ICONS[grammar.query_support];
        row.appendChild(queryCell);
        
        // Incremental support
        const incrementalCell = document.createElement('td');
        incrementalCell.className = 'status-icon';
        incrementalCell.textContent = SUPPORT_ICONS[grammar.incremental_support];
        row.appendChild(incrementalCell);
        
        // Status
        const statusCell = document.createElement('td');
        statusCell.textContent = `${grammar.completion_percentage}% Complete`;
        statusCell.className = grammar.completion_percentage >= 80 ? 'status-full' : 
                                grammar.completion_percentage >= 50 ? 'status-partial' : 
                                'status-none';
        row.appendChild(statusCell);
        
        tbody.appendChild(row);
    });
}

function updatePerformanceMetrics(performance) {
    // Parse speed comparison
    const speedPercentage = Math.min(150, performance.comparison_to_c);
    document.getElementById('parse-speed-bar').style.width = `${speedPercentage / 1.5}%`;
    document.getElementById('parse-speed-label').textContent = 
        `${performance.parse_speed_mb_per_sec} MB/s (${speedPercentage >= 100 ? '+' : ''}${speedPercentage - 100}% vs C)`;
    
    // Memory efficiency
    const memoryEfficiency = Math.max(0, 200 - performance.memory_bytes_per_node) / 2;
    document.getElementById('memory-bar').style.width = `${memoryEfficiency}%`;
    document.getElementById('memory-label').textContent = 
        `${performance.memory_bytes_per_node} bytes/node`;
}

function updateCorpusResults(corpus) {
    document.getElementById('total-grammars').textContent = corpus.total_grammars;
    document.getElementById('passing-grammars').textContent = corpus.passing;
    document.getElementById('failing-grammars').textContent = corpus.failing;
    document.getElementById('pass-rate').textContent = `${corpus.pass_rate.toFixed(1)}%`;
    
    // Update recent changes
    const changesList = document.getElementById('changes-list');
    changesList.innerHTML = '';
    
    corpus.recent_changes.forEach(change => {
        const li = document.createElement('li');
        li.textContent = change;
        changesList.appendChild(li);
    });
}

function updateAdoptionMetrics(adoption) {
    document.getElementById('github-stars').textContent = adoption.github_stars.toLocaleString();
    document.getElementById('downloads').textContent = adoption.crates_io_downloads.toLocaleString();
    document.getElementById('grammar-prs').textContent = adoption.grammar_prs;
    document.getElementById('contributors').textContent = adoption.active_contributors;
}

function showError(message) {
    const main = document.querySelector('main');
    main.innerHTML = `
        <div class="card">
            <h2>Error</h2>
            <p>${message}</p>
            <p>Please ensure the dashboard data file exists and is accessible.</p>
        </div>
    `;
}

// Load data when page loads
document.addEventListener('DOMContentLoaded', loadDashboardData);

// Refresh data every 5 minutes
setInterval(loadDashboardData, 5 * 60 * 1000);