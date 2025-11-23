import { describe, it, expect } from 'vitest';

describe('SymbolList component logic', () => {
	it('should organize symbols from events', () => {
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
					symbol_kind: 'function',
					symbol_signature: 'fn main()'
				}
			},
			{
				event_type: 'ingest.chunk_created.v1',
				timestamp: '2025-01-01T00:00:02.000Z',
				payload: {
					file_path: 'src/main.rs',
					chunk_id: 'chunk_124',
					symbol_name: 'Config',
					symbol_kind: 'struct',
					symbol_signature: 'struct Config { ... }'
				}
			}
		];

		// Simulate the derived logic from component
		const symbols: Array<{
			name: string;
			kind: string;
			signature: string;
			filePath: string;
			language: string;
			chunkId: string;
		}> = [];

		const fileMetadata = new Map<string, { language: string }>();

		for (const event of events) {
			const eventType = event.event_type || '';
			const payload = event.payload || {};

			if (eventType.includes('file_detected')) {
				fileMetadata.set(payload.file_path, {
					language: payload.language || 'unknown'
				});
			} else if (eventType.includes('chunk_created')) {
				const filePath = payload.file_path || 'unknown';
				const fileInfo = fileMetadata.get(filePath);

				symbols.push({
					name: payload.symbol_name || 'unnamed',
					kind: payload.symbol_kind || 'unknown',
					signature: payload.symbol_signature || '',
					filePath,
					language: fileInfo?.language || 'unknown',
					chunkId: payload.chunk_id || ''
				});
			}
		}

		expect(symbols).toHaveLength(2);
		expect(symbols[0].name).toBe('main');
		expect(symbols[0].kind).toBe('function');
		expect(symbols[0].signature).toBe('fn main()');
		expect(symbols[0].language).toBe('rust');
		expect(symbols[1].name).toBe('Config');
		expect(symbols[1].kind).toBe('struct');
	});

	it('should group symbols by file', () => {
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
					symbol_kind: 'function',
					symbol_signature: 'fn main()'
				}
			},
			{
				event_type: 'ingest.chunk_created.v1',
				timestamp: '2025-01-01T00:00:03.000Z',
				payload: {
					file_path: 'src/lib.py',
					chunk_id: 'chunk_124',
					symbol_name: 'process',
					symbol_kind: 'function',
					symbol_signature: 'def process():'
				}
			}
		];

		const symbols: Array<{
			name: string;
			kind: string;
			signature: string;
			filePath: string;
			language: string;
			chunkId: string;
		}> = [];

		const fileMetadata = new Map<string, { language: string }>();

		for (const event of events) {
			const eventType = event.event_type || '';
			const payload = event.payload || {};

			if (eventType.includes('file_detected')) {
				fileMetadata.set(payload.file_path, {
					language: payload.language || 'unknown'
				});
			} else if (eventType.includes('chunk_created')) {
				const filePath = payload.file_path || 'unknown';
				const fileInfo = fileMetadata.get(filePath);

				symbols.push({
					name: payload.symbol_name || 'unnamed',
					kind: payload.symbol_kind || 'unknown',
					signature: payload.symbol_signature || '',
					filePath,
					language: fileInfo?.language || 'unknown',
					chunkId: payload.chunk_id || ''
				});
			}
		}

		// Group by file
		const grouped = new Map<string, typeof symbols>();
		for (const symbol of symbols) {
			if (!grouped.has(symbol.filePath)) {
				grouped.set(symbol.filePath, []);
			}
			grouped.get(symbol.filePath)!.push(symbol);
		}

		const symbolData = Array.from(grouped.entries()).map(([filePath, symbols]) => ({
			filePath,
			symbols
		}));

		expect(symbolData).toHaveLength(2);
		expect(symbolData[0].filePath).toBe('src/main.rs');
		expect(symbolData[1].filePath).toBe('src/lib.py');
		expect(symbolData[0].symbols).toHaveLength(1);
		expect(symbolData[1].symbols).toHaveLength(1);
	});

	it('should filter symbols by search query', () => {
		const symbols = [
			{ name: 'main', kind: 'function', signature: 'fn main()', filePath: 'src/main.rs', language: 'rust', chunkId: 'c1' },
			{ name: 'process', kind: 'function', signature: 'fn process()', filePath: 'src/lib.rs', language: 'rust', chunkId: 'c2' },
			{ name: 'Config', kind: 'struct', signature: 'struct Config', filePath: 'src/config.rs', language: 'rust', chunkId: 'c3' }
		];

		const searchQuery = 'proc';
		const filtered = symbols.filter(symbol =>
			symbol.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
			symbol.signature.toLowerCase().includes(searchQuery.toLowerCase()) ||
			symbol.filePath.toLowerCase().includes(searchQuery.toLowerCase())
		);

		expect(filtered).toHaveLength(1);
		expect(filtered[0].name).toBe('process');
	});

	it('should filter symbols by kind', () => {
		const symbols = [
			{ name: 'main', kind: 'function', signature: 'fn main()', filePath: 'src/main.rs', language: 'rust', chunkId: 'c1' },
			{ name: 'process', kind: 'function', signature: 'fn process()', filePath: 'src/lib.rs', language: 'rust', chunkId: 'c2' },
			{ name: 'Config', kind: 'struct', signature: 'struct Config', filePath: 'src/config.rs', language: 'rust', chunkId: 'c3' }
		];

		const selectedKind: string = 'struct';
		const filtered = symbols.filter(symbol =>
			selectedKind === 'all' || symbol.kind.toLowerCase() === selectedKind.toLowerCase()
		);

		expect(filtered).toHaveLength(1);
		expect(filtered[0].name).toBe('Config');
		expect(filtered[0].kind).toBe('struct');
	});

	it('should return correct symbol color classes', () => {
		function getSymbolColor(kind: string): string {
			switch (kind.toLowerCase()) {
				case 'function':
				case 'method':
					return 'bg-purple-100 text-purple-700 border-purple-300';
				case 'struct':
				case 'class':
					return 'bg-blue-100 text-blue-700 border-blue-300';
				case 'enum':
					return 'bg-orange-100 text-orange-700 border-orange-300';
				case 'trait':
				case 'interface':
					return 'bg-green-100 text-green-700 border-green-300';
				case 'const':
				case 'static':
					return 'bg-gray-100 text-gray-700 border-gray-300';
				case 'module':
					return 'bg-cyan-100 text-cyan-700 border-cyan-300';
				default:
					return 'bg-gray-100 text-gray-500 border-gray-300';
			}
		}

		expect(getSymbolColor('function')).toContain('purple');
		expect(getSymbolColor('struct')).toContain('blue');
		expect(getSymbolColor('enum')).toContain('orange');
		expect(getSymbolColor('trait')).toContain('green');
	});

	it('should extract unique symbol kinds', () => {
		const symbolData = [
			{
				filePath: 'src/main.rs',
				symbols: [
					{ name: 'main', kind: 'function', signature: '', filePath: '', language: '', chunkId: '' },
					{ name: 'Config', kind: 'struct', signature: '', filePath: '', language: '', chunkId: '' }
				]
			},
			{
				filePath: 'src/lib.rs',
				symbols: [
					{ name: 'process', kind: 'function', signature: '', filePath: '', language: '', chunkId: '' },
					{ name: 'State', kind: 'enum', signature: '', filePath: '', language: '', chunkId: '' }
				]
			}
		];

		const kinds = new Set<string>();
		symbolData.forEach(file => {
			file.symbols.forEach(symbol => {
				kinds.add(symbol.kind);
			});
		});

		const symbolKinds = Array.from(kinds).sort();

		expect(symbolKinds).toEqual(['enum', 'function', 'struct']);
	});
});
