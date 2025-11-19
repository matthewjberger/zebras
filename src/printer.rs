#[cfg(not(target_arch = "wasm32"))]
use std::net::TcpStream;
#[cfg(not(target_arch = "wasm32"))]
use std::io::{Write, Read};
#[cfg(not(target_arch = "wasm32"))]
use std::time::Duration;

#[derive(Debug, Clone, PartialEq)]
pub struct ZplPrinter {
    pub name: String,
    pub ip: String,
    pub port: u16,
}

#[cfg(not(target_arch = "wasm32"))]
impl ZplPrinter {
    pub fn new(ip: String, port: u16) -> Self {
        Self {
            name: format!("ZPL Printer @ {}", ip),
            ip,
            port,
        }
    }
}


#[cfg(not(target_arch = "wasm32"))]
pub fn send_to_printer(printer: &ZplPrinter, zpl: &str) -> Result<(), String> {
    let addr = format!("{}:{}", printer.ip, printer.port);

    let mut stream = TcpStream::connect_timeout(
        &addr.parse().map_err(|e| format!("Invalid address: {}", e))?,
        Duration::from_secs(5),
    )
    .map_err(|e| format!("Failed to connect to printer: {}", e))?;

    stream
        .set_write_timeout(Some(Duration::from_secs(5)))
        .map_err(|e| format!("Failed to set timeout: {}", e))?;

    stream
        .write_all(zpl.as_bytes())
        .map_err(|e| format!("Failed to send data: {}", e))?;

    stream
        .flush()
        .map_err(|e| format!("Failed to flush data: {}", e))?;

    Ok(())
}

#[cfg(target_arch = "wasm32")]
pub fn send_to_printer(_printer: &ZplPrinter, _zpl: &str) -> Result<(), String> {
    Err("Printer support is not available in WASM".to_string())
}

#[cfg(not(target_arch = "wasm32"))]
pub fn query_printer(printer: &ZplPrinter, query: &str) -> Result<String, String> {
    let addr = format!("{}:{}", printer.ip, printer.port);

    let mut stream = TcpStream::connect_timeout(
        &addr.parse().map_err(|e| format!("Invalid address: {}", e))?,
        Duration::from_secs(5),
    )
    .map_err(|e| format!("Failed to connect to printer: {}", e))?;

    stream
        .set_write_timeout(Some(Duration::from_secs(5)))
        .map_err(|e| format!("Failed to set timeout: {}", e))?;

    stream
        .set_read_timeout(Some(Duration::from_secs(5)))
        .map_err(|e| format!("Failed to set read timeout: {}", e))?;

    stream
        .write_all(query.as_bytes())
        .map_err(|e| format!("Failed to send query: {}", e))?;

    stream
        .flush()
        .map_err(|e| format!("Failed to flush: {}", e))?;

    let mut buffer = Vec::new();
    let mut temp_buffer = [0u8; 4096];
    let start_time = std::time::Instant::now();

    loop {
        match stream.read(&mut temp_buffer) {
            Ok(0) => break,
            Ok(bytes_read) => {
                buffer.extend_from_slice(&temp_buffer[..bytes_read]);
                if buffer.len() > 0 && buffer.contains(&0x03) {
                    break;
                }
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                if start_time.elapsed() > Duration::from_secs(5) {
                    break;
                }
                std::thread::sleep(Duration::from_millis(100));
                continue;
            }
            Err(e) => return Err(format!("Read error: {}", e)),
        }

        if start_time.elapsed() > Duration::from_secs(5) {
            break;
        }
    }

    if buffer.is_empty() {
        return Err("No response from printer".to_string());
    }

    let response = String::from_utf8_lossy(&buffer).to_string();
    Ok(response)
}

#[cfg(target_arch = "wasm32")]
pub fn query_printer(_printer: &ZplPrinter, _query: &str) -> Result<String, String> {
    Err("Printer support is not available in WASM".to_string())
}
