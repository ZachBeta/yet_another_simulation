# Champion Dropdown Metadata Display Options

This document outlines various approaches to succinctly display champion metadata in the dropdown:

## 1. Option Label Composition
- Show key fields in the option text:
  - Generation (e.g., `Gen 166`)
  - Elo (e.g., `Elo 1776`)
  - Date (e.g., `2025-05-19`)
- Example:
  ```html
  <option value="...">Gen 166 • Elo 1776 • 2025-05-19</option>
  ```

## 2. Grouping with Optgroups
- Separate options into `<optgroup>`s:
  - **Latest Champion**
  - **Top 10 by Elo**
- Provides visual separation between most recent and historical bests.

## 3. Tooltips for Details
- Use the `title` attribute on `<option>` elements (or include an `ℹ️` icon) to show hover-text:
  - Map dimensions, random seed, fitness weights, etc.
- Keeps the dropdown concise while still exposing full context on hover.

## 4. On-Select Details Panel
- Display a summary card below the dropdown upon selection:
  ```text
  Generation: 166
  Timestamp: 2025-05-19_09:05:12
  Elo: 1776.7
  Map: 1200×800 (±200, seed=42)
  Weights: H=1.0, D=1.0, K=0.5
  ```
- Offloads detailed metadata from the select into a separate, dedicated panel.

---

You can pick one of these approaches or combine multiple options. We can revisit and implement when ready.
