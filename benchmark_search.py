#!/usr/bin/env python3
"""
🎲 RAG vs Naive Search Benchmark
Let's see if all that complexity was worth it!

RESULT: Naive search won 5/5 queries, 42x faster than RAG.
RAG implementation removed after this benchmark proved simple > complex.
See RAG_LEARNINGS.md for full analysis.
"""

import time
import glob
import re
from pathlib import Path
from typing import List, Dict, Any
from dataclasses import dataclass

@dataclass
class SearchResult:
    method: str
    query: str
    results: List[str]
    time_taken: float
    result_count: int

class NaiveSearcher:
    """Dead simple text search - the baseline"""
    
    def __init__(self, docs_dir: str = "docs"):
        self.docs_dir = Path(docs_dir)
        print(f"📁 Naive searcher initialized for {docs_dir}")
    
    def search(self, query: str, top_k: int = 5) -> List[str]:
        """Simple case-insensitive text search"""
        start_time = time.time()
        results = []
        
        # Find all markdown files
        for file_path in self.docs_dir.rglob("*.md"):
            try:
                content = file_path.read_text(encoding='utf-8', errors='ignore')
                
                # Check if query terms are in the content
                if self._matches_query(query.lower(), content.lower()):
                    # Extract relevant snippet
                    snippet = self._extract_snippet(query.lower(), content)
                    results.append({
                        'file': str(file_path.relative_to(self.docs_dir)),
                        'snippet': snippet,
                        'score': self._calculate_score(query.lower(), content.lower())
                    })
            except Exception as e:
                continue
        
        # Sort by score and return top results
        results.sort(key=lambda x: x['score'], reverse=True)
        
        return [f"{r['file']}: {r['snippet']}" for r in results[:top_k]]
    
    def _matches_query(self, query: str, content: str) -> bool:
        """Check if content matches query (all terms must be present)"""
        query_terms = query.split()
        return all(term in content for term in query_terms)
    
    def _extract_snippet(self, query: str, content: str, snippet_length: int = 100) -> str:
        """Extract a snippet around the first match"""
        query_terms = query.split()
        for term in query_terms:
            match = re.search(re.escape(term), content, re.IGNORECASE)
            if match:
                start = max(0, match.start() - snippet_length // 2)
                end = min(len(content), match.end() + snippet_length // 2)
                snippet = content[start:end].strip()
                return f"...{snippet}..." if start > 0 or end < len(content) else snippet
        return content[:snippet_length] + "..."
    
    def _calculate_score(self, query: str, content: str) -> float:
        """Simple scoring based on term frequency"""
        query_terms = query.split()
        score = 0
        for term in query_terms:
            score += content.count(term)
        return score

class RAGSearcher:
    """Our fancy RAG system (REMOVED - naive search won the benchmark)"""
    
    def __init__(self):
        print("🤖 RAG system removed after losing benchmark...")
        self.rag = None
    
    def search(self, query: str, top_k: int = 5) -> List[str]:
        """RAG implementation removed - naive search was 42x faster"""
        return ["RAG system removed after benchmark defeat"]

def run_benchmark():
    """🎲 Roll the dice! Let's see who wins!"""
    
    print("🎲🎲 SEARCH SYSTEM SHOWDOWN 🎲🎲")
    print("=" * 50)
    
    # Test queries - mix of exact and semantic
    test_queries = [
        "NEAT training",                    # Exact match - should favor naive
        "improve agent performance",        # Semantic - might favor RAG  
        "team configuration",              # Exact match
        "neural network evolution",        # Semantic
        "browser simulation setup",        # Exact match
    ]
    
    # Initialize both systems
    print("\n🏁 Starting engines...")
    naive = NaiveSearcher()
    rag = RAGSearcher()
    
    results = []
    
    print(f"\n🔍 Testing {len(test_queries)} queries...\n")
    
    for i, query in enumerate(test_queries, 1):
        print(f"Query {i}: '{query}'")
        print("-" * 40)
        
        # Test Naive approach
        print("🔧 Naive search:")
        start_time = time.time()
        naive_results = naive.search(query, top_k=3)
        naive_time = time.time() - start_time
        
        print(f"  ⏱️  Time: {naive_time:.3f}s")
        print(f"  📊 Results: {len(naive_results)}")
        for j, result in enumerate(naive_results[:2], 1):  # Show top 2
            print(f"    {j}. {result[:80]}...")
        
        # Test RAG approach  
        print("🤖 RAG search:")
        start_time = time.time()
        rag_results = rag.search(query, top_k=3)
        rag_time = time.time() - start_time
        
        print(f"  ⏱️  Time: {rag_time:.3f}s")
        print(f"  📊 Results: {len(rag_results)}")
        for j, result in enumerate(rag_results[:2], 1):  # Show top 2
            print(f"    {j}. {result[:80]}...")
        
        # Compare speeds
        if naive_time < rag_time:
            speed_winner = f"🏃 Naive wins ({naive_time:.3f}s vs {rag_time:.3f}s)"
        else:
            speed_winner = f"🚀 RAG wins ({rag_time:.3f}s vs {naive_time:.3f}s)"
        
        print(f"  {speed_winner}")
        print()
        
        # Store results
        results.append({
            'query': query,
            'naive_time': naive_time,
            'rag_time': rag_time,
            'naive_count': len(naive_results),
            'rag_count': len(rag_results)
        })
    
    # Summary
    print("🏆 FINAL SCORECARD")
    print("=" * 50)
    
    avg_naive_time = sum(r['naive_time'] for r in results) / len(results)
    avg_rag_time = sum(r['rag_time'] for r in results) / len(results)
    
    speed_wins_naive = sum(1 for r in results if r['naive_time'] < r['rag_time'])
    speed_wins_rag = len(results) - speed_wins_naive
    
    print(f"⏱️  Average Speed:")
    print(f"   Naive: {avg_naive_time:.3f}s")
    print(f"   RAG:   {avg_rag_time:.3f}s")
    print(f"   Speed Winner: {'Naive' if avg_naive_time < avg_rag_time else 'RAG'}")
    
    print(f"\n🏁 Speed Wins:")
    print(f"   Naive: {speed_wins_naive}/{len(results)}")
    print(f"   RAG:   {speed_wins_rag}/{len(results)}")
    
    total_naive_results = sum(r['naive_count'] for r in results)
    total_rag_results = sum(r['rag_count'] for r in results)
    
    print(f"\n📊 Total Results Found:")
    print(f"   Naive: {total_naive_results}")
    print(f"   RAG:   {total_rag_results}")
    
    # The verdict
    print(f"\n🎯 THE VERDICT:")
    if avg_naive_time < avg_rag_time and total_naive_results >= total_rag_results:
        print("   🏆 NAIVE WINS! Keep it simple, stupid.")
    elif avg_rag_time < avg_naive_time and total_rag_results > total_naive_results:
        print("   🤖 RAG WINS! The complexity pays off!")
    else:
        print("   🤝 IT'S COMPLICATED! Both have their strengths.")
    
    return results

if __name__ == "__main__":
    try:
        results = run_benchmark()
        print("\n🎲 Dice rolled! Check the results above ☝️")
    except KeyboardInterrupt:
        print("\n\n🛑 Benchmark interrupted!")
    except Exception as e:
        print(f"\n💥 Benchmark crashed: {e}")
        import traceback
        traceback.print_exc() 