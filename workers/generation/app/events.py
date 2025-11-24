"""Event emission for generation worker."""

import json
import logging
from typing import Any, Dict, List, Optional

logger = logging.getLogger(__name__)


def emit_event(event_type: str, payload: Dict[str, Any], status: Optional[str] = None):
    """Emit an event to stdout in JSON format.
    
    Args:
        event_type: Type of event (e.g., "generation.started.v1")
        payload: Event payload data
        status: Optional status (e.g., "success", "failed")
    """
    event = {
        "type": event_type,
        "payload": payload,
    }
    
    if status:
        event["status"] = status
    
    try:
        print(json.dumps(event), flush=True)
    except Exception as e:
        logger.error(f"Failed to emit event {event_type}: {e}")


def emit_generation_started(symbol_count: int, estimated_tokens: int):
    """Emit generation started event.
    
    Args:
        symbol_count: Number of symbols to document
        estimated_tokens: Estimated total tokens
    """
    emit_event(
        "generation.started.v1",
        {
            "symbol_count": symbol_count,
            "estimated_tokens": estimated_tokens,
        }
    )


def emit_symbol_documented(symbol_id: str, token_count: int):
    """Emit symbol documented event.
    
    Args:
        symbol_id: ID of documented symbol
        token_count: Tokens used for this symbol
    """
    emit_event(
        "generation.symbol_documented.v1",
        {
            "symbol_id": symbol_id,
            "token_count": token_count,
        }
    )


def emit_generation_completed(
    total_tokens: int,
    total_cost: float,
    duration_ms: int,
    status: str = "success",
    warnings: Optional[List[str]] = None
):
    """Emit generation completed event.
    
    Args:
        total_tokens: Total tokens used
        total_cost: Total cost in dollars
        duration_ms: Duration in milliseconds
        status: Status (success or failed)
        warnings: Optional list of warning messages
    """
    emit_event(
        "generation.completed.v1",
        {
            "total_tokens": total_tokens,
            "total_cost": total_cost,
            "duration_ms": duration_ms,
            "warnings": warnings or [],
        },
        status=status
    )
