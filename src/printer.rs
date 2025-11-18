#[cfg(not(target_arch = "wasm32"))]
use std::net::{TcpStream, IpAddr};
#[cfg(not(target_arch = "wasm32"))]
use std::io::Write;
#[cfg(not(target_arch = "wasm32"))]
use std::time::Duration;

#[derive(Debug, Clone, PartialEq)]
pub enum ConnectionType {
    Tcp,
    Serial,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ZplPrinter {
    pub name: String,
    pub connection_type: ConnectionType,
    pub ip: String,
    pub port: u16,
    pub serial_port: String,
    pub baud_rate: u32,
}

#[cfg(not(target_arch = "wasm32"))]
impl ZplPrinter {
    pub fn new_tcp(ip: String, port: u16) -> Self {
        Self {
            name: format!("ZPL Printer @ {}", ip),
            connection_type: ConnectionType::Tcp,
            ip,
            port,
            serial_port: String::new(),
            baud_rate: 9600,
        }
    }

    pub fn new_serial(port_name: String, baud_rate: u32) -> Self {
        Self {
            name: format!("ZPL Printer @ {}", port_name),
            connection_type: ConnectionType::Serial,
            ip: String::new(),
            port: 0,
            serial_port: port_name,
            baud_rate,
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub fn scan_for_printers() -> Vec<ZplPrinter> {
    use std::net::SocketAddr;

    let mut printers = Vec::new();

    let local_ip = match local_ip_address::local_ip() {
        Ok(ip) => ip,
        Err(_) => return printers,
    };

    let base_ip = match local_ip {
        IpAddr::V4(ipv4) => {
            let octets = ipv4.octets();
            format!("{}.{}.{}", octets[0], octets[1], octets[2])
        }
        _ => return printers,
    };

    for i in 1..255 {
        let ip = format!("{}.{}", base_ip, i);
        let addr = match format!("{}:9100", ip).parse::<SocketAddr>() {
            Ok(addr) => addr,
            Err(_) => continue,
        };

        match TcpStream::connect_timeout(&addr, Duration::from_millis(100)) {
            Ok(_) => {
                printers.push(ZplPrinter::new_tcp(ip, 9100));
            }
            Err(_) => continue,
        }
    }

    printers
}

#[cfg(not(target_arch = "wasm32"))]
pub fn scan_for_serial_printers() -> Vec<ZplPrinter> {
    use std::io::Read;

    let mut printers = Vec::new();

    let ports = match serialport::available_ports() {
        Ok(ports) => ports,
        Err(_) => return printers,
    };

    let baud_rates = [9600, 19200, 115200];

    for port_info in ports {
        let port_name = port_info.port_name.clone();

        for &baud_rate in &baud_rates {
            if let Ok(mut port) = serialport::new(&port_name, baud_rate)
                .timeout(Duration::from_millis(500))
                .open()
            {
                let _ = port.clear(serialport::ClearBuffer::Input);

                if port.write_all(b"~HI\r\n").is_ok() && port.flush().is_ok() {
                    std::thread::sleep(Duration::from_millis(100));

                    let mut buffer = [0u8; 1024];
                    let mut total_read = 0;

                    for _ in 0..3 {
                        if let Ok(bytes_read) = port.read(&mut buffer[total_read..]) {
                            total_read += bytes_read;
                            if bytes_read == 0 {
                                break;
                            }
                        } else {
                            break;
                        }
                    }

                    if total_read > 0 {
                        let response = String::from_utf8_lossy(&buffer[..total_read]);
                        let response_lower = response.to_lowercase();
                        if response_lower.contains("zebra")
                            || (response_lower.contains("zpl") || response_lower.contains("epl"))
                            || (response.contains("S/N") && response.contains("MODEL")) {
                            printers.push(ZplPrinter::new_serial(port_name.clone(), baud_rate));
                            break;
                        }
                    }
                }
            }
        }
    }

    printers
}

#[cfg(target_arch = "wasm32")]
pub fn scan_for_printers() -> Vec<ZplPrinter> {
    Vec::new()
}

#[cfg(target_arch = "wasm32")]
pub fn scan_for_serial_printers() -> Vec<ZplPrinter> {
    Vec::new()
}

#[cfg(not(target_arch = "wasm32"))]
pub fn send_to_printer(printer: &ZplPrinter, zpl: &str) -> Result<(), String> {
    match printer.connection_type {
        ConnectionType::Tcp => send_to_tcp_printer(printer, zpl),
        ConnectionType::Serial => send_to_serial_printer(printer, zpl),
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn send_to_tcp_printer(printer: &ZplPrinter, zpl: &str) -> Result<(), String> {
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

#[cfg(not(target_arch = "wasm32"))]
fn send_to_serial_printer(printer: &ZplPrinter, zpl: &str) -> Result<(), String> {
    let mut port = serialport::new(&printer.serial_port, printer.baud_rate)
        .timeout(Duration::from_secs(5))
        .open()
        .map_err(|e| format!("Failed to open serial port: {}", e))?;

    port
        .write_all(zpl.as_bytes())
        .map_err(|e| format!("Failed to send data: {}", e))?;

    port
        .flush()
        .map_err(|e| format!("Failed to flush data: {}", e))?;

    Ok(())
}

#[cfg(target_arch = "wasm32")]
pub fn send_to_printer(_printer: &ZplPrinter, _zpl: &str) -> Result<(), String> {
    Err("Printer support is not available in WASM".to_string())
}
