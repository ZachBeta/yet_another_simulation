---
title: Phase 3 - Advanced Features and Deployment
description: Adding advanced features and preparing for production deployment
difficulty: Advanced
time_required: 4-5 hours
prerequisites:
  - Completion of Phase 1 & 2
  - Basic Docker knowledge
  - Familiarity with CI/CD concepts
---

# Phase 3: Advanced Features and Production Deployment

## Overview
In this final phase, we'll enhance our knowledge base with advanced features and prepare it for production deployment.

## Step 1: Add Document Versioning

Create `versioning.py`:

```python
import shutil
from pathlib import Path
from datetime import datetime
from typing import List, Optional
import hashlib

class DocumentVersioner:
    """Handles document versioning and history."""
    
    def __init__(self, base_path: str = "knowledge/versions"):
        self.base_path = Path(base_path)
        self.base_path.mkdir(parents=True, exist_ok=True)
    
    def _get_version_dir(self, doc_id: str) -> Path:
        """Get the version directory for a document."""
        return self.base_path / doc_id
    
    def _generate_version_id(self, content: str) -> str:
        """Generate a unique version ID based on content hash."""
        return hashlib.sha256(content.encode()).hexdigest()[:12]
    
    def create_version(self, doc_id: str, content: str, metadata: dict) -> str:
        """Create a new version of a document."""
        version_id = self._generate_version_id(content)
        version_dir = self._get_version_dir(doc_id)
        version_dir.mkdir(parents=True, exist_ok=True)
        
        # Save version
        version_file = version_dir / f"{version_id}.md"
        version_meta = version_dir / f"{version_id}.json"
        
        with open(version_file, 'w', encoding='utf-8') as f:
            f.write(content)
            
        version_data = {
            'version_id': version_id,
            'created_at': datetime.utcnow().isoformat(),
            'metadata': metadata
        }
        
        with open(version_meta, 'w', encoding='utf-8') as f:
            json.dump(version_data, f, indent=2)
            
        # Update current version
        current_link = version_dir / "current"
        if current_link.exists():
            current_link.unlink()
        current_link.symlink_to(f"{version_id}.md")
        
        return version_id
    
    def get_version(self, doc_id: str, version_id: str) -> Optional[dict]:
        """Get a specific version of a document."""
        version_dir = self._get_version_dir(doc_id)
        version_file = version_dir / f"{version_id}.md"
        version_meta = version_dir / f"{version_id}.json"
        
        if not version_file.exists() or not version_meta.exists():
            return None
            
        with open(version_file, 'r', encoding='utf-8') as f:
            content = f.read()
            
        with open(version_meta, 'r', encoding='utf-8') as f:
            metadata = json.load(f)
            
        return {
            'content': content,
            'metadata': metadata
        }
    
    def list_versions(self, doc_id: str) -> List[dict]:
        """List all versions of a document."""
        version_dir = self._get_version_dir(doc_id)
        if not version_dir.exists():
            return []
            
        versions = []
        for meta_file in version_dir.glob("*.json"):
            if meta_file.stem == 'current':
                continue
                
            with open(meta_file, 'r', encoding='utf-8') as f:
                version_data = json.load(f)
                versions.append({
                    'version_id': meta_file.stem,
                    'created_at': version_data['created_at'],
                    'metadata': version_data['metadata']
                })
                
        return sorted(versions, key=lambda x: x['created_at'], reverse=True)
```

## Step 2: Add User Authentication

Create `auth.py`:

```python
from fastapi import Depends, HTTPException, status
from fastapi.security import OAuth2PasswordBearer
from jose import JWTError, jwt
from passlib.context import CryptContext
from datetime import datetime, timedelta
from typing import Optional
import secrets

# Configuration
SECRET_KEY = secrets.token_urlsafe(32)
ALGORITHM = "HS256"
ACCESS_TOKEN_EXPIRE_MINUTES = 30

# Password hashing
pwd_context = CryptContext(schemes=["bcrypt"], deprecated="auto")
oauth2_scheme = OAuth2PasswordBearer(tokenUrl="token")

# Mock user database (replace with real DB in production)
fake_users_db = {
    "admin": {
        "username": "admin",
        "hashed_password": pwd_context.hash("admin"),
        "disabled": False,
    }
}

def verify_password(plain_password: str, hashed_password: str) -> bool:
    return pwd_context.verify(plain_password, hashed_password)

def get_password_hash(password: str) -> str:
    return pwd_context.hash(password)

def create_access_token(data: dict, expires_delta: Optional[timedelta] = None):
    to_encode = data.copy()
    if expires_delta:
        expire = datetime.utcnow() + expires_delta
    else:
        expire = datetime.utcnow() + timedelta(minutes=15)
    to_encode.update({"exp": expire})
    encoded_jwt = jwt.encode(to_encode, SECRET_KEY, algorithm=ALGORITHM)
    return encoded_jwt

async def get_current_user(token: str = Depends(oauth2_scheme)):
    credentials_exception = HTTPException(
        status_code=status.HTTP_401_UNAUTHORIZED,
        detail="Could not validate credentials",
        headers={"WWW-Authenticate": "Bearer"},
    )
    try:
        payload = jwt.decode(token, SECRET_KEY, algorithms=[ALGORITHM])
        username: str = payload.get("sub")
        if username is None:
            raise credentials_exception
    except JWTError:
        raise credentials_exception
        
    user = fake_users_db.get(username)
    if user is None:
        raise credentials_exception
    return user
```

## Step 3: Docker Setup

Create `Dockerfile`:

```dockerfile
FROM python:3.9-slim

WORKDIR /app

# Install system dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    build-essential \
    && rm -rf /var/lib/apt/lists/*

# Install Python dependencies
COPY requirements.txt .
RUN pip install --no-cache-dir -r requirements.txt

# Copy application code
COPY . .

# Create necessary directories
RUN mkdir -p /app/knowledge/documents /app/knowledge/search_index /app/knowledge/versions

# Run the application
CMD ["uvicorn", "knowledge_base.api:app", "--host", "0.0.0.0", "--port", "8000"]
```

Create `docker-compose.yml`:

```yaml
version: '3.8'

services:
  knowledge-base:
    build: .
    ports:
      - "8000:8000"
    volumes:
      - ./knowledge:/app/knowledge
    environment:
      - PYTHONUNBUFFERED=1
    restart: unless-stopped
```

## Step 4: CI/CD Pipeline

Create `.github/workflows/deploy.yml`:

```yaml
name: Deploy Knowledge Base

on:
  push:
    branches: [ main ]
  pull_request:
    branches: [ main ]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v2
    - name: Set up Python
      uses: actions/setup-python@v2
      with:
        python-version: '3.9'
    - name: Install dependencies
      run: |
        python -m pip install --upgrade pip
        pip install -r requirements.txt
        pip install pytest pytest-cov
    - name: Run tests
      run: |
        pytest --cov=knowledge_base tests/

  deploy:
    needs: test
    if: github.ref == 'refs/heads/main'
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v2
    - name: Deploy to production
      env:
        DOCKERHUB_USERNAME: ${{ secrets.DOCKERHUB_USERNAME }}
        DOCKERHUB_TOKEN: ${{ secrets.DOCKERHUB_TOKEN }}
      run: |
        echo "${{ secrets.DEPLOY_KEY }}" > deploy_key
        chmod 600 deploy_key
        ssh -o StrictHostKeyChecking=no -i deploy_key user@server "
          cd /path/to/knowledge-base && \
          git pull && \
          docker-compose up -d --build
        "
```

## Step 5: Monitoring and Logging

Create `logging_config.py`:

```python
import logging
import logging.handlers
import os
from pathlib import Path

def setup_logging():
    """Configure logging for the application."""
    log_dir = Path("logs")
    log_dir.mkdir(exist_ok=True)
    
    # Root logger
    logger = logging.getLogger()
    logger.setLevel(logging.INFO)
    
    # File handler (rotating)
    file_handler = logging.handlers.RotatingFileHandler(
        log_dir / "knowledge_base.log",
        maxBytes=10*1024*1024,  # 10MB
        backupCount=5
    )
    file_handler.setFormatter(logging.Formatter(
        '%(asctime)s - %(name)s - %(levelname)s - %(message)s'
    ))
    
    # Console handler
    console_handler = logging.StreamHandler()
    console_handler.setFormatter(logging.Formatter(
        '%(asctime)s - %(levelname)s - %(message)s'
    ))
    
    # Add handlers
    logger.addHandler(file_handler)
    logger.addHandler(console_handler)
    
    # Suppress noisy loggers
    logging.getLogger('whoosh').setLevel(logging.WARNING)
    logging.getLogger('fastapi').setLevel(logging.INFO)
    logging.getLogger('uvicorn').setLevel(logging.INFO)
```

## Step 6: Deployment

1. Build and run with Docker Compose:
   ```bash
   docker-compose up -d --build
   ```

2. Set up a reverse proxy (Nginx example):
   ```nginx
   server {
       listen 80;
       server_name knowledge.example.com;

       location / {
           proxy_pass http://localhost:8000;
           proxy_set_header Host $host;
           proxy_set_header X-Real-IP $remote_addr;
           proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
       }
   }
   ```

3. Set up SSL with Let's Encrypt:
   ```bash
   sudo certbot --nginx -d knowledge.example.com
   ```

## Next Steps

- Set up automated backups
- Implement rate limiting
- Add API documentation with Swagger/ReDoc
- Set up monitoring (Prometheus/Grafana)
- Implement caching

## Common Issues

1. **File Permissions**: Ensure the Docker user has write access to mounted volumes
2. **Memory Usage**: Monitor and adjust Whoosh index settings for large document sets
3. **Security**: Rotate secrets and implement proper access controls

## Resources

- [Docker documentation](https://docs.docker.com/)
- [FastAPI deployment](https://fastapi.tiangolo.com/deployment/)
- [GitHub Actions](https://docs.github.com/en/actions)
