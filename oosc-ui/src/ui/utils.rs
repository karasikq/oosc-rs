use crossterm::event::KeyCode;

pub fn keycode_to_string(code: KeyCode) -> String {
    match code {
        KeyCode::Char(c) => format!("{}", c),
        _ => "".to_owned(),
    }
}
