# OrchStack

**Token-aware AI coding agent orchestration for Claude Code and Codex.**

OrchStack is a local-first development workspace for turning software specifications into executable tasks, assigning them to AI coding agents, observing their work in real time, reviewing Git changes, and controlling context/token growth.

The first Alpha intentionally focuses on one question:

> Can multi-agent coding stay observable and useful without multiplying context and cached-input usage?

## Alpha Goals

- Rust backend/core
- SvelteKit web application
- Kanban-first workflow
- Issue = design/specification
- Task = executable work unit
- Run = logical execution attempt
- Session = provider/CLI execution session inside a Run
- Claude Code and Codex CLI executors
- Reuse existing CLI subscription authentication
- Role / Soul / Skills / Tools / MCP based Agents
- Automatic Task → Role → Agent assignment
- Realtime Agent / Task / Run diagram
- Streaming logs and execution status
- Local Git status/diff visibility
- GitHub Issue import/link
- Review → Done workflow
- ContextBuilder / ContextManifest
- TokenLedger / ContextProfiler
- Context growth and repeated-context detection

## Token Efficiency Is a Core Requirement

OrchStack must not hide token growth behind a single cumulative number.

Every model call should be attributable as far as the provider allows:

- new input
- cached input / cache read
- cache creation / cache write
- output
- estimated active context
- context source breakdown
- repeated context
- session bootstrap/rotation cost

Stored information is **not** automatically model context.

Raw transcripts, logs, full Issues, entire repositories, and complete Tool/MCP catalogs are kept separate from the context sent to Claude or Codex. Context is assembled per task under an explicit budget.

The Alpha does **not** depend on model-native compact as its primary strategy. Structural pruning and checkpoints come first.

## Alpha Baseline Stack

### Backend

- Rust
- Tokio
- Axum
- Utoipa
- SeaORM
- SQLite

#### Run the backend

```bash
cd backend && cargo run          # http://127.0.0.1:8080/health
```

| Env | Default | |
| --- | --- | --- |
| `DATABASE_URL` | `sqlite://orchstack.db?mode=rwc` | SQLite file, created on first run from `data/sqlite.sql` |
| `BIND_ADDR` | `127.0.0.1:8080` | HTTP listen address |

### Frontend

- Node.js
- SvelteKit
- shadcn-svelte
- Tailwind CSS
- Bits UI
- Svelte-native motion/animation layer

### Executors

- Claude Code CLI
- Codex CLI

OrchStack authentication is separate from provider authentication. The Alpha reuses the authenticated environment of the installed official CLI rather than implementing its own user login.

## Work Model

```text
Project
  └─ Issue        # design / specification
      └─ Task     # executable work
          └─ Run  # logical execution attempt
              └─ Session
```

The Kanban, live diagram, logs, and review views are projections of the same underlying state.

## GitHub Planning Structure

Development planning is maintained in GitHub Issues so long design discussions do not become detached from implementation.

### 1. EPIC

The final destination and Alpha completion criteria.

- #1 OrchStack Alpha — Token-Efficient AI Coding Agent Orchestration

### 2. AREA

Each major product/architecture category maintains its current checklist and child design issues.

- #2 Domain & Workflow
- #3 Backend Core
- #4 Agent Runtime
- #8 Context & Token Efficiency
- #5 Frontend UX
- #6 Git & GitHub
- #7 Ops & Alpha Delivery

### 3. DESIGN

DESIGN issues are implementation blueprints. They contain scope, constraints, implementation tasks, and acceptance criteria.

Current design issues:

- #9 Core Domain Model
- #10 Command → Event → Projection
- #11 Rust Service Baseline
- #12 Realtime API
- #13 Executor Abstraction
- #14 Agent Profile / Assignment / Session Lifecycle
- #15 ContextBuilder / ContextManifest
- #16 TokenLedger / ContextProfiler
- #17 Context Growth Guard
- #18 SvelteKit Workbench
- #19 Agent Configuration UX
- #20 Live Team Diagram
- #21 Local Git Safety
- #22 GitHub Issue Integration
- #23 Ops & Token Health
- #24 Review → Done / Retry / Alpha E2E

## First Alpha Exclusions

The first Alpha intentionally excludes:

- BrainPrint / long-term shared memory
- OrchStack user login and multi-user permissions
- automatic Git branch management
- Git worktrees
- automatic pull request creation
- complex workflow editors
- full conversation replay as default memory

BrainPrint can be added later through a replaceable memory/context provider, but retrieved memory must still pass through the same ContextBuilder and TokenLedger.

## Alpha Success Path

```text
Issue / GitHub Issue
        ↓
      Task
        ↓
 Role / Agent assignment
        ↓
   ContextBuilder
        ↓
 Claude Code / Codex
        ↓
 Logs + Diagram + Kanban
        ↓
    Git Changes
        ↓
      Review
        ↓
       Done
```

At every model call, token/context usage remains observable.

## Project Status

Early Alpha planning and implementation.

The current source of truth for scope and implementation order is the GitHub Issue hierarchy, starting with **#1**.
