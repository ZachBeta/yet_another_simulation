# Agent API Contract

## Types & Interfaces (JSDoc Style)

```js
/**
 * @typedef {Object} AgentPercept
 * @property {{x:number, y:number, team:Team, health:number}} self
 * @property {Array<{x:number, y:number, team:Team, health:number, id:number}>} visibleAgents
 * @property {Array<{x:number, y:number}>} audibleEvents
 */

/**
 * @typedef {Object} AgentAction
 * @property {'move'|'attack'|'wait'} type
 * @property {number} [dx]
 * @property {number} [dy]
 * @property {number} [targetId]
 */
```

---

## Agent Class (API Contract)

```js
/**
 * @class
 */
class Agent {
  /**
   * @param {number} x
   * @param {number} y
   * @param {Team} team
   * @param {Behavior} behavior
   */
  constructor(x, y, team, behavior) { ... }

  /**
   * Decide what to do based on percept.
   * @param {AgentPercept} percept
   * @returns {AgentAction}
   */
  decide(percept) { ... }
}
```

---

## Behavior Contract

```js
/**
 * @interface
 */
class Behavior {
  /**
   * @param {AgentPercept} percept
   * @param {Agent} self
   * @returns {AgentAction}
   */
  decide(percept, self) { ... }
}
```

---

## Simulation Loop

```js
for (const agent of agents) {
  const percept = buildPerceptForAgent(agent, agents, events);
  const action = agent.decide(percept);
  plannedActions.push({ agent, action });
}
for (const { agent, action } of plannedActions) {
  applyAction(agent, action, agents);
}
```

---

## applyAction Function

```js
/**
 * @param {Agent} agent
 * @param {AgentAction} action
 * @param {Agent[]} agents
 */
function applyAction(agent, action, agents) {
  switch (action.type) {
    case 'move':
      agent.x += action.dx;
      agent.y += action.dy;
      break;
    case 'attack':
      const target = agents.find(a => a.id === action.targetId);
      if (target) target.health -= agent.attackDamage;
      break;
    case 'wait':
    default:
      // do nothing
      break;
  }
}
```

---

## Minimal Percept Builder

```js
function buildPerceptForAgent(agent, allAgents, events) {
  return {
    self: { x: agent.x, y: agent.y, team: agent.team, health: agent.health },
    visibleAgents: allAgents.filter(a => a !== agent), // for now, all others
    audibleEvents: events // for now, all events
  };
}
```
