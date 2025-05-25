# UI Design Guidelines

This document outlines the design patterns and best practices for the simulation's user interface components.

## Table of Contents

1. [Champion Selection](#1-champion-selection)
   - [1.1 Dropdown Design](#11-dropdown-design)
   - [1.2 Metadata Display](#12-metadata-display)
   - [1.3 Grouping and Organization](#13-grouping-and-organization)
2. [Common Components](#2-common-components)
   - [2.1 Buttons](#21-buttons)
   - [2.2 Forms](#22-forms)
   - [2.3 Modals](#23-modals)
3. [Responsive Design](#3-responsive-design)
4. [Accessibility](#4-accessibility)
5. [Performance Considerations](#5-performance-considerations)

## 1. Champion Selection

### 1.1 Dropdown Design

The champion selection dropdown is a key interface element that allows users to choose between different trained models.

**Basic Structure:**
```html
<select id="championSelect" class="champion-dropdown">
  <!-- Options will be populated dynamically -->
</select>
```

### 1.2 Metadata Display

Champion options should display the following information in a concise format:

**Option Label Format:**
```
[Generation] • [Elo] • [Date]
```

**Example:**
```html
<option value="champion_166">Gen 166 • Elo 1776 • 2025-05-19</option>
```

### 1.3 Grouping and Organization

Use `<optgroup>` elements to organize champions into logical groups:

```html
<select id="championSelect">
  <optgroup label="Latest Champions">
    <option value="champion_166">Gen 166 • Elo 1776 • 2025-05-19</option>
    <option value="champion_165">Gen 165 • Elo 1750 • 2025-05-18</option>
  </optgroup>
  <optgroup label="Top Performers">
    <option value="champion_150">Gen 150 • Elo 1800 • 2025-05-15</option>
    <option value="champion_140">Gen 140 • Elo 1780 • 2025-05-10</option>
  </optgroup>
</select>
```

### 1.4 Detailed Information

Display additional metadata when a champion is selected:

```html
<div id="championDetails" class="champion-details">
  <h3>Champion Details</h3>
  <div class="detail-row">
    <span class="detail-label">Generation:</span>
    <span id="champGen">166</span>
  </div>
  <div class="detail-row">
    <span class="detail-label">Timestamp:</span>
    <span id="champTimestamp">2025-05-19 09:05:12</span>
  </div>
  <div class="detail-row">
    <span class="detail-label">Elo Rating:</span>
    <span id="champElo">1776.7</span>
  </div>
  <div class="detail-row">
    <span class="detail-label">Map:</span>
    <span id="champMap">1200×800 (±200, seed=42)</span>
  </div>
  <div class="detail-row">
    <span class="detail-label">Weights:</span>
    <span id="champWeights">H=1.0, D=1.0, K=0.5</span>
  </div>
</div>
```

## 2. Common Components

### 2.1 Buttons

**Standard Button:**
```html
<button class="btn btn-primary">Primary Action</button>
<button class="btn btn-secondary">Secondary Action</button>
<button class="btn btn-danger">Danger Action</button>
```

**Button with Icon:**
```html
<button class="btn btn-icon">
  <i class="fas fa-play"></i> Start Simulation
</button>
```

### 2.2 Forms

**Form Group:**
```html
<div class="form-group">
  <label for="simulationSpeed">Simulation Speed</label>
  <select id="simulationSpeed" class="form-control">
    <option value="0.5">0.5x</option>
    <option value="1" selected>1x</option>
    <option value="2">2x</option>
    <option value="5">5x</option>
  </select>
</div>
```

### 2.3 Modals

**Modal Structure:**
```html
<div class="modal" id="settingsModal">
  <div class="modal-content">
    <div class="modal-header">
      <h2>Settings</h2>
      <button class="close-btn">&times;</button>
    </div>
    <div class="modal-body">
      <!-- Modal content goes here -->
    </div>
    <div class="modal-footer">
      <button class="btn btn-secondary">Cancel</button>
      <button class="btn btn-primary">Save Changes</button>
    </div>
  </div>
</div>
```

## 3. Responsive Design

- Use CSS Grid or Flexbox for layouts
- Implement mobile-first design principles
- Ensure all interactive elements are at least 44×44px on touch devices
- Test on various screen sizes (mobile, tablet, desktop)

## 4. Accessibility

- Use semantic HTML elements
- Ensure proper contrast ratios (minimum 4.5:1 for normal text)
- Add ARIA attributes where necessary
- Support keyboard navigation
- Provide text alternatives for non-text content

## 5. Performance Considerations

- Lazy load non-critical resources
- Minimize DOM updates
- Use CSS transforms and opacity for animations
- Implement virtual scrolling for long lists
- Optimize images and other assets

## 6. Implementation Example

### JavaScript for Champion Selection

```javascript
// Initialize champion dropdown
document.addEventListener('DOMContentLoaded', () => {
  const championSelect = document.getElementById('championSelect');
  const championDetails = document.getElementById('championDetails');
  
  // Load champions from API or local data
  async function loadChampions() {
    try {
      const response = await fetch('/api/champions');
      const champions = await response.json();
      
      // Clear existing options
      championSelect.innerHTML = '';
      
      // Add latest champions group
      const latestGroup = document.createElement('optgroup');
      latestGroup.label = 'Latest Champions';
      
      champions.slice(0, 3).forEach(champ => {
        latestGroup.appendChild(createChampionOption(champ));
      });
      
      // Add top performers group
      const topGroup = document.createElement('optgroup');
      topGroup.label = 'Top Performers';
      
      [...champions]
        .sort((a, b) => b.elo - a.elo)
        .slice(0, 5)
        .forEach(champ => {
          topGroup.appendChild(createChampionOption(champ));
        });
      
      championSelect.appendChild(latestGroup);
      championSelect.appendChild(topGroup);
      
      // Select first champion by default
      if (champions.length > 0) {
        updateChampionDetails(champions[0]);
      }
    } catch (error) {
      console.error('Failed to load champions:', error);
    }
  }
  
  // Create option element for a champion
  function createChampionOption(champion) {
    const option = document.createElement('option');
    option.value = champion.id;
    option.textContent = `Gen ${champion.generation} • Elo ${Math.round(champion.elo)} • ${champion.date}`;
    option.dataset.champion = JSON.stringify(champion);
    return option;
  }
  
  // Update champion details when selection changes
  championSelect.addEventListener('change', (e) => {
    const selectedOption = e.target.selectedOptions[0];
    if (selectedOption && selectedOption.dataset.champion) {
      const champion = JSON.parse(selectedOption.dataset.champion);
      updateChampionDetails(champion);
    }
  });
  
  // Update the champion details panel
  function updateChampionDetails(champion) {
    document.getElementById('champGen').textContent = champion.generation;
    document.getElementById('champTimestamp').textContent = champion.timestamp;
    document.getElementById('champElo').textContent = champion.elo.toFixed(1);
    document.getElementById('champMap').textContent = 
      `${champion.map.width}×${champion.map.height} (±${champion.map.variation}, seed=${champion.map.seed})`;
    document.getElementById('champWeights').textContent = 
      `H=${champion.weights.health}, D=${champion.weights.damage}, K=${champion.weights.kills}`;
  }
  
  // Initial load
  loadChampions();
});
```

### CSS for Champion Dropdown

```css
/* Champion Dropdown */
.champion-dropdown {
  width: 100%;
  padding: 0.5rem;
  border: 1px solid #ccc;
  border-radius: 4px;
  font-size: 1rem;
  background-color: white;
  color: #333;
}

/* Champion Details Panel */
.champion-details {
  margin-top: 1rem;
  padding: 1rem;
  border: 1px solid #eee;
  border-radius: 4px;
  background-color: #f9f9f9;
}

.champion-details h3 {
  margin-top: 0;
  margin-bottom: 1rem;
  font-size: 1.1rem;
  color: #333;
}

.detail-row {
  display: flex;
  margin-bottom: 0.5rem;
  line-height: 1.4;
}

.detail-label {
  font-weight: bold;
  width: 120px;
  color: #666;
}

/* Responsive adjustments */
@media (max-width: 768px) {
  .champion-details {
    font-size: 0.9rem;
  }
  
  .detail-row {
    flex-direction: column;
    margin-bottom: 0.75rem;
  }
  
  .detail-label {
    margin-bottom: 0.25rem;
    width: 100%;
  }
}
```

## 7. Best Practices

1. **Consistency**
   - Use consistent spacing, colors, and typography throughout the application
   - Maintain a consistent interaction model

2. **Feedback**
   - Provide visual feedback for user actions
   - Show loading states for asynchronous operations
   - Display helpful error messages

3. **Performance**
   - Debounce or throttle event handlers where appropriate
   - Use CSS transitions for smooth animations
   - Optimize images and other assets

4. **Testing**
   - Test on multiple browsers and devices
   - Verify keyboard navigation
   - Check color contrast and readability
