use uefi::{proto::console::text::Color, system};

pub fn set_fg_color(fg: Color) {
    system::with_stdout(|stdout| {
        stdout.set_color(fg, Color::Black).expect("failed to set color");
    });
}

pub fn clear() {
    system::with_stdout(|stdout| {
        stdout.clear().expect("failed to clear screen");
    });
}