pub fn render_stars(val: u8) -> String {
    let mut s = String::new();
    for _ in 0..val {
        s.push('*');
    }
    for _ in val..5 {
        s.push('-');
    }
    s
}
