use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::Line;
use ratatui::widgets::Paragraph;
use ratatui::Frame;

const PIXEL_ROWS: usize = 10;
const SPACING_PIXELS: usize = 2;

fn glyph(character: char) -> &'static [&'static str; PIXEL_ROWS] {
    match character {
        '0' => &[
            " ###### ",
            "########",
            "##    ##",
            "##    ##",
            "##    ##",
            "##    ##",
            "##    ##",
            "##    ##",
            "########",
            " ###### ",
        ],
        '1' => &[
            "   ##   ",
            "  ###   ",
            " ####   ",
            "   ##   ",
            "   ##   ",
            "   ##   ",
            "   ##   ",
            "   ##   ",
            "   ##   ",
            "   ##   ",
        ],
        '2' => &[
            " ###### ",
            "########",
            "##    ##",
            "      ##",
            "    ####",
            "  ####  ",
            " ###    ",
            "##      ",
            "########",
            "########",
        ],
        '3' => &[
            " ###### ",
            "########",
            "      ##",
            "      ##",
            "  ######",
            "  ######",
            "      ##",
            "      ##",
            "########",
            " ###### ",
        ],
        '4' => &[
            "##    ##",
            "##    ##",
            "##    ##",
            "##    ##",
            "########",
            "########",
            "      ##",
            "      ##",
            "      ##",
            "      ##",
        ],
        '5' => &[
            "########",
            "########",
            "##      ",
            "##      ",
            "####### ",
            "########",
            "      ##",
            "      ##",
            "########",
            " ###### ",
        ],
        '6' => &[
            " ###### ",
            "########",
            "##      ",
            "##      ",
            "####### ",
            "########",
            "##    ##",
            "##    ##",
            "########",
            " ###### ",
        ],
        '7' => &[
            "########",
            "########",
            "      ##",
            "     ## ",
            "     ## ",
            "    ##  ",
            "    ##  ",
            "   ##   ",
            "   ##   ",
            "   ##   ",
        ],
        '8' => &[
            " ###### ",
            "########",
            "##    ##",
            "##    ##",
            " ###### ",
            " ###### ",
            "##    ##",
            "##    ##",
            "########",
            " ###### ",
        ],
        '9' => &[
            " ###### ",
            "########",
            "##    ##",
            "##    ##",
            "########",
            " #######",
            "      ##",
            "      ##",
            "########",
            " ###### ",
        ],
        ':' => &[
            "    ",
            "    ",
            " ## ",
            " ## ",
            "    ",
            "    ",
            " ## ",
            " ## ",
            "    ",
            "    ",
        ],
        '.' => &[
            "    ",
            "    ",
            "    ",
            "    ",
            "    ",
            "    ",
            "    ",
            "    ",
            " ## ",
            " ## ",
        ],
        ' ' => &[
            "    ",
            "    ",
            "    ",
            "    ",
            "    ",
            "    ",
            "    ",
            "    ",
            "    ",
            "    ",
        ],
        _ => &[
            "      ",
            "      ",
            "      ",
            "      ",
            "######",
            "######",
            "      ",
            "      ",
            "      ",
            "      ",
        ],
    }
}

fn pixel_width(text: &str) -> usize {
    let glyph_widths: usize = text.chars().map(|character| glyph(character)[0].len()).sum();
    glyph_widths + text.chars().count().saturating_sub(1) * SPACING_PIXELS
}

fn hanging_margin_pixels() -> usize {
    glyph('-')[0].len() + SPACING_PIXELS
}

fn scale_for(digits: &str, area: Rect) -> usize {
    let width_budget = pixel_width(digits) + 2 * hanging_margin_pixels();
    let width_scale = area.width as usize / width_budget;
    let height_scale = area.height as usize * 2 / PIXEL_ROWS;
    width_scale.min(height_scale)
}

pub fn render(frame: &mut Frame, area: Rect, text: &str, style: Style) -> Rect {
    let negative = text.starts_with('-');
    let digits = text.trim_start_matches('-');
    let scale = scale_for(digits, area);
    if scale == 0 {
        return render_plain(frame, area, text, style);
    }
    let lines = build_lines(digits, scale);
    let centered = center(area, lines[0].chars().count() as u16, lines.len() as u16);
    render_block(frame, centered, lines, style);
    if negative {
        let minus_lines = build_lines("-", scale);
        let minus_area = Rect {
            x: centered.x - (hanging_margin_pixels() * scale) as u16,
            width: minus_lines[0].chars().count() as u16,
            ..centered
        };
        render_block(frame, minus_area, minus_lines, style);
    }
    centered
}

fn composition_width(text: &str, suffix: &str, scale: usize) -> usize {
    let half = scale / 2;
    let suffix_width = if half >= 1 {
        (SPACING_PIXELS + pixel_width(suffix)) * half
    } else {
        1 + suffix.chars().count()
    };
    pixel_width(text) * scale + suffix_width
}

pub fn render_with_suffix(
    frame: &mut Frame,
    area: Rect,
    text: &str,
    suffix: &str,
    style: Style,
    suffix_style: Style,
) -> Rect {
    let height_scale = area.height as usize * 2 / PIXEL_ROWS;
    let scale = (1..=height_scale)
        .rev()
        .find(|scale| composition_width(text, suffix, *scale) <= area.width as usize);
    let Some(scale) = scale else {
        return render_plain(frame, area, &format!("{text}{suffix}"), style);
    };
    let lines = build_lines(text, scale);
    let main_width = lines[0].chars().count() as u16;
    let height = lines.len() as u16;
    let total_width = composition_width(text, suffix, scale) as u16;
    let main_area = Rect {
        x: area.x + area.width.saturating_sub(total_width) / 2,
        y: area.y + area.height.saturating_sub(height) / 2,
        width: main_width,
        height,
    };
    render_block(frame, main_area, lines, style);
    let half = scale / 2;
    if half >= 1 {
        let suffix_lines = build_lines(suffix, half);
        let suffix_height = suffix_lines.len() as u16;
        let suffix_area = Rect {
            x: main_area.x + main_width + (SPACING_PIXELS * half) as u16,
            y: main_area.y + height - suffix_height,
            width: suffix_lines[0].chars().count() as u16,
            height: suffix_height,
        };
        render_block(frame, suffix_area, suffix_lines, suffix_style);
    } else {
        let baseline = Rect {
            x: main_area.x + main_width + 1,
            y: main_area.y + height - 1,
            width: suffix.chars().count() as u16,
            height: 1,
        };
        frame.render_widget(Paragraph::new(suffix.to_string()).style(suffix_style), baseline);
    }
    main_area
}

fn render_block(frame: &mut Frame, area: Rect, lines: Vec<String>, style: Style) {
    let paragraph = Paragraph::new(lines.into_iter().map(Line::from).collect::<Vec<_>>())
        .style(style);
    frame.render_widget(paragraph, area);
}

fn render_plain(frame: &mut Frame, area: Rect, text: &str, style: Style) -> Rect {
    let centered = center(area, text.chars().count() as u16, 1);
    frame.render_widget(Paragraph::new(text.to_string()).style(style), centered);
    centered
}

fn build_lines(text: &str, scale: usize) -> Vec<String> {
    let glyphs: Vec<&[&str; PIXEL_ROWS]> = text.chars().map(glyph).collect();
    let mut lines = Vec::with_capacity(PIXEL_ROWS * scale / 2);
    for line_index in 0..PIXEL_ROWS * scale / 2 {
        let upper_row = line_index * 2 / scale;
        let lower_row = (line_index * 2 + 1) / scale;
        let mut line = String::new();
        for (glyph_index, rows) in glyphs.iter().enumerate() {
            if glyph_index > 0 {
                line.extend(std::iter::repeat_n(' ', SPACING_PIXELS * scale));
            }
            for column in 0..rows[0].len() * scale {
                let source_column = column / scale;
                let upper = rows[upper_row].as_bytes()[source_column] == b'#';
                let lower = rows[lower_row].as_bytes()[source_column] == b'#';
                line.push(match (upper, lower) {
                    (true, true) => '█',
                    (true, false) => '▀',
                    (false, true) => '▄',
                    (false, false) => ' ',
                });
            }
        }
        lines.push(line);
    }
    lines
}

fn center(area: Rect, width: u16, height: u16) -> Rect {
    let x = area.x + area.width.saturating_sub(width) / 2;
    let y = area.y + area.height.saturating_sub(height) / 2;
    Rect {
        x,
        y,
        width: width.min(area.width),
        height: height.min(area.height),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn glyphs_are_rectangular_and_digits_share_width() {
        for character in "0123456789:.- ".chars() {
            let rows = glyph(character);
            assert!(rows.iter().all(|row| row.len() == rows[0].len()));
        }
        let digit_width = glyph('0')[0].len();
        for digit in "123456789".chars() {
            assert_eq!(glyph(digit)[0].len(), digit_width);
        }
    }

    #[test]
    fn rendered_width_ignores_sign() {
        let widths: Vec<usize> = ["00:05", "00:00", "59:59"]
            .iter()
            .map(|text| build_lines(text, 2)[0].chars().count())
            .collect();
        assert!(widths.windows(2).all(|pair| pair[0] == pair[1]));
    }

    #[test]
    fn half_blocks_cover_full_pixel_pairs() {
        let minus = build_lines("-", 1);
        assert_eq!(minus.len(), PIXEL_ROWS / 2);
        assert!(minus[2].contains('█'));
        assert!(!minus[2].contains('▀'));
    }

    #[test]
    fn odd_scale_maps_half_pixels() {
        let zero = build_lines("0", 1);
        assert!(zero[0].contains('▀') || zero[0].contains('█'));
        assert_eq!(zero[0].chars().count(), 8);
    }

    #[test]
    fn scale_fits_area_with_hanging_margins() {
        let area = Rect::new(0, 0, 120, 30);
        let scale = scale_for("00:05", area);
        assert!(scale >= 1);
        assert!((pixel_width("00:05") + 2 * hanging_margin_pixels()) * scale <= 120);
        assert!(PIXEL_ROWS * scale / 2 <= 30);
    }

    #[test]
    fn composition_includes_suffix_at_every_scale() {
        assert_eq!(
            composition_width("00:05", ".042", 2),
            pixel_width("00:05") * 2 + (SPACING_PIXELS + pixel_width(".042"))
        );
        assert_eq!(
            composition_width("00:05", ".042", 1),
            pixel_width("00:05") + 1 + 4
        );
        for width in (100..400usize).step_by(10) {
            let chosen = (1..=12)
                .rev()
                .find(|scale| composition_width("00:05", ".042", *scale) <= width);
            if let Some(scale) = chosen {
                assert!(composition_width("00:05", ".042", scale) <= width);
                assert!(scale >= 2 || width < composition_width("00:05", ".042", 2));
            }
        }
    }

    #[test]
    fn tiny_area_falls_back_to_plain() {
        assert_eq!(scale_for("00:05", Rect::new(0, 0, 20, 4)), 0);
    }
}
