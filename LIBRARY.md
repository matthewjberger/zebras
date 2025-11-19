# Zebras Library

The Zebras crate provides a Rust library for working with Zebra ZPL printers programmatically, without requiring the GUI.

## Usage

Add to your `Cargo.toml`:

```toml
[dependencies]
zebras = { path = "../zebras" }
# or when published:
# zebras = "0.1.0"
```

## Examples

### Creating ZPL Commands

```rust
use zebras::zpl::{ZplCommand, FontOrientation, commands_to_zpl};

fn main() {
    let commands = vec![
        ZplCommand::StartFormat,
        ZplCommand::FieldOrigin { x: 50, y: 50 },
        ZplCommand::Font {
            orientation: FontOrientation::Normal,
            height: 50,
            width: 50,
        },
        ZplCommand::FieldData {
            data: "Hello, Zebra!".to_string(),
        },
        ZplCommand::FieldSeparator,
        ZplCommand::EndFormat,
    ];

    let zpl = commands_to_zpl(&commands);
    println!("{}", zpl);
}
```

### Sending to Printer

```rust
use zebras::printer::{ZplPrinter, send_to_printer};

fn main() -> Result<(), String> {
    let printer = ZplPrinter::new("10.73.27.7".to_string(), 9100);
    let zpl = "^XA^FO50,50^A0N,50,50^FDHello World^FS^XZ";

    send_to_printer(&printer, zpl)?;
    Ok(())
}
```

### Querying Printer Status

```rust
use zebras::printer::{ZplPrinter, query_printer};
use zebras::printer_status::PrinterStatus;

fn main() -> Result<(), String> {
    let printer = ZplPrinter::new("10.73.27.7".to_string(), 9100);
    let response = query_printer(&printer, "~HQES\r\n")?;

    match PrinterStatus::parse(&response) {
        Ok(status) => {
            if status.is_ok() {
                println!("Printer OK");
            } else {
                println!("Errors: {:?}", status.errors.to_descriptions());
                println!("Warnings: {:?}", status.warnings.to_descriptions());
            }
        }
        Err(e) => eprintln!("Parse error: {}", e),
    }

    Ok(())
}
```

### Rendering ZPL with Labelary API

```rust
use zebras::labelary::LabelaryClient;

fn main() -> Result<(), String> {
    let client = LabelaryClient::default();
    let zpl = "^XA^FO50,50^A0N,50,50^FDTest^FS^XZ";

    let png_bytes = client.render_sync(zpl)?;
    std::fs::write("label.png", png_bytes).unwrap();

    Ok(())
}
```

### Working with Graphics

```rust
use zebras::zpl::{ZplCommand, image_to_zpl_hex};
use image::open;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load an image
    let img = open("logo.png")?;

    // Convert to ZPL hex data
    let hex_data = image_to_zpl_hex(&img, 128);

    // Create download graphic command
    let download = ZplCommand::DownloadGraphic {
        name: "LOGO".to_string(),
        width: img.width(),
        height: img.height(),
        data: hex_data,
    };

    // Recall the graphic
    let recall = ZplCommand::RecallGraphic {
        name: "LOGO".to_string(),
        magnification_x: 2,
        magnification_y: 2,
    };

    Ok(())
}
```

### Memory Status Query

```rust
use zebras::printer::{ZplPrinter, query_printer};
use zebras::printer_status::PrinterInfo;

fn main() -> Result<(), String> {
    let printer = ZplPrinter::new("10.73.27.7".to_string(), 9100);
    let response = query_printer(&printer, "~HM\r\n")?;

    if let Some(memory) = PrinterInfo::parse_memory_status(&response) {
        println!("Total RAM: {} KB", memory.total_ram_kb);
        println!("Available: {} KB", memory.current_available_kb);

        let usage = memory.max_available_kb - memory.current_available_kb;
        let percent = (usage as f32 / memory.max_available_kb as f32) * 100.0;
        println!("Usage: {:.1}%", percent);
    }

    Ok(())
}
```

## Modules

- `zpl` - ZPL command types and serialization
- `printer` - Printer communication (send, query, scan)
- `printer_status` - Status parsing and interpretation
- `labelary` - Labelary API client for rendering ZPL to images

## Platform Support

- Full support on desktop platforms (Linux, macOS, Windows)
- Limited support on WASM (printer communication not available)
