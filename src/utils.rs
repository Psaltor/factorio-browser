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

/// Parse Factorio rich text color tags like [color=red]text[/color]
/// Also converts newlines to <br> tags
pub fn parse_rich_text(text: &str) -> Html {
    let mut result: Vec<Html> = Vec::new();
    let mut remaining = text;

    while !remaining.is_empty() {
        // Look for [color=...]
        if let Some(start) = remaining.find("[color=") {
            // Add text before the tag
            if start > 0 {
                let before = &remaining[..start];
                result.push(text_with_newlines(before));
            }

            // Find the end of the opening tag
            let after_start = &remaining[start + 7..]; // skip "[color="
            if let Some(tag_end) = after_start.find(']') {
                let color = &after_start[..tag_end];
                let after_tag = &after_start[tag_end + 1..];

                // Find the closing tag
                if let Some(close) = after_tag.find("[/color]") {
                    let content = &after_tag[..close];
                    let css_color = factorio_color_to_css(color);
                    
                    // Recursively parse content (for nested tags)
                    let inner = parse_rich_text(content);
                    result.push(html! {
                        <span style={format!("color: {}", css_color)}>{inner}</span>
                    });

                    remaining = &after_tag[close + 8..]; // skip "[/color]"
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

/// Convert Factorio color names/values to CSS colors
fn factorio_color_to_css(color: &str) -> String {
    // Handle RGB format: r=1,g=0.5,b=0 or just comma-separated values
    if color.contains('=') || color.contains(',') {
        return parse_rgb_color(color);
    }

    // Handle hex colors
    if color.starts_with('#') {
        return color.to_string();
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

