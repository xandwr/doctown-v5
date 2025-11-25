use std::collections::HashMap;
use std::io::{self, Cursor, Read, Seek, SeekFrom, Write};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum EmbeddingsError {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),

    #[error("Invalid header: expected magic bytes")]
    InvalidHeader,

    #[error("Invalid dimensions: got {0}, expected {1}")]
    InvalidDimensions(u32, u32),

    #[error("Chunk ID not found: {0}")]
    ChunkNotFound(String),
}

pub type Result<T> = std::result::Result<T, EmbeddingsError>;

const MAGIC: &[u8; 8] = b"DOCTEMBD"; // DOCTown EMBeDdings
const VERSION: u32 = 1;

/// Header for embeddings binary format
///
/// Format:
/// - 8 bytes: magic ("DOCTEMBD")
/// - 4 bytes: version (u32, little-endian)
/// - 4 bytes: num_vectors (u32, little-endian)
/// - 4 bytes: dimensions (u32, little-endian)
/// - 4 bytes: index_offset (u32, little-endian) - offset to index section
///
/// Followed by:
/// - num_vectors * dimensions * 4 bytes: f32 vectors (little-endian)
/// - Index section: chunk_id → vector offset mapping
#[derive(Debug, Clone)]
pub struct EmbeddingsHeader {
    pub version: u32,
    pub num_vectors: u32,
    pub dimensions: u32,
    pub index_offset: u32,
}

impl EmbeddingsHeader {
    pub fn new(num_vectors: u32, dimensions: u32, index_offset: u32) -> Self {
        Self {
            version: VERSION,
            num_vectors,
            dimensions,
            index_offset,
        }
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        writer.write_all(MAGIC)?;
        writer.write_all(&self.version.to_le_bytes())?;
        writer.write_all(&self.num_vectors.to_le_bytes())?;
        writer.write_all(&self.dimensions.to_le_bytes())?;
        writer.write_all(&self.index_offset.to_le_bytes())?;
        Ok(())
    }

    pub fn read<R: Read>(reader: &mut R) -> Result<Self> {
        let mut magic_buf = [0u8; 8];
        reader.read_exact(&mut magic_buf)?;
        if &magic_buf != MAGIC {
            return Err(EmbeddingsError::InvalidHeader);
        }

        let mut u32_buf = [0u8; 4];

        reader.read_exact(&mut u32_buf)?;
        let version = u32::from_le_bytes(u32_buf);

        reader.read_exact(&mut u32_buf)?;
        let num_vectors = u32::from_le_bytes(u32_buf);

        reader.read_exact(&mut u32_buf)?;
        let dimensions = u32::from_le_bytes(u32_buf);

        reader.read_exact(&mut u32_buf)?;
        let index_offset = u32::from_le_bytes(u32_buf);

        Ok(Self {
            version,
            num_vectors,
            dimensions,
            index_offset,
        })
    }

    pub fn size() -> usize {
        8 + 4 + 4 + 4 + 4 // magic + version + num_vectors + dimensions + index_offset
    }
}

/// Writer for embeddings binary format
pub struct EmbeddingsWriter {
    dimensions: u32,
    vectors: Vec<(String, Vec<f32>)>, // (chunk_id, vector)
}

impl EmbeddingsWriter {
    pub fn new(dimensions: u32) -> Self {
        Self {
            dimensions,
            vectors: Vec::new(),
        }
    }

    /// Add a vector for a chunk
    pub fn add_vector(&mut self, chunk_id: String, vector: Vec<f32>) -> Result<()> {
        if vector.len() != self.dimensions as usize {
            return Err(EmbeddingsError::InvalidDimensions(
                vector.len() as u32,
                self.dimensions,
            ));
        }
        self.vectors.push((chunk_id, vector));
        Ok(())
    }

    /// Write to bytes
    pub fn write(&self) -> Result<Vec<u8>> {
        let mut buffer = Vec::new();

        // Calculate index offset
        let header_size = EmbeddingsHeader::size();
        let vectors_size = self.vectors.len() * self.dimensions as usize * 4;
        let index_offset = (header_size + vectors_size) as u32;

        // Write header
        let header =
            EmbeddingsHeader::new(self.vectors.len() as u32, self.dimensions, index_offset);
        header.write(&mut buffer)?;

        // Write vectors
        for (_, vector) in &self.vectors {
            for &value in vector {
                buffer.write_all(&value.to_le_bytes())?;
            }
        }

        // Write index: chunk_id → byte offset
        // Format: u32 num_entries, then for each entry:
        //   u32 chunk_id_len, chunk_id bytes, u32 offset
        buffer.write_all(&(self.vectors.len() as u32).to_le_bytes())?;

        let mut offset = EmbeddingsHeader::size() as u32;
        for (chunk_id, vector) in &self.vectors {
            let chunk_id_bytes = chunk_id.as_bytes();
            buffer.write_all(&(chunk_id_bytes.len() as u32).to_le_bytes())?;
            buffer.write_all(chunk_id_bytes)?;
            buffer.write_all(&offset.to_le_bytes())?;
            offset += (vector.len() * 4) as u32;
        }

        Ok(buffer)
    }
}

/// Reader for embeddings binary format
pub struct EmbeddingsReader {
    data: Vec<u8>,
    header: EmbeddingsHeader,
    index: HashMap<String, u32>, // chunk_id → byte offset
}

impl EmbeddingsReader {
    /// Read from bytes
    pub fn read(data: Vec<u8>) -> Result<Self> {
        let mut cursor = Cursor::new(&data);

        // Read header
        let header = EmbeddingsHeader::read(&mut cursor)?;

        // Read index
        cursor.seek(SeekFrom::Start(header.index_offset as u64))?;
        let mut u32_buf = [0u8; 4];
        cursor.read_exact(&mut u32_buf)?;
        let num_entries = u32::from_le_bytes(u32_buf);

        let mut index = HashMap::new();
        for _ in 0..num_entries {
            // Read chunk_id length
            cursor.read_exact(&mut u32_buf)?;
            let chunk_id_len = u32::from_le_bytes(u32_buf);

            // Read chunk_id
            let mut chunk_id_bytes = vec![0u8; chunk_id_len as usize];
            cursor.read_exact(&mut chunk_id_bytes)?;
            let chunk_id = String::from_utf8_lossy(&chunk_id_bytes).to_string();

            // Read offset
            cursor.read_exact(&mut u32_buf)?;
            let offset = u32::from_le_bytes(u32_buf);

            index.insert(chunk_id, offset);
        }

        Ok(Self {
            data,
            header,
            index,
        })
    }

    /// Get header information
    pub fn header(&self) -> &EmbeddingsHeader {
        &self.header
    }

    /// Get number of vectors
    pub fn num_vectors(&self) -> usize {
        self.header.num_vectors as usize
    }

    /// Get dimensions
    pub fn dimensions(&self) -> usize {
        self.header.dimensions as usize
    }

    /// Get vector by chunk_id
    pub fn get_vector(&self, chunk_id: &str) -> Result<Vec<f32>> {
        let offset = self
            .index
            .get(chunk_id)
            .ok_or_else(|| EmbeddingsError::ChunkNotFound(chunk_id.to_string()))?;

        let start = *offset as usize;
        let end = start + (self.header.dimensions as usize * 4);

        if end > self.data.len() {
            return Err(EmbeddingsError::Io(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "Vector data truncated",
            )));
        }

        let mut vector = Vec::with_capacity(self.header.dimensions as usize);
        for i in (start..end).step_by(4) {
            let bytes = [
                self.data[i],
                self.data[i + 1],
                self.data[i + 2],
                self.data[i + 3],
            ];
            vector.push(f32::from_le_bytes(bytes));
        }

        Ok(vector)
    }

    /// Check if chunk_id exists
    pub fn has_chunk(&self, chunk_id: &str) -> bool {
        self.index.contains_key(chunk_id)
    }

    /// Get all chunk IDs
    pub fn chunk_ids(&self) -> Vec<String> {
        self.index.keys().cloned().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_header_roundtrip() {
        let header = EmbeddingsHeader::new(100, 384, 1000);
        let mut buffer = Vec::new();
        header.write(&mut buffer).unwrap();

        let mut cursor = Cursor::new(&buffer);
        let read_header = EmbeddingsHeader::read(&mut cursor).unwrap();

        assert_eq!(read_header.version, VERSION);
        assert_eq!(read_header.num_vectors, 100);
        assert_eq!(read_header.dimensions, 384);
        assert_eq!(read_header.index_offset, 1000);
    }

    #[test]
    fn test_embeddings_roundtrip() {
        let mut writer = EmbeddingsWriter::new(384);

        // Add some test vectors
        for i in 0..10 {
            let chunk_id = format!("chunk_{}", i);
            let vector: Vec<f32> = (0..384).map(|j| (i * 100 + j) as f32 * 0.01).collect();
            writer.add_vector(chunk_id, vector).unwrap();
        }

        // Write to bytes
        let bytes = writer.write().unwrap();

        // Read back
        let reader = EmbeddingsReader::read(bytes).unwrap();

        assert_eq!(reader.num_vectors(), 10);
        assert_eq!(reader.dimensions(), 384);

        // Verify vectors
        for i in 0..10 {
            let chunk_id = format!("chunk_{}", i);
            let vector = reader.get_vector(&chunk_id).unwrap();
            assert_eq!(vector.len(), 384);
            assert_eq!(vector[0], (i * 100) as f32 * 0.01);
        }
    }

    #[test]
    fn test_invalid_dimensions() {
        let mut writer = EmbeddingsWriter::new(384);
        let result = writer.add_vector("chunk_1".to_string(), vec![1.0, 2.0, 3.0]);
        assert!(matches!(
            result,
            Err(EmbeddingsError::InvalidDimensions(_, _))
        ));
    }

    #[test]
    fn test_chunk_not_found() {
        let writer = EmbeddingsWriter::new(384);
        let bytes = writer.write().unwrap();
        let reader = EmbeddingsReader::read(bytes).unwrap();

        let result = reader.get_vector("nonexistent");
        assert!(matches!(result, Err(EmbeddingsError::ChunkNotFound(_))));
    }

    #[test]
    fn test_random_access() {
        let mut writer = EmbeddingsWriter::new(3);
        writer
            .add_vector("a".to_string(), vec![1.0, 2.0, 3.0])
            .unwrap();
        writer
            .add_vector("b".to_string(), vec![4.0, 5.0, 6.0])
            .unwrap();
        writer
            .add_vector("c".to_string(), vec![7.0, 8.0, 9.0])
            .unwrap();

        let bytes = writer.write().unwrap();
        let reader = EmbeddingsReader::read(bytes).unwrap();

        // Access in different order
        assert_eq!(reader.get_vector("c").unwrap(), vec![7.0, 8.0, 9.0]);
        assert_eq!(reader.get_vector("a").unwrap(), vec![1.0, 2.0, 3.0]);
        assert_eq!(reader.get_vector("b").unwrap(), vec![4.0, 5.0, 6.0]);
    }
}
