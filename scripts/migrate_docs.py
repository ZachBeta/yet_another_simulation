#!/usr/bin/env python3
"""
Document migration and analysis module for the knowledge base CLI.
"""

import os
import json
import asyncio
import hashlib
from pathlib import Path
from typing import Dict, List, Optional, Tuple
from dataclasses import dataclass, asdict
from enum import Enum
import re

# Directory constants
DOCS_DIR = Path("docs")
KNOWLEDGE_BASE = DOCS_DIR / "knowledge"
ARCHIVE_DIR = DOCS_DIR / "archive"

class DocumentCategory(Enum):
    """Document categories for organization"""
    AGENTS = "agents"
    TRAINING = "training"
    GAMEPLAY = "gameplay"
    DEVELOPMENT = "development"
    GUIDES = "guides"
    REFERENCE = "reference"
    MISC = "misc"

@dataclass
class DocumentAnalysis:
    """Analysis result for a document"""
    file_path: Path
    category: str
    confidence: float
    reason: str
    title: Optional[str] = None
    summary: Optional[str] = None
    
    def to_dict(self) -> Dict:
        """Convert to dictionary for JSON serialization"""
        return {
            'file_path': str(self.file_path),
            'category': self.category,
            'confidence': self.confidence,
            'reason': self.reason,
            'title': self.title,
            'summary': self.summary
        }

def analyze_file(file_path: Path) -> DocumentAnalysis:
    """Analyze a single file and determine its category"""
    try:
        content = file_path.read_text(encoding='utf-8', errors='ignore')
        
        # Extract title
        title = extract_title(content, file_path)
        
        # Categorize based on content
        category, confidence, reason = categorize_content(content, file_path, title)
        
        return DocumentAnalysis(
            file_path=file_path,
            category=category,
            confidence=confidence,
            reason=reason,
            title=title
        )
        
    except Exception as e:
        return DocumentAnalysis(
            file_path=file_path,
            category="misc",
            confidence=0.1,
            reason=f"Error reading file: {e}",
            title=file_path.stem
        )

def extract_title(content: str, file_path: Path) -> str:
    """Extract title from content or filename"""
    # Try to find markdown title
    title_match = re.search(r'^#\s+(.+?)$', content, re.MULTILINE)
    if title_match:
        return title_match.group(1).strip()
    
    # Fall back to filename
    return file_path.stem.replace('_', ' ').replace('-', ' ').title()

def categorize_content(content: str, file_path: Path, title: str) -> Tuple[str, float, str]:
    """Categorize content based on keywords and patterns"""
    content_lower = content.lower()
    title_lower = title.lower()
    filename_lower = file_path.name.lower()
    
    # Count keyword matches for better scoring
    training_keywords = ['neat', 'training', 'evolution', 'fitness', 'tournament', 'population', 'genome', 'generation', 'mutation', 'crossover', 'neural network', 'neuroevolution']
    agent_keywords = ['agent', 'sensor', 'perception', 'brain', 'ai behavior', 'decision', 'worldview', 'action']
    gameplay_keywords = ['gameplay', 'combat', 'battle', 'simulation', 'physics', 'match', 'health', 'damage', 'weapon', 'shield']
    development_keywords = ['setup', 'install', 'build', 'development', 'architecture', 'cargo', 'rust', 'wasm', 'implementation']
    guide_keywords = ['tutorial', 'guide', 'how to', 'step by step', 'walkthrough', 'implementation guide']
    reference_keywords = ['api', 'reference', 'documentation', 'spec', 'interface', 'contract']
    
    # Score each category
    training_score = sum(1 for kw in training_keywords if kw in content_lower)
    agent_score = sum(1 for kw in agent_keywords if kw in content_lower)
    gameplay_score = sum(1 for kw in gameplay_keywords if kw in content_lower)
    development_score = sum(1 for kw in development_keywords if kw in content_lower)
    guide_score = sum(1 for kw in guide_keywords if kw in content_lower)
    reference_score = sum(1 for kw in reference_keywords if kw in content_lower)
    
    # Add filename bonus
    if any(kw in filename_lower for kw in ['neat', 'train', 'evolution', 'fitness']):
        training_score += 2
    if any(kw in filename_lower for kw in ['agent', 'brain', 'sensor']):
        agent_score += 2
    if any(kw in filename_lower for kw in ['game', 'combat', 'battle', 'simulation']):
        gameplay_score += 2
    if any(kw in filename_lower for kw in ['setup', 'dev', 'build', 'architecture']):
        development_score += 2
    if any(kw in filename_lower for kw in ['tutorial', 'guide', 'howto']):
        guide_score += 2
    if any(kw in filename_lower for kw in ['api', 'spec', 'ref']):
        reference_score += 2
    
    # Find the best category
    scores = {
        'training': training_score,
        'agents': agent_score,
        'gameplay': gameplay_score,
        'development': development_score,
        'guides': guide_score,
        'reference': reference_score
    }
    
    best_category = max(scores, key=scores.get)
    best_score = scores[best_category]
    
    if best_score == 0:
        return "misc", 0.3, "No clear category matches found"
    
    # Calculate confidence based on score
    confidence = min(0.5 + (best_score * 0.1), 0.95)
    
    # Special handling for tutorials/guides
    if guide_score > 0 and best_score > 1:
        if best_category == 'training':
            return "training", confidence, f"Training tutorial/guide (score: {best_score})"
        elif best_category == 'agents':
            return "agents", confidence, f"Agent tutorial/guide (score: {best_score})"
        elif best_category == 'gameplay':
            return "gameplay", confidence, f"Gameplay tutorial/guide (score: {best_score})"
        elif best_category == 'development':
            return "development", confidence, f"Development tutorial/guide (score: {best_score})"
        else:
            return "guides", confidence, f"General tutorial/guide (score: {best_score})"
    
    return best_category, confidence, f"Best match with score: {best_score}"

async def categorize_with_openrouter(file_path: Path) -> DocumentAnalysis:
    """Categorize using OpenRouter API (placeholder for now)"""
    # For now, fall back to rule-based categorization
    return analyze_file(file_path)

async def process_files(file_paths: List[Path], dry_run: bool = False, interactive: bool = True) -> List[DocumentAnalysis]:
    """Process multiple files for categorization"""
    results = []
    
    for file_path in file_paths:
        print(f"📄 Analyzing: {file_path}")
        
        analysis = analyze_file(file_path)
        results.append(analysis)
        
        print(f"   Category: {analysis.category}")
        print(f"   Confidence: {analysis.confidence:.1%}")
        print(f"   Reason: {analysis.reason}")
        
        if not dry_run and not interactive:
            # Auto-move files if not in dry-run mode and not interactive
            target_folder = get_target_folder(analysis.category)
            target_dir = DOCS_DIR / target_folder
            target_path = target_dir / file_path.name
            
            # Handle conflicts
            counter = 1
            original_target = target_path
            while target_path.exists():
                stem = original_target.stem
                suffix = original_target.suffix
                target_path = original_target.parent / f"{stem}_{counter}{suffix}"
                counter += 1
            
            # Create directory and move file
            target_dir.mkdir(parents=True, exist_ok=True)
            file_path.rename(target_path)
            print(f"   ✅ Moved to: {target_path}")
        
        print()
    
    return results

def get_target_folder(category: str) -> str:
    """Get the target folder name for a category"""
    folder_map = {
        "agents": "🤖 agents",
        "training": "🧠 training", 
        "gameplay": "🎮 gameplay",
        "development": "🔧 development",
        "guides": "📖 guides",
        "reference": "📚 reference",
        "misc": "misc"
    }
    return folder_map.get(category, category) 