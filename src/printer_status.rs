use std::fmt;

#[derive(Debug, Clone, PartialEq, Default)]
pub struct PrinterInfo {
    pub serial_number: Option<String>,
    pub hardware_address: Option<String>,
    pub odometer: Option<OdometerInfo>,
    pub printhead_life: Option<PrintheadInfo>,
    pub plug_and_play: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct OdometerInfo {
    pub total_print_length: String,
    pub total_labels: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrintheadInfo {
    pub used_inches: String,
    pub total_labels: String,
}

impl PrinterInfo {
    pub fn parse_serial_number(response: &str) -> Option<String> {
        for line in response.lines() {
            let line = line.trim();
            if line.starts_with('"') && line.ends_with('"') {
                return Some(line.trim_matches('"').to_string());
            }
            if !line.is_empty() && !line.starts_with('<') {
                return Some(line.to_string());
            }
        }
        None
    }

    pub fn parse_hardware_address(response: &str) -> Option<String> {
        for line in response.lines() {
            let line = line.trim();
            if line.len() == 12 && line.chars().all(|c| c.is_ascii_hexdigit()) {
                let formatted = format!(
                    "{}:{}:{}:{}:{}:{}",
                    &line[0..2], &line[2..4], &line[4..6],
                    &line[6..8], &line[8..10], &line[10..12]
                );
                return Some(formatted);
            }
            if !line.is_empty() && !line.starts_with('<') {
                return Some(line.to_string());
            }
        }
        None
    }

    pub fn parse_odometer(response: &str) -> Option<OdometerInfo> {
        let lines: Vec<&str> = response.lines().collect();
        if lines.len() >= 2 {
            let print_length = lines[0].trim().to_string();
            let labels = lines[1].trim().to_string();
            return Some(OdometerInfo {
                total_print_length: print_length,
                total_labels: labels,
            });
        }
        None
    }

    pub fn parse_printhead_life(response: &str) -> Option<PrintheadInfo> {
        let lines: Vec<&str> = response.lines().collect();
        if lines.len() >= 2 {
            let used_inches = lines[0].trim().to_string();
            let labels = lines[1].trim().to_string();
            return Some(PrintheadInfo {
                used_inches,
                total_labels: labels,
            });
        }
        None
    }

    pub fn parse_plug_and_play(response: &str) -> Option<String> {
        let cleaned: String = response.lines()
            .map(|l| l.trim())
            .filter(|l| !l.is_empty())
            .collect::<Vec<_>>()
            .join("\n");
        if !cleaned.is_empty() {
            Some(cleaned)
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrinterStatus {
    pub errors: ErrorFlags,
    pub warnings: WarningFlags,
}

impl PrinterStatus {
    pub fn parse(response: &str) -> Result<Self, String> {
        let mut errors = ErrorFlags::empty();
        let mut warnings = WarningFlags::empty();

        for line in response.lines() {
            let line = line.trim();
            if line.starts_with("ERRORS:") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 4 {
                    let hex_value = parts[3];
                    let error_value = u32::from_str_radix(hex_value, 16)
                        .map_err(|_| format!("Invalid hex value: {}", hex_value))?;
                    errors = ErrorFlags::from_hex(error_value);
                }
            } else if line.starts_with("WARNINGS:") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 4 {
                    let hex_value = parts[3];
                    let warning_value = u32::from_str_radix(hex_value, 16)
                        .map_err(|_| format!("Invalid hex value: {}", hex_value))?;
                    warnings = WarningFlags::from_hex(warning_value);
                }
            }
        }

        Ok(PrinterStatus { errors, warnings })
    }

    pub fn is_ok(&self) -> bool {
        self.errors.is_empty() && self.warnings.is_empty()
    }

    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    pub fn has_warnings(&self) -> bool {
        !self.warnings.is_empty()
    }
}

impl fmt::Display for PrinterStatus {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        if self.is_ok() {
            return write!(formatter, "Printer Status: OK");
        }

        writeln!(formatter, "Printer Status:")?;

        if self.has_errors() {
            writeln!(formatter, "\nErrors:")?;
            for error in self.errors.to_descriptions() {
                writeln!(formatter, "  - {}", error)?;
            }
        }

        if self.has_warnings() {
            writeln!(formatter, "\nWarnings:")?;
            for warning in self.warnings.to_descriptions() {
                writeln!(formatter, "  - {}", warning)?;
            }
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ErrorFlags {
    bits: u32,
}

impl ErrorFlags {
    pub const MEDIA_OUT: u32 = 0x00000001;
    pub const RIBBON_OUT: u32 = 0x00000002;
    pub const HEAD_OPEN: u32 = 0x00000004;
    pub const CUTTER_FAULT: u32 = 0x00000008;
    pub const PRINTHEAD_OVER_TEMPERATURE: u32 = 0x00000010;
    pub const MOTOR_OVER_TEMPERATURE: u32 = 0x00000020;
    pub const BAD_PRINTHEAD_ELEMENT: u32 = 0x00000040;
    pub const PRINTHEAD_DETECTION_ERROR: u32 = 0x00000080;
    pub const INVALID_FIRMWARE_CONFIG: u32 = 0x00000100;
    pub const PRINTHEAD_THERMISTOR_OPEN: u32 = 0x00000200;
    pub const PAUSED: u32 = 0x00001000;
    pub const RETRACT_FUNCTION_TIMED_OUT: u32 = 0x00002000;
    pub const BLACK_MARK_CALIBRATE_ERROR: u32 = 0x00004000;
    pub const BLACK_MARK_NOT_FOUND: u32 = 0x00008000;
    pub const PAPER_JAM_DURING_RETRACT: u32 = 0x00010000;
    pub const PRESENTER_NOT_RUNNING: u32 = 0x00020000;
    pub const PAPER_FEED_ERROR: u32 = 0x00040000;
    pub const CLEAR_PAPER_PATH_FAILED: u32 = 0x00080000;

    pub const fn empty() -> Self {
        Self { bits: 0 }
    }

    pub const fn from_hex(bits: u32) -> Self {
        Self { bits }
    }

    pub const fn is_empty(&self) -> bool {
        self.bits == 0
    }

    pub const fn contains(&self, flag: u32) -> bool {
        (self.bits & flag) != 0
    }

    pub fn to_descriptions(&self) -> Vec<String> {
        let mut descriptions = Vec::new();

        if self.contains(Self::MEDIA_OUT) {
            descriptions.push("Media out or not loaded".to_string());
        }
        if self.contains(Self::RIBBON_OUT) {
            descriptions.push("Ribbon out or not loaded".to_string());
        }
        if self.contains(Self::HEAD_OPEN) {
            descriptions.push("Head open / Cover open".to_string());
        }
        if self.contains(Self::CUTTER_FAULT) {
            descriptions.push("Cutter fault".to_string());
        }
        if self.contains(Self::PRINTHEAD_OVER_TEMPERATURE) {
            descriptions.push("Printhead over temperature".to_string());
        }
        if self.contains(Self::MOTOR_OVER_TEMPERATURE) {
            descriptions.push("Motor over temperature".to_string());
        }
        if self.contains(Self::BAD_PRINTHEAD_ELEMENT) {
            descriptions.push("Bad printhead element".to_string());
        }
        if self.contains(Self::PRINTHEAD_DETECTION_ERROR) {
            descriptions.push("Printhead detection error".to_string());
        }
        if self.contains(Self::INVALID_FIRMWARE_CONFIG) {
            descriptions.push("Invalid firmware configuration".to_string());
        }
        if self.contains(Self::PRINTHEAD_THERMISTOR_OPEN) {
            descriptions.push("Printhead thermistor open".to_string());
        }
        if self.contains(Self::PAUSED) {
            descriptions.push("Printer paused".to_string());
        }
        if self.contains(Self::RETRACT_FUNCTION_TIMED_OUT) {
            descriptions.push("Retract function timed out (KR403 only)".to_string());
        }
        if self.contains(Self::BLACK_MARK_CALIBRATE_ERROR) {
            descriptions.push("Black mark calibrate error (KR403 only)".to_string());
        }
        if self.contains(Self::BLACK_MARK_NOT_FOUND) {
            descriptions.push("Black mark not found (KR403 only)".to_string());
        }
        if self.contains(Self::PAPER_JAM_DURING_RETRACT) {
            descriptions.push("Paper jam during retract (KR403 only)".to_string());
        }
        if self.contains(Self::PRESENTER_NOT_RUNNING) {
            descriptions.push("Presenter not running (KR403 only)".to_string());
        }
        if self.contains(Self::PAPER_FEED_ERROR) {
            descriptions.push("Paper feed error (KR403 only)".to_string());
        }
        if self.contains(Self::CLEAR_PAPER_PATH_FAILED) {
            descriptions.push("Clear paper path failed (KR403 only)".to_string());
        }

        descriptions
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WarningFlags {
    bits: u32,
}

impl WarningFlags {
    pub const NEED_TO_CALIBRATE_MEDIA: u32 = 0x00000001;
    pub const CLEAN_PRINTHEAD: u32 = 0x00000002;
    pub const REPLACE_PRINTHEAD: u32 = 0x00000004;
    pub const PAPER_NEAR_END_SENSOR: u32 = 0x00000008;
    pub const SENSOR_1_PAPER_BEFORE_HEAD: u32 = 0x00000010;
    pub const SENSOR_2_BLACK_MARK: u32 = 0x00000020;
    pub const SENSOR_3_PAPER_AFTER_HEAD: u32 = 0x00000040;
    pub const SENSOR_4_LOOP_READY: u32 = 0x00000080;
    pub const SENSOR_5_PRESENTER: u32 = 0x00000100;
    pub const SENSOR_6_RETRACT_READY: u32 = 0x00000200;
    pub const SENSOR_7_IN_RETRACT: u32 = 0x00000400;
    pub const SENSOR_8_AT_BIN: u32 = 0x00000800;

    pub const fn empty() -> Self {
        Self { bits: 0 }
    }

    pub const fn from_hex(bits: u32) -> Self {
        Self { bits }
    }

    pub const fn is_empty(&self) -> bool {
        self.bits == 0
    }

    pub const fn contains(&self, flag: u32) -> bool {
        (self.bits & flag) != 0
    }

    pub fn to_descriptions(&self) -> Vec<String> {
        let mut descriptions = Vec::new();

        if self.contains(Self::NEED_TO_CALIBRATE_MEDIA) {
            descriptions.push("Need to calibrate media".to_string());
        }
        if self.contains(Self::CLEAN_PRINTHEAD) {
            descriptions.push("Clean printhead".to_string());
        }
        if self.contains(Self::REPLACE_PRINTHEAD) {
            descriptions.push("Replace printhead".to_string());
        }
        if self.contains(Self::PAPER_NEAR_END_SENSOR) {
            descriptions.push("Paper near end sensor (KR403 only)".to_string());
        }
        if self.contains(Self::SENSOR_1_PAPER_BEFORE_HEAD) {
            descriptions.push("Sensor 1: Paper before head (KR403 only)".to_string());
        }
        if self.contains(Self::SENSOR_2_BLACK_MARK) {
            descriptions.push("Sensor 2: Black mark (KR403 only)".to_string());
        }
        if self.contains(Self::SENSOR_3_PAPER_AFTER_HEAD) {
            descriptions.push("Sensor 3: Paper after head (KR403 only)".to_string());
        }
        if self.contains(Self::SENSOR_4_LOOP_READY) {
            descriptions.push("Sensor 4: Loop ready (KR403 only)".to_string());
        }
        if self.contains(Self::SENSOR_5_PRESENTER) {
            descriptions.push("Sensor 5: Presenter (KR403 only)".to_string());
        }
        if self.contains(Self::SENSOR_6_RETRACT_READY) {
            descriptions.push("Sensor 6: Retract ready (KR403 only)".to_string());
        }
        if self.contains(Self::SENSOR_7_IN_RETRACT) {
            descriptions.push("Sensor 7: In retract (KR403 only)".to_string());
        }
        if self.contains(Self::SENSOR_8_AT_BIN) {
            descriptions.push("Sensor 8: At bin (KR403 only)".to_string());
        }

        descriptions
    }
}
