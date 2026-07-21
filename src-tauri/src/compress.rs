use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression;
use std::fs::File;
use std::io::{self, BufRead, BufReader, BufWriter, Read, Write};
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

/// Stream a gzip file as bounded, owned lines.
pub fn decompressed_lines(
    gz_path: &Path,
    max_bytes: u64,
    max_line_bytes: usize,
) -> Result<impl Iterator<Item = Result<String, io::Error>>, io::Error> {
    let file = File::open(gz_path)?;
    let decoder = GzDecoder::new(BufReader::new(file));
    let mut reader = BufReader::with_capacity(64 * 1024, decoder);
    let mut line = Vec::new();
    let mut total_bytes = 0u64;
    let mut finished = false;

    Ok(std::iter::from_fn(move || {
        if finished {
            return None;
        }

        line.clear();
        let mut read_any = false;

        loop {
            let buffer = match reader.fill_buf() {
                Ok(buffer) => buffer,
                Err(error) => {
                    finished = true;
                    return Some(Err(error));
                }
            };
            if buffer.is_empty() {
                finished = true;
                break;
            }

            read_any = true;
            let newline = buffer.iter().position(|byte| *byte == b'\n');
            let consumed = newline.map_or(buffer.len(), |position| position + 1);
            let content_bytes = newline.unwrap_or(consumed);

            let Some(next_total) = total_bytes.checked_add(consumed as u64) else {
                finished = true;
                return Some(Err(io::Error::other("Decompressed snapshot size overflow")));
            };
            total_bytes = next_total;
            if total_bytes > max_bytes {
                finished = true;
                return Some(Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("Decompressed snapshot exceeds the {max_bytes}-byte inspection limit"),
                )));
            }

            if line.len().saturating_add(content_bytes) > max_line_bytes {
                finished = true;
                return Some(Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("Snapshot line exceeds the {max_line_bytes}-byte inspection limit"),
                )));
            }

            line.extend_from_slice(&buffer[..content_bytes]);
            reader.consume(consumed);

            if newline.is_some() {
                break;
            }
        }

        if !read_any {
            return None;
        }

        if line.last() == Some(&b'\r') {
            line.pop();
        }
        Some(Ok(String::from_utf8_lossy(&line).into_owned()))
    }))
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn streams_decompressed_lines_with_size_limits() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("snapshot.sql.gz");
        compress_from_reader(std::io::Cursor::new(b"one\ntwo\n"), &path).unwrap();

        let lines = decompressed_lines(&path, 8, 3)
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        assert_eq!(lines, ["one", "two"]);

        assert!(decompressed_lines(&path, 7, 3)
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .is_err());
        assert!(decompressed_lines(&path, 8, 2)
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .is_err());
    }
}
