# 📁 Human-Readable Documentation Structure

## Current Problems
- Deep nesting (`concepts/training` instead of `training`)
- Technical jargon ("concepts" not user-friendly)
- Inconsistent hierarchy
- Unclear category boundaries

## Proposed Structure

```
docs/
├── 🤖 agents/              # Everything about AI agents
│   ├── how-agents-work.md
│   ├── sensors-and-input.md
│   ├── decision-making.md
│   └── evaluation.md
│
├── 🧠 training/            # NEAT evolution & learning
│   ├── neat-basics.md
│   ├── training-pipeline.md
│   ├── experiments.md
│   └── tournaments.md
│
├── 🎮 gameplay/            # Game mechanics & simulation
│   ├── combat-system.md
│   ├── physics.md
│   ├── team-battles.md
│   └── browser-simulation.md
│
├── 🔧 development/         # Building & contributing
│   ├── getting-started.md
│   ├── architecture.md
│   ├── python-services.md
│   └── performance.md
│
├── 📖 guides/              # Step-by-step tutorials
│   ├── quick-start.md
│   ├── training-new-models.md
│   └── running-tournaments.md
│
└── 📚 reference/           # API docs & specs
    ├── api/
    ├── cli/
    └── specifications/
```

## Why This Works Better

### 🎯 **Clear Mental Models**
- **agents** = "How do the AI players work?"
- **training** = "How do they learn and evolve?"
- **gameplay** = "What are the rules of the game?"
- **development** = "How do I build/modify this?"
- **guides** = "Show me how to do X"
- **reference** = "Look up technical details"

### 🚀 **User Journey Aligned**
1. **New users** → `guides/quick-start.md`
2. **Understanding the game** → `gameplay/`
3. **Understanding AI** → `agents/` → `training/`
4. **Contributing** → `development/`
5. **API lookup** → `reference/`

### 🧹 **Flat & Scannable**
- No deep nesting (max 2 levels)
- Emoji icons for quick scanning
- Descriptive folder names
- Clear file naming conventions

## Category Mapping Rules

```json
{
  "agents": {
    "patterns": ["agent", "perception", "brain", "decision", "sensor"],
    "files": ["AGENT_", "SENSOR_"],
    "description": "AI behavior, sensors, decision-making"
  },
  "training": {
    "patterns": ["training", "neat", "evolution", "fitness", "tournament"],
    "files": ["TRAINING", "NEAT_"],
    "description": "Learning, evolution, optimization"
  },
  "gameplay": {
    "patterns": ["gameplay", "combat", "simulation", "physics", "battle"],
    "files": ["GAMEPLAY", "SIMULATION", "COMBAT"],
    "description": "Game mechanics, physics, rules"
  },
  "development": {
    "patterns": ["development", "architecture", "setup", "build", "performance"],
    "files": ["DEVELOPMENT", "ARCHITECTURE", "PERFORMANCE"],
    "description": "Building, setup, architecture"
  },
  "guides": {
    "patterns": ["tutorial", "guide", "howto", "walkthrough"],
    "files": ["GUIDE_", "TUTORIAL_"],
    "description": "Step-by-step instructions"
  },
  "reference": {
    "patterns": ["api", "reference", "specification", "interface"],
    "files": ["API_", "SPEC_"],
    "description": "Technical references, APIs"
  }
}
```

## Migration Strategy

1. **Update categorization rules** in `doc_rules.json`
2. **Test with `kb categorize --dry-run`**
3. **Migrate incrementally** with `kb analyze`
4. **Update file names** to be more descriptive
5. **Add emoji icons** to folder names for visual scanning 