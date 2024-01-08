use std::fmt::Display;

use crossterm::event::KeyCode;

pub fn keycode_to_string(code: KeyCode) -> String {
    match code {
        KeyCode::Char(c) => format!("{}", c),
        _ => "".to_owned(),
    }
}

pub fn keycode_to_string_prefixed<'a, T>(code: Option<KeyCode>, prefix: T, postfix: T) -> String
where
    T: Into<&'a str> + Display,
{
    if code.is_none() {
        return "".to_owned();
    }
    let code = code.unwrap();
    match code {
        KeyCode::Char(c) => format!("{}{}{}", prefix, c, postfix),
        _ => "".to_owned(),
    }
}
