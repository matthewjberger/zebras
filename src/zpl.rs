use enum2str::EnumStr;
use image::{DynamicImage, GenericImageView, Rgba};

#[allow(dead_code)]
#[derive(EnumStr, Debug, Clone, Copy)]
pub enum ZplPrefix {
    #[enum2str("^")]
    Caret,
    #[enum2str("~")]
    Tilde,
}

#[allow(dead_code)]
#[derive(EnumStr, Debug, Clone, PartialEq)]
pub enum ZplCommand {
    #[enum2str("XA")]
    StartFormat,
    #[enum2str("XZ")]
    EndFormat,
    #[enum2str("FO")]
    FieldOrigin { x: u32, y: u32 },
    #[enum2str("A0")]
    Font {
        orientation: FontOrientation,
        height: u32,
        width: u32,
    },
    #[enum2str("FD")]
    FieldData { data: String },
    #[enum2str("FS")]
    FieldSeparator,
    #[enum2str("GB")]
    GraphicBox {
        width: u32,
        height: u32,
        thickness: u32,
    },
    #[enum2str("CF")]
    ChangeFont { font: String, size: u32 },
    #[enum2str("FW")]
    FieldOrientation { rotation: FieldRotation },
    #[enum2str("BY")]
    BarcodeFieldDefault { width: u32, ratio: f32, height: u32 },
    #[enum2str("BC")]
    Code128Barcode {
        orientation: FieldOrientation,
        height: u32,
        print_interpretation: bool,
        print_above: bool,
        check_digit: bool,
        mode: FieldOrientation,
    },
    #[enum2str("GFA")]
    GraphicField {
        width: u32,
        height: u32,
        data: String,
    },
    #[enum2str("MMD")]
    MediaModeDelayed,
    #[enum2str("JK")]
    CutNow,
}

impl ZplCommand {
    pub fn command_name(&self) -> &str {
        match self {
            ZplCommand::StartFormat => "Start Format (^XA)",
            ZplCommand::EndFormat => "End Format (^XZ)",
            ZplCommand::FieldOrigin { .. } => "Field Origin (^FO)",
            ZplCommand::Font { .. } => "Font (^A0)",
            ZplCommand::FieldData { .. } => "Field Data (^FD)",
            ZplCommand::FieldSeparator => "Field Separator (^FS)",
            ZplCommand::GraphicBox { .. } => "Graphic Box (^GB)",
            ZplCommand::ChangeFont { .. } => "Change Font (^CF)",
            ZplCommand::FieldOrientation { .. } => "Field Orientation (^FW)",
            ZplCommand::BarcodeFieldDefault { .. } => "Barcode Field Default (^BY)",
            ZplCommand::Code128Barcode { .. } => "Code 128 Barcode (^BC)",
            ZplCommand::GraphicField { .. } => "Graphic Field (^GFA)",
            ZplCommand::MediaModeDelayed => "Media Mode Delayed (^MMD)",
            ZplCommand::CutNow => "Cut Now (~JK)",
        }
    }

    pub fn all_command_types() -> Vec<(&'static str, ZplCommand)> {
        vec![
            ("Start Format (^XA)", ZplCommand::StartFormat),
            ("End Format (^XZ)", ZplCommand::EndFormat),
            (
                "Field Origin (^FO)",
                ZplCommand::FieldOrigin { x: 0, y: 0 },
            ),
            (
                "Font (^A0)",
                ZplCommand::Font {
                    orientation: FontOrientation::Normal,
                    height: 30,
                    width: 30,
                },
            ),
            (
                "Field Data (^FD)",
                ZplCommand::FieldData {
                    data: String::new(),
                },
            ),
            ("Field Separator (^FS)", ZplCommand::FieldSeparator),
            (
                "Graphic Box (^GB)",
                ZplCommand::GraphicBox {
                    width: 100,
                    height: 100,
                    thickness: 1,
                },
            ),
            (
                "Graphic Field (^GFA)",
                ZplCommand::GraphicField {
                    width: 32,
                    height: 32,
                    data: String::new(),
                },
            ),
        ]
    }

    pub fn to_zpl_string(&self) -> String {
        match self {
            ZplCommand::StartFormat => "^XA".to_string(),
            ZplCommand::EndFormat => "^XZ".to_string(),
            ZplCommand::FieldOrigin { x, y } => format!("^FO{},{}", x, y),
            ZplCommand::Font {
                orientation,
                height,
                width,
            } => format!("^A0{},{},{}", orientation, height, width),
            ZplCommand::FieldData { data } => format!("^FD{}", data),
            ZplCommand::FieldSeparator => "^FS".to_string(),
            ZplCommand::GraphicBox {
                width,
                height,
                thickness,
            } => format!("^GB{},{},{}", width, height, thickness),
            ZplCommand::ChangeFont { font, size } => format!("^CF{},{}", font, size),
            ZplCommand::FieldOrientation { rotation } => format!("^FW{}", rotation),
            ZplCommand::BarcodeFieldDefault {
                width,
                ratio,
                height,
            } => format!("^BY{},{},{}", width, ratio, height),
            ZplCommand::Code128Barcode {
                orientation,
                height,
                print_interpretation,
                print_above,
                check_digit,
                mode,
            } => format!(
                "^BC{},{},{},{},{},{}",
                orientation,
                height,
                if *print_interpretation { "Y" } else { "N" },
                if *print_above { "Y" } else { "N" },
                if *check_digit { "Y" } else { "N" },
                mode
            ),
            ZplCommand::GraphicField { width, height, data } => {
                let bytes_per_row = (width + 7) / 8;
                let total_bytes = bytes_per_row * height;
                let clean_data = data.replace(",", "").replace(" ", "").replace("\n", "").replace("\r", "").to_uppercase();
                format!("^GFA,{},{},{},{}", total_bytes, total_bytes, bytes_per_row, clean_data)
            }
            ZplCommand::MediaModeDelayed => "^MMD".to_string(),
            ZplCommand::CutNow => "~JK".to_string(),
        }
    }
}

impl Default for ZplCommand {
    fn default() -> Self {
        ZplCommand::FieldSeparator
    }
}

pub fn commands_to_zpl(commands: &[ZplCommand]) -> String {
    commands
        .iter()
        .map(|cmd| cmd.to_zpl_string())
        .collect::<Vec<_>>()
        .join("\n")
}

#[derive(EnumStr, Debug, Clone, Copy, PartialEq)]
pub enum FontOrientation {
    #[enum2str("N")]
    Normal,
    #[enum2str("R")]
    Rotated90,
    #[enum2str("I")]
    Rotated180,
    #[enum2str("B")]
    Rotated270,
}

#[derive(EnumStr, Debug, Clone, Copy, PartialEq)]
pub enum FieldOrientation {
    #[enum2str("N")]
    Normal,
    #[enum2str("R")]
    Rotated90,
    #[enum2str("I")]
    Rotated180,
    #[enum2str("B")]
    Rotated270,
}

#[derive(EnumStr, Debug, Clone, Copy, PartialEq)]
pub enum FieldRotation {
    #[enum2str("N")]
    Normal,
    #[enum2str("R")]
    Rotated90,
    #[enum2str("I")]
    Rotated180,
    #[enum2str("B")]
    Rotated270,
}

pub struct ZplLabel {
    commands: Vec<ZplCommand>,
}

impl ZplLabel {
    pub fn new() -> Self {
        Self {
            commands: vec![ZplCommand::StartFormat],
        }
    }

    pub fn field_origin(mut self, x: u32, y: u32) -> Self {
        self.commands.push(ZplCommand::FieldOrigin { x, y });
        self
    }

    pub fn font(mut self, orientation: FontOrientation, height: u32, width: u32) -> Self {
        self.commands.push(ZplCommand::Font {
            orientation,
            height,
            width,
        });
        self
    }

    pub fn field_data(mut self, data: impl Into<String>) -> Self {
        self.commands
            .push(ZplCommand::FieldData { data: data.into() });
        self
    }

    pub fn field_separator(mut self) -> Self {
        self.commands.push(ZplCommand::FieldSeparator);
        self
    }

    pub fn graphic_box(mut self, width: u32, height: u32, thickness: u32) -> Self {
        self.commands.push(ZplCommand::GraphicBox {
            width,
            height,
            thickness,
        });
        self
    }

    pub fn graphic_field(mut self, width: u32, height: u32, data: impl Into<String>) -> Self {
        self.commands.push(ZplCommand::GraphicField {
            width,
            height,
            data: data.into(),
        });
        self
    }

    pub fn build(mut self) -> String {
        self.commands.push(ZplCommand::EndFormat);
        self.to_zpl()
    }

    fn to_zpl(&self) -> String {
        self.commands
            .iter()
            .map(|cmd| match cmd {
                ZplCommand::StartFormat => format!("^XA"),
                ZplCommand::EndFormat => format!("^XZ"),
                ZplCommand::FieldOrigin { x, y } => format!("^FO{},{}", x, y),
                ZplCommand::Font {
                    orientation,
                    height,
                    width,
                } => {
                    format!("^A0{},{},{}", orientation, height, width)
                }
                ZplCommand::FieldData { data } => format!("^FD{}", data),
                ZplCommand::FieldSeparator => format!("^FS"),
                ZplCommand::GraphicBox {
                    width,
                    height,
                    thickness,
                } => {
                    format!("^GB{},{},{}", width, height, thickness)
                }
                ZplCommand::ChangeFont { font, size } => format!("^CF{},{}", font, size),
                ZplCommand::FieldOrientation { rotation } => format!("^FW{}", rotation),
                ZplCommand::BarcodeFieldDefault {
                    width,
                    ratio,
                    height,
                } => {
                    format!("^BY{},{},{}", width, ratio, height)
                }
                ZplCommand::Code128Barcode {
                    orientation,
                    height,
                    print_interpretation,
                    print_above,
                    check_digit,
                    mode,
                } => {
                    format!(
                        "^BC{},{},{},{},{},{}",
                        orientation,
                        height,
                        if *print_interpretation { "Y" } else { "N" },
                        if *print_above { "Y" } else { "N" },
                        if *check_digit { "Y" } else { "N" },
                        mode
                    )
                }
                ZplCommand::GraphicField { width, height, data } => {
                    let bytes_per_row = (width + 7) / 8;
                    let total_bytes = bytes_per_row * height;
                    let clean_data = data.replace(",", "").replace(" ", "").replace("\n", "").replace("\r", "").to_uppercase();
                    format!("^GFA,{},{},{},{}", total_bytes, total_bytes, bytes_per_row, clean_data)
                }
                ZplCommand::MediaModeDelayed => format!("^MMD"),
                ZplCommand::CutNow => format!("~JK"),
            })
            .collect::<Vec<_>>()
            .join("\n")
    }
}

impl Default for ZplLabel {
    fn default() -> Self {
        Self::new()
    }
}

pub fn image_to_zpl_hex(image: &DynamicImage, threshold: u8) -> String {
    let width = image.width();
    let height = image.height();
    let bytes_per_row = ((width + 7) / 8) as usize;

    let mut hex_lines = Vec::new();

    for y in 0..height {
        let mut row_bytes = vec![0u8; bytes_per_row];

        for x in 0..width {
            let pixel = image.get_pixel(x, y);
            let grayscale = rgb_to_grayscale(pixel);

            if grayscale < threshold {
                let byte_index = (x / 8) as usize;
                let bit_index = 7 - (x % 8);
                row_bytes[byte_index] |= 1 << bit_index;
            }
        }

        let row_hex: String = row_bytes
            .iter()
            .map(|byte| format!("{:02X}", byte))
            .collect();
        hex_lines.push(row_hex);
    }

    hex_lines.join("")
}

fn rgb_to_grayscale(pixel: Rgba<u8>) -> u8 {
    let r = pixel[0] as u32;
    let g = pixel[1] as u32;
    let b = pixel[2] as u32;
    ((r * 299 + g * 587 + b * 114) / 1000) as u8
}

pub fn create_graphic_field_from_image(
    image: &DynamicImage,
    threshold: u8,
) -> ZplCommand {
    let width = image.width();
    let height = image.height();
    let data = image_to_zpl_hex(image, threshold);

    ZplCommand::GraphicField {
        width,
        height,
        data,
    }
}

pub fn parse_graphic_field_from_zpl(zpl: &str) -> Option<(u32, u32, String)> {
    let zpl_upper = zpl.to_uppercase();

    if let Some(gf_start) = zpl_upper.find("^GF") {
        let after_gf = &zpl[gf_start + 3..];

        if after_gf.starts_with("A,") || after_gf.to_uppercase().starts_with("A,") {
            let gfa_section = &zpl[gf_start + 5..];

            let end_pos = gfa_section.find('^').unwrap_or(gfa_section.len());
            let gfa_data = &gfa_section[..end_pos];
            let parts: Vec<&str> = gfa_data.split(',').collect();

            if parts.len() >= 4 {
                let total_bytes = parts[0].trim().parse::<u32>().ok()?;
                let bytes_per_row = parts[2].trim().parse::<u32>().ok()?;

                let hex_data_parts: Vec<&str> = parts[3..].iter()
                    .flat_map(|s| s.split_whitespace())
                    .collect();
                let hex_data = hex_data_parts.join("")
                    .replace(",", "")
                    .replace(" ", "")
                    .replace("\n", "")
                    .replace("\r", "")
                    .to_uppercase();

                let height = if bytes_per_row > 0 { total_bytes / bytes_per_row } else { 0 };
                let width = bytes_per_row * 8;

                return Some((width, height, hex_data));
            }
        }
    }

    None
}

