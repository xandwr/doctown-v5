"""Event emission for embedding operations."""

import json
import logging
from datetime import datetime
from typing import Dict, Any

logger = logging.getLogger(__name__)


def emit_event(event_type: str, payload: Dict[str, Any]):
    """Emit an event (currently just logs, can be extended to send to event bus).
    
    Args:
        event_type: Event type (e.g., "embedding.batch_started.v1")
        payload: Event payload data
    """
    event = {
        "type": event_type,
        "timestamp": datetime.utcnow().isoformat() + "Z",
        "payload": payload
    }
    logger.info(f"Event: {json.dumps(event)}")


def emit_batch_started(batch_id: str, chunk_count: int):
    """Emit event when embedding batch starts.
    
    Args:
        batch_id: Unique identifier for the batch
        chunk_count: Number of chunks in the batch
    """
    emit_event("embedding.batch_started.v1", {
        "batch_id": batch_id,
        "chunk_count": chunk_count
    })


def emit_batch_completed(batch_id: str, chunk_count: int, duration_ms: float):
    """Emit event when embedding batch completes.
    
    Args:
        batch_id: Unique identifier for the batch
        chunk_count: Number of chunks embedded
        duration_ms: Time taken to embed the batch in milliseconds
    """
    emit_event("embedding.batch_completed.v1", {
        "batch_id": batch_id,
        "chunk_count": chunk_count,
        "duration_ms": round(duration_ms, 2)
    })
