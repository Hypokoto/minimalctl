use super::schema::SpatialNode;

pub fn parse_layout(input: &str) -> Option<SpatialNode> {
    let (_, layout_str) = input.split_once(',')?;
    let (node, remaining) = parse_node(layout_str)?;
    if !remaining.is_empty() {
        None
    } else {
        Some(node)
    }
}

fn parse_node(input: &str) -> Option<(SpatialNode, &str)> {
    let (width_str, rest) = input.split_once('x')?;
    let width: u16 = width_str.parse().ok()?;

    let (height_str, rest) = rest.split_once(',')?;
    let height: u16 = height_str.parse().ok()?;

    let (x_str, rest) = rest.split_once(',')?;
    let x: u16 = x_str.parse().ok()?;

    let mut delim_idx = None;
    for (i, c) in rest.char_indices() {
        if c == ',' || c == '{' || c == '[' {
            delim_idx = Some((i, c));
            break;
        }
    }

    let (i, _c) = delim_idx?;
    let y_str = &rest[..i];
    let y: u16 = y_str.parse().ok()?;
    let rest = &rest[i..];

    if let Some(rest_id) = rest.strip_prefix(',') {
        let mut id_end = rest_id.len();
        for (idx, c) in rest_id.char_indices() {
            if !c.is_ascii_digit() {
                id_end = idx;
                break;
            }
        }
        let id_str = &rest_id[..id_end];
        let pane_id: u32 = id_str.parse().ok()?;
        let remaining = &rest_id[id_end..];

        Some((
            SpatialNode::Pane {
                width,
                height,
                x,
                y,
                pane_id,
            },
            remaining,
        ))
    } else if rest.starts_with('{') || rest.starts_with('[') {
        let is_horiz = rest.starts_with('{');
        let close_char = if is_horiz { '}' } else { ']' };
        let mut children = Vec::new();
        let mut inner_rest = &rest[1..];

        loop {
            if inner_rest.is_empty() {
                return None;
            }
            if inner_rest.starts_with(close_char) {
                inner_rest = &inner_rest[1..];
                break;
            }

            let (child, next_rest) = parse_node(inner_rest)?;
            children.push(child);
            inner_rest = next_rest;

            if inner_rest.starts_with(',') {
                inner_rest = &inner_rest[1..];
            } else if !inner_rest.starts_with(close_char) {
                return None;
            }
        }

        if is_horiz {
            Some((
                SpatialNode::HorizontalSplit {
                    width,
                    height,
                    x,
                    y,
                    children,
                },
                inner_rest,
            ))
        } else {
            Some((
                SpatialNode::VerticalSplit {
                    width,
                    height,
                    x,
                    y,
                    children,
                },
                inner_rest,
            ))
        }
    } else {
        None
    }
}
