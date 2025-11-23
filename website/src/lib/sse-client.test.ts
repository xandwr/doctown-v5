/**
 * Unit tests for SSE Client
 * These tests verify event parsing and error handling
 */

import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest';
import { SSEClient } from './sse-client';

describe('SSEClient', () => {
	let mockEventSource: {
		close: ReturnType<typeof vi.fn>;
		onopen: (() => void) | null;
		onmessage: ((event: MessageEvent) => void) | null;
		onerror: ((event: Event) => void) | null;
		readyState: number;
	};

	beforeEach(() => {
		// Mock EventSource
		mockEventSource = {
			close: vi.fn(),
			onopen: null,
			onmessage: null,
			onerror: null,
			readyState: 0
		};

		// @ts-expect-error - Mocking global EventSource
		global.EventSource = vi.fn(() => mockEventSource);
	});

	afterEach(() => {
		vi.restoreAllMocks();
	});

	it('should parse incoming JSON events', () => {
		const messages: unknown[] = [];
		const client = new SSEClient('http://test.com', {
			onMessage: (event) => messages.push(event)
		});

		client.connect();
		mockEventSource.readyState = 1; // OPEN

		// Simulate receiving a message
		const testData = { type: 'test', payload: { value: 123 } };
		mockEventSource.onmessage?.({
			data: JSON.stringify(testData)
		} as MessageEvent);

		expect(messages).toHaveLength(1);
		expect(messages[0]).toEqual(testData);
	});

	it('should handle invalid JSON gracefully', () => {
		const errors: Error[] = [];
		const client = new SSEClient('http://test.com', {
			onMessage: () => {},
			onError: (error) => errors.push(error)
		});

		client.connect();
		mockEventSource.readyState = 1; // OPEN

		// Simulate receiving invalid JSON
		mockEventSource.onmessage?.({
			data: 'invalid json{'
		} as MessageEvent);

		expect(errors).toHaveLength(1);
		expect(errors[0].message).toContain('Failed to parse event data');
	});

	it('should call onOpen when connection opens', () => {
		let opened = false;
		const client = new SSEClient('http://test.com', {
			onMessage: () => {},
			onOpen: () => {
				opened = true;
			}
		});

		client.connect();
		mockEventSource.readyState = 1; // OPEN
		mockEventSource.onopen?.();

		expect(opened).toBe(true);
	});

	it('should call onClose when connection is closed', () => {
		let closed = false;
		const client = new SSEClient('http://test.com', {
			onMessage: () => {},
			onClose: () => {
				closed = true;
			}
		});

		client.connect();
		client.close();

		expect(closed).toBe(true);
		expect(mockEventSource.close).toHaveBeenCalled();
	});

	it('should report connection status correctly', () => {
		const client = new SSEClient('http://test.com', {
			onMessage: () => {}
		});

		expect(client.isConnected()).toBe(false);

		client.connect();

		// After connect, the EventSource is created but not yet open
		expect(client.isConnected()).toBe(false);

		// Simulate the connection opening
		mockEventSource.readyState = 1; // OPEN
		expect(client.isConnected()).toBe(true);

		client.close();
		expect(client.isConnected()).toBe(false);
	});
});
