use uefi::{print, println, proto::console::text::Color};

use crate::colors::set_fg_color;

pub fn print_header() {
    set_fg_color(Color::LightGreen);
    print!("reginald ");
    set_fg_color(Color::White);
    print!("bootloader v{}.{}.{} ", 0, 0, 1);

    #[cfg(debug_assertions)]
    {
        set_fg_color(Color::LightRed);
        print!("debug ");
    }

    #[cfg(not(debug_assertions))]
    {
        set_fg_color(Color::LightGreen);
        print!("release ");
    }

    set_fg_color(Color::White);
    println!();
}
