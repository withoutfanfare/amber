use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression;
use std::fs::File;
use std::io::{self, BufReader, BufWriter, Read, Write};
use std::path::Path;

/// Compress a raw SQL file to .sql.gz using streaming gzip.
/// Returns the compressed file size in bytes.
pub fn compress_file(input: &Path, output: &Path) -> Result<u64, io::Error> {
    let input_file = File::open(input)?;
    let mut reader = BufReader::with_capacity(64 * 1024, input_file);

    let output_file = File::create(output)?;
    let writer = BufWriter::new(output_file);
    let mut encoder = GzEncoder::new(writer, Compression::default());

    // Stream in 64KB chunks to keep memory usage low
    io::copy(&mut reader, &mut encoder)?;

    encoder.finish()?;

    Ok(std::fs::metadata(output)?.len())
}

/// Decompress a .sql.gz file to raw SQL, streaming to a writer.
/// Used for restore: decompress -> pipe into mysql/psql stdin.
pub fn decompress_to_writer<W: Write>(gz_path: &Path, writer: &mut W) -> Result<(), io::Error> {
    let file = File::open(gz_path)?;
    let reader = BufReader::new(file);
    let mut decoder = GzDecoder::new(reader);

    // Stream in 64KB chunks
    let mut buffer = vec![0u8; 64 * 1024];
    loop {
        let bytes_read = decoder.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        writer.write_all(&buffer[..bytes_read])?;
    }

    Ok(())
}

/// Compress subprocess stdout directly to a .sql.gz file (zero intermediate file).
/// Reads from the child process stdout and streams through gzip to disk.
pub fn compress_from_reader<R: Read>(reader: R, output: &Path) -> Result<u64, io::Error> {
    let output_file = File::create(output)?;
    let writer = BufWriter::new(output_file);
    let mut encoder = GzEncoder::new(writer, Compression::default());

    let mut buf_reader = BufReader::with_capacity(64 * 1024, reader);
    io::copy(&mut buf_reader, &mut encoder)?;

    encoder.finish()?;

    Ok(std::fs::metadata(output)?.len())
}
