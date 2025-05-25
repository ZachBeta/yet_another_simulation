# Knowledge Base Implementation Plan

## Overview
This document outlines the implementation plan for a simple, file-based knowledge base system with colocated markdown content and JSON metadata files.

## Directory Structure

```
knowledge/
├── documents/          # All knowledge base documents
│   ├── getting-started/
│   │   ├── index.md
│   │   └── index.metadata.json
│   └── api-reference/
│       ├── overview.md
│       ├── overview.metadata.json
│       └── ...
└── categories.json     # Category definitions and hierarchy
```

## File Naming Convention
- Each document is a directory with an `index.md` and `index.metadata.json`
- Example: `getting-started/index.md` and `getting-started/index.metadata.json`
- For non-index documents: `topic-name.md` and `topic-name.metadata.json`

## Document Structure

### Markdown Files (`*.md`)
- Raw markdown content
- Standardized YAML frontmatter for essential metadata
- Content follows standard markdown formatting

Example `index.md`:
```markdown
---
title: Getting Started
description: Introduction to the knowledge base
---

# Getting Started

Welcome to the knowledge base...
```

### Metadata Files (`*.metadata.json`)
- JSON file with the same base name as the markdown file
- Contains structured metadata about the document

Example `index.metadata.json`:
```json
{
    "id": "getting-started",
    "title": "Getting Started",
    "description": "Introduction to the knowledge base",
    "created_at": "2025-05-24T10:00:00Z",
    "updated_at": "2025-05-24T10:00:00Z",
    "categories": ["introduction", "getting-started"],
    "tags": ["beginner", "tutorial"],
    "status": "published",
    "author": "system",
    "related_documents": ["api-reference"]
}
```

## Core Operations

### 1. Document Management

#### Add Document
```python
async def add_document(
    path: str,
    content: str,
    metadata: Optional[dict] = None
) -> str:
    """
    Add or update a document
    
    Args:
        path: Relative path without extension (e.g., 'getting-started/index')
        content: Markdown content
        metadata: Optional metadata dictionary
    """
```

#### Get Document
```python
async def get_document(path: str) -> tuple[str, dict]:
    """
    Get document content and metadata
    
    Returns:
        Tuple of (content, metadata)
    """
```

### 2. Search and Filter

#### Basic Search
```python
async def search(
    query: Optional[str] = None,
    filters: Optional[dict] = None,
    limit: int = 20
) -> list[dict]:
    """
    Search documents with optional filters
    
    Args:
        query: Optional text search query
        filters: Dictionary of filters (e.g., {"categories": ["tutorial"]})
        limit: Maximum number of results
    """
```

## Implementation Phases

### Phase 1: Core Functionality (Week 1)
1. Set up basic directory structure
2. Implement file system operations
3. Add document CRUD operations
4. Basic metadata handling

### Phase 2: Search and Organization (Week 2)
1. Implement basic search functionality
2. Add category/tag filtering
3. Add sorting and pagination
4. Basic validation

### Phase 3: Integration (Week 3)
1. Add CLI interface
2. Add basic web API (FastAPI)
3. Documentation and examples
4. Basic error handling

## Next Steps
1. Review and approve this plan
2. Set up initial directory structure
3. Begin Phase 1 implementation

## Open Questions
- Should we support document versioning?
- What are the required metadata fields?
- Any specific search requirements beyond basic text search?
