use std::env;

pub struct MultiWriter {
    file: Option<std::fs::File>,
}

impl MultiWriter {
    pub fn new() -> Self {
        let log_path =
            env::var("LOG_FILE_PATH").unwrap_or_else(|_| "/var/log/jito-bell/app.log".to_string());

        let file = match std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_path)
        {
            Ok(file) => Some(file),
            Err(e) => {
                eprintln!("Failed to open log file {}: {}", log_path, e);
                None
            }
        };

        Self { file }
    }
}

impl std::io::Write for MultiWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        // Always write to stdout
        std::io::stdout().write(buf)?;

        // Also write to file if available
        if let Some(file) = &mut self.file {
            file.write(buf)?;
        }

        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        std::io::stdout().flush()?;
        if let Some(file) = &mut self.file {
            file.flush()?;
        }
        Ok(())
    }
}
