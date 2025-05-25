---
title: Phase 2 - Search and Organization
description: Implementing search functionality and document organization
difficulty: Intermediate
time_required: 3-4 hours
prerequisites:
  - Completion of Phase 1
  - Basic understanding of search algorithms
  - Familiarity with Python data structures
---

# Phase 2: Search and Document Organization

## Overview
In this phase, we'll enhance our knowledge base with search capabilities and better document organization through categories and tags.

## Step 1: Update Dependencies

Add these to your existing requirements:
```bash
pip install fastapi uvicorn whoosh python-multipart
```

## Step 2: Implement Search Index

Create `search.py`:

```python
from pathlib import Path
from whoosh import index
from whoosh.fields import Schema, TEXT, KEYWORD, ID, DATETIME, STORED
from whoosh.analysis import StemmingAnalyzer
from whoosh.qparser import QueryParser, MultifieldParser
import os

class SearchEngine:
    def __init__(self, index_dir: str = "knowledge/search_index"):
        self.index_dir = Path(index_dir)
        self.schema = Schema(
            id=ID(stored=True, unique=True),
            title=TEXT(stored=True, analyzer=StemmingAnalyzer()),
            content=TEXT(analyzer=StemmingAnalyzer()),
            categories=KEYWORD(stored=True, commas=True, lowercase=True),
            tags=KEYWORD(stored=True, commas=True, lowercase=True),
            description=TEXT(stored=True),
            created_at=DATETIME(stored=True, sortable=True),
            path=STORED
        )
        self._ensure_index()
    
    def _ensure_index(self):
        """Create index directory if it doesn't exist."""
        if not self.index_dir.exists():
            self.index_dir.mkdir(parents=True)
            self.ix = index.create_in(str(self.index_dir), self.schema)
        else:
            self.ix = index.open_dir(str(self.index_dir))
    
    def index_document(self, doc_id: str, metadata: dict, content: str):
        """Add or update a document in the search index."""
        writer = self.ix.writer()
        
        writer.update_document(
            id=doc_id,
            title=metadata.get('title', ''),
            content=content,
            categories=','.join(metadata.get('categories', [])),
            tags=','.join(metadata.get('tags', [])),
            description=metadata.get('description', ''),
            created_at=metadata.get('created_at'),
            path=str(Path(metadata.get('path', '')))
        )
        writer.commit()
    
    def search(self, query_str: str, limit: int = 10, **filters):
        """Search documents with optional filters."""
        with self.ix.searcher() as searcher:
            # Build query
            query_parser = MultifieldParser(
                ["title", "content", "description"], 
                schema=self.ix.schema
            )
            
            # Parse query string
            query = query_parser.parse(query_str)
            
            # Apply filters
            filtered = query
            for field, value in filters.items():
                if value:
                    filtered = filtered & QueryParser(field, self.ix.schema).parse(str(value))
            
            # Execute search
            results = searcher.search(filtered, limit=limit)
            return [dict(r) for r in results]
    
    def delete_document(self, doc_id: str):
        """Remove a document from the search index."""
        writer = self.ix.writer()
        writer.delete_by_term('id', doc_id)
        writer.commit()
```

## Step 3: Update Document Storage

Modify `storage.py` to integrate search:

```python
class DocumentStorage:
    def __init__(self, base_path: str = "knowledge"):
        self.base_path = Path(base_path)
        self.documents_path = self.base_path / "documents"
        self.search_engine = SearchEngine()
        self._ensure_directories()
    
    def save_document(self, doc: Document) -> str:
        # ... existing save logic ...
        
        # Update search index
        self.search_engine.index_document(
            doc_id=doc.metadata.id,
            metadata=doc.metadata.dict(),
            content=doc.content
        )
        
        return doc.metadata.id
    
    def search_documents(self, query: str = "*", limit: int = 10, **filters) -> List[dict]:
        """Search documents with optional filters."""
        return self.search_engine.search(query, limit=limit, **filters)
    
    def delete_document(self, doc_id: str) -> bool:
        """Delete a document and its search index."""
        try:
            doc_dir = self.documents_path / doc_id
            if doc_dir.exists():
                import shutil
                shutil.rmtree(doc_dir)
                self.search_engine.delete_document(doc_id)
                return True
            return False
        except Exception as e:
            raise StorageError(f"Failed to delete document {doc_id}: {str(e)}")
```

## Step 4: Add Web API (Optional)

Create `api.py`:

```python
from fastapi import FastAPI, HTTPException, Query
from fastapi.middleware.cors import CORSMiddleware
from typing import List, Optional
import uvicorn

from knowledge_base.storage import DocumentStorage, Document, DocumentMetadata
from knowledge_base.models import Document as DocModel

app = FastAPI(title="Knowledge Base API")
storage = DocumentStorage()

# CORS middleware
app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)

@app.post("/documents/")
async def create_document(document: DocModel):
    """Create a new document."""
    try:
        doc_id = storage.save_document(document)
        return {"id": doc_id, "status": "created"}
    except Exception as e:
        raise HTTPException(status_code=400, detail=str(e))

@app.get("/documents/{doc_id}")
async def get_document(doc_id: str):
    """Get a document by ID."""
    try:
        doc = storage.get_document(doc_id)
        return {"content": doc.content, "metadata": doc.metadata.dict()}
    except Exception as e:
        raise HTTPException(status_code=404, detail=str(e))

@app.get("/search/")
async def search_documents(
    q: str = Query("*"),
    limit: int = 10,
    category: Optional[str] = None,
    tag: Optional[str] = None
):
    """Search documents with optional filters."""
    filters = {}
    if category:
        filters["categories"] = category
    if tag:
        filters["tags"] = tag
        
    return storage.search_documents(q, limit=limit, **filters)

if __name__ == "__main__":
    uvicorn.run(app, host="0.0.0.0", port=8000)
```

## Step 5: Testing the Search

1. Start the API:
   ```bash
   python -m knowledge_base.api
   ```

2. Test search with curl:
   ```bash
   # Search for documents
   curl "http://localhost:8000/search/?q=test&category=example"
   
   # Get a specific document
   curl http://localhost:8000/documents/test-doc
   ```

## Step 6: Frontend Integration (Basic Example)

Create `static/index.html`:

```html
<!DOCTYPE html>
<html>
<head>
    <title>Knowledge Base</title>
    <script src="https://unpkg.com/htmx.org@1.9.0"></script>
    <style>
        .search-result { margin: 1em 0; padding: 1em; border: 1px solid #ddd; }
        .highlight { background-color: yellow; }
    </style>
</head>
<body>
    <h1>Knowledge Base Search</h1>
    
    <input type="text" 
           hx-get="/search/" 
           hx-trigger="keyup changed delay:500ms"
           hx-target="#results"
           name="q"
           placeholder="Search...">
    
    <div id="results">
        <!-- Results will appear here -->
    </div>
</body>
</html>
```

## Next Steps

- Add pagination to search results
- Implement document versioning
- Add user authentication
- Set up automated testing

## Common Issues

1. **Search Index Corruption**: Delete the `knowledge/search_index` directory to rebuild
2. **Performance**: For large document sets, consider batching index updates
3. **Memory Usage**: Whoosh keeps the index in memory during searches

## Resources

- [Whoosh documentation](https://whoosh.readthedocs.io/)
- [FastAPI documentation](https://fastapi.tiangolo.com/)
