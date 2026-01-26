"""
CEREBRO API Server

FastAPI server providing REST endpoints and WebSocket support
for the Cerebro Intelligence System.
"""

import asyncio
import json
import os
from contextlib import asynccontextmanager
from datetime import datetime, timezone
from typing import Any, Dict, List, Optional
from pathlib import Path

from fastapi import FastAPI, HTTPException, WebSocket, WebSocketDisconnect, Query
from fastapi.middleware.cors import CORSMiddleware
from fastapi.responses import JSONResponse
from pydantic import BaseModel, Field
import logging

from phantom.intelligence.core import (
    CerebroIntelligence,
    IntelligenceType,
    ThreatLevel,
    Project,
)
from phantom.intelligence.analyzer import IntelligenceAnalyzer
from phantom.intelligence.briefing import BriefingGenerator, BriefingType
from phantom.registry.scanner import ProjectScanner
from phantom.registry.indexer import KnowledgeIndexer

logger = logging.getLogger("cerebro.api")

# Global instances
cerebro: Optional[CerebroIntelligence] = None
scanner: Optional[ProjectScanner] = None
indexer: Optional[KnowledgeIndexer] = None
analyzer: Optional[IntelligenceAnalyzer] = None
briefing_gen: Optional[BriefingGenerator] = None

# WebSocket connections
active_connections: List[WebSocket] = []


# ==================== Pydantic Models ====================

class QueryRequest(BaseModel):
    """Request for intelligence query."""
    query: str = Field(..., min_length=1, max_length=1000)
    types: Optional[List[str]] = None
    projects: Optional[List[str]] = None
    limit: int = Field(default=10, ge=1, le=100)
    semantic: bool = Field(default=True, description="Use semantic search")


class QueryResponse(BaseModel):
    """Response for intelligence query."""
    query: str
    results: List[Dict[str, Any]]
    total: int
    search_type: str


class ProjectResponse(BaseModel):
    """Response for project info."""
    name: str
    path: str
    description: str
    languages: List[str]
    status: str
    health_score: float
    last_commit: Optional[str]
    last_indexed: Optional[str]
    metadata: Dict[str, Any]


class EcosystemStatus(BaseModel):
    """Ecosystem status response."""
    total_projects: int
    active_projects: int
    health_score: float
    total_intelligence: int
    alerts_count: int
    last_scan: Optional[str]


class BriefingRequest(BaseModel):
    """Request for briefing generation."""
    type: str = Field(default="daily")
    project: Optional[str] = None
    format: str = Field(default="json")  # json or markdown


class ScanRequest(BaseModel):
    """Request for ecosystem scan."""
    full_scan: bool = Field(default=False)
    collect_intelligence: bool = Field(default=True)


# ==================== Lifespan ====================

@asynccontextmanager
async def lifespan(app: FastAPI):
    """Initialize and cleanup resources."""
    global cerebro, scanner, indexer, analyzer, briefing_gen

    # Configuration
    arch_path = os.getenv("CEREBRO_ARCH_PATH", "/home/kernelcore/arch")
    data_dir = os.getenv("CEREBRO_DATA_DIR", "./data/intelligence")

    logger.info(f"Initializing Cerebro Intelligence System...")
    logger.info(f"Arch path: {arch_path}")
    logger.info(f"Data dir: {data_dir}")

    # Initialize components
    cerebro = CerebroIntelligence(
        arch_path=arch_path,
        data_dir=data_dir,
    )

    scanner = ProjectScanner(cerebro, arch_path)
    indexer = KnowledgeIndexer(cerebro, cache_dir=f"{data_dir}/embeddings")
    analyzer = IntelligenceAnalyzer(cerebro)
    briefing_gen = BriefingGenerator(cerebro)

    logger.info("Cerebro Intelligence System initialized")

    yield

    # Cleanup
    if cerebro:
        cerebro.shutdown()
    logger.info("Cerebro Intelligence System shutdown")


# ==================== Create App ====================

def create_app() -> FastAPI:
    """Create the FastAPI application."""
    app = FastAPI(
        title="CEREBRO Intelligence API",
        description="Central Intelligence System for ~/arch ecosystem",
        version="1.0.0",
        lifespan=lifespan,
    )

    # CORS middleware
    app.add_middleware(
        CORSMiddleware,
        allow_origins=["*"],  # Configure appropriately for production
        allow_credentials=True,
        allow_methods=["*"],
        allow_headers=["*"],
    )

    return app


app = create_app()


# ==================== Health & Status ====================

@app.get("/health")
async def health_check():
    """Health check endpoint."""
    return {
        "status": "healthy",
        "service": "cerebro-intelligence",
        "timestamp": datetime.now(timezone.utc).isoformat(),
    }


@app.get("/status", response_model=EcosystemStatus)
async def get_status():
    """Get ecosystem status."""
    if not cerebro:
        raise HTTPException(status_code=503, detail="Service not initialized")

    status = cerebro.get_ecosystem_status()
    alerts = cerebro.get_alerts()

    return EcosystemStatus(
        total_projects=status.total_projects,
        active_projects=status.active_projects,
        health_score=cerebro.calculate_health_score(),
        total_intelligence=status.total_intelligence,
        alerts_count=len(alerts),
        last_scan=status.last_scan.isoformat() if status.last_scan else None,
    )


# ==================== Projects ====================

@app.get("/projects", response_model=List[ProjectResponse])
async def list_projects(
    status: Optional[str] = Query(None, description="Filter by status"),
    language: Optional[str] = Query(None, description="Filter by language"),
    sort_by: str = Query("health_score", description="Sort field"),
    order: str = Query("desc", description="Sort order (asc/desc)"),
):
    """List all projects."""
    if not cerebro:
        raise HTTPException(status_code=503, detail="Service not initialized")

    projects = cerebro.list_projects()

    # Filter by status
    if status:
        projects = [p for p in projects if p.status.value == status]

    # Filter by language
    if language:
        projects = [p for p in projects if language in p.languages]

    # Sort
    reverse = order == "desc"
    if sort_by == "health_score":
        projects.sort(key=lambda p: p.health_score, reverse=reverse)
    elif sort_by == "name":
        projects.sort(key=lambda p: p.name, reverse=reverse)
    elif sort_by == "last_commit":
        projects.sort(
            key=lambda p: p.last_commit or datetime.min.replace(tzinfo=timezone.utc),
            reverse=reverse
        )

    return [
        ProjectResponse(
            name=p.name,
            path=str(p.path),
            description=p.description,
            languages=p.languages,
            status=p.status.value,
            health_score=p.health_score,
            last_commit=p.last_commit.isoformat() if p.last_commit else None,
            last_indexed=p.last_indexed.isoformat() if p.last_indexed else None,
            metadata=p.metadata,
        )
        for p in projects
    ]


@app.get("/projects/{project_name}")
async def get_project(project_name: str):
    """Get detailed project information."""
    if not cerebro or not analyzer:
        raise HTTPException(status_code=503, detail="Service not initialized")

    project = cerebro.get_project(project_name)
    if not project:
        raise HTTPException(status_code=404, detail=f"Project not found: {project_name}")

    # Run analysis
    analysis = analyzer.analyze_project(project)

    return {
        "project": ProjectResponse(
            name=project.name,
            path=str(project.path),
            description=project.description,
            languages=project.languages,
            status=project.status.value,
            health_score=project.health_score,
            last_commit=project.last_commit.isoformat() if project.last_commit else None,
            last_indexed=project.last_indexed.isoformat() if project.last_indexed else None,
            metadata=project.metadata,
        ),
        "analysis": analysis,
    }


# ==================== Intelligence ====================

@app.post("/query", response_model=QueryResponse)
async def query_intelligence(request: QueryRequest):
    """Query the intelligence database."""
    if not cerebro or not indexer:
        raise HTTPException(status_code=503, detail="Service not initialized")

    # Convert type strings to enums
    types = None
    if request.types:
        types = [IntelligenceType(t) for t in request.types]

    if request.semantic:
        # Use semantic search
        results = indexer.semantic_query(
            query=request.query,
            top_k=request.limit,
        )
        search_type = "semantic"
    else:
        # Use keyword search
        items = cerebro.query_intelligence(
            query=request.query,
            types=types,
            projects=request.projects,
            limit=request.limit,
        )
        results = [item.to_dict() for item in items]
        search_type = "keyword"

    return QueryResponse(
        query=request.query,
        results=results,
        total=len(results),
        search_type=search_type,
    )


@app.get("/intelligence/stats")
async def get_intelligence_stats():
    """Get intelligence statistics."""
    if not cerebro or not indexer:
        raise HTTPException(status_code=503, detail="Service not initialized")

    intel_by_type = {}
    for intel_type in IntelligenceType:
        count = sum(
            1 for i in cerebro._intelligence.values()
            if i.type == intel_type
        )
        intel_by_type[intel_type.value] = count

    intel_by_threat = {}
    for threat_level in ThreatLevel:
        count = sum(
            1 for i in cerebro._intelligence.values()
            if i.threat_level == threat_level
        )
        intel_by_threat[threat_level.value] = count

    return {
        "total": len(cerebro._intelligence),
        "by_type": intel_by_type,
        "by_threat_level": intel_by_threat,
        "indexer_stats": indexer.get_stats(),
    }


# ==================== Briefings ====================

@app.post("/briefing")
async def generate_briefing(request: BriefingRequest):
    """Generate an intelligence briefing."""
    if not briefing_gen:
        raise HTTPException(status_code=503, detail="Service not initialized")

    try:
        briefing_type = BriefingType(request.type)
    except ValueError:
        raise HTTPException(
            status_code=400,
            detail=f"Invalid briefing type: {request.type}. Valid types: {[t.value for t in BriefingType]}"
        )

    briefing = briefing_gen.generate(briefing_type, request.project)

    if request.format == "markdown":
        return JSONResponse(
            content={"markdown": briefing_gen.to_markdown(briefing)},
            media_type="application/json",
        )

    return briefing


@app.get("/briefing/daily")
async def get_daily_briefing():
    """Get the daily briefing."""
    if not briefing_gen:
        raise HTTPException(status_code=503, detail="Service not initialized")

    return briefing_gen.generate(BriefingType.DAILY)


@app.get("/briefing/executive")
async def get_executive_briefing():
    """Get the executive summary."""
    if not briefing_gen:
        raise HTTPException(status_code=503, detail="Service not initialized")

    return briefing_gen.generate(BriefingType.EXECUTIVE)


# ==================== Scanning ====================

@app.post("/scan")
async def trigger_scan(request: ScanRequest):
    """Trigger an ecosystem scan."""
    if not scanner or not indexer:
        raise HTTPException(status_code=503, detail="Service not initialized")

    if request.collect_intelligence:
        stats = scanner.full_scan_with_intelligence()
    else:
        projects = scanner.scan(full_scan=request.full_scan)
        stats = {"projects_found": len(projects)}

    # Index new intelligence
    indexed = indexer.index_all()
    stats["indexed_items"] = indexed

    # Broadcast update to WebSocket clients
    await broadcast({
        "type": "scan_complete",
        "stats": stats,
    })

    return stats


# ==================== Alerts ====================

@app.get("/alerts")
async def get_alerts():
    """Get current alerts."""
    if not cerebro:
        raise HTTPException(status_code=503, detail="Service not initialized")

    return cerebro.get_alerts()


# ==================== Dependencies Graph ====================

@app.get("/graph")
async def get_dependency_graph():
    """Get project dependency graph."""
    if not analyzer:
        raise HTTPException(status_code=503, detail="Service not initialized")

    graph = analyzer.find_dependencies_graph()

    # Convert to nodes and edges for visualization
    nodes = [{"id": name, "label": name} for name in graph.keys()]
    edges = []
    for source, targets in graph.items():
        for target in targets:
            edges.append({"source": source, "target": target})

    return {
        "nodes": nodes,
        "edges": edges,
    }


# ==================== WebSocket ====================

@app.websocket("/ws")
async def websocket_endpoint(websocket: WebSocket):
    """WebSocket endpoint for real-time updates."""
    await websocket.accept()
    active_connections.append(websocket)

    try:
        # Send initial status
        if cerebro:
            status = cerebro.get_ecosystem_status()
            await websocket.send_json({
                "type": "status",
                "data": status.to_dict(),
            })

        # Keep connection alive and handle messages
        while True:
            data = await websocket.receive_text()
            message = json.loads(data)

            # Handle different message types
            if message.get("type") == "ping":
                await websocket.send_json({"type": "pong"})
            elif message.get("type") == "subscribe":
                # Handle subscriptions
                pass

    except WebSocketDisconnect:
        active_connections.remove(websocket)


async def broadcast(message: Dict[str, Any]):
    """Broadcast message to all connected WebSocket clients."""
    for connection in active_connections:
        try:
            await connection.send_json(message)
        except Exception:
            pass


# ==================== AI Features ====================

@app.post("/summarize")
async def summarize_project(project_name: str):
    """Generate AI summary for a project."""
    if not cerebro:
        raise HTTPException(status_code=503, detail="Service not initialized")

    project = cerebro.get_project(project_name)
    if not project:
        raise HTTPException(status_code=404, detail=f"Project not found: {project_name}")

    # Get project intelligence
    intel_items = cerebro.query_intelligence(
        query="",
        projects=[project_name],
        limit=50,
    )

    # Build summary (placeholder for LLM integration)
    summary = {
        "project": project_name,
        "description": project.description,
        "languages": project.languages,
        "health_score": project.health_score,
        "key_points": [
            f"Primary languages: {', '.join(project.languages)}",
            f"Health score: {project.health_score}%",
            f"Status: {project.status.value}",
            f"Intelligence items: {len(intel_items)}",
        ],
        "recommendations": [],
    }

    if project.health_score < 50:
        summary["recommendations"].append("Consider improving documentation and test coverage")

    return summary


# ==================== Run Server ====================

if __name__ == "__main__":
    import uvicorn
    uvicorn.run(
        "phantom.api.server:app",
        host="0.0.0.0",
        port=8000,
        reload=True,
    )
