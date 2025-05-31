pub fn str(str: &[u8]) -> &str {
    std::str::from_utf8(str)
        .unwrap_or("[err invaild utf-8]")
        .trim_end_matches('\0')
}

pub fn pretty_byte(size: u32) -> String {
    match size {
        a if a < 1024 => format!("{:.2} B", a),
        a if a < 1024 * 1024 => format!("{:.2} KB", a as f32 / 1024.),
        a @ _ => format!("{:.2} MB", a as f32 / 1024. / 1024.),
    }
}
