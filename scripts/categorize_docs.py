#!/usr/bin/env python3
"""
Rule-based document categorization module.
"""

import json
from pathlib import Path
from typing import Dict, List, Optional
from dataclasses import dataclass

@dataclass
class DocumentAnalysis:
    """Analysis result for a document"""
    file_path: Path
    category: str
    confidence: float
    reason: str
    title: Optional[str] = None

class DocumentCategorizer:
    """Rule-based document categorizer"""
    
    def __init__(self, rules_file: str):
        """Initialize with rules from JSON file"""
        self.rules_file = Path(rules_file)
        self.rules = self._load_rules()
    
    def _load_rules(self) -> Dict:
        """Load categorization rules from JSON file"""
        if self.rules_file.exists():
            try:
                with open(self.rules_file, 'r') as f:
                    return json.load(f)
            except Exception as e:
                print(f"Warning: Could not load rules from {self.rules_file}: {e}")
        
        # Default rules if file doesn't exist
        return {
            "categories": {
                "agents": {
                    "keywords": ["agent", "ai", "brain", "sensor", "perception", "behavior", "decision"],
                    "filename_patterns": ["agent", "ai", "brain", "sensor"],
                    "weight": 1.0
                },
                "training": {
                    "keywords": ["neat", "training", "evolution", "fitness", "tournament", "learn", "optimize"],
                    "filename_patterns": ["train", "neat", "evolution", "fitness"],
                    "weight": 1.0
                },
                "gameplay": {
                    "keywords": ["game", "combat", "battle", "physics", "simulation", "team", "fight"],
                    "filename_patterns": ["game", "combat", "battle", "sim"],
                    "weight": 1.0
                },
                "development": {
                    "keywords": ["setup", "install", "build", "dev", "contribute", "architecture", "performance"],
                    "filename_patterns": ["setup", "install", "dev", "build"],
                    "weight": 1.0
                },
                "guides": {
                    "keywords": ["tutorial", "guide", "how", "step", "start", "begin"],
                    "filename_patterns": ["tutorial", "guide", "howto"],
                    "weight": 1.0
                },
                "reference": {
                    "keywords": ["api", "spec", "documentation", "interface", "reference"],
                    "filename_patterns": ["api", "spec", "ref", "doc"],
                    "weight": 1.0
                }
            }
        }
    
    def categorize_file(self, file_path: Path) -> DocumentAnalysis:
        """Categorize a single file"""
        try:
            content = file_path.read_text(encoding='utf-8', errors='ignore')
            return self._categorize_content(file_path, content)
        except Exception as e:
            return DocumentAnalysis(
                file_path=file_path,
                category="misc",
                confidence=0.1,
                reason=f"Error reading file: {e}"
            )
    
    def _categorize_content(self, file_path: Path, content: str) -> DocumentAnalysis:
        """Categorize based on content and filename"""
        content_lower = content.lower()
        filename_lower = file_path.name.lower()
        
        # Extract title
        title = self._extract_title(content, file_path)
        
        # Score each category
        scores = {}
        reasons = {}
        
        for category, rules in self.rules["categories"].items():
            score = 0.0
            reason_parts = []
            
            # Content keyword matching
            keyword_matches = sum(1 for keyword in rules["keywords"] if keyword in content_lower)
            if keyword_matches > 0:
                content_score = (keyword_matches / len(rules["keywords"])) * 0.7
                score += content_score
                reason_parts.append(f"{keyword_matches} content keywords")
            
            # Filename pattern matching
            filename_matches = sum(1 for pattern in rules["filename_patterns"] if pattern in filename_lower)
            if filename_matches > 0:
                filename_score = (filename_matches / len(rules["filename_patterns"])) * 0.3
                score += filename_score
                reason_parts.append(f"{filename_matches} filename patterns")
            
            # Apply category weight
            score *= rules.get("weight", 1.0)
            
            scores[category] = score
            reasons[category] = ", ".join(reason_parts) if reason_parts else "no matches"
        
        # Find best category
        if not scores or max(scores.values()) == 0:
            return DocumentAnalysis(
                file_path=file_path,
                category="misc",
                confidence=0.2,
                reason="No clear category matches found",
                title=title
            )
        
        best_category = max(scores, key=scores.get)
        confidence = min(scores[best_category], 1.0)
        
        return DocumentAnalysis(
            file_path=file_path,
            category=best_category,
            confidence=confidence,
            reason=f"Best match: {reasons[best_category]}",
            title=title
        )
    
    def _extract_title(self, content: str, file_path: Path) -> str:
        """Extract title from content or filename"""
        import re
        
        # Try to find markdown title
        title_match = re.search(r'^#\s+(.+?)$', content, re.MULTILINE)
        if title_match:
            return title_match.group(1).strip()
        
        # Fall back to filename
        return file_path.stem.replace('_', ' ').replace('-', ' ').title() 