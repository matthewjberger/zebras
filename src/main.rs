use nightshade::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    launch(Zebras::default())?;
    Ok(())
}

struct Zebras {
    zpl_text: String,
    original_zpl: String,
    rendered_image: Option<egui::TextureHandle>,
    is_dirty: bool,
    error_message: Option<String>,
    is_loading: bool,
    needs_initial_render: bool,
}

impl Default for Zebras {
    fn default() -> Self {
        let default_zpl = "^XA\n^FO50,50\n^A0N,50,50\n^FDHello World!^FS\n^FO50,150\n^GB300,2,2^FS\n^FO50,200\n^A0N,30,30\n^FDZebra ZPL Simulator^FS\n^XZ".to_string();
        Self {
            zpl_text: default_zpl.clone(),
            original_zpl: default_zpl,
            rendered_image: None,
            is_dirty: false,
            error_message: None,
            is_loading: false,
            needs_initial_render: true,
        }
    }
}

impl Zebras {
    fn render_zpl(&mut self, ui_context: &egui::Context) {
        println!("Starting ZPL render...");
        self.error_message = None;
        self.is_loading = true;

        println!("Fetching from Labelary API...");
        match Self::fetch_labelary_image(&self.zpl_text) {
            Ok(image_data) => {
                println!("Received {} bytes from API", image_data.len());
                let image = image::load_from_memory(&image_data);
                match image {
                    Ok(img) => {
                        println!(
                            "Successfully decoded image: {}x{}",
                            img.width(),
                            img.height()
                        );
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
                        println!("Texture loaded successfully");
                    }
                    Err(e) => {
                        println!("Failed to decode image: {}", e);
                        self.error_message = Some(format!("Failed to decode image: {}", e));
                    }
                }
            }
            Err(e) => {
                println!("API request failed: {}", e);
                self.error_message = Some(format!("Failed to render ZPL: {}", e));
            }
        }

        self.original_zpl = self.zpl_text.clone();
        self.is_dirty = false;
        self.is_loading = false;
    }

    fn fetch_labelary_image(zpl: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        println!("Sending ZPL to Labelary API:");
        println!("{}", zpl);

        let client = reqwest::blocking::Client::new();
        let response = client
            .post("http://api.labelary.com/v1/printers/8dpmm/labels/4x6/0/")
            .header("Accept", "image/png")
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(zpl.to_string())
            .send()?;

        println!("Response status: {}", response.status());

        if !response.status().is_success() {
            let error_body = response.text()?;
            println!("Error response body: {}", error_body);
            return Err(format!("API returned status with error: {}", error_body).into());
        }

        let bytes = response.bytes()?.to_vec();
        println!("Received {} bytes", bytes.len());
        Ok(bytes)
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
        if self.needs_initial_render {
            println!("Performing initial render...");
            self.needs_initial_render = false;
            self.render_zpl(ui_context);
        }

        egui::TopBottomPanel::top("top_panel").show(ui_context, |ui| {
            ui.horizontal(|ui| {
                ui.heading("ZPL Simulator");
                ui.separator();

                let button_enabled = self.is_dirty && !self.is_loading;
                let button_text = if self.is_loading {
                    "Loading..."
                } else {
                    "Apply Changes"
                };
                let button = egui::Button::new(button_text);

                if ui.add_enabled(button_enabled, button).clicked() {
                    self.render_zpl(ui_context);
                }

                if self.is_loading {
                    ui.spinner();
                }

                if let Some(ref error) = self.error_message {
                    ui.label(egui::RichText::new(error).color(egui::Color32::RED));
                }
            });
        });

        egui::CentralPanel::default().show(ui_context, |ui| {
            let available_width = ui.available_width();
            let panel_width = (available_width - 10.0) / 2.0;

            ui.horizontal(|ui| {
                ui.allocate_ui(egui::vec2(panel_width, ui.available_height()), |ui| {
                    ui.vertical(|ui| {
                        ui.heading("ZPL Code");
                        ui.separator();

                        let text_edit = egui::TextEdit::multiline(&mut self.zpl_text)
                            .code_editor()
                            .desired_width(f32::INFINITY)
                            .desired_rows(30);

                        if ui.add(text_edit).changed() {
                            self.is_dirty = self.zpl_text != self.original_zpl;
                        }
                    });
                });

                ui.separator();

                ui.allocate_ui(egui::vec2(panel_width, ui.available_height()), |ui| {
                    ui.vertical(|ui| {
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
                            ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                                let size = texture.size_vec2();
                                let max_width = ui.available_width();
                                let max_height = ui.available_height();

                                println!(
                                    "Texture size: {:?}, available: {}x{}",
                                    size, max_width, max_height
                                );

                                let scale = (max_width / size.x).min(max_height / size.y).min(2.0);
                                let display_size = size * scale;

                                println!("Scale: {}, display_size: {:?}", scale, display_size);

                                ui.add(egui::Image::new(egui::load::SizedTexture::new(
                                    texture.id(),
                                    display_size,
                                )));
                            });
                        } else {
                            ui.centered_and_justified(|ui| {
                                ui.label("Loading...");
                            });
                        }
                    });
                });
            });
        });
    }

    fn on_keyboard_input(&mut self, world: &mut World, key_code: KeyCode, key_state: KeyState) {
        if matches!((key_code, key_state), (KeyCode::KeyQ, KeyState::Pressed)) {
            world.resources.window.should_exit = true;
        }
    }
}
