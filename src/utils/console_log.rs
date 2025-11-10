use modern_terminal::{
    components::text::{
        Text, TextAlignment
    }, 
    core::style::Style
};

pub fn header(text: String) -> Box<Text> {
    Box::new(Text {
        align:    TextAlignment::Center,
        styles: vec![Style::Bold, Style::Foreground("green".to_string())],
        text: text.to_string(),
    })
}

pub fn field(text: String) -> Box<Text> {
    Box::new(Text {
        align:    TextAlignment::Center,
        styles: vec![Style::Bold, Style::Foreground("white".to_string())],
        text,
    })
}
