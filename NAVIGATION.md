# Navigation Guide - Find What You Need Fast

**Lost?** **Overwhelmed?** **Not sure where to start?**

This guide helps you find the right document for your needs.

---

## 🎯 I Want To...

### ...Start Using adze RIGHT NOW (5 minutes)
→ **[QUICK_START.md](./QUICK_START.md)** - Get parsing in 5 minutes

### ...Learn adze Properly (30 minutes)
→ **[docs/GETTING_STARTED.md](./docs/GETTING_STARTED.md)** - Complete tutorial

### ...Understand How It Works
→ **[ARCHITECTURE.md](./ARCHITECTURE.md)** - Visual architecture guide with diagrams

### ...Answer a Specific Question
→ **[FAQ.md](./FAQ.md)** - Common questions answered

### ...See Working Examples
→ **[example/src/](./example/src/)** - Real grammars (arithmetic, JSON, etc.)

### ...Contribute to adze
→ **[CONTRIBUTING.md](./CONTRIBUTING.md)** - Development setup
→ **[GAPS.md](./GAPS.md)** - 43 tasks ready to pick up

### ...Know What's Implemented vs What's Planned
→ **[CURRENT_STATUS_2025-11.md](./CURRENT_STATUS_2025-11.md)** - v0.6.1 status
→ **[ROADMAP.md](./ROADMAP.md)** - v0.7.0 and beyond

### ...Track Current Development
→ **[IMPLEMENTATION_PLAN.md](./IMPLEMENTATION_PLAN.md)** - Week-by-week v0.7.0 schedule
→ **[docs/progress/](./docs/progress/)** - Weekly updates

### ...Debug a Problem
→ **[FAQ.md](./FAQ.md)** - Common issues section
→ **[docs/TROUBLESHOOTING.md](./docs/TROUBLESHOOTING.md)** (coming v0.7.0)

### ...Understand Performance
→ **[docs/PERFORMANCE_BASELINE.md](./docs/PERFORMANCE_BASELINE.md)** - Current performance

### ...Look Up API Details
→ **[API_DOCUMENTATION.md](./API_DOCUMENTATION.md)** - Complete API reference

---

## 📚 Documents by Purpose

### Getting Started
| Document | Purpose | Time Needed |
|----------|---------|-------------|
| [README.md](./README.md) | Project overview | 5 min |
| [QUICK_START.md](./QUICK_START.md) | Get parsing now | 5 min |
| [docs/GETTING_STARTED.md](./docs/GETTING_STARTED.md) | Full tutorial | 30 min |
| [example/src/](./example/src/) | Working examples | 10 min |

### Understanding
| Document | Purpose | Time Needed |
|----------|---------|-------------|
| [ARCHITECTURE.md](./ARCHITECTURE.md) | How components fit together | 15 min |
| [FAQ.md](./FAQ.md) | Common questions | 10 min |
| [API_DOCUMENTATION.md](./API_DOCUMENTATION.md) | API details | As needed |

### Contributing
| Document | Purpose | Time Needed |
|----------|---------|-------------|
| [CONTRIBUTING.md](./CONTRIBUTING.md) | How to contribute | 10 min |
| [GAPS.md](./GAPS.md) | Available tasks | 20 min |
| [IMPLEMENTATION_PLAN.md](./IMPLEMENTATION_PLAN.md) | v0.7.0 schedule | 15 min |
| [docs/dev-workflow.md](./docs/dev-workflow.md) | Dev commands | 5 min |

### Status & Planning
| Document | Purpose | Time Needed |
|----------|---------|-------------|
| [CURRENT_STATUS_2025-11.md](./CURRENT_STATUS_2025-11.md) | What works today | 10 min |
| [ROADMAP.md](./ROADMAP.md) | Future plans | 10 min |
| [CHANGELOG.md](./CHANGELOG.md) | Version history | As needed |
| [docs/progress/](./docs/progress/) | Weekly updates | 5 min |

---

## 🔍 Documents by Audience

### New Users
1. Start: [QUICK_START.md](./QUICK_START.md)
2. Learn: [docs/GETTING_STARTED.md](./docs/GETTING_STARTED.md)
3. Examples: [example/src/](./example/src/)
4. Questions: [FAQ.md](./FAQ.md)

### Grammar Authors
1. Tutorial: [docs/GETTING_STARTED.md](./docs/GETTING_STARTED.md)
2. Patterns: [GAPS.md](./GAPS.md) - Grammar patterns section
3. API: [API_DOCUMENTATION.md](./API_DOCUMENTATION.md)
4. Examples: [example/src/](./example/src/)

### Contributors
1. Setup: [CONTRIBUTING.md](./CONTRIBUTING.md)
2. Tasks: [GAPS.md](./GAPS.md)
3. Schedule: [IMPLEMENTATION_PLAN.md](./IMPLEMENTATION_PLAN.md)
4. Architecture: [ARCHITECTURE.md](./ARCHITECTURE.md)

### Project Maintainers
1. Status: [CURRENT_STATUS_2025-11.md](./CURRENT_STATUS_2025-11.md)
2. Plan: [IMPLEMENTATION_PLAN.md](./IMPLEMENTATION_PLAN.md)
3. Roadmap: [ROADMAP.md](./ROADMAP.md)
4. Progress: [docs/progress/](./docs/progress/)

---

## 📖 Document Relationships

```
Entry Points
├── README.md ─────────────────┬─> Start here
│                              │
├── QUICK_START.md ────────────┤   5-minute guide
│                              │
└── FAQ.md ────────────────────┘   Quick answers

Learning Path
├── docs/GETTING_STARTED.md ───┬─> Tutorial
│                              │
├── example/src/ ──────────────┤   Examples
│                              │
├── ARCHITECTURE.md ───────────┤   How it works
│                              │
└── API_DOCUMENTATION.md ──────┘   Reference

Contributing Path
├── CONTRIBUTING.md ───────────┬─> How to help
│                              │
├── GAPS.md ───────────────────┤   What to do
│                              │
├── IMPLEMENTATION_PLAN.md ────┤   When to do it
│                              │
└── docs/dev-workflow.md ──────┘   Commands

Status & Planning
├── CURRENT_STATUS_2025-11.md ─┬─> Now
│                              │
├── ROADMAP.md ────────────────┤   Future
│                              │
├── CHANGELOG.md ──────────────┤   Past
│                              │
└── docs/progress/ ────────────┘   Weekly
```

---

## 🗂️ File Organization

### Root Directory
```
adze/
├── README.md                    # Start here
├── QUICK_START.md              # 5-minute guide
├── FAQ.md                      # Common questions
├── ARCHITECTURE.md             # System design
├── CONTRIBUTING.md             # How to contribute
├── GAPS.md                     # Task list (43 tasks)
├── IMPLEMENTATION_PLAN.md      # v0.7.0 schedule
├── ROADMAP.md                  # Long-term vision
├── CURRENT_STATUS_2025-11.md   # v0.6.1 status
├── CHANGELOG.md                # Version history
├── API_DOCUMENTATION.md        # API reference
└── NAVIGATION.md               # This file!
```

### docs/ Directory
```
docs/
├── GETTING_STARTED.md          # Full tutorial
├── PERFORMANCE_BASELINE.md     # Performance data
├── dev-workflow.md             # Dev commands
├── progress/                   # Weekly updates
│   └── WEEK1_PROGRESS.md
└── ... (other guides)
```

### example/ Directory
```
example/src/
├── arithmetic.rs               # Expression grammar
├── json.rs                     # JSON parser
├── repetition.rs              # List patterns
└── ... (more examples)
```

---

## 💡 Quick Tips

### First Time Here?
1. Read [README.md](./README.md) (5 min)
2. Try [QUICK_START.md](./QUICK_START.md) (5 min)
3. Browse [example/src/](./example/src/) (10 min)

### Want to Understand It?
1. [ARCHITECTURE.md](./ARCHITECTURE.md) - Visual overview
2. [docs/GETTING_STARTED.md](./docs/GETTING_STARTED.md) - Deep dive
3. [FAQ.md](./FAQ.md) - Common questions

### Ready to Contribute?
1. [CONTRIBUTING.md](./CONTRIBUTING.md) - Setup
2. [GAPS.md](./GAPS.md) - Pick a task (start with "good first issue")
3. [IMPLEMENTATION_PLAN.md](./IMPLEMENTATION_PLAN.md) - See the plan

### Stuck?
1. Check [FAQ.md](./FAQ.md)
2. Search [GitHub Issues](https://github.com/EffortlessMetrics/adze/issues)
3. Ask in [GitHub Discussions](https://github.com/EffortlessMetrics/adze/discussions)

---

## 🔗 External Links

- **GitHub**: https://github.com/EffortlessMetrics/adze
- **Crates.io**: https://crates.io/crates/adze
- **Issues**: https://github.com/EffortlessMetrics/adze/issues
- **Discussions**: https://github.com/EffortlessMetrics/adze/discussions

---

**Still can't find what you need?**
Open a [GitHub Discussion](https://github.com/EffortlessMetrics/adze/discussions) and we'll help!
