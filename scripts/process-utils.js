#!/usr/bin/env node
/**
 * Process utilities for adze CI/orchestration
 * Provides robust process management without ps dependency and EAGAIN handling
 */

const { spawn } = require('child_process');
const fs = require('fs');
const path = require('path');

const LOCK_DIR = path.join(__dirname, '..', '.locks');

/**
 * Robust process runner that avoids ps dependency and handles EAGAIN
 * Uses process groups for reliable cleanup
 */
class ProcessRunner {
  constructor() {
    // Ensure lock directory exists
    if (!fs.existsSync(LOCK_DIR)) {
      fs.mkdirSync(LOCK_DIR, { recursive: true });
    }
  }

  /**
   * Run command with process group management and EAGAIN handling
   */
  async run(cmd, args = [], opts = {}) {
    const { timeoutMs = 30 * 60 * 1000, ...spawnOpts } = opts;
    
    const child = spawn(cmd, args, {
      detached: true,                 // put child in its own process group
      stdio: 'inherit',
      ...spawnOpts,
    });

    const pgid = -child.pid;         // negative => process group

    const done = new Promise((resolve, reject) => {
      child.on('exit', code => resolve(code ?? 0));
      child.on('error', err => {
        if (err?.code === 'EAGAIN') {
          // Backoff + single retry to handle transient fork failure
          console.warn(`EAGAIN error, retrying in 100ms: ${cmd} ${args.join(' ')}`);
          setTimeout(async () => {
            try { 
              resolve(await this.run(cmd, args, opts)); 
            } catch (e) { 
              reject(e); 
            }
          }, 100);
          return;
        }
        reject(err);
      });
    });

    const timer = setTimeout(() => {
      console.log(`Timeout reached, killing process group: ${pgid}`);
      try { 
        process.kill(pgid, 'SIGKILL'); 
      } catch (e) {
        console.warn(`Failed to kill process group ${pgid}:`, e.message);
      }
    }, timeoutMs);

    try { 
      return await done; 
    } finally { 
      clearTimeout(timer); 
    }
  }

  /**
   * Global lock-based debouncing to prevent duplicate agent runs
   */
  async withGlobalLock(lockName, fn, options = {}) {
    const { 
      retries = 8, 
      factor = 1.4, 
      minTimeout = 50, 
      maxTimeout = 500,
      staleLockMs = 5 * 60 * 1000 // 5 minutes
    } = options;

    const lockPath = path.join(LOCK_DIR, `${lockName}.lock`);
    let attempt = 0;

    while (attempt < retries) {
      try {
        // Check if lock exists and is stale
        if (fs.existsSync(lockPath)) {
          const stats = fs.statSync(lockPath);
          const age = Date.now() - stats.mtime.getTime();
          if (age > staleLockMs) {
            console.warn(`Removing stale lock: ${lockPath} (age: ${Math.round(age/1000)}s)`);
            fs.unlinkSync(lockPath);
          }
        }

        // Try to acquire lock
        fs.writeFileSync(lockPath, JSON.stringify({
          pid: process.pid,
          timestamp: Date.now(),
          lockName
        }), { flag: 'wx' }); // fail if exists

        console.log(`Acquired lock: ${lockName}`);
        
        try {
          return await fn();
        } finally {
          // Release lock
          try {
            fs.unlinkSync(lockPath);
            console.log(`Released lock: ${lockName}`);
          } catch (e) {
            console.warn(`Failed to release lock ${lockPath}:`, e.message);
          }
        }
      } catch (err) {
        if (err.code === 'EEXIST') {
          // Lock exists, wait and retry
          const delay = Math.min(
            minTimeout * Math.pow(factor, attempt), 
            maxTimeout
          );
          console.log(`Lock ${lockName} held by another process, retrying in ${delay}ms (attempt ${attempt + 1}/${retries})`);
          await new Promise(resolve => setTimeout(resolve, delay));
          attempt++;
        } else {
          throw err;
        }
      }
    }

    throw new Error(`Failed to acquire lock ${lockName} after ${retries} attempts`);
  }

  /**
   * Cleanup stale locks (call this periodically or at startup)
   */
  cleanupStaleLocks(staleLockMs = 5 * 60 * 1000) {
    if (!fs.existsSync(LOCK_DIR)) return;
    
    const files = fs.readdirSync(LOCK_DIR);
    let cleaned = 0;
    
    for (const file of files) {
      if (!file.endsWith('.lock')) continue;
      
      const lockPath = path.join(LOCK_DIR, file);
      try {
        const stats = fs.statSync(lockPath);
        const age = Date.now() - stats.mtime.getTime();
        if (age > staleLockMs) {
          fs.unlinkSync(lockPath);
          cleaned++;
          console.log(`Cleaned stale lock: ${file} (age: ${Math.round(age/1000)}s)`);
        }
      } catch (e) {
        // File might have been removed by another process
        console.warn(`Failed to check/clean lock ${file}:`, e.message);
      }
    }
    
    if (cleaned > 0) {
      console.log(`Cleaned ${cleaned} stale locks`);
    }
  }
}

/**
 * Agent orchestration utilities
 */
class AgentRunner {
  constructor() {
    this.processRunner = new ProcessRunner();
  }

  /**
   * Run Claude agent with debouncing
   */
  async runAgent(agentName, context = {}) {
    const lockName = `agent-${agentName}`;
    
    return this.processRunner.withGlobalLock(lockName, async () => {
      console.log(`Running agent: ${agentName}`);
      
      // This would be replaced with actual agent invocation logic
      // For now, just simulate some work
      await new Promise(resolve => setTimeout(resolve, 1000));
      
      console.log(`Agent ${agentName} completed`);
      return { success: true, agent: agentName, context };
    });
  }

  /**
   * Run cleanup tasks with resource limits
   */
  async runCleanupTasks(tasks = []) {
    const lockName = 'pr-cleanup-batch';
    
    return this.processRunner.withGlobalLock(lockName, async () => {
      console.log(`Running ${tasks.length} cleanup tasks`);
      
      for (const task of tasks) {
        console.log(`  - ${task.name}`);
        
        if (task.command) {
          const [cmd, ...args] = task.command.split(' ');
          await this.processRunner.run(cmd, args, {
            timeoutMs: task.timeoutMs || 60000,
          });
        }
      }
      
      console.log('Cleanup tasks completed');
    });
  }
}

// CLI interface
if (require.main === module) {
  const [,, command, ...args] = process.argv;
  
  const runner = new ProcessRunner();
  const agentRunner = new AgentRunner();

  switch (command) {
    case 'run':
      const [cmd, ...cmdArgs] = args;
      runner.run(cmd, cmdArgs)
        .then(code => process.exit(code))
        .catch(err => {
          console.error('Process failed:', err.message);
          process.exit(1);
        });
      break;
      
    case 'agent':
      const agentName = args[0];
      if (!agentName) {
        console.error('Usage: process-utils.js agent <agent-name>');
        process.exit(1);
      }
      agentRunner.runAgent(agentName)
        .then(() => process.exit(0))
        .catch(err => {
          console.error('Agent failed:', err.message);
          process.exit(1);
        });
      break;
      
    case 'cleanup-locks':
      runner.cleanupStaleLocks();
      console.log('Lock cleanup completed');
      break;
      
    default:
      console.log(`Usage: ${path.basename(process.argv[1])} <command> [args...]

Commands:
  run <cmd> [args...]     - Run command with robust process management
  agent <agent-name>      - Run Claude agent with debouncing
  cleanup-locks           - Clean up stale locks

Examples:
  ./scripts/process-utils.js run cargo test -p adze-python
  ./scripts/process-utils.js agent pr-cleanup-reviewer
  ./scripts/process-utils.js cleanup-locks
`);
      process.exit(1);
  }
}

module.exports = { ProcessRunner, AgentRunner };