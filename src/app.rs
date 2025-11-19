use std::sync::{Arc, Mutex};

use zebras::{
    labelary::LabelaryClient,
    printer::ZplPrinter,
    printer_status::*,
    zpl::{FieldOrientation, FontOrientation, ZplCommand, commands_to_zpl},
};

pub struct Zebras {
    zpl_commands: Vec<ZplCommand>,
    rendered_image: Option<egui::TextureHandle>,
    is_dirty: bool,
    error_message: Option<String>,
    is_loading: bool,
    needs_initial_render: bool,
    pending_response: Arc<Mutex<Option<Result<Vec<u8>, String>>>>,
    show_raw_text: bool,
    raw_zpl_mode: bool,
    raw_zpl_input: String,
    printers: Vec<ZplPrinter>,
    selected_printer: Option<usize>,
    print_status: Option<String>,
    manual_ip: String,
    image_load_status: Option<String>,
    graphic_threshold: u8,
    needs_render_after_image: bool,
    pending_query_result: Arc<Mutex<Option<Result<String, String>>>>,
    query_response: Option<String>,
    is_querying: bool,
    parsed_status: Option<PrinterStatus>,
    printer_info: PrinterInfo,
    last_query_type: Option<String>,
    show_query_window: bool,
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
            show_raw_text: false,
            raw_zpl_mode: false,
            raw_zpl_input: String::new(),
            printers: Vec::new(),
            selected_printer: None,
            print_status: None,
            manual_ip: "10.73.27.7".to_string(),
            image_load_status: None,
            graphic_threshold: 128,
            needs_render_after_image: false,
            pending_query_result: Arc::new(Mutex::new(None)),
            query_response: None,
            is_querying: false,
            parsed_status: None,
            printer_info: PrinterInfo::default(),
            last_query_type: None,
            show_query_window: false,
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
                        color: None,
                        rounding: None,
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
            (
                "Download & Reuse Graphic",
                vec![
                    ZplCommand::DownloadGraphic {
                        name: "LOGO".to_string(),
                        width: 32,
                        height: 32,
                        data: "FFFFFFFFFFFFFFFFC0000003C0000003C0000003C0000003C0000003C0000003C0000003C0000003C0000003C0000003C0000003C0000003C0000003C0000003C0000003C0000003C0000003C0000003C0000003C0000003C0000003C0000003C0000003C0000003C0000003C0000003C0000003C0000003FFFFFFFFFFFFFFFF".to_string(),
                    },
                    ZplCommand::StartFormat,
                    ZplCommand::FieldOrigin { x: 50, y: 50 },
                    ZplCommand::Font {
                        orientation: FontOrientation::Normal,
                        height: 30,
                        width: 30,
                    },
                    ZplCommand::FieldData {
                        data: "Stored Graphics Demo".to_string(),
                    },
                    ZplCommand::FieldSeparator,
                    ZplCommand::FieldOrigin { x: 50, y: 100 },
                    ZplCommand::RecallGraphic {
                        name: "LOGO".to_string(),
                        magnification_x: 1,
                        magnification_y: 1,
                    },
                    ZplCommand::FieldSeparator,
                    ZplCommand::FieldOrigin { x: 150, y: 100 },
                    ZplCommand::RecallGraphic {
                        name: "LOGO".to_string(),
                        magnification_x: 2,
                        magnification_y: 2,
                    },
                    ZplCommand::FieldSeparator,
                    ZplCommand::FieldOrigin { x: 50, y: 200 },
                    ZplCommand::RecallGraphic {
                        name: "LOGO".to_string(),
                        magnification_x: 1,
                        magnification_y: 1,
                    },
                    ZplCommand::FieldSeparator,
                    ZplCommand::FieldOrigin { x: 150, y: 200 },
                    ZplCommand::RecallGraphic {
                        name: "LOGO".to_string(),
                        magnification_x: 1,
                        magnification_y: 1,
                    },
                    ZplCommand::FieldSeparator,
                    ZplCommand::EndFormat,
                ],
            ),
            (
                "Framed Label with Image",
                vec![
                    ZplCommand::StartFormat,
                    ZplCommand::FieldOrigin { x: 40, y: 20 },
                    ZplCommand::GraphicBox {
                        width: 760,
                        height: 570,
                        thickness: 8,
                        color: Some('B'),
                        rounding: Some(2),
                    },
                    ZplCommand::FieldSeparator,
                    ZplCommand::FieldOrigin { x: 40, y: 150 },
                    ZplCommand::GraphicBox {
                        width: 760,
                        height: 0,
                        thickness: 8,
                        color: Some('B'),
                        rounding: None,
                    },
                    ZplCommand::FieldSeparator,
                    ZplCommand::FieldOrigin { x: 0, y: 30 },
                    ZplCommand::GraphicField {
                        width: 400,
                        height: 86,
                        data: String::new(),
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

    fn save_template(&mut self) {
        #[cfg(not(target_arch = "wasm32"))]
        {
            if let Some(path) = rfd::FileDialog::new()
                .add_filter("JSON Template", &["json"])
                .set_file_name("template.json")
                .save_file()
            {
                match serde_json::to_string_pretty(&self.zpl_commands) {
                    Ok(json) => match std::fs::write(&path, json) {
                        Ok(_) => {
                            self.print_status =
                                Some(format!("Template saved to {:?}", path.file_name()));
                        }
                        Err(error) => {
                            self.print_status = Some(format!("Failed to save: {}", error));
                        }
                    },
                    Err(error) => {
                        self.print_status = Some(format!("Failed to serialize: {}", error));
                    }
                }
            }
        }

        #[cfg(target_arch = "wasm32")]
        {
            self.print_status = Some("Template save not available in WASM".to_string());
        }
    }

    fn load_template(&mut self) {
        #[cfg(not(target_arch = "wasm32"))]
        {
            if let Some(path) = rfd::FileDialog::new()
                .add_filter("JSON Template", &["json"])
                .pick_file()
            {
                match std::fs::read_to_string(&path) {
                    Ok(json) => match serde_json::from_str::<Vec<ZplCommand>>(&json) {
                        Ok(commands) => {
                            self.zpl_commands = commands;
                            self.is_dirty = true;
                            self.print_status =
                                Some(format!("Template loaded from {:?}", path.file_name()));
                        }
                        Err(error) => {
                            self.print_status =
                                Some(format!("Failed to parse template: {}", error));
                        }
                    },
                    Err(error) => {
                        self.print_status = Some(format!("Failed to read file: {}", error));
                    }
                }
            }
        }

        #[cfg(target_arch = "wasm32")]
        {
            self.print_status = Some("Template load not available in WASM".to_string());
        }
    }

    fn send_to_printer(&mut self) {
        if let Some(idx) = self.selected_printer {
            if let Some(printer) = self.printers.get(idx) {
                let mut zpl = String::new();

                zpl.push_str("^XA^MMT^XZ\n");

                zpl.push_str(&self.get_zpl_text());

                match zebras::printer::send_to_printer(printer, &zpl) {
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
                let new_index = self.printers.len() - 1;
                self.selected_printer = Some(new_index);
                self.print_status = Some(format!("Added and selected printer at {}", ip));
                self.manual_ip.clear();
            } else {
                let existing_index = self.printers.iter().position(|p| p.ip == ip);
                self.selected_printer = existing_index;
                self.print_status = Some(format!("Printer at {} already exists, selected", ip));
            }
        }
    }

    fn query_printer(&mut self, query_type: &str, ctx: &egui::Context) {
        if let Some(idx) = self.selected_printer {
            if let Some(printer) = self.printers.get(idx).cloned() {
                self.is_querying = true;
                self.query_response = Some("Querying printer...".to_string());
                self.last_query_type = Some(query_type.to_string());

                let query = if query_type == "HM" {
                    format!("~{}\r\n", query_type)
                } else {
                    format!("~HQ{}\r\n", query_type)
                };
                let ctx = ctx.clone();
                let pending_result = Arc::clone(&self.pending_query_result);

                #[cfg(not(target_arch = "wasm32"))]
                {
                    std::thread::spawn(move || {
                        let response = zebras::printer::query_printer(&printer, &query);
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

    fn query_all(&mut self, ctx: &egui::Context) {
        if let Some(idx) = self.selected_printer {
            if let Some(printer) = self.printers.get(idx).cloned() {
                self.is_querying = true;
                self.query_response = Some("Starting comprehensive query...\n\n".to_string());
                self.last_query_type = Some("ALL".to_string());

                let ctx = ctx.clone();
                let pending_result = Arc::clone(&self.pending_query_result);

                #[cfg(not(target_arch = "wasm32"))]
                {
                    std::thread::spawn(move || {
                        let queries = vec![
                            ("PRINTER STATUS (ES)", "~HQES\r\n"),
                            ("HOST STATUS (HS)", "~HQHS\r\n"),
                            ("HOST IDENTIFICATION (HI)", "~HQHI\r\n"),
                            ("SERIAL NUMBER (SN)", "~HQSN\r\n"),
                            ("HARDWARE ADDRESS (HA)", "~HQHA\r\n"),
                            ("ODOMETER (OD)", "~HQOD\r\n"),
                            ("PRINTHEAD LIFE (PH)", "~HQPH\r\n"),
                            ("PRINT CONFIGURATION (PR)", "~HQPR\r\n"),
                            ("CONFIGURATION STATUS (CM)", "~HQCM\r\n"),
                            ("BATTERY CAPACITY (BC)", "~HQBC\r\n"),
                            ("USB DEVICE ID (UI)", "~HQUI\r\n"),
                            ("LABEL DIMENSIONS (LD)", "~HQLD\r\n"),
                            ("LABEL COUNT (LC)", "~HQLC\r\n"),
                            ("FILE SYSTEM INFO (FS)", "~HQFS\r\n"),
                            ("NETWORK ROUTER (NR)", "~HQNR\r\n"),
                            ("MAINTENANCE ALERT (MA)", "~HQMA\r\n"),
                            ("SENSOR/MEDIA STATUS (SM)", "~HQSM\r\n"),
                            ("ALERTS (AL)", "~HQAL\r\n"),
                            ("FIRMWARE VERSION (FW)", "~HQFW\r\n"),
                            ("SUPPLIES STATUS (ST)", "~HQST\r\n"),
                            ("DARKNESS SETTINGS (DA)", "~HQDA\r\n"),
                            ("PLUG AND PLAY (PP)", "~HQPP\r\n"),
                            ("HOST RAM STATUS (HM)", "~HM\r\n"),
                        ];

                        let total = queries.len();
                        for (index, (name, query)) in queries.iter().enumerate() {
                            let progress = format!("[{}/{}] ", index + 1, total);
                            let mut section = format!("=== {} ===\n", name);

                            match zebras::printer::query_printer(&printer, query) {
                                Ok(response) => {
                                    if response.trim().is_empty() {
                                        section.push_str("(No response or not supported)\n");
                                    } else if name == &"HOST RAM STATUS (HM)" {
                                        if let Some(memory) =
                                            zebras::printer_status::PrinterInfo::parse_memory_status(
                                                &response,
                                            )
                                        {
                                            let used_kb = memory
                                                .max_available_kb
                                                .saturating_sub(memory.current_available_kb);
                                            let usage_percent = if memory.max_available_kb > 0 {
                                                (used_kb as f32 / memory.max_available_kb as f32
                                                    * 100.0)
                                                    as u32
                                            } else {
                                                0
                                            };
                                            section.push_str(&format!(
                                                "Total RAM Installed:       {} KB\nMaximum Available:         {} KB\nCurrently Available:       {} KB\nMemory Used:               {} KB\nMemory Usage:              {}%\n",
                                                memory.total_ram_kb,
                                                memory.max_available_kb,
                                                memory.current_available_kb,
                                                used_kb,
                                                usage_percent
                                            ));
                                        } else {
                                            section.push_str(&response);
                                        }
                                    } else {
                                        section.push_str(&response);
                                    }
                                }
                                Err(e) => {
                                    section.push_str(&format!("Error: {}\n", e));
                                }
                            }
                            section.push_str("\n\n");

                            if let Ok(mut guard) = pending_result.lock() {
                                let current = guard
                                    .as_ref()
                                    .and_then(|r| r.as_ref().ok())
                                    .map(|s| s.clone())
                                    .unwrap_or_else(|| {
                                        format!("Starting comprehensive query...\n\n")
                                    });

                                let is_complete = index == total - 1;
                                let complete_marker = if is_complete {
                                    "\n___COMPLETE___\n"
                                } else {
                                    ""
                                };
                                *guard = Some(Ok(format!(
                                    "{}{}{}{}",
                                    current, progress, section, complete_marker
                                )));
                            }
                            ctx.request_repaint();
                        }
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

    fn render_zpl(&mut self, ctx: &egui::Context) {
        self.error_message = None;
        self.is_loading = true;

        let zpl = self.get_zpl_text();

        let ctx = ctx.clone();
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
            ZplCommand::StartFormat
            | ZplCommand::EndFormat
            | ZplCommand::FieldSeparator
            | ZplCommand::MediaModeDelayed
            | ZplCommand::MediaModeTearOff
            | ZplCommand::CutNow => {}
            ZplCommand::FieldOrigin { x, y } => {
                ui.horizontal(|ui| {
                    ui.label("X:");
                    if ui.add(egui::DragValue::new(x).speed(1)).lost_focus() {
                        self.is_dirty = true;
                    }
                    ui.label("Y:");
                    if ui.add(egui::DragValue::new(y).speed(1)).lost_focus() {
                        self.is_dirty = true;
                    }
                });
            }
            ZplCommand::Font { height, width, .. } => {
                ui.horizontal(|ui| {
                    ui.label("Height:");
                    if ui.add(egui::DragValue::new(height).speed(1)).lost_focus() {
                        self.is_dirty = true;
                    }
                    ui.label("Width:");
                    if ui.add(egui::DragValue::new(width).speed(1)).lost_focus() {
                        self.is_dirty = true;
                    }
                });
            }
            ZplCommand::FieldData { data } => {
                ui.horizontal(|ui| {
                    ui.label("Text:");
                    if ui.text_edit_singleline(data).lost_focus() {
                        self.is_dirty = true;
                    }
                });
            }
            ZplCommand::GraphicBox {
                width,
                height,
                thickness,
                color,
                rounding,
            } => {
                ui.push_id(idx, |ui| {
                    ui.vertical(|ui| {
                        ui.horizontal(|ui| {
                            ui.label("W:");
                            if ui.add(egui::DragValue::new(width).speed(1)).lost_focus() {
                                self.is_dirty = true;
                            }
                            ui.label("H:");
                            if ui.add(egui::DragValue::new(height).speed(1)).lost_focus() {
                                self.is_dirty = true;
                            }
                            ui.label("T:");
                            if ui.add(egui::DragValue::new(thickness).speed(1)).lost_focus() {
                                self.is_dirty = true;
                            }
                        });
                        ui.horizontal(|ui| {
                            ui.label("Color:");
                            let mut color_selection = match color {
                                Some('B') => 0,
                                Some('W') => 1,
                                _ => 2,
                            };
                            let prev_selection = color_selection;
                            ui.radio_value(&mut color_selection, 0, "Black");
                            ui.radio_value(&mut color_selection, 1, "White");
                            ui.radio_value(&mut color_selection, 2, "Default");
                            if color_selection != prev_selection {
                                *color = match color_selection {
                                    0 => Some('B'),
                                    1 => Some('W'),
                                    _ => None,
                                };
                                self.is_dirty = true;
                            }
                        });
                        ui.horizontal(|ui| {
                            ui.label("Rounding:");
                            let mut rounding_value = rounding.unwrap_or(0);
                            let response = ui.add(egui::Slider::new(&mut rounding_value, 0..=8));
                            if response.changed() {
                                *rounding = if rounding_value > 0 {
                                    Some(rounding_value)
                                } else {
                                    None
                                };
                            }
                            if response.lost_focus() {
                                self.is_dirty = true;
                            }
                        });
                        ui.label(egui::RichText::new("Tip: For horizontal line, set height=1-5. For vertical line, set width=1-5").small().color(egui::Color32::GRAY));
                    });
                });
            }
            ZplCommand::GraphicField {
                width,
                height,
                data,
            } => {
                ui.vertical(|ui| {
                    ui.label(
                        egui::RichText::new("Note: Add Field Origin (^FO) command before this")
                            .small()
                            .color(egui::Color32::GRAY),
                    );
                    ui.horizontal(|ui| {
                        ui.label("W:");
                        if ui.add(egui::DragValue::new(width).speed(1)).lost_focus() {
                            self.is_dirty = true;
                        }
                        ui.label("H:");
                        if ui.add(egui::DragValue::new(height).speed(1)).lost_focus() {
                            self.is_dirty = true;
                        }
                    });
                    ui.horizontal(|ui| {
                        ui.label("Data (hex):");
                        if ui.text_edit_singleline(data).lost_focus() {
                            self.is_dirty = true;
                        }
                    });
                    ui.label(format!("Data length: {} chars", data.len()));
                    ui.separator();
                    ui.label("Load from image:");
                    ui.horizontal(|ui| {
                        if ui.button("Select Image").clicked() {
                            self.image_load_status = Some("Opening file dialog...".to_string());
                            if let Some(path) = rfd::FileDialog::new()
                                .add_filter("Image", &["png", "jpg", "jpeg", "bmp", "gif"])
                                .pick_file()
                            {
                                self.image_load_status =
                                    Some(format!("Loading {:?}...", path.file_name()));
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
                                        *data = zebras::zpl::image_to_zpl_hex(
                                            &resized_image,
                                            self.graphic_threshold,
                                        );
                                        self.image_load_status = Some(format!(
                                            "Image loaded! {} chars - rendering...",
                                            data.len()
                                        ));
                                        self.needs_render_after_image = true;
                                    }
                                    Err(e) => {
                                        self.image_load_status =
                                            Some(format!("Error loading image: {}", e));
                                    }
                                }
                            } else {
                                self.image_load_status = Some("No file selected".to_string());
                            }
                        }
                    });
                    ui.horizontal(|ui| {
                        ui.label("Threshold:");
                        ui.add(egui::Slider::new(&mut self.graphic_threshold, 0..=255));
                    });
                    ui.label(
                        egui::RichText::new("(Lower = more black, Higher = more white)")
                            .small()
                            .color(egui::Color32::GRAY),
                    );
                    if let Some(ref status) = self.image_load_status {
                        ui.label(egui::RichText::new(status).color(egui::Color32::LIGHT_BLUE));
                    }
                });
            }
            ZplCommand::DownloadGraphic {
                name,
                width,
                height,
                data,
            } => {
                ui.vertical(|ui| {
                    ui.label(
                        egui::RichText::new("Stores graphic in printer memory")
                            .small()
                            .color(egui::Color32::GRAY),
                    );
                    ui.horizontal(|ui| {
                        ui.label("Name:");
                        if ui.text_edit_singleline(name).lost_focus() {
                            self.is_dirty = true;
                        }
                    });
                    ui.horizontal(|ui| {
                        ui.label("W:");
                        if ui.add(egui::DragValue::new(width).speed(1)).lost_focus() {
                            self.is_dirty = true;
                        }
                        ui.label("H:");
                        if ui.add(egui::DragValue::new(height).speed(1)).lost_focus() {
                            self.is_dirty = true;
                        }
                    });
                    ui.horizontal(|ui| {
                        ui.label("Data (hex):");
                        if ui.text_edit_singleline(data).lost_focus() {
                            self.is_dirty = true;
                        }
                    });
                    ui.label(format!("Data length: {} chars", data.len()));
                    ui.separator();
                    ui.label("Load from image:");
                    ui.horizontal(|ui| {
                        if ui.button("Select Image").clicked() {
                            self.image_load_status = Some("Opening file dialog...".to_string());
                            if let Some(path) = rfd::FileDialog::new()
                                .add_filter("Image", &["png", "jpg", "jpeg", "bmp", "gif"])
                                .pick_file()
                            {
                                self.image_load_status =
                                    Some(format!("Loading {:?}...", path.file_name()));
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
                                        *data = zebras::zpl::image_to_zpl_hex(
                                            &resized_image,
                                            self.graphic_threshold,
                                        );
                                        self.image_load_status = Some(format!(
                                            "Image loaded! {} chars - rendering...",
                                            data.len()
                                        ));
                                        self.needs_render_after_image = true;
                                    }
                                    Err(e) => {
                                        self.image_load_status =
                                            Some(format!("Error loading image: {}", e));
                                    }
                                }
                            } else {
                                self.image_load_status = None;
                            }
                        }
                    });
                    ui.horizontal(|ui| {
                        ui.label("Threshold:");
                        ui.add(egui::Slider::new(&mut self.graphic_threshold, 0..=255));
                    });
                    ui.label(
                        egui::RichText::new("(Lower = more black, Higher = more white)")
                            .small()
                            .color(egui::Color32::GRAY),
                    );
                    if let Some(ref status) = self.image_load_status {
                        ui.label(egui::RichText::new(status).color(egui::Color32::LIGHT_BLUE));
                    }
                });
            }
            ZplCommand::RecallGraphic {
                name,
                magnification_x,
                magnification_y,
            } => {
                ui.vertical(|ui| {
                    ui.label(egui::RichText::new("Note: Add Field Origin (^FO) before this. Graphic must be stored via ~DG first").small().color(egui::Color32::GRAY));
                    ui.horizontal(|ui| {
                        ui.label("Name:");
                        if ui.text_edit_singleline(name).lost_focus() {
                            self.is_dirty = true;
                        }
                    });
                    ui.horizontal(|ui| {
                        ui.label("Mag X:");
                        if ui.add(egui::DragValue::new(magnification_x).speed(1).range(1..=10)).lost_focus() {
                            self.is_dirty = true;
                        }
                        ui.label("Mag Y:");
                        if ui.add(egui::DragValue::new(magnification_y).speed(1).range(1..=10)).lost_focus() {
                            self.is_dirty = true;
                        }
                    });
                });
            }
            _ => {
                ui.label("(Complex command - not yet editable)");
            }
        }
    }

    fn process_image_response(&mut self, image_data: Vec<u8>, ctx: &egui::Context) {
        let image = image::load_from_memory(&image_data);
        match image {
            Ok(img) => {
                let size = [img.width() as _, img.height() as _];
                let rgba = img.to_rgba8();
                let pixels = rgba.as_flat_samples();
                let color_image = egui::ColorImage::from_rgba_unmultiplied(size, pixels.as_slice());
                let texture =
                    ctx.load_texture("zpl_render", color_image, egui::TextureOptions::LINEAR);
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

impl eframe::App for Zebras {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let pending_result = if let Ok(mut guard) = self.pending_response.try_lock() {
            guard.take()
        } else {
            None
        };

        if let Some(response) = pending_result {
            match response {
                Ok(image_data) => {
                    self.process_image_response(image_data, ctx);
                }
                Err(e) => {
                    self.error_message = Some(e);
                    self.is_loading = false;
                }
            }
        }

        let pending_query = if let Ok(mut guard) = self.pending_query_result.try_lock() {
            if let Some(ref result) = *guard {
                if let Ok(response) = result {
                    if let Some(ref query_type) = self.last_query_type {
                        if query_type == "ALL" {
                            let is_complete = response.contains("___COMPLETE___");
                            if is_complete {
                                guard.take()
                            } else {
                                let new_response = response.replace("___COMPLETE___", "");
                                if self.query_response.as_ref() != Some(&new_response) {
                                    self.query_response = Some(new_response);
                                }
                                None
                            }
                        } else {
                            guard.take()
                        }
                    } else {
                        guard.take()
                    }
                } else {
                    guard.take()
                }
            } else {
                None
            }
        } else {
            None
        };

        if let Some(query_result) = pending_query {
            self.is_querying = false;
            match query_result {
                Ok(response) => {
                    let cleaned_response = response.replace("___COMPLETE___", "");
                    if let Some(ref query_type) = self.last_query_type {
                        match query_type.as_str() {
                            "ES" => match PrinterStatus::parse(&cleaned_response) {
                                Ok(status) => {
                                    self.parsed_status = Some(status);
                                    self.query_response = None;
                                }
                                Err(e) => {
                                    self.query_response = Some(format!(
                                        "Failed to parse status: {}\n\nRaw response:\n{}",
                                        e, cleaned_response
                                    ));
                                    self.parsed_status = None;
                                }
                            },
                            "SN" => {
                                self.printer_info.serial_number =
                                    PrinterInfo::parse_serial_number(&cleaned_response);
                                self.query_response = Some(cleaned_response.clone());
                                self.parsed_status = None;
                            }
                            "HA" => {
                                self.printer_info.hardware_address =
                                    PrinterInfo::parse_hardware_address(&cleaned_response);
                                self.query_response = Some(cleaned_response.clone());
                                self.parsed_status = None;
                            }
                            "OD" => {
                                self.printer_info.odometer =
                                    PrinterInfo::parse_odometer(&cleaned_response);
                                self.query_response = Some(cleaned_response.clone());
                                self.parsed_status = None;
                            }
                            "PH" => {
                                self.printer_info.printhead_life =
                                    PrinterInfo::parse_printhead_life(&cleaned_response);
                                self.query_response = Some(cleaned_response.clone());
                                self.parsed_status = None;
                            }
                            "PP" => {
                                self.printer_info.plug_and_play =
                                    PrinterInfo::parse_plug_and_play(&cleaned_response);
                                self.query_response = Some(cleaned_response.clone());
                                self.parsed_status = None;
                            }
                            "HS" => {
                                self.printer_info.host_status =
                                    PrinterInfo::parse_host_status(&cleaned_response);
                                self.query_response = Some(cleaned_response.clone());
                                self.parsed_status = None;
                            }
                            "SM" => {
                                self.printer_info.sensor_media_status =
                                    PrinterInfo::parse_sensor_media_status(&cleaned_response);
                                self.query_response = Some(cleaned_response.clone());
                                self.parsed_status = None;
                            }
                            "AL" => {
                                self.printer_info.alerts =
                                    PrinterInfo::parse_alerts(&cleaned_response);
                                self.query_response = Some(cleaned_response.clone());
                                self.parsed_status = None;
                            }
                            "ST" => {
                                self.printer_info.supplies_status =
                                    PrinterInfo::parse_supplies_status(&cleaned_response);
                                self.query_response = Some(cleaned_response.clone());
                                self.parsed_status = None;
                            }
                            "BC" => {
                                self.printer_info.battery_capacity =
                                    PrinterInfo::parse_battery_capacity(&cleaned_response);
                                self.query_response = Some(cleaned_response.clone());
                                self.parsed_status = None;
                            }
                            "LD" => {
                                self.printer_info.label_dimensions =
                                    PrinterInfo::parse_label_dimensions(&cleaned_response);
                                self.query_response = Some(cleaned_response.clone());
                                self.parsed_status = None;
                            }
                            "FW" => {
                                self.printer_info.firmware_version =
                                    PrinterInfo::parse_firmware_version(&cleaned_response);
                                self.query_response = Some(cleaned_response.clone());
                                self.parsed_status = None;
                            }
                            "HM" => {
                                if let Some(memory) =
                                    PrinterInfo::parse_memory_status(&cleaned_response)
                                {
                                    self.printer_info.memory_status = Some(memory);
                                    let used_kb = memory
                                        .max_available_kb
                                        .saturating_sub(memory.current_available_kb);
                                    let usage_percent = if memory.max_available_kb > 0 {
                                        (used_kb as f32 / memory.max_available_kb as f32 * 100.0)
                                            as u32
                                    } else {
                                        0
                                    };
                                    let formatted = format!(
                                        "HOST RAM STATUS\n\nTotal RAM Installed:       {} KB\nMaximum Available:         {} KB\nCurrently Available:       {} KB\nMemory Used:               {} KB\nMemory Usage:              {}%",
                                        memory.total_ram_kb,
                                        memory.max_available_kb,
                                        memory.current_available_kb,
                                        used_kb,
                                        usage_percent
                                    );
                                    self.query_response = Some(formatted);
                                } else {
                                    self.query_response = Some(format!(
                                        "Failed to parse memory status\n\nRaw response:\n{}",
                                        cleaned_response
                                    ));
                                }
                                self.parsed_status = None;
                            }
                            "ALL" => {
                                let sections: Vec<&str> = cleaned_response.split("===").collect();
                                for section in sections {
                                    if section.contains("PRINTER STATUS") {
                                        let status_part = section.split("===").next().unwrap_or("");
                                        if let Ok(status) = PrinterStatus::parse(status_part) {
                                            self.parsed_status = Some(status);
                                        }
                                    } else if section.contains("SERIAL NUMBER") {
                                        let lines: Vec<&str> = section.lines().skip(1).collect();
                                        let data = lines.join("\n");
                                        self.printer_info.serial_number =
                                            PrinterInfo::parse_serial_number(&data);
                                    } else if section.contains("HARDWARE ADDRESS") {
                                        let lines: Vec<&str> = section.lines().skip(1).collect();
                                        let data = lines.join("\n");
                                        self.printer_info.hardware_address =
                                            PrinterInfo::parse_hardware_address(&data);
                                    } else if section.contains("ODOMETER") {
                                        let lines: Vec<&str> = section.lines().skip(1).collect();
                                        let data = lines.join("\n");
                                        self.printer_info.odometer =
                                            PrinterInfo::parse_odometer(&data);
                                    } else if section.contains("PRINTHEAD LIFE") {
                                        let lines: Vec<&str> = section.lines().skip(1).collect();
                                        let data = lines.join("\n");
                                        self.printer_info.printhead_life =
                                            PrinterInfo::parse_printhead_life(&data);
                                    } else if section.contains("PLUG AND PLAY") {
                                        let lines: Vec<&str> = section.lines().skip(1).collect();
                                        let data = lines.join("\n");
                                        self.printer_info.plug_and_play =
                                            PrinterInfo::parse_plug_and_play(&data);
                                    } else if section.contains("HOST RAM STATUS") {
                                        let lines: Vec<&str> = section.lines().skip(1).collect();
                                        let data = lines.join("\n");
                                        self.printer_info.memory_status =
                                            PrinterInfo::parse_memory_status(&data);
                                    }
                                }
                                self.query_response = Some(cleaned_response.clone());
                            }
                            _ => {
                                self.query_response = Some(cleaned_response.clone());
                                self.parsed_status = None;
                            }
                        }
                    } else {
                        self.query_response = Some(cleaned_response.clone());
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
            self.render_zpl(ctx);
        }

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("ZPL Simulator");
                ui.separator();

                ui.label("Preset:");
                let preset_response = egui::ComboBox::from_id_salt(ui.next_auto_id())
                    .selected_text("Load...")
                    .show_ui(ui, |ui| {
                        let mut clicked_preset = None;
                        egui::ScrollArea::vertical()
                            .max_height(300.0)
                            .show(ui, |ui| {
                                for (name, _) in Self::get_presets() {
                                    if ui.selectable_label(false, name).clicked() {
                                        clicked_preset = Some(name);
                                    }
                                }
                            });
                        clicked_preset
                    });

                if let Some(inner) = preset_response.inner {
                    if let Some(preset_name) = inner {
                        self.load_preset(preset_name);
                        self.render_zpl(ctx);
                        self.is_dirty = false;
                    }
                }

                ui.separator();

                if ui.button("Save Template").clicked() {
                    self.save_template();
                }

                if ui.button("Load Template").clicked() {
                    self.load_template();
                    self.render_zpl(ctx);
                    self.is_dirty = false;
                }

                ui.separator();

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
                    self.render_zpl(ctx);
                }

                if self.is_loading {
                    ui.spinner();
                }

                ui.separator();

                ui.horizontal(|ui| {
                    ui.label("Manual IP:");
                    let response = ui.text_edit_singleline(&mut self.manual_ip);

                    let enter_pressed =
                        response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));

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

                    egui::ComboBox::from_id_salt(ui.next_auto_id())
                        .selected_text(selected_text)
                        .show_ui(ui, |ui| {
                            egui::ScrollArea::vertical()
                                .max_height(300.0)
                                .show(ui, |ui| {
                                    for (idx, printer) in self.printers.iter().enumerate() {
                                        if ui
                                            .selectable_label(
                                                Some(idx) == self.selected_printer,
                                                &printer.name,
                                            )
                                            .clicked()
                                        {
                                            self.selected_printer = Some(idx);
                                        }
                                    }
                                });
                        });

                    if ui
                        .add_enabled(
                            self.selected_printer.is_some(),
                            egui::Button::new("Send to Printer"),
                        )
                        .clicked()
                    {
                        self.send_to_printer();
                    }

                    ui.separator();

                    if ui.button("Query Printer...").clicked() {
                        self.show_query_window = true;
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

        egui::CentralPanel::default().show(ctx, |ui| {
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
                            let available_height = ui.available_height() - 50.0;
                            egui::ScrollArea::vertical()
                                .auto_shrink([false, false])
                                .max_height(available_height)
                                .show(ui, |ui| {
                                    if ui.add(
                                        egui::TextEdit::multiline(&mut self.raw_zpl_input)
                                            .code_editor()
                                            .desired_width(f32::INFINITY)
                                            .desired_rows(25)
                                    ).lost_focus() {
                                        self.is_dirty = true;
                                    }
                                });
                            ui.horizontal(|ui| {
                                if ui.button("Clear").clicked() {
                                    self.raw_zpl_input.clear();
                                    self.is_dirty = true;
                                }
                                if ui.button("Copy to Clipboard").clicked() {
                                    #[cfg(not(target_arch = "wasm32"))]
                                    {
                                        use arboard::Clipboard;
                                        if let Ok(mut clipboard) = Clipboard::new() {
                                            if let Err(_) = clipboard.set_text(&self.raw_zpl_input) {
                                                self.print_status = Some("Failed to copy to clipboard".to_string());
                                            }
                                        }
                                    }
                                    #[cfg(target_arch = "wasm32")]
                                    {
                                        ui.ctx().copy_text(self.raw_zpl_input.clone());
                                    }
                                }
                                let example_response = egui::ComboBox::from_id_salt(ui.next_auto_id())
                                    .selected_text("Load Example...")
                                    .show_ui(ui, |ui| {
                                        let mut selected = None;
                                        egui::ScrollArea::vertical()
                                            .max_height(200.0)
                                            .show(ui, |ui| {
                                                if ui.selectable_label(false, "Hello World").clicked() {
                                                    selected = Some("^XA\n^FO50,50\n^A0N,50,50\n^FDHello World^FS\n^FO50,150\n^GB300,2,2^FS\n^FO50,200\n^A0N,30,30\n^FDRaw ZPL Entry^FS\n^XZ");
                                                }
                                                if ui.selectable_label(false, "Graphic Test (32x32)").clicked() {
                                                    selected = Some("^XA\n^FO50,50^A0N,30,30^FDGraphic Below:^FS\n^FO50,100^GFA,128,128,4,FFFFFFFFFFFFFFFFC0000003C0000003C0000003C0000003C0000003C0000003C0000003C0000003C0000003C0000003C0000003C0000003C0000003C0000003C0000003C0000003C0000003C0000003C0000003C0000003C0000003C0000003C0000003C0000003C0000003C0000003C0000003C0000003FFFFFFFFFFFFFFFF^FS\n^FO50,200^A0N,30,30^FD32x32 Box^FS\n^XZ");
                                                }
                                                if ui.selectable_label(false, "Large Graphic (64x64)").clicked() {
                                                    selected = Some("^XA\n^FO100,100\n^GFA,512,512,8,FFFFFFFFFFFFFFFF80000000000000018000000000000001800000000000000180000000000000018000000000000001800000000000000180000000000000018000000000000001800000000000000180000000000000018000000000000001800000000000000180000000000000018000000000000001800000000000000180000000000000018000000000000001800000000000000180000000000000018000000000000001800000000000000180000000000000018000000000000001800000000000000180000000000000018000000000000001800000000000000180000000000000018000000000000001800000000000000180000000000000018000000000000001800000000000000180000000000000018000000000000001800000000000000180000000000000018000000000000001800000000000000180000000000000018000000000000001800000000000000180000000000000018000000000000001800000000000000180000000000000018000000000000001800000000000000180000000000000018000000000000001800000000000000180000000000000018000000000000001800000000000000180000000000000018000000000000001800000000000000180000000000000018000000000000001800000000000000180000000000000018000000000000001800000000000000180000000000000018000000000000001800000000000000180000000000000018000000000000001FFFFFFFFFFFFFFFF^FS\n^XZ");
                                                }
                                            });
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
                            ui.horizontal(|ui| {
                                ui.label("Generated ZPL:");
                                if ui.button("Copy to Clipboard").clicked() {
                                    let zpl_text = self.get_zpl_text();
                                    #[cfg(not(target_arch = "wasm32"))]
                                    {
                                        use arboard::Clipboard;
                                        if let Ok(mut clipboard) = Clipboard::new() {
                                            if let Err(_) = clipboard.set_text(&zpl_text) {
                                                self.print_status = Some("Failed to copy to clipboard".to_string());
                                            }
                                        }
                                    }
                                    #[cfg(target_arch = "wasm32")]
                                    {
                                        ui.ctx().copy_text(zpl_text);
                                    }
                                }
                            });
                            let available_height = ui.available_height();
                            egui::ScrollArea::vertical()
                                .auto_shrink([false, false])
                                .max_height(available_height)
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
                            ui.label("Add Command:");
                            ui.horizontal(|ui| {
                                if ui.button("Field Origin").clicked() {
                                    self.zpl_commands.push(ZplCommand::FieldOrigin { x: 0, y: 0 });
                                    self.is_dirty = true;
                                }
                                if ui.button("Field Data").clicked() {
                                    self.zpl_commands.push(ZplCommand::FieldData { data: String::new() });
                                    self.is_dirty = true;
                                }
                                if ui.button("Field Sep").clicked() {
                                    self.zpl_commands.push(ZplCommand::FieldSeparator);
                                    self.is_dirty = true;
                                }
                                if ui.button("Font").clicked() {
                                    self.zpl_commands.push(ZplCommand::Font {
                                        orientation: FontOrientation::Normal,
                                        height: 30,
                                        width: 30,
                                    });
                                    self.is_dirty = true;
                                }
                            });
                            ui.horizontal(|ui| {
                                if ui.button("Graphic Box").clicked() {
                                    self.zpl_commands.push(ZplCommand::GraphicBox {
                                        width: 100,
                                        height: 100,
                                        thickness: 1,
                                        color: None,
                                        rounding: None,
                                    });
                                    self.is_dirty = true;
                                }
                                if ui.button("Graphic Field").clicked() {
                                    self.zpl_commands.push(ZplCommand::GraphicField {
                                        width: 32,
                                        height: 32,
                                        data: String::new(),
                                    });
                                    self.is_dirty = true;
                                }

                                let response = egui::ComboBox::from_id_salt(ui.next_auto_id())
                                    .selected_text("More...")
                                    .show_ui(ui, |ui| {
                                        let mut selected = None;
                                        egui::ScrollArea::vertical()
                                            .max_height(300.0)
                                            .show(ui, |ui| {
                                                if ui.selectable_label(false, "Start Format (^XA)").clicked() {
                                                    selected = Some(ZplCommand::StartFormat);
                                                }
                                                if ui.selectable_label(false, "End Format (^XZ)").clicked() {
                                                    selected = Some(ZplCommand::EndFormat);
                                                }
                                                if ui.selectable_label(false, "Download Graphic (~DG)").clicked() {
                                                    selected = Some(ZplCommand::DownloadGraphic {
                                                        name: "GRAPHIC".to_string(),
                                                        width: 32,
                                                        height: 32,
                                                        data: String::new(),
                                                    });
                                                }
                                                if ui.selectable_label(false, "Recall Graphic (^XG)").clicked() {
                                                    selected = Some(ZplCommand::RecallGraphic {
                                                        name: "GRAPHIC".to_string(),
                                                        magnification_x: 1,
                                                        magnification_y: 1,
                                                    });
                                                }
                                                if ui.selectable_label(false, "Barcode Default (^BY)").clicked() {
                                                    selected = Some(ZplCommand::BarcodeFieldDefault {
                                                        width: 2,
                                                        ratio: 3.0,
                                                        height: 80,
                                                    });
                                                }
                                                if ui.selectable_label(false, "Code 128 Barcode (^BC)").clicked() {
                                                    selected = Some(ZplCommand::Code128Barcode {
                                                        orientation: FieldOrientation::Normal,
                                                        height: 80,
                                                        print_interpretation: true,
                                                        print_above: false,
                                                        check_digit: false,
                                                        mode: FieldOrientation::Normal,
                                                    });
                                                }
                                                if ui.selectable_label(false, "Media Mode Delayed (^MMD)").clicked() {
                                                    selected = Some(ZplCommand::MediaModeDelayed);
                                                }
                                                if ui.selectable_label(false, "Media Mode Tear-off (^MMT)").clicked() {
                                                    selected = Some(ZplCommand::MediaModeTearOff);
                                                }
                                                if ui.selectable_label(false, "Cut Now (~JK)").clicked() {
                                                    selected = Some(ZplCommand::CutNow);
                                                }
                                            });
                                        selected
                                    });

                                if let Some(inner) = response.inner {
                                    if let Some(command) = inner {
                                        self.zpl_commands.push(command);
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
                                                    egui::CollapsingHeader::new(
                                                        egui::RichText::new(
                                                            self.zpl_commands[idx].command_name()
                                                        ).strong()
                                                    )
                                                    .default_open(false)
                                                    .show(ui, |ui| {
                                                        self.render_command_editor(ui, idx);
                                                    });
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

                                    if self.needs_render_after_image {
                                        self.needs_render_after_image = false;
                                        self.render_zpl(ctx);
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
                    });
            });
        });

        if self.show_query_window {
            let mut show_window = self.show_query_window;
            egui::Window::new("Printer Query")
                .default_width(500.0)
                .default_height(400.0)
                .open(&mut show_window)
                .show(ctx, |ui| {
                    ui.heading("Query Printer");
                    ui.add_space(10.0);

                    let query_button_enabled = self.selected_printer.is_some() && !self.is_querying;

                    ui.horizontal(|ui| {
                        if ui
                            .add_enabled(query_button_enabled, egui::Button::new("Query All"))
                            .clicked()
                        {
                            self.query_all(ui.ctx());
                        }

                        ui.label("Or select individual:");
                        egui::ComboBox::from_id_salt(ui.next_auto_id())
                            .selected_text("Select...")
                            .show_ui(ui, |ui| {
                                egui::ScrollArea::vertical()
                                    .max_height(400.0)
                                    .show(ui, |ui| {
                                        if ui
                                            .add_enabled(
                                                query_button_enabled,
                                                egui::Button::new("Printer Status (ES)"),
                                            )
                                            .clicked()
                                        {
                                            self.query_printer("ES", ui.ctx());
                                        }
                                        if ui
                                            .add_enabled(
                                                query_button_enabled,
                                                egui::Button::new("Host Status (HS)"),
                                            )
                                            .clicked()
                                        {
                                            self.query_printer("HS", ui.ctx());
                                        }
                                        if ui
                                            .add_enabled(
                                                query_button_enabled,
                                                egui::Button::new("Host Identification (HI)"),
                                            )
                                            .clicked()
                                        {
                                            self.query_printer("HI", ui.ctx());
                                        }
                                        if ui
                                            .add_enabled(
                                                query_button_enabled,
                                                egui::Button::new("Serial Number (SN)"),
                                            )
                                            .clicked()
                                        {
                                            self.query_printer("SN", ui.ctx());
                                        }
                                        if ui
                                            .add_enabled(
                                                query_button_enabled,
                                                egui::Button::new("Hardware Address (HA)"),
                                            )
                                            .clicked()
                                        {
                                            self.query_printer("HA", ui.ctx());
                                        }
                                        if ui
                                            .add_enabled(
                                                query_button_enabled,
                                                egui::Button::new("Odometer (OD)"),
                                            )
                                            .clicked()
                                        {
                                            self.query_printer("OD", ui.ctx());
                                        }
                                        if ui
                                            .add_enabled(
                                                query_button_enabled,
                                                egui::Button::new("Printhead Life (PH)"),
                                            )
                                            .clicked()
                                        {
                                            self.query_printer("PH", ui.ctx());
                                        }
                                        if ui
                                            .add_enabled(
                                                query_button_enabled,
                                                egui::Button::new("Print Configuration (PR)"),
                                            )
                                            .clicked()
                                        {
                                            self.query_printer("PR", ui.ctx());
                                        }
                                        if ui
                                            .add_enabled(
                                                query_button_enabled,
                                                egui::Button::new("Config Status (CM)"),
                                            )
                                            .clicked()
                                        {
                                            self.query_printer("CM", ui.ctx());
                                        }
                                        if ui
                                            .add_enabled(
                                                query_button_enabled,
                                                egui::Button::new("Battery Capacity (BC)"),
                                            )
                                            .clicked()
                                        {
                                            self.query_printer("BC", ui.ctx());
                                        }
                                        if ui
                                            .add_enabled(
                                                query_button_enabled,
                                                egui::Button::new("USB Device ID (UI)"),
                                            )
                                            .clicked()
                                        {
                                            self.query_printer("UI", ui.ctx());
                                        }
                                        if ui
                                            .add_enabled(
                                                query_button_enabled,
                                                egui::Button::new("Label Dimensions (LD)"),
                                            )
                                            .clicked()
                                        {
                                            self.query_printer("LD", ui.ctx());
                                        }
                                        if ui
                                            .add_enabled(
                                                query_button_enabled,
                                                egui::Button::new("Label Count (LC)"),
                                            )
                                            .clicked()
                                        {
                                            self.query_printer("LC", ui.ctx());
                                        }
                                        if ui
                                            .add_enabled(
                                                query_button_enabled,
                                                egui::Button::new("File System Info (FS)"),
                                            )
                                            .clicked()
                                        {
                                            self.query_printer("FS", ui.ctx());
                                        }
                                        if ui
                                            .add_enabled(
                                                query_button_enabled,
                                                egui::Button::new("Network Router (NR)"),
                                            )
                                            .clicked()
                                        {
                                            self.query_printer("NR", ui.ctx());
                                        }
                                        if ui
                                            .add_enabled(
                                                query_button_enabled,
                                                egui::Button::new("Maintenance Alert (MA)"),
                                            )
                                            .clicked()
                                        {
                                            self.query_printer("MA", ui.ctx());
                                        }
                                        if ui
                                            .add_enabled(
                                                query_button_enabled,
                                                egui::Button::new("Sensor/Media Status (SM)"),
                                            )
                                            .clicked()
                                        {
                                            self.query_printer("SM", ui.ctx());
                                        }
                                        if ui
                                            .add_enabled(
                                                query_button_enabled,
                                                egui::Button::new("Alerts (AL)"),
                                            )
                                            .clicked()
                                        {
                                            self.query_printer("AL", ui.ctx());
                                        }
                                        if ui
                                            .add_enabled(
                                                query_button_enabled,
                                                egui::Button::new("Firmware Version (FW)"),
                                            )
                                            .clicked()
                                        {
                                            self.query_printer("FW", ui.ctx());
                                        }
                                        if ui
                                            .add_enabled(
                                                query_button_enabled,
                                                egui::Button::new("Supplies Status (ST)"),
                                            )
                                            .clicked()
                                        {
                                            self.query_printer("ST", ui.ctx());
                                        }
                                        if ui
                                            .add_enabled(
                                                query_button_enabled,
                                                egui::Button::new("Darkness Settings (DA)"),
                                            )
                                            .clicked()
                                        {
                                            self.query_printer("DA", ui.ctx());
                                        }
                                        if ui
                                            .add_enabled(
                                                query_button_enabled,
                                                egui::Button::new("Plug and Play (PP)"),
                                            )
                                            .clicked()
                                        {
                                            self.query_printer("PP", ui.ctx());
                                        }
                                        if ui
                                            .add_enabled(
                                                query_button_enabled,
                                                egui::Button::new("Host RAM Status (HM)"),
                                            )
                                            .clicked()
                                        {
                                            self.query_printer("HM", ui.ctx());
                                        }
                                    });
                            });

                        if self.is_querying {
                            ui.spinner();
                        }
                    });

                    ui.add_space(10.0);
                    ui.separator();
                    ui.add_space(10.0);

                    egui::ScrollArea::vertical()
                        .auto_shrink([false, false])
                        .show(ui, |ui| {
                            if let Some(ref status) = self.parsed_status {
                                let mut clear_status = false;

                                ui.horizontal(|ui| {
                                    ui.heading("Printer Status");
                                    if ui.button("Clear").clicked() {
                                        clear_status = true;
                                    }
                                });
                                ui.separator();
                                ui.add_space(8.0);

                                if status.is_ok() {
                                    ui.label(
                                        egui::RichText::new("✓ Printer is operating normally")
                                            .color(egui::Color32::GREEN)
                                            .size(16.0)
                                            .strong(),
                                    );
                                } else {
                                    if status.has_errors() {
                                        ui.label(
                                            egui::RichText::new("⚠ ERRORS DETECTED")
                                                .color(egui::Color32::RED)
                                                .size(16.0)
                                                .strong(),
                                        );
                                        ui.add_space(10.0);
                                        for error in status.errors.to_descriptions() {
                                            ui.horizontal(|ui| {
                                                ui.label(
                                                    egui::RichText::new("•")
                                                        .color(egui::Color32::RED)
                                                        .size(14.0),
                                                );
                                                ui.label(
                                                    egui::RichText::new(error)
                                                        .color(egui::Color32::RED)
                                                        .size(13.0),
                                                );
                                            });
                                            ui.add_space(3.0);
                                        }
                                        ui.add_space(12.0);
                                    }

                                    if status.has_warnings() {
                                        ui.label(
                                            egui::RichText::new("⚠ Warnings")
                                                .color(egui::Color32::YELLOW)
                                                .size(16.0)
                                                .strong(),
                                        );
                                        ui.add_space(10.0);
                                        for warning in status.warnings.to_descriptions() {
                                            ui.horizontal(|ui| {
                                                ui.label(
                                                    egui::RichText::new("•")
                                                        .color(egui::Color32::YELLOW)
                                                        .size(14.0),
                                                );
                                                ui.label(
                                                    egui::RichText::new(warning)
                                                        .color(egui::Color32::YELLOW)
                                                        .size(13.0),
                                                );
                                            });
                                            ui.add_space(3.0);
                                        }
                                    }
                                }

                                ui.add_space(15.0);

                                if clear_status {
                                    self.parsed_status = None;
                                }
                            }

                            let has_printer_info = self.printer_info.serial_number.is_some()
                                || self.printer_info.hardware_address.is_some()
                                || self.printer_info.odometer.is_some()
                                || self.printer_info.printhead_life.is_some()
                                || self.printer_info.plug_and_play.is_some()
                                || self.printer_info.memory_status.is_some();

                            if has_printer_info {
                                let mut clear_info = false;

                                ui.horizontal(|ui| {
                                    ui.heading("Printer Information");
                                    if ui.button("Clear").clicked() {
                                        clear_info = true;
                                    }
                                });
                                ui.separator();
                                ui.add_space(8.0);

                                if let Some(ref serial) = self.printer_info.serial_number {
                                    ui.label(egui::RichText::new("Serial Number:").strong());
                                    ui.label(serial);
                                    ui.add_space(8.0);
                                }

                                if let Some(ref mac) = self.printer_info.hardware_address {
                                    ui.label(
                                        egui::RichText::new("Hardware Address (MAC):").strong(),
                                    );
                                    ui.label(mac);
                                    ui.add_space(8.0);
                                }

                                if let Some(ref odometer) = self.printer_info.odometer {
                                    ui.label(egui::RichText::new("Odometer:").strong());
                                    ui.label(format!(
                                        "Total Print Length: {}",
                                        odometer.total_print_length
                                    ));
                                    ui.label(format!("Total Labels: {}", odometer.total_labels));
                                    ui.add_space(8.0);
                                }

                                if let Some(ref printhead) = self.printer_info.printhead_life {
                                    ui.label(egui::RichText::new("Printhead Life:").strong());
                                    ui.label(format!("Used Inches: {}", printhead.used_inches));
                                    ui.label(format!("Total Labels: {}", printhead.total_labels));
                                    ui.add_space(8.0);
                                }

                                if let Some(ref pnp) = self.printer_info.plug_and_play {
                                    ui.label(egui::RichText::new("Plug and Play Info:").strong());
                                    ui.label(pnp);
                                    ui.add_space(8.0);
                                }

                                if let Some(ref memory) = self.printer_info.memory_status {
                                    ui.label(egui::RichText::new("Memory Status:").strong());
                                    ui.label(format!("Total RAM: {} KB", memory.total_ram_kb));
                                    ui.label(format!(
                                        "Max Available: {} KB",
                                        memory.max_available_kb
                                    ));
                                    ui.label(format!(
                                        "Currently Available: {} KB",
                                        memory.current_available_kb
                                    ));
                                    let used_kb = memory
                                        .max_available_kb
                                        .saturating_sub(memory.current_available_kb);
                                    let usage_percent = if memory.max_available_kb > 0 {
                                        (used_kb as f32 / memory.max_available_kb as f32 * 100.0)
                                            as u32
                                    } else {
                                        0
                                    };
                                    ui.label(format!("Memory Usage: {}%", usage_percent));
                                    ui.add_space(8.0);
                                }

                                ui.add_space(15.0);

                                if clear_info {
                                    self.printer_info = PrinterInfo::default();
                                }
                            }

                            if self.query_response.is_some() {
                                let response_text = self.query_response.clone().unwrap();
                                let mut clear_response = false;
                                let mut copy_response = false;

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

                                ui.add(
                                    egui::TextEdit::multiline(&mut response_text.as_str())
                                        .code_editor()
                                        .desired_width(f32::INFINITY)
                                        .desired_rows(10)
                                        .interactive(false),
                                );

                                if clear_response {
                                    self.query_response = None;
                                }
                                if copy_response {
                                    ui.ctx().copy_text(response_text);
                                }
                            }

                            if self.parsed_status.is_none() && self.query_response.is_none() {
                                ui.vertical_centered(|ui| {
                                    ui.add_space(50.0);
                                    ui.label(
                                        egui::RichText::new("No query results yet")
                                            .color(egui::Color32::GRAY)
                                            .size(14.0),
                                    );
                                    ui.label(
                                        egui::RichText::new("Select a query type above")
                                            .color(egui::Color32::GRAY)
                                            .size(12.0),
                                    );
                                });
                            }
                        });
                });
            self.show_query_window = show_window;
        }
    }
}
