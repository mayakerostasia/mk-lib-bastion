# Session Files Guide

This document explains all the planning and summary files created during development.

---

## 📍 Location

All session-specific files are in:
```
/home/mayak/.copilot/session-state/586f0987-c8b9-4029-af24-6dd553fa4c87/
```

**Note:** These files are NOT in the git repository. They're in the Copilot CLI session workspace.

---

## 📁 File Organization

### `/plan.md` - Main Plan
The master plan tracking all refactoring tasks. Organized by phase (1-7) with task breakdowns, dependencies, and success criteria.

**Updated:** Throughout development  
**Use:** Track overall progress, find next tasks

### `/checkpoints/` - Work Summaries
Detailed summaries of completed work sessions:
- `001-frame-rkyv-serialization-and-r.md` - Frame serialization work
- `002-fleet-refactoring-and-agent-pa.md` - Fleet refactoring
- `003-agent-visualization-and-llm-en.md` - Viz planning
- `004-visualization-and-llm-infrastructure.md` - Current session

**Updated:** After major milestones  
**Use:** Understand what was done and why

### `/files/` - Analysis & Design Documents
Detailed analysis and design documents:

#### Configuration
- `config-analysis.md` - Problems with old config system
- `config-design.md` - Unified config system design

#### Base API
- `baseapi-analysis.md` - Base API structure issues
- `baseapi-design.md` - Proposed new structure

#### Transports
- `iggy-nats-comparison.md` - NATS vs Iggy comparison
- `transport-abstraction-analysis.md` - Abstraction recommendations

#### Features
- `event-system-design.md` - Event observability design
- `http-listener-api-doc.md` - HTTP API documentation

#### Session Work
- `visualization-plan.md` - Initial viz planning
- `viz-transport-plan.md` - Transport integration plan
- `transport-integration-complete.md` - Completion notes

**Updated:** During analysis/design phases  
**Use:** Understand design decisions

---

## 📄 Repository Files (Git-Tracked)

These files ARE in the git repository:

### Planning & Status
- `/STATUS_REPORT.md` - Current project status (NEW)
- `/SESSION_ACCOMPLISHMENTS.md` - Recent session achievements (NEW)
- `/NEXT_SESSION_PLAN.md` - Phase 5 roadmap (NEW)

### Architecture
- `/ARCHITECTURE.md` - System architecture
- `/MIGRATION.md` - Breaking changes guide
- `/FUTURE_REFACTORING.md` - v3.0.0 plans

### User Guides
- `/README.md` - Main project overview
- `/README.LLM-SETUP.md` - Quick LLM setup (NEW)
- `/LLM_TESTING.md` - Comprehensive LLM guide (NEW)

### Developer Guides
- `/DOCKER_BUILD_NOTES.md` - Container build guide (NEW)
- `/DOCKER_VALIDATION_REPORT.md` - Container status (NEW)
- `/README.SESSION-FILES.md` - This file (NEW)

### Scripts
- `/validate-docker.sh` - Docker validation script (NEW)

---

## 🗺️ How to Navigate

### "What should I work on next?"
→ Read `/NEXT_SESSION_PLAN.md` (in repo)

### "What was just done?"
→ Read `/SESSION_ACCOMPLISHMENTS.md` (in repo)

### "What's the overall status?"
→ Read `/STATUS_REPORT.md` (in repo)

### "Why was X designed this way?"
→ Check `/files/*-design.md` (in session)

### "What's broken or incomplete?"
→ Read `/plan.md` (in session), look for tasks not marked done

### "How do I set up LLMs?"
→ Read `/README.LLM-SETUP.md` (in repo)

### "How do I build containers?"
→ Read `/DOCKER_BUILD_NOTES.md` (in repo)

---

## 📊 File Relationships

```
Session Files (Not in Git)
├── plan.md ───────────────┐
│   └── Master task list   │
├── checkpoints/           │
│   └── What was done      │  ┌──> STATUS_REPORT.md
├── files/                 │  │    (Current state)
│   ├── *-analysis.md ─────┼──┤
│   ├── *-design.md ───────┘  │
│   └── *-plan.md             │
                              │
Repository Files (In Git)    │
├── SESSION_ACCOMPLISHMENTS ──┤
│   (Recent work)            │
├── NEXT_SESSION_PLAN ────────┤
│   (Future work)            │
├── ARCHITECTURE.md ──────────┤
├── MIGRATION.md             │
└── Other docs ───────────────┘
```

---

## 🎯 Quick Reference

### Start New Session
1. Read `STATUS_REPORT.md` - Know current state
2. Read `NEXT_SESSION_PLAN.md` - Know what's next
3. Review last checkpoint - Understand recent context
4. Check `plan.md` - See task status

### During Session
1. Update `plan.md` - Mark tasks complete
2. Create analysis docs in `/files/` as needed
3. Update code and docs in repo

### End Session
1. Create new checkpoint in `/checkpoints/`
2. Update `SESSION_ACCOMPLISHMENTS.md`
3. Update `STATUS_REPORT.md`
4. Create/update `NEXT_SESSION_PLAN.md`

---

## 🔍 Finding Information

| Question | File to Read |
|----------|--------------|
| What tasks remain? | `plan.md` |
| What just happened? | `SESSION_ACCOMPLISHMENTS.md` |
| What's next? | `NEXT_SESSION_PLAN.md` |
| Current status? | `STATUS_REPORT.md` |
| How does X work? | `ARCHITECTURE.md` |
| How do I setup Y? | `README.md` or specific README |
| Why was Z chosen? | `/files/*-design.md` |
| What changed? | `MIGRATION.md` |
| Future plans? | `FUTURE_REFACTORING.md` |

---

## 📝 File Naming Conventions

### Session Files
- `*-analysis.md` - Problem analysis
- `*-design.md` - Solution design
- `*-plan.md` - Implementation plan
- `*-complete.md` - Completion notes

### Checkpoint Files
- `NNN-short-description.md` - Numbered chronologically

### Repository Files
- `README.*.md` - User-facing guides
- `*_NOTES.md` - Technical notes
- `*_REPORT.md` - Status reports
- `*.md` - General documentation

---

## 🎓 Best Practices

### When Creating Analysis Docs
- Be specific about the problem
- Provide concrete examples
- List alternatives considered
- Recommend a solution with reasoning

### When Creating Design Docs
- Show before/after
- Include code examples
- Document trade-offs
- Define success criteria

### When Creating Plan Docs
- Break into small tasks
- Define dependencies
- Estimate complexity
- Provide context

### When Creating Summaries
- What was built?
- Why those decisions?
- What worked well?
- What didn't?
- What's next?

---

## 🚀 New Developer Onboarding

If you're new to this project:

1. **Start here:** `/README.md` (main overview)
2. **Then read:** `/ARCHITECTURE.md` (how it works)
3. **Then read:** `/STATUS_REPORT.md` (current state)
4. **Then read:** Latest checkpoint (recent work)
5. **Then read:** `/NEXT_SESSION_PLAN.md` (what's next)

After that, you'll understand:
- What the project does
- How it's architected
- What state it's in
- What was recently done
- What needs doing next

---

## 📞 Questions?

If something is unclear:
1. Check if there's a design doc in `/files/`
2. Check checkpoint summaries for context
3. Check `ARCHITECTURE.md` for high-level view
4. Check specific crate READMEs for details

---

*Last Updated: 2026-02-27*
