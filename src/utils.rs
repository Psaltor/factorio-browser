use yew::prelude::*;

/// Convert plain text to Html, preserving newlines as <br> tags
fn text_with_newlines(text: &str) -> Html {
    let parts: Vec<Html> = text
        .split('\n')
        .enumerate()
        .flat_map(|(i, line)| {
            if i > 0 {
                vec![html! { <br /> }, html! { <>{line}</> }]
            } else {
                vec![html! { <>{line}</> }]
            }
        })
        .collect();
    html! { <>{for parts}</> }
}

/// Find the next rich text tag ([color=...] or [font=...])
fn find_next_tag(text: &str) -> Option<(usize, &str)> {
    let color_pos = text.find("[color=");
    let font_pos = text.find("[font=");
    
    match (color_pos, font_pos) {
        (Some(c), Some(f)) => {
            if c < f {
                Some((c, "color"))
            } else {
                Some((f, "font"))
            }
        }
        (Some(c), None) => Some((c, "color")),
        (None, Some(f)) => Some((f, "font")),
        (None, None) => None,
    }
}

/// Parse Factorio rich text tags: [color=...][/color] and [font=...][/font]
/// Also converts newlines to <br> tags
pub fn parse_rich_text(text: &str) -> Html {
    let mut result: Vec<Html> = Vec::new();
    let mut remaining = text;

    while !remaining.is_empty() {
        if let Some((start, tag_type)) = find_next_tag(remaining) {
            // Add text before the tag
            if start > 0 {
                let before = &remaining[..start];
                result.push(text_with_newlines(before));
            }

            let tag_prefix = format!("[{}=", tag_type);
            let close_tag = format!("[/{}]", tag_type);
            let prefix_len = tag_prefix.len();
            let close_len = close_tag.len();

            // Find the end of the opening tag
            let after_start = &remaining[start + prefix_len..];
            if let Some(tag_end) = after_start.find(']') {
                let value = &after_start[..tag_end];
                let after_tag = &after_start[tag_end + 1..];

                // Find the closing tag
                if let Some(close) = after_tag.find(&close_tag) {
                    let content = &after_tag[..close];
                    
                    // Recursively parse content (for nested tags)
                    let inner = parse_rich_text(content);
                    
                    let styled = match tag_type {
                        "color" => {
                            let css_color = factorio_color_to_css(value);
                            html! {
                                <span style={format!("color: {}", css_color)}>{inner}</span>
                            }
                        }
                        "font" => {
                            let css_style = factorio_font_to_css(value);
                            html! {
                                <span style={css_style}>{inner}</span>
                            }
                        }
                        _ => inner,
                    };
                    
                    result.push(styled);
                    remaining = &after_tag[close + close_len..];
                    continue;
                }
            }
            // Malformed tag, treat as plain text
            result.push(text_with_newlines(&remaining[..start + 1]));
            remaining = &remaining[start + 1..];
        } else {
            // No more tags, add remaining text
            result.push(text_with_newlines(remaining));
            break;
        }
    }

    html! { <>{for result}</> }
}

/// Convert Factorio font names to CSS styles
fn factorio_font_to_css(font: &str) -> String {
    match font.to_lowercase().as_str() {
        "default" => "".to_string(),
        "default-bold" => "font-weight: 700".to_string(),
        "default-semibold" => "font-weight: 600".to_string(),
        "default-small" => "font-size: 0.85em".to_string(),
        "default-small-bold" => "font-size: 0.85em; font-weight: 700".to_string(),
        "default-small-semibold" => "font-size: 0.85em; font-weight: 600".to_string(),
        "default-large" => "font-size: 1.2em".to_string(),
        "default-large-bold" => "font-size: 1.2em; font-weight: 700".to_string(),
        "default-large-semibold" => "font-size: 1.2em; font-weight: 600".to_string(),
        "heading-1" => "font-size: 1.5em; font-weight: 700".to_string(),
        "heading-2" => "font-size: 1.25em; font-weight: 700".to_string(),
        _ => "".to_string(), // Default for unknown fonts
    }
}

/// Convert Factorio color names/values to CSS colors
fn factorio_color_to_css(color: &str) -> String {
    // Handle RGB format: r=1,g=0.5,b=0 or just comma-separated values
    if color.contains('=') || color.contains(',') {
        return parse_rgb_color(color);
    }

    // Handle hex colors
    if color.starts_with('#') {
        let cleaned = color.trim_start_matches('#');
        if cleaned.len() == 6 && cleaned.chars().all(|c| c.is_ascii_hexdigit()) {
            return color.to_string();
        }
        return "inherit".to_string();
    }

    // Named colors (Factorio uses these)
    match color.to_lowercase().as_str() {
        "red" => "#ff0000".to_string(),
        "green" => "#00ff00".to_string(),
        "blue" => "#0000ff".to_string(),
        "yellow" => "#ffff00".to_string(),
        "orange" => "#ffa500".to_string(),
        "pink" | "magenta" => "#ff00ff".to_string(),
        "cyan" | "aqua" => "#00ffff".to_string(),
        "white" => "#ffffff".to_string(),
        "black" => "#000000".to_string(),
        "gray" | "grey" => "#808080".to_string(),
        "purple" => "#800080".to_string(),
        "brown" => "#8b4513".to_string(),
        "acid" => "#b0ff00".to_string(),
        "default" => "inherit".to_string(),
        _ => {
            // Only allow valid 6-digit hex colors, reject everything else for security
            let cleaned = color.trim_start_matches('#');
            if cleaned.len() == 6 && cleaned.chars().all(|c| c.is_ascii_hexdigit()) {
                format!("#{}", cleaned)
            } else {
                "inherit".to_string()
            }
        }
    }
}

/// Parse RGB color format: "r=1,g=0.5,b=0" or "1,0.5,0"
fn parse_rgb_color(color: &str) -> String {
    let mut r = 1.0f32;
    let mut g = 1.0f32;
    let mut b = 1.0f32;

    for part in color.split(',') {
        let part = part.trim();
        if let Some(val) = part.strip_prefix("r=") {
            r = val.parse().unwrap_or(1.0);
        } else if let Some(val) = part.strip_prefix("g=") {
            g = val.parse().unwrap_or(1.0);
        } else if let Some(val) = part.strip_prefix("b=") {
            b = val.parse().unwrap_or(1.0);
        }
    }

    // Factorio uses 0-1 range, convert to 0-255
    let r = (r.clamp(0.0, 1.0) * 255.0) as u8;
    let g = (g.clamp(0.0, 1.0) * 255.0) as u8;
    let b = (b.clamp(0.0, 1.0) * 255.0) as u8;

    format!("rgb({}, {}, {})", r, g, b)
}
