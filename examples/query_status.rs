use zebras::{
    printer::{ZplPrinter, query_printer},
    printer_status::{PrinterInfo, PrinterStatus},
};

fn main() -> Result<(), String> {
    println!("Zebras Library - Printer Status Query Example");
    println!("==============================================\n");

    let printer_ip = "10.73.27.7";
    let printer = ZplPrinter::new(printer_ip.to_string(), 9100);

    println!("Querying printer at {}...\n", printer_ip);

    println!("1. Printer Status (Errors/Warnings):");
    match query_printer(&printer, "~HQES\r\n") {
        Ok(response) => match PrinterStatus::parse(&response) {
            Ok(status) => {
                if status.is_ok() {
                    println!("   ✓ Printer is operating normally");
                } else {
                    if !status.errors.is_empty() {
                        println!("   Errors:");
                        for error in status.errors.to_descriptions() {
                            println!("     - {}", error);
                        }
                    }
                    if !status.warnings.is_empty() {
                        println!("   Warnings:");
                        for warning in status.warnings.to_descriptions() {
                            println!("     - {}", warning);
                        }
                    }
                }
            }
            Err(e) => println!("   Parse error: {}", e),
        },
        Err(e) => println!("   Query error: {}", e),
    }

    println!("\n2. Serial Number:");
    match query_printer(&printer, "~HQSN\r\n") {
        Ok(response) => {
            if let Some(serial) = PrinterInfo::parse_serial_number(&response) {
                println!("   {}", serial);
            } else {
                println!("   Unable to parse");
            }
        }
        Err(e) => println!("   Query error: {}", e),
    }

    println!("\n3. Memory Status:");
    match query_printer(&printer, "~HM\r\n") {
        Ok(response) => {
            if let Some(memory) = PrinterInfo::parse_memory_status(&response) {
                println!("   Total RAM: {} KB", memory.total_ram_kb);
                println!("   Maximum Available: {} KB", memory.max_available_kb);
                println!("   Currently Available: {} KB", memory.current_available_kb);

                let used = memory
                    .max_available_kb
                    .saturating_sub(memory.current_available_kb);
                let percent = if memory.max_available_kb > 0 {
                    (used as f32 / memory.max_available_kb as f32) * 100.0
                } else {
                    0.0
                };
                println!("   Memory Usage: {:.1}%", percent);
            } else {
                println!("   Unable to parse");
            }
        }
        Err(e) => println!("   Query error: {}", e),
    }

    println!("\n4. Firmware Version:");
    match query_printer(&printer, "~HQFW\r\n") {
        Ok(response) => {
            if let Some(firmware) = PrinterInfo::parse_firmware_version(&response) {
                println!("   {}", firmware);
            } else {
                println!("   Unable to parse");
            }
        }
        Err(e) => println!("   Query error: {}", e),
    }

    Ok(())
}
