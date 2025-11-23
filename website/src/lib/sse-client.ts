/**
 * SSE (Server-Sent Events) Client
 * Wraps EventSource with error handling, reconnection, and JSON parsing
 */

export interface SSEClientOptions {
	onMessage: (event: unknown) => void;
	onError?: (error: Error) => void;
	onOpen?: () => void;
	onClose?: () => void;
	maxReconnectAttempts?: number;
	reconnectDelay?: number;
}

export class SSEClient {
	private eventSource: EventSource | null = null;
	private url: string;
	private options: Required<SSEClientOptions>;
	private reconnectAttempts = 0;
	private reconnectTimeout: number | null = null;
	private isClosed = false;

	constructor(url: string, options: SSEClientOptions) {
		this.url = url;
		this.options = {
			onMessage: options.onMessage,
			onError: options.onError || (() => {}),
			onOpen: options.onOpen || (() => {}),
			onClose: options.onClose || (() => {}),
			maxReconnectAttempts: options.maxReconnectAttempts || 3,
			reconnectDelay: options.reconnectDelay || 1000
		};
	}

	/**
	 * Connect to the SSE endpoint
	 */
	connect(): void {
		if (this.isClosed) {
			return;
		}

		try {
			this.eventSource = new EventSource(this.url);

			this.eventSource.onopen = () => {
				this.reconnectAttempts = 0;
				this.options.onOpen();
			};

			this.eventSource.onmessage = (event) => {
				try {
					const data = JSON.parse(event.data);
					this.options.onMessage(data);
				} catch (error) {
					this.options.onError(
						new Error(
							`Failed to parse event data: ${error instanceof Error ? error.message : 'Unknown error'}`
						)
					);
				}
			};

			this.eventSource.onerror = (event) => {
				console.error('SSE error:', event);

				// Close the current connection
				this.eventSource?.close();
				this.eventSource = null;

				// Attempt reconnection if not manually closed
				if (!this.isClosed && this.reconnectAttempts < this.options.maxReconnectAttempts) {
					this.reconnectAttempts++;
					this.options.onError(
						new Error(
							`Connection lost. Reconnecting (attempt ${this.reconnectAttempts}/${this.options.maxReconnectAttempts})...`
						)
					);

					this.reconnectTimeout = window.setTimeout(() => {
						this.connect();
					}, this.options.reconnectDelay * this.reconnectAttempts);
				} else if (!this.isClosed) {
					this.options.onError(new Error('Connection lost. Max reconnection attempts reached.'));
					this.close();
				}
			};
		} catch (error) {
			this.options.onError(
				new Error(`Failed to connect: ${error instanceof Error ? error.message : 'Unknown error'}`)
			);
		}
	}

	/**
	 * Close the SSE connection
	 */
	close(): void {
		this.isClosed = true;

		if (this.reconnectTimeout !== null) {
			clearTimeout(this.reconnectTimeout);
			this.reconnectTimeout = null;
		}

		if (this.eventSource) {
			this.eventSource.close();
			this.eventSource = null;
		}

		this.options.onClose();
	}

	/**
	 * Check if the client is connected
	 */
	isConnected(): boolean {
		return this.eventSource !== null && this.eventSource.readyState === EventSource.OPEN;
	}
}
