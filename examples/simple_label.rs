use zebras::{
    printer::{ZplPrinter, send_to_printer},
    zpl::{ZplCommand, FontOrientation, commands_to_zpl},
};

fn main() -> Result<(), String> {
    let commands = vec![
        ZplCommand::StartFormat,
        ZplCommand::FieldOrigin { x: 50, y: 50 },
        ZplCommand::Font {
            orientation: FontOrientation::Normal,
            height: 50,
            width: 50,
        },
        ZplCommand::FieldData {
            data: "Hello from Zebras Library!".to_string(),
        },
        ZplCommand::FieldSeparator,
        ZplCommand::FieldOrigin { x: 50, y: 150 },
        ZplCommand::GraphicBox {
            width: 400,
            height: 3,
            thickness: 3,
            color: None,
            rounding: None,
        },
        ZplCommand::FieldSeparator,
        ZplCommand::FieldOrigin { x: 50, y: 200 },
        ZplCommand::Font {
            orientation: FontOrientation::Normal,
            height: 30,
            width: 30,
        },
        ZplCommand::FieldData {
            data: "Programmatic ZPL Generation".to_string(),
        },
        ZplCommand::FieldSeparator,
        ZplCommand::EndFormat,
    ];

    let zpl = commands_to_zpl(&commands);

    println!("Generated ZPL:");
    println!("{}", zpl);
    println!();

    println!("To send to printer, uncomment the code below:");
    println!("// let printer = ZplPrinter::new(\"10.73.27.7\".to_string(), 9100);");
    println!("// send_to_printer(&printer, &zpl)?;");

    Ok(())
}
