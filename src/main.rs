use nightshade::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    launch(Zebras)?;
    Ok(())
}

#[derive(Default)]
struct Zebras;

impl State for Zebras {
    fn title(&self) -> &str {
        "Zebras"
    }

    fn initialize(&mut self, world: &mut World) {
        world.resources.user_interface.enabled = true;
    }

    fn ui(&mut self, _world: &mut World, ui_context: &egui::Context) {
        egui::CentralPanel::default().show(ui_context, |ui| {
            ui.heading("ZPL Simulator");
        });
    }

    fn on_keyboard_input(&mut self, world: &mut World, key_code: KeyCode, key_state: KeyState) {
        if matches!((key_code, key_state), (KeyCode::KeyQ, KeyState::Pressed)) {
            world.resources.window.should_exit = true;
        }
    }
}
