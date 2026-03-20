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

/// Compress with progress callback — reports bytes processed during streaming compression.
/// The callback receives `(bytes_processed_so_far, current_table_name)`.
pub fn compress_from_reader_with_progress<R: Read, F>(
    reader: R,
    output: &Path,
    mut on_progress: F,
) -> Result<u64, io::Error>
where
    F: FnMut(u64, Option<&str>),
{
    let output_file = File::create(output)?;
    let writer = BufWriter::new(output_file);
    let mut encoder = GzEncoder::new(writer, Compression::default());

    let mut buf_reader = BufReader::with_capacity(64 * 1024, reader);
    let mut buffer = vec![0u8; 64 * 1024];
    let mut total_bytes: u64 = 0;
    let mut current_table: Option<String> = None;
    let mut last_report = std::time::Instant::now();

    loop {
        let bytes_read = buf_reader.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }

        // Scan for table markers in the SQL output
        let chunk = &buffer[..bytes_read];
        if let Ok(text) = std::str::from_utf8(chunk) {
            for line in text.lines() {
                let trimmed = line.trim();
                // mysqldump: -- Dumping data for table `tablename`
                // pg_dump: -- Name: tablename; Type: TABLE DATA
                if trimmed.starts_with("-- Dumping data for table") {
                    if let Some(name) = trimmed.split('`').nth(1) {
                        current_table = Some(name.to_string());
                    }
                } else if trimmed.starts_with("-- Name:") && trimmed.contains("TABLE DATA") {
                    if let Some(name) = trimmed.strip_prefix("-- Name: ") {
                        if let Some(end) = name.find(';') {
                            current_table = Some(name[..end].trim().to_string());
                        }
                    }
                }
            }
        }

        encoder.write_all(chunk)?;
        total_bytes += bytes_read as u64;

        // Report progress at most every 500ms to avoid flooding the channel
        if last_report.elapsed() >= std::time::Duration::from_millis(500) {
            on_progress(total_bytes, current_table.as_deref());
            last_report = std::time::Instant::now();
        }
    }

    // Final progress report
    on_progress(total_bytes, current_table.as_deref());

    encoder.finish()?;

    Ok(std::fs::metadata(output)?.len())
}
