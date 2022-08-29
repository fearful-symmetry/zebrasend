use serde_derive::Deserialize;

#[derive(Deserialize, Clone)]
pub struct MessageStyle {
    #[serde(default = "default_font_size")]
    pub font_size: i32,
    #[serde(default)]
    pub font_width: i32,
    #[serde(default)]
    pub invert: bool,
    #[serde(default = "default_font")]
    pub font: String,
    #[serde(default)]
    pub line_padding: i32,
    #[serde(default)]
    pub start_point: i32,
}

fn default_font_size() -> i32 {
    35
}
fn default_font() -> String {
    "A".to_string()
}

impl MessageStyle {
    pub fn create_zpl_message(self, message: Vec<String>) -> String {
        let font_size = self.font_size;

        let mut fo_acc = self.start_point;
        let mut label_body = String::new();
        // Create formatting for each individual line
        for line in message {
            label_body = format!("{}^FO10,{}^FD{}^FS", label_body, fo_acc, line);
            fo_acc += font_size + self.line_padding;
        }

        let mut invert = "N";
        if self.invert {
            invert = "I"
        }

        let zpl = format!(
            "^XA^CF{},{},{}^PO{}{}^XZ",
            self.font, font_size, self.font_width, invert, label_body
        );
        zpl
    }
}
