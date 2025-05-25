---
title: Phase 1 - Core Knowledge Base Functionality
description: Setting up the basic knowledge base structure and core operations
difficulty: Intermediate
time_required: 2-3 hours
prerequisites:
  - Python 3.8+
  - Basic understanding of file I/O in Python
  - Familiarity with JSON and Markdown
---

# Phase 1: Core Knowledge Base Implementation

## Overview
In this phase, we'll implement the basic building blocks of our knowledge base system, focusing on document storage and retrieval with metadata support.

## Step 1: Project Setup

1. Create a new Python virtual environment:
   ```bash
   python -m venv venv
   source venv/bin/activate  # On Windows: venv\Scripts\activate
   ```

2. Install required packages:
   ```bash
   pip install pydantic python-dateutil
   ```

## Step 2: Directory Structure

Create the following structure:
```
knowledge_base/
├── __init__.py
├── models.py      # Data models
├── storage.py     # File operations
└── exceptions.py  # Custom exceptions
```

## Step 3: Define Data Models

In `models.py`:

```python
from datetime import datetime
from pathlib import Path
from typing import List, Optional, Dict, Any
from pydantic import BaseModel, Field

class DocumentMetadata(BaseModel):
    """Schema for document metadata."""
    id: str
    title: str
    description: str = ""
    created_at: datetime = Field(default_factory=datetime.utcnow)
    updated_at: datetime = Field(default_factory=datetime.utcnow)
    categories: List[str] = []
    tags: List[str] = []
    status: str = "draft"  # draft, published, archived
    author: str = "system"
    custom_fields: Dict[str, Any] = {}

class Document(BaseModel):
    """Represents a document with content and metadata."""
    content: str
    metadata: DocumentMetadata
```

## Step 4: Implement Storage Layer

In `storage.py`:

```python
import json
from pathlib import Path
from typing import Optional, Tuple
from datetime import datetime
from .models import Document, DocumentMetadata
from .exceptions import DocumentNotFoundError, StorageError

class DocumentStorage:
    """Handles storage and retrieval of documents."""
    
    def __init__(self, base_path: str = "knowledge"):
        self.base_path = Path(base_path)
        self.documents_path = self.base_path / "documents"
        self._ensure_directories()
    
    def _ensure_directories(self):
        """Create required directories if they don't exist."""
        self.documents_path.mkdir(parents=True, exist_ok=True)
    
    def _get_document_paths(self, doc_id: str) -> Tuple[Path, Path]:
        """Get paths for document and metadata files."""
        doc_dir = self.documents_path / doc_id
        return (doc_dir / "index.md", doc_dir / "index.metadata.json")
    
    def save_document(self, doc: Document) -> str:
        """Save a document to storage."""
        try:
            # Update timestamps
            now = datetime.utcnow()
            if not doc.metadata.created_at:
                doc.metadata.created_at = now
            doc.metadata.updated_at = now
            
            # Ensure document directory exists
            doc_dir = self.documents_path / doc.metadata.id
            doc_dir.mkdir(exist_ok=True)
            
            # Save content and metadata
            content_path, meta_path = self._get_document_paths(doc.metadata.id)
            
            # Save markdown content
            with open(content_path, 'w', encoding='utf-8') as f:
                f.write(doc.content)
            
            # Save metadata
            with open(meta_path, 'w', encoding='utf-8') as f:
                json.dump(doc.metadata.dict(), f, indent=2, default=str)
            
            return doc.metadata.id
            
        except Exception as e:
            raise StorageError(f"Failed to save document: {str(e)}")
    
    def get_document(self, doc_id: str) -> Document:
        """Retrieve a document by ID."""
        try:
            content_path, meta_path = self._get_document_paths(doc_id)
            
            if not content_path.exists() or not meta_path.exists():
                raise DocumentNotFoundError(f"Document {doc_id} not found")
            
            # Load metadata
            with open(meta_path, 'r', encoding='utf-8') as f:
                metadata = DocumentMetadata(**json.load(f))
            
            # Load content
            with open(content_path, 'r', encoding='utf-8') as f:
                content = f.read()
            
            return Document(content=content, metadata=metadata)
            
        except json.JSONDecodeError as e:
            raise StorageError(f"Invalid metadata format for document {doc_id}: {str(e)}")
        except Exception as e:
            raise StorageError(f"Failed to retrieve document {doc_id}: {str(e)}")
```

## Step 5: Basic Usage Example

Create `example.py`:

```python
from datetime import datetime
from knowledge_base.storage import DocumentStorage
from knowledge_base.models import Document, DocumentMetadata

# Initialize storage
storage = DocumentStorage()

# Create a new document
doc = Document(
    content="# Hello, World!\n\nThis is a test document.",
    metadata=DocumentMetadata(
        id="test-doc",
        title="Test Document",
        description="A simple test document",
        categories=["test", "example"],
        tags=["demo"]
    )
)

# Save the document
doc_id = storage.save_document(doc)
print(f"Saved document with ID: {doc_id}")

# Retrieve the document
retrieved = storage.get_document("test-doc")
print(f"Retrieved document: {retrieved.metadata.title}")
print(f"Content: {retrieved.content[:50]}...")
```

## Testing

1. Run the example:
   ```bash
   python example.py
   ```

2. Verify the files were created:
   ```
   knowledge/
   └── documents/
       └── test-doc/
           ├── index.md
           └── index.metadata.json
   ```

## Next Steps

- Add error handling and validation
- Implement document updates and deletion
- Add basic search functionality

## Common Issues

1. **Permission Errors**: Ensure the script has write access to the knowledge directory
2. **JSON Serialization**: All datetime objects are automatically converted to ISO format strings
3. **Document IDs**: Must be URL-safe and unique

## Resources

- [Python pathlib documentation](https://docs.python.org/3/library/pathlib.html)
- [Pydantic documentation](https://pydantic-docs.helpmanual.io/)
