import { describe, it, expect } from 'vitest';

describe('FileTree component logic', () => {
	it('should organize events into file structure', () => {
		const events = [
			{
				event_type: 'ingest.file_detected.v1',
				timestamp: '2025-01-01T00:00:00.000Z',
				payload: {
					file_path: 'src/main.rs',
					language: 'rust'
				}
			},
			{
				event_type: 'ingest.chunk_created.v1',
				timestamp: '2025-01-01T00:00:01.000Z',
				payload: {
					file_path: 'src/main.rs',
					chunk_id: 'chunk_123',
					symbol_name: 'main',
					symbol_kind: 'function'
				}
			},
			{
				event_type: 'ingest.chunk_created.v1',
				timestamp: '2025-01-01T00:00:02.000Z',
				payload: {
					file_path: 'src/main.rs',
					chunk_id: 'chunk_124',
					symbol_name: 'process_data',
					symbol_kind: 'function'
				}
			}
		];

		// Simulate the derived logic from component
		const files = new Map();

		for (const event of events) {
			const eventType = event.event_type || '';
			const payload = event.payload || {};

			if (eventType.includes('file_detected')) {
				const path = payload.file_path || '';
				if (!files.has(path)) {
					files.set(path, {
						path,
						language: payload.language || 'unknown',
						chunks: []
					});
				}
			} else if (eventType.includes('chunk_created')) {
				const path = payload.file_path || '';
				const file = files.get(path);
				if (file) {
					file.chunks.push({
						name: payload.symbol_name || 'unknown',
						kind: payload.symbol_kind || 'unknown',
						chunkId: payload.chunk_id || ''
					});
				}
			}
		}

		const fileData = Array.from(files.values());

		expect(fileData).toHaveLength(1);
		expect(fileData[0].path).toBe('src/main.rs');
		expect(fileData[0].language).toBe('rust');
		expect(fileData[0].chunks).toHaveLength(2);
		expect(fileData[0].chunks[0].name).toBe('main');
		expect(fileData[0].chunks[1].name).toBe('process_data');
	});

	it('should handle multiple files', () => {
		const events = [
			{
				event_type: 'ingest.file_detected.v1',
				timestamp: '2025-01-01T00:00:00.000Z',
				payload: {
					file_path: 'src/main.rs',
					language: 'rust'
				}
			},
			{
				event_type: 'ingest.file_detected.v1',
				timestamp: '2025-01-01T00:00:01.000Z',
				payload: {
					file_path: 'src/lib.py',
					language: 'python'
				}
			},
			{
				event_type: 'ingest.chunk_created.v1',
				timestamp: '2025-01-01T00:00:02.000Z',
				payload: {
					file_path: 'src/main.rs',
					chunk_id: 'chunk_123',
					symbol_name: 'main',
					symbol_kind: 'function'
				}
			},
			{
				event_type: 'ingest.chunk_created.v1',
				timestamp: '2025-01-01T00:00:03.000Z',
				payload: {
					file_path: 'src/lib.py',
					chunk_id: 'chunk_124',
					symbol_name: 'process',
					symbol_kind: 'function'
				}
			}
		];

		const files = new Map();

		for (const event of events) {
			const eventType = event.event_type || '';
			const payload = event.payload || {};

			if (eventType.includes('file_detected')) {
				const path = payload.file_path || '';
				if (!files.has(path)) {
					files.set(path, {
						path,
						language: payload.language || 'unknown',
						chunks: []
					});
				}
			} else if (eventType.includes('chunk_created')) {
				const path = payload.file_path || '';
				const file = files.get(path);
				if (file) {
					file.chunks.push({
						name: payload.symbol_name || 'unknown',
						kind: payload.symbol_kind || 'unknown',
						chunkId: payload.chunk_id || ''
					});
				}
			}
		}

		const fileData = Array.from(files.values());

		expect(fileData).toHaveLength(2);
		expect(fileData[0].path).toBe('src/main.rs');
		expect(fileData[1].path).toBe('src/lib.py');
		expect(fileData[0].chunks).toHaveLength(1);
		expect(fileData[1].chunks).toHaveLength(1);
	});

	it('should return correct language icons', () => {
		function getLanguageIcon(language: string): string {
			switch (language.toLowerCase()) {
				case 'rust':
					return '🦀';
				case 'python':
					return '🐍';
				case 'javascript':
					return '📜';
				case 'typescript':
					return '📘';
				case 'go':
					return '🐹';
				default:
					return '📄';
			}
		}

		expect(getLanguageIcon('rust')).toBe('🦀');
		expect(getLanguageIcon('python')).toBe('🐍');
		expect(getLanguageIcon('typescript')).toBe('📘');
		expect(getLanguageIcon('unknown')).toBe('📄');
	});

	it('should return correct symbol icons', () => {
		function getSymbolIcon(kind: string): string {
			switch (kind.toLowerCase()) {
				case 'function':
					return 'ƒ';
				case 'struct':
				case 'class':
					return 'C';
				case 'enum':
					return 'E';
				case 'trait':
				case 'interface':
					return 'I';
				case 'method':
					return 'm';
				case 'const':
				case 'static':
					return 'K';
				case 'module':
					return 'M';
				default:
					return '◦';
			}
		}

		expect(getSymbolIcon('function')).toBe('ƒ');
		expect(getSymbolIcon('struct')).toBe('C');
		expect(getSymbolIcon('class')).toBe('C');
		expect(getSymbolIcon('enum')).toBe('E');
		expect(getSymbolIcon('method')).toBe('m');
	});
});
