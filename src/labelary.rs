#[cfg(not(target_arch = "wasm32"))]
use reqwest::blocking;

#[cfg(target_arch = "wasm32")]
use reqwest_wasm as reqwest;

pub struct LabelaryClient {
    base_url: String,
    dpmm: u8,
    width: f32,
    height: f32,
}

impl LabelaryClient {
    pub fn new(dpmm: u8, width: f32, height: f32) -> Self {
        Self {
            base_url: "http://api.labelary.com/v1/printers".to_string(),
            dpmm,
            width,
            height,
        }
    }

    fn get_url(&self) -> String {
        format!(
            "{}/{}dpmm/labels/{}x{}/0/",
            self.base_url, self.dpmm, self.width, self.height
        )
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn render_sync(&self, zpl: &str) -> Result<Vec<u8>, String> {
        let client = blocking::Client::new();
        let response = client
            .post(self.get_url())
            .header("Accept", "image/png")
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(zpl.to_string())
            .send();

        match response {
            Ok(resp) => {
                if resp.status().is_success() {
                    match resp.bytes() {
                        Ok(bytes) => Ok(bytes.to_vec()),
                        Err(e) => Err(format!("Failed to read response bytes: {}", e)),
                    }
                } else {
                    Err(format!("API returned status: {}", resp.status()))
                }
            }
            Err(e) => Err(format!("Request failed: {}", e)),
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn convert_image_to_zpl_sync(&self, image_bytes: Vec<u8>) -> Result<String, String> {
        if image_bytes.is_empty() {
            return Err("Image data is empty".to_string());
        }

        let image_format = image::guess_format(&image_bytes)
            .map_err(|e| format!("Unable to detect image format: {}", e))?;

        let extension = match image_format {
            image::ImageFormat::Png => "png",
            image::ImageFormat::Jpeg => "jpg",
            image::ImageFormat::Gif => "gif",
            image::ImageFormat::Bmp => "bmp",
            _ => {
                return Err(format!(
                    "Unsupported image format: {:?}. Use PNG, JPG, GIF, or BMP",
                    image_format
                ));
            }
        };

        let client = blocking::Client::new();
        let part =
            blocking::multipart::Part::bytes(image_bytes).file_name(format!("image.{}", extension));

        let form = blocking::multipart::Form::new().part("file", part);

        let response = client
            .post("http://api.labelary.com/v1/graphics")
            .multipart(form)
            .send();

        match response {
            Ok(resp) => {
                let status = resp.status();
                if status.is_success() {
                    match resp.text() {
                        Ok(zpl) => {
                            if zpl.is_empty() {
                                Err("API returned empty response".to_string())
                            } else {
                                Ok(zpl)
                            }
                        }
                        Err(e) => Err(format!("Failed to read response text: {}", e)),
                    }
                } else {
                    let error_body = resp
                        .text()
                        .unwrap_or_else(|_| "Unable to read error body".to_string());
                    Err(format!("API error ({}): {}", status.as_u16(), error_body))
                }
            }
            Err(e) => Err(format!("Network error: {}", e)),
        }
    }

    #[cfg(target_arch = "wasm32")]
    pub async fn render_async(&self, zpl: &str) -> Result<Vec<u8>, String> {
        let client = reqwest::Client::new();
        let response = client
            .post(self.get_url())
            .header("Accept", "image/png")
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(zpl.to_string())
            .send()
            .await;

        match response {
            Ok(resp) => {
                if resp.status().is_success() {
                    match resp.bytes().await {
                        Ok(bytes) => Ok(bytes.to_vec()),
                        Err(e) => Err(format!("Failed to read response bytes: {}", e)),
                    }
                } else {
                    Err(format!("API returned status: {}", resp.status()))
                }
            }
            Err(e) => Err(format!("Request failed: {}", e)),
        }
    }
}

impl Default for LabelaryClient {
    fn default() -> Self {
        Self::new(8, 4.0, 6.0)
    }
}
