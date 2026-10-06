use termimad::crossterm::style::Color::*;
use termimad::MadSkin;

pub fn render_markdown(markdown: &str) -> Result<(), String> {
    let mut skin = MadSkin::default();

    // Customize the skin for "minimal" theme
    skin.set_headers_fg(AnsiValue(208)); // Orange-ish
    skin.bold.set_fg(Yellow);
    skin.italic.set_fg(AnsiValue(204));

    // Inline code styling
    skin.inline_code.set_fg(AnsiValue(14));
    skin.inline_code.set_bg(AnsiValue(236));

    skin.print_text(markdown);
    Ok(())
}
