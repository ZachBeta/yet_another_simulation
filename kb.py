#!/usr/bin/env python3
"""
🧠 Knowledge Base CLI

Unified interface for managing the neural network battle simulation knowledge base.
Consolidates document analysis, categorization, and querying into a single tool.
"""

import os
import sys
import json
import time
import argparse
from pathlib import Path
from typing import Dict, List, Optional, Tuple
from dataclasses import dataclass, asdict
from enum import Enum
import re

# Add scripts directory to path for imports
sys.path.append(str(Path(__file__).parent / "scripts"))

try:
    from migrate_docs import (
        DocumentAnalysis, DocumentCategory, 
        categorize_with_openrouter, analyze_file,
        DOCS_DIR, KNOWLEDGE_BASE, ARCHIVE_DIR
    )
    from categorize_docs import DocumentCategorizer
except ImportError as e:
    print(f"Error importing modules: {e}")
    print("Make sure you're running from the project root directory")
    sys.exit(1)

# Add this after the imports, before the KnowledgeBase class
CATEGORY_FOLDERS = {
    "agents": "🤖 agents",
    "training": "🧠 training", 
    "gameplay": "🎮 gameplay",
    "development": "🔧 development",
    "guides": "📖 guides",
    "reference": "📚 reference",
    "misc": "misc"
}

def get_target_folder(category: str) -> str:
    """Get the target folder name for a category"""
    return CATEGORY_FOLDERS.get(category, category)

class KnowledgeBase:
    """Main knowledge base interface"""
    
    def __init__(self):
        self.docs_dir = DOCS_DIR
        self.kb_dir = KNOWLEDGE_BASE
        self.archive_dir = ARCHIVE_DIR
        
    def status(self) -> Dict:
        """Get knowledge base statistics"""
        stats = {
            'total_docs': 0,
            'kb_docs': 0,
            'unorganized_docs': 0,
            'categories': {},
            'total_size': 0
        }
        
        # Count all markdown files
        all_docs = list(self.docs_dir.rglob("*.md"))
        stats['total_docs'] = len(all_docs)
        
        # Count organized docs in emoji folders
        organized_docs = []
        for category, folder in CATEGORY_FOLDERS.items():
            folder_path = self.docs_dir / folder
            if folder_path.exists():
                folder_docs = list(folder_path.glob("*.md"))
                organized_docs.extend(folder_docs)
                if folder_docs:
                    stats['categories'][folder] = len(folder_docs)
        
        stats['kb_docs'] = len(organized_docs)
        
        # Count unorganized docs (not in emoji folders or archive)
        unorganized = []
        for doc in all_docs:
            is_organized = any(doc.is_relative_to(self.docs_dir / folder) 
                             for folder in CATEGORY_FOLDERS.values() 
                             if (self.docs_dir / folder).exists())
            
            is_archived = (self.archive_dir.exists() and doc.is_relative_to(self.archive_dir))
            
            if not is_organized and not is_archived:
                unorganized.append(doc)
                stats['total_size'] += doc.stat().st_size
        
        stats['unorganized_docs'] = len(unorganized)
        return stats

def cmd_status(args):
    """Show knowledge base status"""
    kb = KnowledgeBase()
    stats = kb.status()
    
    print("🧠 Knowledge Base Status")
    print("=" * 40)
    print(f"📄 Total documents: {stats['total_docs']}")
    print(f"🗂️  Organized: {stats['kb_docs']}")
    print(f"📦 Unorganized: {stats['unorganized_docs']}")
    
    if stats['categories']:
        print(f"\n📊 Categories:")
        for category, count in sorted(stats['categories'].items()):
            print(f"   {category}: {count} docs")
    
    if stats['unorganized_docs'] > 0:
        size_mb = stats['total_size'] / (1024 * 1024)
        print(f"\n💡 Tip: Run 'kb analyze' to process {stats['unorganized_docs']} unorganized docs ({size_mb:.1f}MB)")

def cmd_analyze(args):
    """Analyze and categorize documents using LLM"""
    from migrate_docs import process_files
    import asyncio
    
    # Find unorganized markdown files
    kb = KnowledgeBase()
    all_docs = list(kb.docs_dir.rglob("*.md"))
    unorganized = []
    
    for doc in all_docs:
        if not (doc.is_relative_to(kb.kb_dir) or 
               (kb.archive_dir.exists() and doc.is_relative_to(kb.archive_dir))):
            unorganized.append(doc)
    
    if not unorganized:
        print("✅ All documents are already organized!")
        return
    
    print(f"🔍 Found {len(unorganized)} unorganized documents")
    
    if args.limit:
        unorganized = unorganized[:args.limit]
        print(f"📝 Processing first {len(unorganized)} documents (--limit {args.limit})")
    
    # Process files
    asyncio.run(process_files(unorganized, dry_run=args.dry_run, interactive=not args.yes))

def cmd_categorize(args):
    """Categorize documents using rule-based approach"""
    rules_path = Path(__file__).parent / 'scripts' / 'doc_rules.json'
    categorizer = DocumentCategorizer(str(rules_path))
    
    # Find markdown files to categorize
    source_dir = Path(args.source)
    markdown_files = list(source_dir.glob('*.md'))
    
    if not markdown_files:
        print(f"No markdown files found in {source_dir}")
        return
    
    print(f"🏷️  Categorizing {len(markdown_files)} files using rule-based approach")
    
    categorized = []
    for file_path in markdown_files:
        doc = categorizer.categorize_file(file_path)
        categorized.append(doc)
        
        print(f"📄 {file_path.name}")
        print(f"   Category: {doc.category}")
        print(f"   Confidence: {doc.confidence:.1%}")
        print(f"   Reason: {doc.reason}")
    
    # Show summary
    print(f"\n📊 Summary by category:")
    by_category = {}
    for doc in categorized:
        by_category.setdefault(doc.category, []).append(doc)
    
    for category, docs in sorted(by_category.items()):
        print(f"   {category}: {len(docs)} files")

def cmd_query(args):
    """Query the knowledge base with smart Q&A"""
    if not args.question:
        # Interactive mode
        print("🔍 Knowledge Base Query (Interactive Mode)")
        print("=" * 45)
        print("Ask questions about your neural network battle simulation!")
        print("Type 'exit' or 'quit' to stop, 'help' for tips.")
        print()
        
        while True:
            try:
                question = input("❓ Your question: ").strip()
                if question.lower() in ['exit', 'quit', 'q']:
                    print("👋 Goodbye!")
                    break
                elif question.lower() == 'help':
                    print("\n💡 Tips:")
                    print("• Ask about agents: 'How do agents make decisions?'")
                    print("• Ask about training: 'How does NEAT evolution work?'")
                    print("• Ask about gameplay: 'What are the combat mechanics?'")
                    print("• Ask about development: 'How do I set up the project?'")
                    print()
                    continue
                elif not question:
                    continue
                
                answer = query_knowledge_base(question)
                print(f"\n🤖 Answer:\n{answer}\n")
                
            except KeyboardInterrupt:
                print("\n👋 Goodbye!")
                break
    else:
        # Single question mode
        print(f"🔍 Querying: {args.question}")
        print("=" * 50)
        answer = query_knowledge_base(args.question)
        print(f"\n🤖 Answer:\n{answer}")

def query_knowledge_base(question: str) -> str:
    """Query the knowledge base and generate an answer"""
    import os
    import requests
    from dotenv import load_dotenv
    
    # Load environment variables
    load_dotenv(Path(__file__).parent / '.env')
    
    try:
        # Step 1: Find relevant documents
        relevant_docs = find_relevant_documents(question)
        
        if not relevant_docs:
            return "❌ I couldn't find any relevant documents for your question. Try rephrasing or asking about agents, training, gameplay, or development."
        
        # Step 2: Extract relevant content
        context = build_context_from_docs(relevant_docs, question)
        
        # Step 3: Generate answer using Qwen3
        answer = generate_answer_with_qwen(question, context)
        
        return answer
        
    except Exception as e:
        return f"❌ Error processing your question: {str(e)}"

def find_relevant_documents(question: str) -> List[Path]:
    """Find documents relevant to the question using keyword matching"""
    docs_dir = Path("docs")
    relevant_docs = []
    
    # Normalize question for searching
    question_lower = question.lower()
    
    # Category-based search hints
    category_keywords = {
        "agents": ["agent", "ai", "brain", "decision", "sensor", "perception", "behavior"],
        "training": ["train", "neat", "evolution", "fitness", "learn", "optimize", "tournament"],
        "gameplay": ["game", "combat", "battle", "physics", "simulation", "team", "fight"],
        "development": ["setup", "install", "build", "dev", "contribute", "architecture", "performance"],
        "guides": ["how", "tutorial", "guide", "step", "start", "begin"],
        "reference": ["api", "spec", "documentation", "interface"]
    }
    
    # Find the most relevant category
    best_category = None
    max_matches = 0
    
    for category, keywords in category_keywords.items():
        matches = sum(1 for keyword in keywords if keyword in question_lower)
        if matches > max_matches:
            max_matches = matches
            best_category = category
    
    # Search in the relevant category first
    if best_category:
        category_folder = CATEGORY_FOLDERS.get(best_category, best_category)
        category_path = docs_dir / category_folder
        
        if category_path.exists():
            for doc in category_path.glob("*.md"):
                relevant_docs.append(doc)
    
    # If no category match or need more docs, search broadly
    if len(relevant_docs) < 3:
        for emoji_folder in CATEGORY_FOLDERS.values():
            folder_path = docs_dir / emoji_folder
            if folder_path.exists():
                for doc in folder_path.glob("*.md"):
                    if doc not in relevant_docs:
                        # Quick relevance check
                        try:
                            content = doc.read_text(encoding='utf-8', errors='ignore').lower()
                            if any(word in content for word in question_lower.split() if len(word) > 2):
                                relevant_docs.append(doc)
                        except:
                            continue
    
    return relevant_docs[:5]  # Limit to top 5 docs

def build_context_from_docs(docs: List[Path], question: str) -> str:
    """Build context string from relevant documents"""
    context_parts = []
    
    for doc in docs:
        try:
            content = doc.read_text(encoding='utf-8', errors='ignore')
            
            # Extract title
            title = doc.stem.replace('_', ' ').title()
            first_line = content.split('\n')[0]
            if first_line.startswith('#'):
                title = first_line.strip('# ')
            
            # Get relevant excerpt (first 1000 chars for now)
            excerpt = content[:1000].strip()
            if len(content) > 1000:
                excerpt += "..."
            
            context_parts.append(f"## {title}\n{excerpt}")
            
        except Exception as e:
            continue
    
    return "\n\n".join(context_parts)

def generate_answer_with_qwen(question: str, context: str) -> str:
    """Generate answer using Qwen3 via OpenRouter"""
    import os
    import requests
    
    api_key = os.getenv("OPENROUTER_API_KEY")
    if not api_key:
        return "❌ OpenRouter API key not found. Please set OPENROUTER_API_KEY in your .env file."
    
    prompt = f"""You are a knowledgeable assistant for a neural network battle simulation project. Answer the user's question based on the provided documentation context.

Context from documentation:
{context}

User question: {question}

Instructions:
- Provide a clear, helpful answer based on the documentation
- If the documentation doesn't contain enough information, say so
- Use technical terms appropriately but explain them when needed
- Be concise but thorough
- Include relevant details from the documentation

Answer:"""

    try:
        response = requests.post(
            "https://openrouter.ai/api/v1/chat/completions",
            headers={
                "Authorization": f"Bearer {api_key}",
                "Content-Type": "application/json"
            },
            json={
                "model": "qwen/qwen-2.5-72b-instruct",  # Updated model
                "messages": [
                    {"role": "user", "content": prompt}
                ],
                "temperature": 0.3,
                "max_tokens": 800
            },
            timeout=30
        )
        
        response.raise_for_status()
        result = response.json()
        
        answer = result['choices'][0]['message']['content'].strip()
        
        # Add source information
        doc_names = [doc.name for doc in find_relevant_documents(question)]
        if doc_names:
            answer += f"\n\n📚 Sources: {', '.join(doc_names[:3])}"
        
        return answer
        
    except requests.exceptions.RequestException as e:
        return f"❌ API request failed: {str(e)}"
    except Exception as e:
        return f"❌ Error generating answer: {str(e)}"

def cmd_benchmark(args):
    """Benchmark knowledge base performance"""
    print("⚡ Knowledge Base Benchmark")
    print("=" * 35)
    print("This feature is coming soon!")
    print()
    print("Planned benchmarks:")
    print("• Categorization accuracy")
    print("• Summary quality assessment") 
    print("• Query response relevance")
    print("• Processing speed metrics")

def cmd_migrate(args):
    """Migrate files to the new human-readable structure"""
    import shutil
    
    # First, categorize files
    rules_path = Path(__file__).parent / 'scripts' / 'doc_rules.json'
    categorizer = DocumentCategorizer(str(rules_path))
    
    # Find markdown files to migrate
    source_dir = Path("docs")
    markdown_files = [f for f in source_dir.glob('*.md') if not f.name.startswith('.')]
    
    if not markdown_files:
        print("No markdown files found to migrate")
        return
    
    print(f"🚚 Migrating {len(markdown_files)} files to new structure")
    print()
    
    moved_count = 0
    
    for file_path in markdown_files:
        doc = categorizer.categorize_file(file_path)
        
        # Get target folder with emoji
        target_folder = get_target_folder(doc.category)
        target_dir = source_dir / target_folder
        target_path = target_dir / file_path.name
        
        # Skip if already in target location
        if file_path.parent == target_dir:
            continue
            
        print(f"📄 {file_path.name}")
        print(f"   Moving to: {target_folder}/")
        print(f"   Reason: {doc.reason}")
        print(f"   Confidence: {doc.confidence:.1%}")
        
        if not args.dry_run:
            # Create target directory if it doesn't exist
            target_dir.mkdir(parents=True, exist_ok=True)
            
            # Move the file
            shutil.move(str(file_path), str(target_path))
            moved_count += 1
            print(f"   ✅ Moved!")
        else:
            print(f"   🔍 Would move (dry-run)")
        print()
    
    if args.dry_run:
        print(f"🔍 Dry run complete. Would move {len(markdown_files)} files")
    else:
        print(f"✅ Migration complete! Moved {moved_count} files to new structure")
        print("\n📁 New structure:")
        for category, folder in CATEGORY_FOLDERS.items():
            folder_path = source_dir / folder
            if folder_path.exists():
                file_count = len(list(folder_path.glob("*.md")))
                if file_count > 0:
                    print(f"   {folder}: {file_count} files")

def cmd_discover(args):
    """Discover all markdown files across the repository"""
    import subprocess
    
    print("🔍 Discovering markdown files across the repository...")
    print("=" * 55)
    
    # Find all markdown files, excluding common non-documentation directories
    try:
        result = subprocess.run([
            'find', '.', '-name', '*.md', '-type', 'f'
        ], capture_output=True, text=True, check=True)
        
        all_files = result.stdout.strip().split('\n')
        
        # Filter out unwanted directories
        excluded_patterns = [
            'node_modules', '.venv', 'venv', '.git', 
            '__pycache__', '.cargo/registry', 'target/debug', 'target/release'
        ]
        
        filtered_files = []
        for file_path in all_files:
            if file_path and not any(pattern in file_path for pattern in excluded_patterns):
                filtered_files.append(Path(file_path))
        
        print(f"📊 Found {len(filtered_files)} markdown files total")
        
        # Categorize by location
        categories = {
            'organized': [],      # Already in our emoji folders
            'docs_root': [],      # In docs/ but not organized
            'sim_core': [],       # In sim_core/
            'root': [],          # In project root
            'other': []          # Elsewhere
        }
        
        for file_path in filtered_files:
            path_str = str(file_path)
            if any(emoji in path_str for emoji in ['🤖', '🧠', '🎮', '🔧', '📖', '📚']):
                categories['organized'].append(file_path)
            elif path_str.startswith('./docs/') and not path_str.startswith('./docs/knowledge/'):
                categories['docs_root'].append(file_path)
            elif path_str.startswith('./sim_core/'):
                categories['sim_core'].append(file_path)
            elif path_str.count('/') == 1:  # Root level files
                categories['root'].append(file_path)
            else:
                categories['other'].append(file_path)
        
        # Display results
        print(f"\n📁 File Distribution:")
        print(f"   ✅ Already organized: {len(categories['organized'])} files")
        print(f"   📄 docs/ (unorganized): {len(categories['docs_root'])} files")
        print(f"   🦀 sim_core/: {len(categories['sim_core'])} files")
        print(f"   📋 root level: {len(categories['root'])} files")
        print(f"   📂 other locations: {len(categories['other'])} files")
        
        # Show unorganized files by location
        if args.show_files:
            for category, files in categories.items():
                if category != 'organized' and files:
                    print(f"\n📂 {category.replace('_', ' ').title()} files:")
                    for file_path in sorted(files)[:10]:  # Show first 10
                        print(f"   • {file_path}")
                    if len(files) > 10:
                        print(f"   ... and {len(files) - 10} more")
        
        # Show actionable suggestions
        unorganized_count = sum(len(files) for cat, files in categories.items() if cat != 'organized')
        if unorganized_count > 0:
            print(f"\n💡 Next steps:")
            print(f"   • Run 'kb harvest' to analyze and organize {unorganized_count} unorganized files")
            print(f"   • Use 'kb harvest --dry-run' to preview the organization")
            print(f"   • Add '--show-files' to see detailed file lists")
        
        return categories
        
    except subprocess.CalledProcessError as e:
        print(f"❌ Error finding files: {e}")
        return None

def cmd_harvest(args):
    """Harvest and organize markdown files from across the repository"""
    import shutil
    
    print("🌾 Harvesting documentation from across the repository...")
    print("=" * 60)
    
    # Find all markdown files manually to exclude archives
    import subprocess
    try:
        result = subprocess.run([
            'find', '.', '-name', '*.md', '-type', 'f'
        ], capture_output=True, text=True, check=True)
        
        all_files = result.stdout.strip().split('\n')
        
        # Filter out unwanted directories and files
        excluded_patterns = [
            'node_modules', '.venv', 'venv', '.git', 
            '__pycache__', '.cargo/registry', 'target/debug', 'target/release',
            'docs/archive'  # Exclude archive by default
        ]
        
        if not args.include_archive:
            excluded_patterns.append('docs/archive')
        
        # Also exclude already organized files
        emoji_patterns = ['🤖', '🧠', '🎮', '🔧', '📖', '📚']
        
        harvestable = []
        for file_path_str in all_files:
            if not file_path_str:
                continue
                
            # Skip excluded patterns
            if any(pattern in file_path_str for pattern in excluded_patterns):
                continue
                
            # Skip already organized files
            if any(emoji in file_path_str for emoji in emoji_patterns):
                continue
                
            # Skip certain metadata files
            filename = Path(file_path_str).name.lower()
            if any(skip in filename for skip in ['readme', 'license', 'privacy', 'changelog']):
                if not args.include_meta:
                    continue
            
            harvestable.append(Path(file_path_str))
        
    except subprocess.CalledProcessError as e:
        print(f"❌ Error finding files: {e}")
        return
    
    if not harvestable:
        print("✅ All relevant files are already organized!")
        return
    
    print(f"🎯 Found {len(harvestable)} files to analyze for organization...")
    
    # Categorize files using the improved system
    organized_count = 0
    skipped_count = 0
    
    for file_path in harvestable:
        try:
            # Skip very small files (likely not documentation)
            if file_path.stat().st_size < 50:
                skipped_count += 1
                continue
                
            # Categorize the file using the improved analysis
            doc = analyze_file(file_path)
            
            # Get target location
            target_folder = get_target_folder(doc.category)
            target_dir = Path("docs") / target_folder
            target_path = target_dir / file_path.name
            
            # Handle name conflicts
            counter = 1
            original_target = target_path
            while target_path.exists():
                stem = original_target.stem
                suffix = original_target.suffix
                target_path = original_target.parent / f"{stem}_{counter}{suffix}"
                counter += 1
            
            print(f"📄 {file_path}")
            print(f"   → {target_folder}/")
            print(f"   Category: {doc.category} (confidence: {doc.confidence:.1%})")
            if doc.confidence < 0.3:
                print(f"   ⚠️  Low confidence - please review")
            
            if args.dry_run:
                print("   🔍 Would copy (dry-run)")
            else:
                # Create target directory
                target_dir.mkdir(parents=True, exist_ok=True)
                
                # Copy (don't move) to preserve original structure
                shutil.copy2(str(file_path), str(target_path))
                organized_count += 1
                print("   ✅ Copied to knowledge base!")
            
            print()
            
        except Exception as e:
            print(f"❌ Error processing {file_path}: {e}")
            skipped_count += 1
            continue
    
    # Summary
    if args.dry_run:
        print(f"🔍 Dry run complete:")
        print(f"   • Would organize: {len(harvestable) - skipped_count} files")
        print(f"   • Would skip: {skipped_count} files")
    else:
        print(f"✅ Harvest complete!")
        print(f"   • Organized: {organized_count} files")
        print(f"   • Skipped: {skipped_count} files")
        print(f"\n📚 Run 'kb status' to see the updated knowledge base")

def cmd_cleanup(args):
    """Clean up duplicate and redundant files in the knowledge base"""
    import hashlib
    from collections import defaultdict
    
    print("🧹 Cleaning up knowledge base duplicates and redundancy...")
    print("=" * 55)
    
    # Find all files in organized folders
    kb_files = []
    for category, folder in CATEGORY_FOLDERS.items():
        folder_path = Path("docs") / folder
        if folder_path.exists():
            for file_path in folder_path.glob("*.md"):
                kb_files.append(file_path)
    
    print(f"📊 Found {len(kb_files)} files in knowledge base")
    
    # Group files by content hash to find duplicates
    file_hashes = defaultdict(list)
    file_sizes = defaultdict(list)
    suspicious_names = []
    
    for file_path in kb_files:
        try:
            content = file_path.read_text(encoding='utf-8', errors='ignore')
            content_hash = hashlib.md5(content.encode()).hexdigest()
            file_hashes[content_hash].append(file_path)
            
            size = file_path.stat().st_size
            file_sizes[size].append(file_path)
            
            # Flag suspicious file names
            name = file_path.name.lower()
            if any(pattern in name for pattern in ['untitled', 'copy', 'duplicate', '(1)', '_1', '_2']):
                # Exclude index_* files - they have their own rename command
                if not name.startswith('index_'):
                    suspicious_names.append(file_path)
                
        except Exception as e:
            print(f"⚠️ Error reading {file_path}: {e}")
    
    # Report findings
    duplicates = {h: files for h, files in file_hashes.items() if len(files) > 1}
    size_duplicates = {s: files for s, files in file_sizes.items() if len(files) > 1}
    
    print(f"\n🔍 Analysis Results:")
    print(f"   📄 Total files: {len(kb_files)}")
    print(f"   🔗 Exact duplicates: {len(duplicates)} groups ({sum(len(files)-1 for files in duplicates.values())} redundant files)")
    print(f"   📏 Same-size files: {len(size_duplicates)} groups")
    print(f"   🚨 Suspicious names: {len(suspicious_names)} files")
    
    # Show duplicates
    if duplicates:
        print(f"\n🔗 Exact Content Duplicates:")
        for content_hash, files in list(duplicates.items())[:5]:  # Show first 5
            print(f"   Hash: {content_hash[:8]}...")
            for i, file_path in enumerate(files):
                marker = "🗑️" if i > 0 else "✅"
                print(f"     {marker} {file_path}")
        if len(duplicates) > 5:
            print(f"   ... and {len(duplicates) - 5} more duplicate groups")
    
    # Show suspicious names
    if suspicious_names:
        print(f"\n🚨 Suspicious File Names (likely auto-generated):")
        for file_path in suspicious_names[:10]:
            print(f"   • {file_path}")
        if len(suspicious_names) > 10:
            print(f"   ... and {len(suspicious_names) - 10} more")
    
    # Show actions
    total_removable = sum(len(files)-1 for files in duplicates.values()) + len(suspicious_names)
    if total_removable > 0:
        print(f"\n💡 Recommended actions:")
        print(f"   • Run 'kb cleanup --remove-duplicates' to remove {sum(len(files)-1 for files in duplicates.values())} exact duplicates")
        print(f"   • Run 'kb cleanup --remove-suspicious' to remove {len(suspicious_names)} suspicious files")
        print(f"   • Run 'kb cleanup --remove-all' to remove both ({total_removable} files total)")
    
    # Execute cleanup if requested
    removed_count = 0
    if args.remove_duplicates or args.remove_all:
        print(f"\n🗑️ Removing exact duplicates...")
        for content_hash, files in duplicates.items():
            # Keep the first file, remove the rest
            for file_path in files[1:]:
                if args.dry_run:
                    print(f"   🔍 Would remove: {file_path}")
                else:
                    file_path.unlink()
                    print(f"   ✅ Removed: {file_path}")
                    removed_count += 1
    
    if args.remove_suspicious or args.remove_all:
        print(f"\n🗑️ Removing suspicious files...")
        for file_path in suspicious_names:
            if file_path not in [f for files in duplicates.values() for f in files[1:]]:  # Don't double-remove
                if args.dry_run:
                    print(f"   🔍 Would remove: {file_path}")
                else:
                    file_path.unlink()
                    print(f"   ✅ Removed: {file_path}")
                    removed_count += 1
    
    if removed_count > 0:
        print(f"\n✅ Cleanup complete! Removed {removed_count} files")
        print(f"📚 Run 'kb status' to see the cleaned knowledge base")
    elif args.dry_run and total_removable > 0:
        print(f"\n🔍 Dry run complete. Would remove {total_removable} files")

def cmd_archive_status(args):
    """Show status of archived documents"""
    archive_dir = Path("docs/archive")
    
    if not archive_dir.exists():
        print("📂 No archive directory found")
        return
    
    archive_files = list(archive_dir.rglob("*.md"))
    print(f"📦 Archive Status")
    print("=" * 30)
    print(f"📄 Archive files: {len(archive_files)}")
    
    if args.show_files:
        print(f"\n📂 Archive contents (first 20):")
        for file_path in sorted(archive_files)[:20]:
            size_kb = file_path.stat().st_size / 1024
            print(f"   • {file_path.name} ({size_kb:.1f}KB)")
        if len(archive_files) > 20:
            print(f"   ... and {len(archive_files) - 20} more files")
    
    print(f"\n💡 To include archive in harvest: kb harvest --include-archive")

def cmd_rename_index_files(args):
    """Intelligently rename index_* files to proper names"""
    print("🔧 Renaming index_* files to proper names...")
    print("=" * 45)
    
    # Find all index_* files in organized folders
    index_files = []
    for category, folder in CATEGORY_FOLDERS.items():
        folder_path = Path("docs") / folder
        if folder_path.exists():
            for file_path in folder_path.glob("index_*.md"):
                index_files.append(file_path)
    
    print(f"📊 Found {len(index_files)} index_* files to rename")
    
    renamed_count = 0
    for file_path in index_files:
        try:
            # Read content to extract title
            content = file_path.read_text(encoding='utf-8', errors='ignore')
            
            # Try to extract a good name from the content
            new_name = None
            
            # Method 1: Look for # Title at the start
            title_match = re.search(r'^#\s+(.+?)$', content, re.MULTILINE)
            if title_match:
                title = title_match.group(1).strip()
                # Clean up the title for filename
                new_name = re.sub(r'[^\w\s-]', '', title).strip()
                new_name = re.sub(r'\s+', '_', new_name)
                new_name = new_name[:50]  # Limit length
            
            # Method 2: If no title, try to guess from content keywords
            if not new_name:
                content_lower = content.lower()
                if 'neat' in content_lower and 'training' in content_lower:
                    new_name = "neat_training_guide"
                elif 'agent' in content_lower and 'sensor' in content_lower:
                    new_name = "agent_sensor_guide"
                elif 'gameplay' in content_lower and 'simulation' in content_lower:
                    new_name = "gameplay_simulation_guide"
                elif 'development' in content_lower and 'setup' in content_lower:
                    new_name = "development_setup_guide"
                else:
                    # Use first few words of content
                    words = content.split()[:5]
                    new_name = "_".join(re.sub(r'[^\w]', '', word) for word in words if word.isalpha())
                    new_name = new_name[:30]
            
            # Fallback: keep as index_X but make it clear it needs manual review
            if not new_name or len(new_name) < 3:
                new_name = f"document_{file_path.stem}"
            
            # Ensure .md extension
            if not new_name.endswith('.md'):
                new_name += '.md'
            
            # Create new path
            new_path = file_path.parent / new_name
            
            # Handle conflicts
            counter = 1
            original_new_path = new_path
            while new_path.exists() and new_path != file_path:
                stem = original_new_path.stem
                suffix = original_new_path.suffix
                new_path = original_new_path.parent / f"{stem}_{counter}{suffix}"
                counter += 1
            
            print(f"📄 {file_path.name}")
            print(f"   → {new_path.name}")
            
            # Show preview of content for verification
            preview = content[:100].replace('\n', ' ').strip()
            if len(content) > 100:
                preview += "..."
            print(f"   Preview: {preview}")
            
            if args.dry_run:
                print("   🔍 Would rename (dry-run)")
            else:
                file_path.rename(new_path)
                print("   ✅ Renamed!")
                renamed_count += 1
            
            print()
            
        except Exception as e:
            print(f"❌ Error processing {file_path}: {e}")
    
    if args.dry_run:
        print(f"🔍 Dry run complete. Would rename {len(index_files)} files")
    else:
        print(f"✅ Rename complete! Renamed {renamed_count} files")
        print(f"📚 Run 'kb status' to see the updated knowledge base")

def cmd_cleanup_old_structure(args):
    """Clean up the old knowledge/documents structure after successful harvest"""
    import shutil
    
    print("🧹 Cleaning up old knowledge base structure...")
    print("=" * 50)
    
    # Files and directories to clean up
    cleanup_targets = [
        "knowledge/documents",  # Old nested structure
        "inbox.md",            # Root level files already harvested
        "RAG_LEARNINGS.md",
        "docs_structure_proposal.md"
    ]
    
    removed_count = 0
    for target in cleanup_targets:
        target_path = Path(target)
        
        if target_path.exists():
            if target_path.is_dir():
                # Count files in directory
                file_count = len(list(target_path.rglob("*")))
                print(f"📂 {target}: {file_count} items")
                
                if args.dry_run:
                    print(f"   🔍 Would remove directory (dry-run)")
                else:
                    shutil.rmtree(target_path)
                    print(f"   ✅ Removed directory!")
                    removed_count += file_count
            else:
                # Single file
                print(f"📄 {target}")
                
                if args.dry_run:
                    print(f"   🔍 Would remove file (dry-run)")
                else:
                    target_path.unlink()
                    print(f"   ✅ Removed file!")
                    removed_count += 1
        else:
            print(f"⚠️  {target}: Not found (already cleaned?)")
    
    if args.dry_run:
        print(f"\n🔍 Dry run complete. Would remove old structure")
    else:
        print(f"\n✅ Cleanup complete! Removed {removed_count} items")
        print(f"📚 Old knowledge structure has been cleaned up")
        print(f"💡 Run 'kb status' to see the clean knowledge base")

def main():
    parser = argparse.ArgumentParser(
        description="🧠 Knowledge Base CLI - Unified interface for document management",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Examples:
  kb status                    # Show knowledge base overview
  kb analyze                   # Analyze unorganized docs with LLM  
  kb analyze --dry-run         # Preview what would be analyzed
  kb analyze --limit 5         # Process only first 5 docs
  kb categorize docs/          # Rule-based categorization
  kb query "NEAT training"     # Search knowledge base (coming soon)
  kb benchmark                 # Performance testing (coming soon)
        """
    )
    
    subparsers = parser.add_subparsers(dest='command', help='Available commands')
    
    # Status command
    status_parser = subparsers.add_parser('status', help='Show knowledge base status')
    
    # Analyze command (LLM-powered)
    analyze_parser = subparsers.add_parser('analyze', help='Analyze documents with LLM')
    analyze_parser.add_argument('--dry-run', action='store_true', 
                               help='Show what would be done without making changes')
    analyze_parser.add_argument('--yes', action='store_true',
                               help='Skip interactive prompts')
    analyze_parser.add_argument('--limit', type=int,
                               help='Limit number of documents to process')
    
    # Categorize command (rule-based)
    categorize_parser = subparsers.add_parser('categorize', help='Rule-based categorization')
    categorize_parser.add_argument('source', nargs='?', default='docs',
                                  help='Source directory (default: docs)')
    categorize_parser.add_argument('--dry-run', action='store_true',
                                  help='Show categorization without moving files')
    
    # Query command (future)
    query_parser = subparsers.add_parser('query', help='Query the knowledge base')
    query_parser.add_argument('question', nargs='?',
                             help='Question to ask the knowledge base')
    
    # Benchmark command (future)
    benchmark_parser = subparsers.add_parser('benchmark', help='Benchmark performance')
    
    # Migrate command
    migrate_parser = subparsers.add_parser('migrate', help='Migrate files to the new structure')
    migrate_parser.add_argument('--dry-run', action='store_true',
                               help='Show what would be done without making changes')
    
    # Discover command
    discover_parser = subparsers.add_parser('discover', help='Discover markdown files across the repository')
    discover_parser.add_argument('--show-files', action='store_true',
                                 help='Show unorganized files by location')
    
    # Harvest command
    harvest_parser = subparsers.add_parser('harvest', help='Harvest and organize markdown files from across the repository')
    harvest_parser.add_argument('--dry-run', action='store_true',
                                help='Show what would be done without making changes')
    harvest_parser.add_argument('--include-meta', action='store_true',
                                help='Include metadata in harvested files')
    harvest_parser.add_argument('--include-archive', action='store_true',
                                help='Include archive files in harvested files')
    
    # Cleanup command
    cleanup_parser = subparsers.add_parser('cleanup', help='Clean up duplicate and redundant files in the knowledge base')
    cleanup_parser.add_argument('--dry-run', action='store_true',
                                help='Show what would be done without making changes')
    cleanup_parser.add_argument('--remove-duplicates', action='store_true',
                                help='Remove exact duplicates')
    cleanup_parser.add_argument('--remove-suspicious', action='store_true',
                                help='Remove suspicious files')
    cleanup_parser.add_argument('--remove-all', action='store_true',
                                help='Remove both exact duplicates and suspicious files')
    
    # Archive status command
    archive_status_parser = subparsers.add_parser('archive-status', help='Show status of archived documents')
    archive_status_parser.add_argument('--show-files', action='store_true',
                                        help='Show archived files')
    
    # Rename index files command
    rename_index_files_parser = subparsers.add_parser('rename-index-files', help='Rename index_* files to proper names')
    rename_index_files_parser.add_argument('--dry-run', action='store_true',
                                           help='Show what would be done without making changes')
    
    # Cleanup old structure command
    cleanup_old_structure_parser = subparsers.add_parser('cleanup-old-structure', help='Clean up the old knowledge/documents structure after successful harvest')
    cleanup_old_structure_parser.add_argument('--dry-run', action='store_true',
                                               help='Show what would be done without making changes')
    
    args = parser.parse_args()
    
    if args.command == 'status':
        cmd_status(args)
    elif args.command == 'analyze':
        cmd_analyze(args)
    elif args.command == 'categorize':
        cmd_categorize(args)
    elif args.command == 'query':
        cmd_query(args)
    elif args.command == 'benchmark':
        cmd_benchmark(args)
    elif args.command == 'migrate':
        cmd_migrate(args)
    elif args.command == 'discover':
        cmd_discover(args)
    elif args.command == 'harvest':
        cmd_harvest(args)
    elif args.command == 'cleanup':
        cmd_cleanup(args)
    elif args.command == 'archive-status':
        cmd_archive_status(args)
    elif args.command == 'rename-index-files':
        cmd_rename_index_files(args)
    elif args.command == 'cleanup-old-structure':
        cmd_cleanup_old_structure(args)
    else:
        parser.print_help()

if __name__ == "__main__":
    main() 