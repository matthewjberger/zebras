use nightshade::prelude::*;
use std::sync::{Arc, Mutex};

use crate::labelary::LabelaryClient;
use crate::printer::ZplPrinter;
use crate::printer_status::PrinterStatus;
use crate::zpl::{commands_to_zpl, parse_graphic_field_from_zpl, FieldOrientation, FontOrientation, ZplCommand};

pub struct Zebras {
    zpl_commands: Vec<ZplCommand>,
    rendered_image: Option<egui::TextureHandle>,
    is_dirty: bool,
    error_message: Option<String>,
    is_loading: bool,
    needs_initial_render: bool,
    pending_response: Arc<Mutex<Option<Result<Vec<u8>, String>>>>,
    pending_scan_result: Arc<Mutex<Option<Vec<ZplPrinter>>>>,
    show_raw_text: bool,
    raw_zpl_mode: bool,
    raw_zpl_input: String,
    printers: Vec<ZplPrinter>,
    selected_printer: Option<usize>,
    is_scanning: bool,
    print_status: Option<String>,
    manual_ip: String,
    image_load_status: Option<String>,
    graphic_threshold: u8,
    pending_query_result: Arc<Mutex<Option<Result<String, String>>>>,
    query_response: Option<String>,
    is_querying: bool,
    parsed_status: Option<PrinterStatus>,
    last_query_type: Option<String>,
    cutting_enabled: bool,
}

impl Default for Zebras {
    fn default() -> Self {
        let default_commands = vec![
            ZplCommand::StartFormat,
            ZplCommand::FieldOrigin { x: 50, y: 50 },
            ZplCommand::Font {
                orientation: FontOrientation::Normal,
                height: 50,
                width: 50,
            },
            ZplCommand::FieldData {
                data: "Hello World!".to_string(),
            },
            ZplCommand::FieldSeparator,
            ZplCommand::FieldOrigin { x: 50, y: 150 },
            ZplCommand::GraphicBox {
                width: 300,
                height: 2,
                thickness: 2,
            },
            ZplCommand::FieldSeparator,
            ZplCommand::FieldOrigin { x: 50, y: 200 },
            ZplCommand::Font {
                orientation: FontOrientation::Normal,
                height: 30,
                width: 30,
            },
            ZplCommand::FieldData {
                data: "Zebra ZPL Simulator".to_string(),
            },
            ZplCommand::FieldSeparator,
            ZplCommand::EndFormat,
        ];

        Self {
            zpl_commands: default_commands,
            rendered_image: None,
            is_dirty: false,
            error_message: None,
            is_loading: false,
            needs_initial_render: true,
            pending_response: Arc::new(Mutex::new(None)),
            pending_scan_result: Arc::new(Mutex::new(None)),
            show_raw_text: false,
            raw_zpl_mode: false,
            raw_zpl_input: String::new(),
            printers: Vec::new(),
            selected_printer: None,
            is_scanning: false,
            print_status: None,
            manual_ip: String::new(),
            image_load_status: None,
            graphic_threshold: 128,
            pending_query_result: Arc::new(Mutex::new(None)),
            query_response: None,
            is_querying: false,
            parsed_status: None,
            last_query_type: None,
            cutting_enabled: false,
        }
    }
}

impl Zebras {
    fn get_zpl_text(&self) -> String {
        if self.raw_zpl_mode {
            self.raw_zpl_input.clone()
        } else {
            commands_to_zpl(&self.zpl_commands)
        }
    }

    fn toggle_cutting(&mut self, enabled: bool) {
        if enabled {
            let has_media_mode = self.zpl_commands.iter().any(|cmd| matches!(cmd, ZplCommand::MediaModeDelayed));
            let has_cut_now = self.zpl_commands.iter().any(|cmd| matches!(cmd, ZplCommand::CutNow));

            if !has_media_mode {
                if let Some(start_idx) = self.zpl_commands.iter().position(|cmd| matches!(cmd, ZplCommand::StartFormat)) {
                    self.zpl_commands.insert(start_idx + 1, ZplCommand::MediaModeDelayed);
                }
            }

            if !has_cut_now {
                if let Some(end_idx) = self.zpl_commands.iter().position(|cmd| matches!(cmd, ZplCommand::EndFormat)) {
                    self.zpl_commands.insert(end_idx, ZplCommand::CutNow);
                }
            }
        } else {
            self.zpl_commands.retain(|cmd| !matches!(cmd, ZplCommand::MediaModeDelayed | ZplCommand::CutNow));
        }
        self.is_dirty = true;
    }

    fn get_presets() -> Vec<(&'static str, Vec<ZplCommand>)> {
        vec![
            (
                "Hello World",
                vec![
                    ZplCommand::StartFormat,
                    ZplCommand::FieldOrigin { x: 50, y: 50 },
                    ZplCommand::Font {
                        orientation: FontOrientation::Normal,
                        height: 50,
                        width: 50,
                    },
                    ZplCommand::FieldData {
                        data: "Hello World!".to_string(),
                    },
                    ZplCommand::FieldSeparator,
                    ZplCommand::FieldOrigin { x: 50, y: 150 },
                    ZplCommand::GraphicBox {
                        width: 300,
                        height: 2,
                        thickness: 2,
                    },
                    ZplCommand::FieldSeparator,
                    ZplCommand::FieldOrigin { x: 50, y: 200 },
                    ZplCommand::Font {
                        orientation: FontOrientation::Normal,
                        height: 30,
                        width: 30,
                    },
                    ZplCommand::FieldData {
                        data: "Zebra ZPL Simulator".to_string(),
                    },
                    ZplCommand::FieldSeparator,
                    ZplCommand::EndFormat,
                ],
            ),
            (
                "Shipping Label",
                vec![
                    ZplCommand::StartFormat,
                    ZplCommand::FieldOrigin { x: 20, y: 20 },
                    ZplCommand::Font {
                        orientation: FontOrientation::Normal,
                        height: 40,
                        width: 40,
                    },
                    ZplCommand::FieldData {
                        data: "SHIP TO:".to_string(),
                    },
                    ZplCommand::FieldSeparator,
                    ZplCommand::FieldOrigin { x: 20, y: 80 },
                    ZplCommand::Font {
                        orientation: FontOrientation::Normal,
                        height: 30,
                        width: 30,
                    },
                    ZplCommand::FieldData {
                        data: "John Smith".to_string(),
                    },
                    ZplCommand::FieldSeparator,
                    ZplCommand::FieldOrigin { x: 20, y: 120 },
                    ZplCommand::FieldData {
                        data: "123 Main Street".to_string(),
                    },
                    ZplCommand::FieldSeparator,
                    ZplCommand::FieldOrigin { x: 20, y: 160 },
                    ZplCommand::FieldData {
                        data: "Anytown, ST 12345".to_string(),
                    },
                    ZplCommand::FieldSeparator,
                    ZplCommand::FieldOrigin { x: 20, y: 220 },
                    ZplCommand::BarcodeFieldDefault {
                        width: 2,
                        ratio: 3.0,
                        height: 80,
                    },
                    ZplCommand::Code128Barcode {
                        orientation: FieldOrientation::Normal,
                        height: 80,
                        print_interpretation: true,
                        print_above: false,
                        check_digit: false,
                        mode: FieldOrientation::Normal,
                    },
                    ZplCommand::FieldData {
                        data: "1234567890".to_string(),
                    },
                    ZplCommand::FieldSeparator,
                    ZplCommand::EndFormat,
                ],
            ),
            (
                "Product Label",
                vec![
                    ZplCommand::StartFormat,
                    ZplCommand::FieldOrigin { x: 30, y: 30 },
                    ZplCommand::Font {
                        orientation: FontOrientation::Normal,
                        height: 35,
                        width: 35,
                    },
                    ZplCommand::FieldData {
                        data: "Product Name".to_string(),
                    },
                    ZplCommand::FieldSeparator,
                    ZplCommand::FieldOrigin { x: 30, y: 80 },
                    ZplCommand::Font {
                        orientation: FontOrientation::Normal,
                        height: 25,
                        width: 25,
                    },
                    ZplCommand::FieldData {
                        data: "SKU: ABC-123".to_string(),
                    },
                    ZplCommand::FieldSeparator,
                    ZplCommand::FieldOrigin { x: 30, y: 110 },
                    ZplCommand::FieldData {
                        data: "Price: $19.99".to_string(),
                    },
                    ZplCommand::FieldSeparator,
                    ZplCommand::FieldOrigin { x: 30, y: 160 },
                    ZplCommand::BarcodeFieldDefault {
                        width: 2,
                        ratio: 3.0,
                        height: 60,
                    },
                    ZplCommand::Code128Barcode {
                        orientation: FieldOrientation::Normal,
                        height: 60,
                        print_interpretation: true,
                        print_above: false,
                        check_digit: false,
                        mode: FieldOrientation::Normal,
                    },
                    ZplCommand::FieldData {
                        data: "ABC123456".to_string(),
                    },
                    ZplCommand::FieldSeparator,
                    ZplCommand::EndFormat,
                ],
            ),
            (
                "Simple Barcode",
                vec![
                    ZplCommand::StartFormat,
                    ZplCommand::FieldOrigin { x: 50, y: 50 },
                    ZplCommand::BarcodeFieldDefault {
                        width: 3,
                        ratio: 3.0,
                        height: 100,
                    },
                    ZplCommand::Code128Barcode {
                        orientation: FieldOrientation::Normal,
                        height: 100,
                        print_interpretation: true,
                        print_above: false,
                        check_digit: false,
                        mode: FieldOrientation::Normal,
                    },
                    ZplCommand::FieldData {
                        data: "9876543210".to_string(),
                    },
                    ZplCommand::FieldSeparator,
                    ZplCommand::EndFormat,
                ],
            ),
            (
                "Multi-Line Text",
                vec![
                    ZplCommand::StartFormat,
                    ZplCommand::FieldOrigin { x: 40, y: 40 },
                    ZplCommand::Font {
                        orientation: FontOrientation::Normal,
                        height: 30,
                        width: 30,
                    },
                    ZplCommand::FieldData {
                        data: "Line 1 of Text".to_string(),
                    },
                    ZplCommand::FieldSeparator,
                    ZplCommand::FieldOrigin { x: 40, y: 80 },
                    ZplCommand::FieldData {
                        data: "Line 2 of Text".to_string(),
                    },
                    ZplCommand::FieldSeparator,
                    ZplCommand::FieldOrigin { x: 40, y: 120 },
                    ZplCommand::FieldData {
                        data: "Line 3 of Text".to_string(),
                    },
                    ZplCommand::FieldSeparator,
                    ZplCommand::FieldOrigin { x: 40, y: 160 },
                    ZplCommand::GraphicBox {
                        width: 320,
                        height: 1,
                        thickness: 1,
                    },
                    ZplCommand::FieldSeparator,
                    ZplCommand::FieldOrigin { x: 40, y: 180 },
                    ZplCommand::Font {
                        orientation: FontOrientation::Normal,
                        height: 25,
                        width: 25,
                    },
                    ZplCommand::FieldData {
                        data: "Line 4 of Text".to_string(),
                    },
                    ZplCommand::FieldSeparator,
                    ZplCommand::EndFormat,
                ],
            ),
            (
                "Simple Graphic",
                vec![
                    ZplCommand::StartFormat,
                    ZplCommand::FieldOrigin { x: 50, y: 50 },
                    ZplCommand::Font {
                        orientation: FontOrientation::Normal,
                        height: 30,
                        width: 30,
                    },
                    ZplCommand::FieldData {
                        data: "Graphic Below:".to_string(),
                    },
                    ZplCommand::FieldSeparator,
                    ZplCommand::FieldOrigin { x: 50, y: 100 },
                    ZplCommand::GraphicField {
                        width: 32,
                        height: 32,
                        data: "FFFFFFFFFFFFFFFFC0000003C0000003C0000003C0000003C0000003C0000003C0000003C0000003C0000003C0000003C0000003C0000003C0000003C0000003C0000003C0000003C0000003C0000003C0000003C0000003C0000003C0000003C0000003C0000003C0000003C0000003C0000003C0000003FFFFFFFFFFFFFFFF".to_string(),
                    },
                    ZplCommand::FieldSeparator,
                    ZplCommand::FieldOrigin { x: 50, y: 200 },
                    ZplCommand::Font {
                        orientation: FontOrientation::Normal,
                        height: 30,
                        width: 30,
                    },
                    ZplCommand::FieldData {
                        data: "32x32 Test Pattern".to_string(),
                    },
                    ZplCommand::FieldSeparator,
                    ZplCommand::EndFormat,
                ],
            ),
        ]
    }

    fn load_preset(&mut self, preset_name: &str) {
        let presets = Self::get_presets();
        if let Some((_, commands)) = presets.iter().find(|(name, _)| *name == preset_name) {
            self.zpl_commands = commands.clone();
            self.is_dirty = true;
        }
    }

    fn scan_for_printers(&mut self) {
        self.is_scanning = true;
        self.print_status = Some("Scanning for printers...".to_string());

        #[cfg(not(target_arch = "wasm32"))]
        {
            use std::thread;
            let result = self.pending_scan_result.clone();

            thread::spawn(move || {
                let printers = crate::printer::scan_for_printers();
                if let Ok(mut guard) = result.lock() {
                    *guard = Some(printers);
                }
            });
        }

        #[cfg(target_arch = "wasm32")]
        {
            self.print_status = Some("Printer scanning not available in WASM".to_string());
            self.is_scanning = false;
        }
    }

    fn send_to_printer(&mut self) {
        if let Some(idx) = self.selected_printer {
            if let Some(printer) = self.printers.get(idx) {
                let zpl = self.get_zpl_text();
                match crate::printer::send_to_printer(printer, &zpl) {
                    Ok(_) => {
                        self.print_status = Some(format!("Sent to {}", printer.name));
                    }
                    Err(e) => {
                        self.print_status = Some(format!("Print error: {}", e));
                    }
                }
            }
        } else {
            self.print_status = Some("No printer selected".to_string());
        }
    }

    fn add_manual_printer(&mut self) {
        let ip = self.manual_ip.trim();

        if ip.is_empty() {
            self.print_status = Some("Please enter an IP address".to_string());
            return;
        }

        if ip.split('.').count() != 4 || !ip.split('.').all(|octet| octet.parse::<u8>().is_ok()) {
            self.print_status = Some("Invalid IP address format".to_string());
            return;
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            let printer = ZplPrinter::new(ip.to_string(), 9100);

            if !self.printers.iter().any(|p| p.ip == ip) {
                self.printers.push(printer);
                self.print_status = Some(format!("Added printer at {}", ip));
                self.manual_ip.clear();
            } else {
                self.print_status = Some(format!("Printer at {} already exists", ip));
            }
        }
    }

    fn query_printer(&mut self, query_type: &str, ui_context: &egui::Context) {
        if let Some(idx) = self.selected_printer {
            if let Some(printer) = self.printers.get(idx).cloned() {
                self.is_querying = true;
                self.query_response = Some("Querying printer...".to_string());
                self.last_query_type = Some(query_type.to_string());

                let query = format!("~HQ{}\r\n", query_type);
                let ctx = ui_context.clone();
                let pending_result = Arc::clone(&self.pending_query_result);

                #[cfg(not(target_arch = "wasm32"))]
                {
                    std::thread::spawn(move || {
                        let response = crate::printer::query_printer(&printer, &query);
                        if let Ok(mut guard) = pending_result.lock() {
                            *guard = Some(response);
                        }
                        ctx.request_repaint();
                    });
                }

                #[cfg(target_arch = "wasm32")]
                {
                    self.query_response = Some("Printer queries not available in WASM".to_string());
                    self.is_querying = false;
                }
            }
        } else {
            self.query_response = Some("No printer selected".to_string());
        }
    }

    fn render_zpl(&mut self, ui_context: &egui::Context) {
        self.error_message = None;
        self.is_loading = true;

        let zpl = self.get_zpl_text();

        let ctx = ui_context.clone();
        let pending_response = Arc::clone(&self.pending_response);
        let client = LabelaryClient::default();

        #[cfg(not(target_arch = "wasm32"))]
        {
            std::thread::spawn(move || {
                let response_data = client.render_sync(&zpl);
                if let Ok(mut guard) = pending_response.lock() {
                    *guard = Some(response_data);
                }
                ctx.request_repaint();
            });
        }

        #[cfg(target_arch = "wasm32")]
        {
            wasm_bindgen_futures::spawn_local(async move {
                let response_data = client.render_async(&zpl).await;
                if let Ok(mut guard) = pending_response.lock() {
                    *guard = Some(response_data);
                }
                ctx.request_repaint();
            });
        }

        self.is_dirty = false;
    }

    fn render_command_editor(&mut self, ui: &mut egui::Ui, idx: usize) {
        let command = &mut self.zpl_commands[idx];
        match command {
            ZplCommand::StartFormat | ZplCommand::EndFormat | ZplCommand::FieldSeparator => {}
            ZplCommand::FieldOrigin { x, y } => {
                ui.horizontal(|ui| {
                    ui.label("X:");
                    if ui.add(egui::DragValue::new(x).speed(1)).changed() {
                        self.is_dirty = true;
                    }
                    ui.label("Y:");
                    if ui.add(egui::DragValue::new(y).speed(1)).changed() {
                        self.is_dirty = true;
                    }
                });
            }
            ZplCommand::Font {
                height, width, ..
            } => {
                ui.horizontal(|ui| {
                    ui.label("Height:");
                    if ui.add(egui::DragValue::new(height).speed(1)).changed() {
                        self.is_dirty = true;
                    }
                    ui.label("Width:");
                    if ui.add(egui::DragValue::new(width).speed(1)).changed() {
                        self.is_dirty = true;
                    }
                });
            }
            ZplCommand::FieldData { data } => {
                ui.horizontal(|ui| {
                    ui.label("Text:");
                    if ui.text_edit_singleline(data).changed() {
                        self.is_dirty = true;
                    }
                });
            }
            ZplCommand::GraphicBox {
                width,
                height,
                thickness,
            } => {
                ui.horizontal(|ui| {
                    ui.label("W:");
                    if ui.add(egui::DragValue::new(width).speed(1)).changed() {
                        self.is_dirty = true;
                    }
                    ui.label("H:");
                    if ui.add(egui::DragValue::new(height).speed(1)).changed() {
                        self.is_dirty = true;
                    }
                    ui.label("T:");
                    if ui.add(egui::DragValue::new(thickness).speed(1)).changed() {
                        self.is_dirty = true;
                    }
                });
            }
            ZplCommand::GraphicField { width, height, data } => {
                ui.vertical(|ui| {
                    ui.label(egui::RichText::new("Note: Add Field Origin (^FO) command before this").small().color(egui::Color32::GRAY));
                    ui.horizontal(|ui| {
                        ui.label("W:");
                        if ui.add(egui::DragValue::new(width).speed(1)).changed() {
                            self.is_dirty = true;
                        }
                        ui.label("H:");
                        if ui.add(egui::DragValue::new(height).speed(1)).changed() {
                            self.is_dirty = true;
                        }
                    });
                    ui.horizontal(|ui| {
                        ui.label("Data (hex):");
                        if ui.text_edit_singleline(data).changed() {
                            self.is_dirty = true;
                        }
                    });
                    ui.label(format!("Data length: {} chars", data.len()));
                    ui.separator();
                    ui.label("Load from image:");
                    ui.horizontal(|ui| {
                        if ui.button("Select Image (Labelary API)").clicked() {
                            self.image_load_status = Some("Opening file dialog...".to_string());
                            if let Some(path) = rfd::FileDialog::new()
                                .add_filter("Image", &["png", "jpg", "jpeg", "bmp", "gif"])
                                .pick_file()
                            {
                                self.image_load_status = Some(format!("Loading {:?}...", path.file_name()));
                                #[cfg(not(target_arch = "wasm32"))]
                                {
                                    match std::fs::read(&path) {
                                        Ok(file_bytes) => {
                                            if file_bytes.len() > 200_000 {
                                                match image::open(&path) {
                                                    Ok(loaded_image) => {
                                                        self.image_load_status = Some(format!(
                                                            "Resizing {}x{} image...",
                                                            loaded_image.width(),
                                                            loaded_image.height()
                                                        ));
                                                        let max_dimension = 1000;
                                                        let scale = (max_dimension as f32 / loaded_image.width().max(loaded_image.height()) as f32).min(1.0);
                                                        let new_width = (loaded_image.width() as f32 * scale) as u32;
                                                        let new_height = (loaded_image.height() as f32 * scale) as u32;
                                                        let resized_image = loaded_image.resize(new_width, new_height, image::imageops::FilterType::Lanczos3);

                                                        self.image_load_status = Some("Encoding as PNG...".to_string());
                                                        let mut png_bytes = Vec::new();
                                                        let encode_result = {
                                                            use image::codecs::png::PngEncoder;
                                                            use image::ImageEncoder;
                                                            let rgba = resized_image.to_rgba8();
                                                            let encoder = PngEncoder::new(&mut png_bytes);
                                                            encoder.write_image(
                                                                rgba.as_raw(),
                                                                resized_image.width(),
                                                                resized_image.height(),
                                                                image::ExtendedColorType::Rgba8
                                                            )
                                                        };

                                                        if let Err(e) = encode_result {
                                                            self.image_load_status = Some(format!("Encode failed: {}", e));
                                                        } else if png_bytes.is_empty() {
                                                            self.image_load_status = Some("Empty PNG data".to_string());
                                                        } else {
                                                            self.image_load_status = Some(format!("Sending {}KB...", png_bytes.len() / 1024));
                                                            let client = LabelaryClient::default();
                                                            match client.convert_image_to_zpl_sync(png_bytes) {
                                                Ok(zpl_response) => {
                                                    self.image_load_status = Some("Parsing ZPL response...".to_string());
                                                    if let Some((parsed_width, parsed_height, hex_data)) = parse_graphic_field_from_zpl(&zpl_response) {
                                                        *width = parsed_width;
                                                        *height = parsed_height;
                                                        *data = hex_data;
                                                        self.is_dirty = true;
                                                        self.image_load_status = Some(format!(
                                                            "Image loaded! {}x{}, {} chars of hex data",
                                                            width, height, data.len()
                                                        ));
                                                    } else {
                                                        self.image_load_status = Some(format!(
                                                            "Failed to parse ZPL response. Response: {}",
                                                            &zpl_response[..zpl_response.len().min(200)]
                                                        ));
                                                    }
                                                }
                                                Err(e) => {
                                                    self.image_load_status = Some(format!("API error: {}", e));
                                                }
                                            }
                                                                                                }
                                                                                            }
                                                                                            Err(e) => {
                                                                                                self.image_load_status = Some(format!("Failed to load image for resizing: {}", e));
                                                                                            }
                                                                                        }
                                            } else {
                                                self.image_load_status = Some(format!("Sending {}KB directly...", file_bytes.len() / 1024));
                                                let client = LabelaryClient::default();
                                                match client.convert_image_to_zpl_sync(file_bytes) {
                                                    Ok(zpl_response) => {
                                                        self.image_load_status = Some("Parsing ZPL response...".to_string());
                                                        if let Some((parsed_width, parsed_height, hex_data)) = parse_graphic_field_from_zpl(&zpl_response) {
                                                            *width = parsed_width;
                                                            *height = parsed_height;
                                                            *data = hex_data;
                                                            self.is_dirty = true;
                                                            self.image_load_status = Some(format!(
                                                                "Image loaded! {}x{}, {} chars of hex data",
                                                                width, height, data.len()
                                                            ));
                                                        } else {
                                                            self.image_load_status = Some(format!(
                                                                "Failed to parse ZPL response. Response: {}",
                                                                &zpl_response[..zpl_response.len().min(200)]
                                                            ));
                                                        }
                                                    }
                                                    Err(e) => {
                                                        self.image_load_status = Some(format!("API error: {}", e));
                                                    }
                                                }
                                            }
                                        }
                                        Err(e) => {
                                            self.image_load_status = Some(format!("Failed to read file: {}", e));
                                        }
                                    }
                                }
                                #[cfg(target_arch = "wasm32")]
                                {
                                    self.image_load_status = Some("Image upload not available in WASM".to_string());
                                }
                            } else {
                                self.image_load_status = Some("No file selected".to_string());
                            }
                        }
                        if ui.button("Select Image (Local)").clicked() {
                            self.image_load_status = Some("Opening file dialog...".to_string());
                            if let Some(path) = rfd::FileDialog::new()
                                .add_filter("Image", &["png", "jpg", "jpeg", "bmp", "gif"])
                                .pick_file()
                            {
                                self.image_load_status = Some(format!("Loading {:?}...", path.file_name()));
                                match image::open(&path) {
                                    Ok(loaded_image) => {
                                        self.image_load_status = Some(format!(
                                            "Resizing {}x{} to {}x{}...",
                                            loaded_image.width(),
                                            loaded_image.height(),
                                            *width,
                                            *height
                                        ));
                                        let resized_image = loaded_image.resize(
                                            *width,
                                            *height,
                                            image::imageops::FilterType::Lanczos3,
                                        );
                                        *data = crate::zpl::image_to_zpl_hex(&resized_image, self.graphic_threshold);
                                        self.is_dirty = true;
                                        self.image_load_status = Some(format!(
                                            "Image loaded! {} chars of hex data",
                                            data.len()
                                        ));
                                    }
                                    Err(e) => {
                                        self.image_load_status = Some(format!("Error loading image: {}", e));
                                    }
                                }
                            } else {
                                self.image_load_status = Some("No file selected".to_string());
                            }
                        }
                    });
                    ui.horizontal(|ui| {
                        ui.label("Threshold (Local only):");
                        ui.add(egui::Slider::new(&mut self.graphic_threshold, 0..=255));
                    });
                    ui.label(egui::RichText::new("(Lower = more black, Higher = more white)").small().color(egui::Color32::GRAY));
                    if let Some(ref status) = self.image_load_status {
                        ui.label(egui::RichText::new(status).color(egui::Color32::LIGHT_BLUE));
                    }
                });
            }
            _ => {
                ui.label("(Complex command - not yet editable)");
            }
        }
    }

    fn process_image_response(&mut self, image_data: Vec<u8>, ui_context: &egui::Context) {
        let image = image::load_from_memory(&image_data);
        match image {
            Ok(img) => {
                let size = [img.width() as _, img.height() as _];
                let rgba = img.to_rgba8();
                let pixels = rgba.as_flat_samples();
                let color_image =
                    egui::ColorImage::from_rgba_unmultiplied(size, pixels.as_slice());
                let texture = ui_context.load_texture(
                    "zpl_render",
                    color_image,
                    egui::TextureOptions::LINEAR,
                );
                self.rendered_image = Some(texture);
                self.is_loading = false;
            }
            Err(e) => {
                self.error_message = Some(format!("Failed to decode image: {}", e));
                self.is_loading = false;
            }
        }
    }
}

impl State for Zebras {
    fn title(&self) -> &str {
        "Zebras - ZPL Simulator"
    }

    fn initialize(&mut self, world: &mut World) {
        world.resources.user_interface.enabled = true;
    }

    fn ui(&mut self, _world: &mut World, ui_context: &egui::Context) {
        let pending_result = if let Ok(mut guard) = self.pending_response.try_lock() {
            guard.take()
        } else {
            None
        };

        if let Some(response) = pending_result {
            match response {
                Ok(image_data) => {
                    self.process_image_response(image_data, ui_context);
                }
                Err(e) => {
                    self.error_message = Some(e);
                    self.is_loading = false;
                }
            }
        }

        let pending_scan = if let Ok(mut guard) = self.pending_scan_result.try_lock() {
            guard.take()
        } else {
            None
        };

        if let Some(mut scanned_printers) = pending_scan {
            self.is_scanning = false;
            if scanned_printers.is_empty() {
                self.print_status = Some("No printers found".to_string());
            } else {
                let mut added_count = 0;
                for scanned in scanned_printers.drain(..) {
                    let already_exists = self.printers.iter().any(|existing| {
                        existing.ip == scanned.ip && !scanned.ip.is_empty()
                    });
                    if !already_exists {
                        self.printers.push(scanned);
                        added_count += 1;
                    }
                }
                if added_count > 0 {
                    self.print_status = Some(format!("Found {} new printer(s)", added_count));
                } else {
                    self.print_status = Some("No new printers found".to_string());
                }
            }
        }

        let pending_query = if let Ok(mut guard) = self.pending_query_result.try_lock() {
            guard.take()
        } else {
            None
        };

        if let Some(query_result) = pending_query {
            self.is_querying = false;
            match query_result {
                Ok(response) => {
                    if let Some(ref query_type) = self.last_query_type {
                        if query_type == "ES" {
                            match PrinterStatus::parse(&response) {
                                Ok(status) => {
                                    self.parsed_status = Some(status);
                                    self.query_response = None;
                                }
                                Err(e) => {
                                    self.query_response = Some(format!("Failed to parse status: {}\n\nRaw response:\n{}", e, response));
                                    self.parsed_status = None;
                                }
                            }
                        } else {
                            self.query_response = Some(response);
                            self.parsed_status = None;
                        }
                    } else {
                        self.query_response = Some(response);
                        self.parsed_status = None;
                    }
                }
                Err(e) => {
                    self.query_response = Some(format!("Query error: {}", e));
                    self.parsed_status = None;
                }
            }
        }

        if self.needs_initial_render {
            self.needs_initial_render = false;
            self.render_zpl(ui_context);
        }

        egui::TopBottomPanel::top("top_panel").show(ui_context, |ui| {
            ui.horizontal(|ui| {
                ui.heading("ZPL Simulator");
                ui.separator();

                ui.label("Preset:");
                let preset_response = egui::ComboBox::from_id_salt("preset_selector")
                    .selected_text("Load...")
                    .show_ui(ui, |ui| {
                        let mut clicked_preset = None;
                        for (name, _) in Self::get_presets() {
                            if ui.selectable_label(false, name).clicked() {
                                clicked_preset = Some(name);
                            }
                        }
                        clicked_preset
                    });
                
                if let Some(inner) = preset_response.inner {
                    if let Some(preset_name) = inner {
                        self.load_preset(preset_name);
                        self.render_zpl(ui_context);
                        self.is_dirty = false;
                    }
                }

                ui.separator();

                if !self.raw_zpl_mode {
                    if ui.checkbox(&mut self.cutting_enabled, "Enable Cutting").changed() {
                        self.toggle_cutting(self.cutting_enabled);
                    }
                    ui.separator();
                }

                let button_enabled = self.is_dirty && !self.is_loading;
                let button_text = if self.is_loading {
                    "Loading..."
                } else {
                    "Apply Changes"
                };
                let button = egui::Button::new(button_text);

                if ui.add_enabled(button_enabled, button).clicked() {
                    let zpl = self.get_zpl_text();
                    println!("Rendering ZPL:\n{}\n", zpl);
                    self.render_zpl(ui_context);
                }

                if self.is_loading {
                    ui.spinner();
                }

                ui.separator();

                if ui.add_enabled(!self.is_scanning, egui::Button::new("Scan for Printers")).clicked() {
                    self.scan_for_printers();
                }

                if self.is_scanning {
                    ui.spinner();
                }

                ui.separator();

                ui.horizontal(|ui| {
                    ui.label("Manual IP:");
                    let response = ui.text_edit_singleline(&mut self.manual_ip);

                    let enter_pressed = response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));

                    if ui.button("Add").clicked() || enter_pressed {
                        self.add_manual_printer();
                    }
                });

                if !self.printers.is_empty() {
                    let selected_text = if let Some(idx) = self.selected_printer {
                        self.printers[idx].name.as_str()
                    } else {
                        "Select..."
                    };

                    egui::ComboBox::from_id_salt("printer_selector")
                        .selected_text(selected_text)
                        .show_ui(ui, |ui| {
                            for (idx, printer) in self.printers.iter().enumerate() {
                                if ui.selectable_label(Some(idx) == self.selected_printer, &printer.name).clicked() {
                                    self.selected_printer = Some(idx);
                                }
                            }
                        });

                    if ui.add_enabled(self.selected_printer.is_some(), egui::Button::new("Send to Printer")).clicked() {
                        self.send_to_printer();
                    }

                    ui.separator();

                    ui.label("Query:");
                    let query_button_enabled = self.selected_printer.is_some() && !self.is_querying;
                    egui::ComboBox::from_id_salt("query_selector")
                        .selected_text("Select Query...")
                        .show_ui(ui, |ui| {
                            if ui.add_enabled(query_button_enabled, egui::Button::new("Printer Status (ES)")).clicked() {
                                self.query_printer("ES", ui.ctx());
                            }
                            if ui.add_enabled(query_button_enabled, egui::Button::new("Serial Number (SN)")).clicked() {
                                self.query_printer("SN", ui.ctx());
                            }
                            if ui.add_enabled(query_button_enabled, egui::Button::new("Hardware Address (HA)")).clicked() {
                                self.query_printer("HA", ui.ctx());
                            }
                            if ui.add_enabled(query_button_enabled, egui::Button::new("Odometer (OD)")).clicked() {
                                self.query_printer("OD", ui.ctx());
                            }
                            if ui.add_enabled(query_button_enabled, egui::Button::new("Printhead Life (PH)")).clicked() {
                                self.query_printer("PH", ui.ctx());
                            }
                            if ui.add_enabled(query_button_enabled, egui::Button::new("Plug and Play (PP)")).clicked() {
                                self.query_printer("PP", ui.ctx());
                            }
                        });

                    if self.is_querying {
                        ui.spinner();
                    }
                }

                if let Some(ref status) = self.print_status {
                    ui.label(egui::RichText::new(status).color(egui::Color32::LIGHT_BLUE));
                }

                if let Some(ref error) = self.error_message {
                    ui.label(egui::RichText::new(error).color(egui::Color32::RED));
                }
            });
        });

        egui::CentralPanel::default().show(ui_context, |ui| {
            ui.columns(2, |columns| {
                columns[0].vertical(|ui| {
                        ui.horizontal(|ui| {
                            ui.heading("ZPL Commands");
                            ui.separator();
                            if ui.checkbox(&mut self.raw_zpl_mode, "Raw ZPL Mode").changed() {
                                if self.raw_zpl_mode {
                                    self.raw_zpl_input = self.get_zpl_text();
                                }
                            }
                            if !self.raw_zpl_mode {
                                ui.checkbox(&mut self.show_raw_text, "Show Raw ZPL");
                            }
                            if ui.button("Copy ZPL").clicked() {
                                ui.ctx().copy_text(self.get_zpl_text());
                            }
                        });
                        ui.separator();

                        if self.raw_zpl_mode {
                            ui.label("Enter raw ZPL code below:");
                            egui::ScrollArea::vertical()
                                .auto_shrink([false, false])
                                .show(ui, |ui| {
                                    if ui.add(
                                        egui::TextEdit::multiline(&mut self.raw_zpl_input)
                                            .code_editor()
                                            .desired_width(f32::INFINITY)
                                            .desired_rows(25)
                                    ).changed() {
                                        self.is_dirty = true;
                                    }
                                });
                            ui.horizontal(|ui| {
                                if ui.button("Clear").clicked() {
                                    self.raw_zpl_input.clear();
                                    self.is_dirty = true;
                                }
                                let example_response = egui::ComboBox::from_id_salt("example_selector")
                                    .selected_text("Load Example...")
                                    .show_ui(ui, |ui| {
                                        let mut selected = None;
                                        if ui.selectable_label(false, "Hello World").clicked() {
                                            selected = Some("^XA\n^FO50,50\n^A0N,50,50\n^FDHello World^FS\n^FO50,150\n^GB300,2,2^FS\n^FO50,200\n^A0N,30,30\n^FDRaw ZPL Entry^FS\n^XZ");
                                        }
                                        if ui.selectable_label(false, "Graphic Test (32x32)").clicked() {
                                            selected = Some("^XA\n^FO50,50^A0N,30,30^FDGraphic Below:^FS\n^FO50,100^GFA,128,128,4,FFFFFFFFFFFFFFFFC0000003C0000003C0000003C0000003C0000003C0000003C0000003C0000003C0000003C0000003C0000003C0000003C0000003C0000003C0000003C0000003C0000003C0000003C0000003C0000003C0000003C0000003C0000003C0000003C0000003C0000003C0000003C0000003FFFFFFFFFFFFFFFF^FS\n^FO50,200^A0N,30,30^FD32x32 Box^FS\n^XZ");
                                        }
                                        if ui.selectable_label(false, "Large Graphic (64x64)").clicked() {
                                            selected = Some("^XA\n^FO100,100\n^GFA,512,512,8,FFFFFFFFFFFFFFFF80000000000000018000000000000001800000000000000180000000000000018000000000000001800000000000000180000000000000018000000000000001800000000000000180000000000000018000000000000001800000000000000180000000000000018000000000000001800000000000000180000000000000018000000000000001800000000000000180000000000000018000000000000001800000000000000180000000000000018000000000000001800000000000000180000000000000018000000000000001800000000000000180000000000000018000000000000001800000000000000180000000000000018000000000000001800000000000000180000000000000018000000000000001800000000000000180000000000000018000000000000001800000000000000180000000000000018000000000000001800000000000000180000000000000018000000000000001800000000000000180000000000000018000000000000001800000000000000180000000000000018000000000000001800000000000000180000000000000018000000000000001800000000000000180000000000000018000000000000001800000000000000180000000000000018000000000000001800000000000000180000000000000018000000000000001800000000000000180000000000000018000000000000001800000000000000180000000000000018000000000000001FFFFFFFFFFFFFFFF^FS\n^XZ");
                                        }
                                        selected
                                    });
                                if let Some(inner) = example_response.inner {
                                    if let Some(example) = inner {
                                        self.raw_zpl_input = example.to_string();
                                        self.is_dirty = true;
                                    }
                                }
                                if ui.button("Paste from Clipboard").clicked() {
                                    #[cfg(not(target_arch = "wasm32"))]
                                    {
                                        use arboard::Clipboard;
                                        if let Ok(mut clipboard) = Clipboard::new() {
                                            if let Ok(text) = clipboard.get_text() {
                                                self.raw_zpl_input = text;
                                                self.is_dirty = true;
                                            }
                                        }
                                    }
                                    #[cfg(target_arch = "wasm32")]
                                    {
                                        ui.ctx().output_mut(|o| {
                                            o.copied_text = "Clipboard not supported in WASM".to_string();
                                        });
                                    }
                                }
                            });
                        } else if self.show_raw_text {
                            egui::ScrollArea::vertical()
                                .auto_shrink([false, false])
                                .show(ui, |ui| {
                                    let zpl_text = self.get_zpl_text();
                                    ui.add(
                                        egui::TextEdit::multiline(&mut zpl_text.as_str())
                                            .code_editor()
                                            .desired_width(f32::INFINITY)
                                            .desired_rows(20)
                                            .interactive(false),
                                    );
                                });
                        } else {
                            ui.horizontal(|ui| {
                                ui.label("Add Command:");
                                let response = egui::ComboBox::from_id_salt("command_selector")
                                    .selected_text("Select...")
                                    .show_ui(ui, |ui| {
                                        let mut selected = None;
                                        for (idx, (name, _)) in
                                            ZplCommand::all_command_types().iter().enumerate()
                                        {
                                            if ui.selectable_label(false, *name).clicked() {
                                                selected = Some(idx);
                                            }
                                        }
                                        selected
                                    });

                                if let Some(inner) = response.inner {
                                    if let Some(idx) = inner {
                                        let (_, template) = &ZplCommand::all_command_types()[idx];
                                        self.zpl_commands.push(template.clone());
                                        self.is_dirty = true;
                                    }
                                }
                            });

                            ui.separator();

                            egui::ScrollArea::vertical()
                                .auto_shrink([false, false])
                                .show(ui, |ui| {
                                    let mut to_remove = None;
                                    let mut to_move_up = None;
                                    let mut to_move_down = None;
                                    let command_count = self.zpl_commands.len();

                                    for idx in 0..command_count {
                                        ui.group(|ui| {
                                            ui.horizontal(|ui| {
                                                ui.label(
                                                    egui::RichText::new(format!("#{}", idx + 1))
                                                        .strong()
                                                        .color(egui::Color32::GRAY),
                                                );

                                                ui.vertical(|ui| {
                                                    ui.label(egui::RichText::new(
                                                        self.zpl_commands[idx].command_name(),
                                                    )
                                                    .strong());
                                                    self.render_command_editor(ui, idx);
                                                });

                                                ui.vertical(|ui| {
                                                    if ui.add_enabled(idx > 0, egui::Button::new("^")).clicked() {
                                                        to_move_up = Some(idx);
                                                    }
                                                    if ui.add_enabled(idx < command_count - 1, egui::Button::new("v")).clicked() {
                                                        to_move_down = Some(idx);
                                                    }
                                                });

                                                if ui.button("🗑").clicked() {
                                                    to_remove = Some(idx);
                                                }
                                            });
                                        });
                                        ui.add_space(4.0);
                                    }

                                    if let Some(idx) = to_remove {
                                        self.zpl_commands.remove(idx);
                                        self.is_dirty = true;
                                    }
                                    if let Some(idx) = to_move_up {
                                        self.zpl_commands.swap(idx, idx - 1);
                                        self.is_dirty = true;
                                    }
                                    if let Some(idx) = to_move_down {
                                        self.zpl_commands.swap(idx, idx + 1);
                                        self.is_dirty = true;
                                    }
                                });
                        }
                    });

                columns[1].vertical(|ui| {
                        ui.heading("Rendered Output");
                        ui.separator();

                        if self.is_loading {
                            ui.centered_and_justified(|ui| {
                                ui.vertical_centered(|ui| {
                                    ui.spinner();
                                    ui.label("Rendering ZPL...");
                                });
                            });
                        } else if let Some(ref texture) = self.rendered_image {
                            egui::ScrollArea::both()
                                .auto_shrink([false, false])
                                .show(ui, |ui| {
                                    let size = texture.size_vec2();
                                    let max_width = ui.available_width();
                                    let max_height = ui.available_height();

                                    let scale = (max_width / size.x)
                                        .min(max_height / size.y)
                                        .min(2.0)
                                        .max(0.5);
                                    let display_size = size * scale;

                                    ui.centered_and_justified(|ui| {
                                        ui.add(egui::Image::new(egui::load::SizedTexture::new(
                                            texture.id(),
                                            display_size,
                                        )));
                                    });
                                });
                        } else {
                            ui.centered_and_justified(|ui| {
                                ui.label("Loading...");
                            });
                        }

                        if let Some(ref status) = self.parsed_status {
                            let mut clear_status = false;

                            ui.add_space(20.0);
                            ui.separator();
                            ui.add_space(10.0);

                            ui.horizontal(|ui| {
                                ui.heading("Printer Status");
                                if ui.button("Clear").clicked() {
                                    clear_status = true;
                                }
                            });
                            ui.separator();
                            ui.add_space(8.0);

                            egui::ScrollArea::vertical()
                                .max_height(300.0)
                                .auto_shrink([false, false])
                                .show(ui, |ui| {
                                    if status.is_ok() {
                                        ui.label(egui::RichText::new("✓ Printer is operating normally")
                                            .color(egui::Color32::GREEN)
                                            .size(16.0)
                                            .strong());
                                    } else {
                                        if status.has_errors() {
                                            ui.label(egui::RichText::new("⚠ ERRORS DETECTED")
                                                .color(egui::Color32::RED)
                                                .size(16.0)
                                                .strong());
                                            ui.add_space(10.0);
                                            for error in status.errors.to_descriptions() {
                                                ui.horizontal(|ui| {
                                                    ui.label(egui::RichText::new("•").color(egui::Color32::RED).size(14.0));
                                                    ui.label(egui::RichText::new(error).color(egui::Color32::RED).size(13.0));
                                                });
                                                ui.add_space(3.0);
                                            }
                                            ui.add_space(12.0);
                                        }

                                        if status.has_warnings() {
                                            ui.label(egui::RichText::new("⚠ Warnings")
                                                .color(egui::Color32::YELLOW)
                                                .size(16.0)
                                                .strong());
                                            ui.add_space(10.0);
                                            for warning in status.warnings.to_descriptions() {
                                                ui.horizontal(|ui| {
                                                    ui.label(egui::RichText::new("•").color(egui::Color32::YELLOW).size(14.0));
                                                    ui.label(egui::RichText::new(warning).color(egui::Color32::YELLOW).size(13.0));
                                                });
                                                ui.add_space(3.0);
                                            }
                                        }
                                    }
                                });

                            if clear_status {
                                self.parsed_status = None;
                            }
                        }

                        if self.query_response.is_some() {
                            let response_text = self.query_response.clone().unwrap();
                            let mut clear_response = false;
                            let mut copy_response = false;

                            ui.add_space(20.0);
                            ui.separator();
                            ui.add_space(10.0);

                            ui.horizontal(|ui| {
                                ui.heading("Query Response");
                                if ui.button("Clear").clicked() {
                                    clear_response = true;
                                }
                                if ui.button("Copy").clicked() {
                                    copy_response = true;
                                }
                            });
                            ui.separator();
                            ui.add_space(8.0);

                            egui::ScrollArea::vertical()
                                .max_height(300.0)
                                .auto_shrink([false, false])
                                .show(ui, |ui| {
                                    ui.add(
                                        egui::TextEdit::multiline(&mut response_text.as_str())
                                            .code_editor()
                                            .desired_width(f32::INFINITY)
                                            .interactive(false)
                                    );
                                });

                            if clear_response {
                                self.query_response = None;
                            }
                            if copy_response {
                                ui.ctx().copy_text(response_text);
                            }
                        }
                    });
            });
        });
    }

    fn on_keyboard_input(&mut self, world: &mut World, key_code: KeyCode, key_state: KeyState) {
        if matches!(
            (key_code, key_state),
            (KeyCode::KeyQ, KeyState::Pressed)
        ) {
            world.resources.window.should_exit = true;
        }
    }
}

