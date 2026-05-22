use crate::dom::tree::DomTree;
use crate::css::matcher::{StyleMatcher, ComputedStyle, StyleValue};
use crate::css::parser::StyleSheet;
use crate::layout::{LayoutItem, LayoutEnv};

pub struct LayoutConverter {
    style_matcher: StyleMatcher,
    layout_env: LayoutEnv,
}

impl LayoutConverter {
    pub fn new(style_matcher: StyleMatcher, layout_env: LayoutEnv) -> Self {
        LayoutConverter {
            style_matcher,
            layout_env,
        }
    }

    pub fn convert_dom(&self, dom_tree: &DomTree) -> Result<Vec<LayoutItem>, String> {
        let mut items = Vec::new();
        let root_id = dom_tree.get_root().ok_or("No root node in DOM tree")?;
        self.convert_node_recursive(dom_tree, root_id, &mut items, 0, 0.0);
        Ok(items)
    }

    fn convert_node_recursive(
        &self,
        dom_tree: &DomTree,
        node_id: usize,
        items: &mut Vec<LayoutItem>,
        depth: u32,
        current_y: f32,
    ) -> f32 {
        let node = match dom_tree.get_node(node_id) {
            Some(n) => n,
            None => return current_y,
        };

        let computed_style = self.style_matcher.compute_style(dom_tree, node_id);
        let layout_item = self.node_to_layout_item(node, &computed_style, depth);
        items.push(layout_item);

        let mut child_y = current_y + 20.0;
        for &child_id in &node.children {
            child_y = self.convert_node_recursive(dom_tree, child_id, items, depth + 1, child_y);
        }

        child_y
    }

    fn node_to_layout_item(
        &self,
        _node: &crate::dom::tree::DomNode,
        computed_style: &ComputedStyle,
        depth: u32,
    ) -> LayoutItem {
        let width = self.parse_length(computed_style, "width", 100.0);
        let height = self.parse_length(computed_style, "height", 20.0);
        let z_index = self.parse_number(computed_style, "z-index", depth as f32);

        let margin = self.parse_margin(computed_style);
        let padding = self.parse_padding(computed_style);
        let flow_type = self.parse_flow_type(computed_style);
        let weight = self.parse_flex_grow(computed_style);
        let flex_shrink = self.parse_flex_shrink(computed_style);

        let (is_absolute, pos_x, pos_y) = self.parse_position(computed_style);
        let (right, bottom) = self.parse_right_bottom(computed_style, is_absolute);
        let is_hide = self.parse_display_hide(computed_style);
        let final_flow_type = if is_absolute { 1 } else { flow_type };

        let bg_color = self.parse_background_color(computed_style);
        let border = self.parse_border_width(computed_style);
        let border_color = self.parse_border_color(computed_style);
        let opacity = self.parse_opacity(computed_style);
        let overflow = self.parse_overflow(computed_style);
        let transform = self.parse_transform(computed_style);

        let (shadow_enabled, shadow_color, shadow_offset, shadow_blur, shadow_spread) = self.parse_box_shadow(computed_style);
        let border_radius = self.parse_border_radius(computed_style);
        let visibility = self.parse_visibility(computed_style);
        let flex_direction = self.parse_flex_direction(computed_style);
        let flex_wrap = self.parse_flex_wrap(computed_style);
        let align_items = self.parse_align_items(computed_style);
        let justify_content = self.parse_justify_content(computed_style);
        let align_content = self.parse_align_content(computed_style);
        let align_self = self.parse_align_self(computed_style);
        let order = self.parse_order(computed_style);
        let flex_basis = self.parse_flex_basis(computed_style);
        let gap = self.parse_gap(computed_style);
        let outline_width = self.parse_outline_width(computed_style);
        let outline_color = self.parse_outline_color(computed_style);
        let cursor = self.parse_cursor(computed_style);
        let pointer_events = self.parse_pointer_events(computed_style);
        let float = self.parse_float(computed_style);
        let clear = self.parse_clear(computed_style);
        let overflow_x = self.parse_overflow_x(computed_style);
        let overflow_y = self.parse_overflow_y(computed_style);
        let line_height = self.parse_line_height(computed_style);
        let letter_spacing = self.parse_letter_spacing(computed_style);
        let word_spacing = self.parse_word_spacing(computed_style);
        let text_indent = self.parse_text_indent(computed_style);
        let white_space = self.parse_white_space(computed_style);
        let max_width = self.parse_max_width(computed_style);
        let min_width = self.parse_min_width(computed_style);
        let max_height = self.parse_max_height(computed_style);
        let min_height = self.parse_min_height(computed_style);

        LayoutItem::new()
            .with_size(width, height)
            .with_margin(margin.0, margin.1, margin.2, margin.3)
            .with_padding(padding.0, padding.1, padding.2, padding.3)
            .with_pos(pos_x, pos_y)
            .with_z_index(z_index)
            .with_flow_type(final_flow_type)
            .with_weight(weight)
            .with_flex_shrink(flex_shrink)
            .with_hide_if(is_hide)
            .with_bg_color(bg_color.0, bg_color.1, bg_color.2, bg_color.3)
            .with_border(border.0, border.1, border.2, border.3)
            .with_border_color(border_color.0, border_color.1, border_color.2, border_color.3)
            .with_opacity(opacity)
            .with_overflow(overflow)
            .with_size_constraint(right, bottom)
            .with_transform(transform.0, transform.1, transform.2, transform.3, transform.4, transform.5)
            .with_shadow_if(shadow_enabled != 0, shadow_color, shadow_offset, shadow_blur, shadow_spread)
            .with_border_radius(border_radius.0, border_radius.1, border_radius.2, border_radius.3)
            .with_visibility(visibility)
            .with_flex_direction(flex_direction)
            .with_flex_wrap(flex_wrap)
            .with_align_items(align_items)
            .with_justify_content(justify_content)
            .with_align_content(align_content)
            .with_align_self(align_self)
            .with_order(order)
            .with_flex_basis(flex_basis)
            .with_gap(gap)
            .with_outline_width(outline_width)
            .with_outline_color(outline_color.0, outline_color.1, outline_color.2, outline_color.3)
            .with_cursor(cursor)
            .with_pointer_events(pointer_events)
            .with_float(float)
            .with_clear(clear)
            .with_overflow_x(overflow_x)
            .with_overflow_y(overflow_y)
            .with_line_height(line_height)
            .with_letter_spacing(letter_spacing)
            .with_word_spacing(word_spacing)
            .with_text_indent(text_indent)
            .with_white_space(white_space)
            .with_max_width(max_width)
            .with_min_width(min_width)
            .with_max_height(max_height)
            .with_min_height(min_height)
    }

    fn parse_box_shadow(&self, style: &ComputedStyle) -> (u32, [f32; 4], [f32; 2], f32, f32) {
        match self.style_matcher.get_property(style, "box-shadow") {
            Some(StyleValue::String(s)) => self.parse_shadow_string(&s),
            _ => (0, [0.0, 0.0, 0.0, 0.0], [0.0, 0.0], 0.0, 0.0),
        }
    }

    fn parse_shadow_string(&self, shadow_str: &str) -> (u32, [f32; 4], [f32; 2], f32, f32) {
        let shadow_str = shadow_str.trim().to_lowercase();
        if shadow_str.is_empty() || shadow_str == "none" {
            return (0, [0.0, 0.0, 0.0, 0.0], [0.0, 0.0], 0.0, 0.0);
        }

        let parts: Vec<&str> = shadow_str.split_whitespace().collect();
        if parts.is_empty() {
            return (0, [0.0, 0.0, 0.0, 0.0], [0.0, 0.0], 0.0, 0.0);
        }

        let h_offset = parts.first()
            .and_then(|p| p.trim_end_matches("px").parse::<f32>().ok())
            .unwrap_or(0.0);
        let v_offset = parts.get(1)
            .and_then(|p| p.trim_end_matches("px").parse::<f32>().ok())
            .unwrap_or(0.0);

        let mut blur = 0.0;
        let mut spread = 0.0;
        let mut color = [0.0, 0.0, 0.0, 0.5];

        for part in parts.iter().skip(2) {
            if part.starts_with('#') || part.starts_with("rgb") || part.starts_with("rgba")
                || ["red", "blue", "green", "black", "white", "transparent"].contains(part) {
                let (r, g, b, a) = self.parse_color(part);
                color = [r, g, b, a];
                break;
            }

            if let Ok(val) = part.trim_end_matches("px").parse::<f32>() {
                if blur == 0.0 {
                    blur = val;
                } else if spread == 0.0 {
                    spread = val;
                }
            }
        }

        (1, color, [h_offset, v_offset], blur, spread)
    }

    fn parse_border_radius(&self, style: &ComputedStyle) -> (f32, f32, f32, f32) {
        match self.style_matcher.get_property(style, "border-radius") {
            Some(StyleValue::Length(v)) => (*v, *v, *v, *v),
            Some(StyleValue::Number(v)) => (*v, *v, *v, *v),
            Some(StyleValue::String(s)) => self.parse_sides_from_string(&s, 0.0),
            _ => (0.0, 0.0, 0.0, 0.0),
        }
    }

    fn parse_sides_from_string(&self, value: &str, default: f32) -> (f32, f32, f32, f32) {
        let values: Vec<f32> = value.split_whitespace()
            .filter_map(|v| self.parse_css_length(v))
            .collect();

        match values.len() {
            1 => (values[0], values[0], values[0], values[0]),
            2 => (values[0], values[1], values[0], values[1]),
            3 => (values[0], values[1], values[2], values[1]),
            4 => (values[0], values[1], values[2], values[3]),
            _ => (default, default, default, default),
        }
    }

    fn parse_visibility(&self, style: &ComputedStyle) -> u32 {
        match self.style_matcher.get_property(style, "visibility") {
            Some(StyleValue::String(s)) => {
                match s.to_lowercase().as_str() {
                    "hidden" => 1,
                    "collapse" => 2,
                    _ => 0,
                }
            }
            _ => 0,
        }
    }

    fn parse_flex_direction(&self, style: &ComputedStyle) -> u32 {
        match self.style_matcher.get_property(style, "flex-direction") {
            Some(StyleValue::String(s)) => {
                match s.to_lowercase().as_str() {
                    "row-reverse" => 1,
                    "column" => 2,
                    "column-reverse" => 3,
                    _ => 0, // row 默认
                }
            }
            _ => 0,
        }
    }

    fn parse_flex_wrap(&self, style: &ComputedStyle) -> u32 {
        match self.style_matcher.get_property(style, "flex-wrap") {
            Some(StyleValue::String(s)) => {
                match s.to_lowercase().as_str() {
                    "wrap" => 1,
                    "wrap-reverse" => 2,
                    _ => 0, // nowrap 默认
                }
            }
            _ => 0,
        }
    }

    fn parse_align_items(&self, style: &ComputedStyle) -> u32 {
        match self.style_matcher.get_property(style, "align-items") {
            Some(StyleValue::String(s)) => {
                match s.to_lowercase().as_str() {
                    "flex-start" => 1,
                    "flex-end" => 2,
                    "center" => 3,
                    "baseline" => 4,
                    _ => 0, // stretch 默认
                }
            }
            _ => 0,
        }
    }

    fn parse_justify_content(&self, style: &ComputedStyle) -> u32 {
        match self.style_matcher.get_property(style, "justify-content") {
            Some(StyleValue::String(s)) => {
                match s.to_lowercase().as_str() {
                    "flex-end" => 1,
                    "center" => 2,
                    "space-between" => 3,
                    "space-around" => 4,
                    "space-evenly" => 5,
                    _ => 0, // flex-start 默认
                }
            }
            _ => 0,
        }
    }

    fn parse_align_content(&self, style: &ComputedStyle) -> u32 {
        match self.style_matcher.get_property(style, "align-content") {
            Some(StyleValue::String(s)) => {
                match s.to_lowercase().as_str() {
                    "flex-start" => 1,
                    "flex-end" => 2,
                    "center" => 3,
                    "space-between" => 4,
                    "space-around" => 5,
                    _ => 0, // stretch 默认
                }
            }
            _ => 0,
        }
    }

    fn parse_align_self(&self, style: &ComputedStyle) -> u32 {
        match self.style_matcher.get_property(style, "align-self") {
            Some(StyleValue::String(s)) => {
                match s.to_lowercase().as_str() {
                    "flex-start" => 1,
                    "flex-end" => 2,
                    "center" => 3,
                    "baseline" => 4,
                    "stretch" => 5,
                    _ => 0, // auto 默认
                }
            }
            _ => 0,
        }
    }

    fn parse_order(&self, style: &ComputedStyle) -> i32 {
        match self.style_matcher.get_property(style, "order") {
            Some(StyleValue::Number(v)) => *v as i32,
            Some(StyleValue::Length(v)) => *v as i32,
            _ => 0,
        }
    }

    fn parse_flex_basis(&self, style: &ComputedStyle) -> f32 {
        match self.style_matcher.get_property(style, "flex-basis") {
            Some(StyleValue::Length(v)) => *v,
            Some(StyleValue::Number(v)) => *v,
            Some(StyleValue::String(s)) => {
                if s.to_lowercase() == "auto" {
                    0.0
                } else {
                    self.parse_css_length(&s).unwrap_or(0.0)
                }
            }
            _ => 0.0,
        }
    }

    fn parse_gap(&self, style: &ComputedStyle) -> f32 {
        match self.style_matcher.get_property(style, "gap") {
            Some(StyleValue::Length(v)) => *v,
            Some(StyleValue::Number(v)) => *v,
            Some(StyleValue::String(s)) => self.parse_css_length(&s).unwrap_or(0.0),
            _ => 0.0,
        }
    }

    fn parse_outline_width(&self, style: &ComputedStyle) -> f32 {
        self.parse_length(style, "outline-width", 0.0)
    }

    fn parse_outline_color(&self, style: &ComputedStyle) -> (f32, f32, f32, f32) {
        match self.style_matcher.get_property(style, "outline-color") {
            Some(StyleValue::Color(s)) => self.parse_color(&s),
            Some(StyleValue::String(s)) => self.parse_color(&s),
            _ => (0.0, 0.0, 0.0, 0.0),
        }
    }

    fn parse_cursor(&self, style: &ComputedStyle) -> u32 {
        match self.style_matcher.get_property(style, "cursor") {
            Some(StyleValue::String(s)) => {
                match s.to_lowercase().as_str() {
                    "pointer" => 1,
                    "text" => 2,
                    "wait" => 3,
                    "crosshair" => 4,
                    "move" => 5,
                    "n-resize" | "s-resize" | "e-resize" | "w-resize" |
                    "ne-resize" | "nw-resize" | "se-resize" | "sw-resize" => 6,
                    _ => 0, // default
                }
            }
            _ => 0,
        }
    }

    fn parse_pointer_events(&self, style: &ComputedStyle) -> u32 {
        match self.style_matcher.get_property(style, "pointer-events") {
            Some(StyleValue::String(s)) => {
                match s.to_lowercase().as_str() {
                    "none" => 1,
                    "visiblepainted" => 2,
                    "visiblefill" => 3,
                    "visiblestroke" => 4,
                    "visible" => 5,
                    "painted" => 6,
                    "fill" => 7,
                    "stroke" => 8,
                    "all" => 9,
                    _ => 0,
                }
            }
            _ => 0,
        }
    }

    fn parse_float(&self, style: &ComputedStyle) -> u32 {
        match self.style_matcher.get_property(style, "float") {
            Some(StyleValue::String(s)) => {
                match s.to_lowercase().as_str() {
                    "left" => 1,
                    "right" => 2,
                    _ => 0, // none 默认
                }
            }
            _ => 0,
        }
    }

    fn parse_clear(&self, style: &ComputedStyle) -> u32 {
        match self.style_matcher.get_property(style, "clear") {
            Some(StyleValue::String(s)) => {
                match s.to_lowercase().as_str() {
                    "left" => 1,
                    "right" => 2,
                    "both" => 3,
                    _ => 0, // none 默认
                }
            }
            _ => 0,
        }
    }

    fn parse_overflow_x(&self, style: &ComputedStyle) -> u32 {
        self.parse_overflow_single(style, "overflow-x")
    }

    fn parse_overflow_y(&self, style: &ComputedStyle) -> u32 {
        self.parse_overflow_single(style, "overflow-y")
    }

    fn parse_overflow_single(&self, style: &ComputedStyle, property: &str) -> u32 {
        match self.style_matcher.get_property(style, property) {
            Some(StyleValue::String(s)) => {
                match s.to_lowercase().as_str() {
                    "hidden" => 1,
                    "scroll" => 2,
                    "auto" => 3,
                    _ => 0, // visible 默认
                }
            }
            Some(StyleValue::Auto) => 3,
            _ => 0,
        }
    }

    fn parse_line_height(&self, style: &ComputedStyle) -> f32 {
        match self.style_matcher.get_property(style, "line-height") {
            Some(StyleValue::Length(v)) => *v,
            Some(StyleValue::Number(v)) => *v,
            Some(StyleValue::String(s)) => self.parse_css_length(&s).unwrap_or(0.0),
            _ => 0.0,
        }
    }

    fn parse_letter_spacing(&self, style: &ComputedStyle) -> f32 {
        match self.style_matcher.get_property(style, "letter-spacing") {
            Some(StyleValue::Length(v)) => *v,
            Some(StyleValue::Number(v)) => *v,
            Some(StyleValue::String(s)) => {
                if s.to_lowercase() == "normal" { 0.0 } else { self.parse_css_length(&s).unwrap_or(0.0) }
            }
            _ => 0.0,
        }
    }

    fn parse_word_spacing(&self, style: &ComputedStyle) -> f32 {
        match self.style_matcher.get_property(style, "word-spacing") {
            Some(StyleValue::Length(v)) => *v,
            Some(StyleValue::Number(v)) => *v,
            Some(StyleValue::String(s)) => {
                if s.to_lowercase() == "normal" { 0.0 } else { self.parse_css_length(&s).unwrap_or(0.0) }
            }
            _ => 0.0,
        }
    }

    fn parse_text_indent(&self, style: &ComputedStyle) -> f32 {
        self.parse_length(style, "text-indent", 0.0)
    }

    fn parse_white_space(&self, style: &ComputedStyle) -> u32 {
        match self.style_matcher.get_property(style, "white-space") {
            Some(StyleValue::String(s)) => {
                match s.to_lowercase().as_str() {
                    "nowrap" => 1,
                    "pre" => 2,
                    "pre-wrap" => 3,
                    "pre-line" => 4,
                    _ => 0, // normal 默认
                }
            }
            _ => 0,
        }
    }

    fn parse_max_width(&self, style: &ComputedStyle) -> f32 {
        match self.style_matcher.get_property(style, "max-width") {
            Some(StyleValue::Length(v)) => *v,
            Some(StyleValue::Number(v)) => *v,
            Some(StyleValue::String(s)) => {
                if s.to_lowercase() == "none" { 0.0 } else { self.parse_css_length(&s).unwrap_or(0.0) }
            }
            _ => 0.0,
        }
    }

    fn parse_min_width(&self, style: &ComputedStyle) -> f32 {
        self.parse_length(style, "min-width", 0.0)
    }

    fn parse_max_height(&self, style: &ComputedStyle) -> f32 {
        match self.style_matcher.get_property(style, "max-height") {
            Some(StyleValue::Length(v)) => *v,
            Some(StyleValue::Number(v)) => *v,
            Some(StyleValue::String(s)) => {
                if s.to_lowercase() == "none" { 0.0 } else { self.parse_css_length(&s).unwrap_or(0.0) }
            }
            _ => 0.0,
        }
    }

    fn parse_min_height(&self, style: &ComputedStyle) -> f32 {
        self.parse_length(style, "min-height", 0.0)
    }

    fn parse_transform(&self, style: &ComputedStyle) -> (f32, f32, f32, f32, f32, f32) {
        match self.style_matcher.get_property(style, "transform") {
            Some(StyleValue::String(s)) => self.parse_transform_string(&s),
            _ => (1.0, 0.0, 0.0, 1.0, 0.0, 0.0),
        }
    }

    fn parse_transform_string(&self, transform_str: &str) -> (f32, f32, f32, f32, f32, f32) {
        if transform_str.starts_with("translate(") && transform_str.ends_with(")") {
            let inner = &transform_str[10..transform_str.len() - 1];
            let parts: Vec<&str> = inner.split(',').map(|s| s.trim()).collect();
            if parts.len() >= 2 {
                let tx = parts[0].trim_end_matches("px").parse::<f32>().unwrap_or(0.0);
                let ty = parts[1].trim_end_matches("px").parse::<f32>().unwrap_or(0.0);
                return (1.0, 0.0, 0.0, 1.0, tx, ty);
            }
        }

        if transform_str.starts_with("rotate(") && transform_str.ends_with(")") {
            let inner = &transform_str[7..transform_str.len() - 1];
            let angle_str = inner.trim_end_matches("deg").trim_end_matches("rad");
            if let Ok(angle) = angle_str.parse::<f32>() {
                let rad = if inner.ends_with("deg") {
                    angle * std::f32::consts::PI / 180.0
                } else {
                    angle
                };
                let cos = rad.cos();
                let sin = rad.sin();
                return (cos, sin, -sin, cos, 0.0, 0.0);
            }
        }

        if transform_str.starts_with("scale(") && transform_str.ends_with(")") {
            let inner = &transform_str[6..transform_str.len() - 1];
            let parts: Vec<&str> = inner.split(',').map(|s| s.trim()).collect();
            let (sx, sy) = if parts.len() >= 2 {
                (parts[0].parse::<f32>().unwrap_or(1.0), parts[1].parse::<f32>().unwrap_or(1.0))
            } else {
                let s = parts.get(0).and_then(|p| p.parse::<f32>().ok()).unwrap_or(1.0);
                (s, s)
            };
            return (sx, 0.0, 0.0, sy, 0.0, 0.0);
        }

        if transform_str.starts_with("matrix(") && transform_str.ends_with(")") {
            let inner = &transform_str[7..transform_str.len() - 1];
            let parts: Vec<&str> = inner.split(',').map(|s| s.trim()).collect();
            if parts.len() >= 6 {
                return (
                    parts[0].parse::<f32>().unwrap_or(1.0),
                    parts[1].parse::<f32>().unwrap_or(0.0),
                    parts[2].parse::<f32>().unwrap_or(0.0),
                    parts[3].parse::<f32>().unwrap_or(1.0),
                    parts[4].parse::<f32>().unwrap_or(0.0),
                    parts[5].parse::<f32>().unwrap_or(0.0),
                );
            }
        }

        (1.0, 0.0, 0.0, 1.0, 0.0, 0.0)
    }

    fn parse_opacity(&self, style: &ComputedStyle) -> f32 {
        match self.style_matcher.get_property(style, "opacity") {
            Some(StyleValue::Number(v)) => v.clamp(0.0, 1.0),
            Some(StyleValue::Length(v)) => v.clamp(0.0, 1.0),
            _ => 1.0,
        }
    }

    fn parse_overflow(&self, style: &ComputedStyle) -> u32 {
        match self.style_matcher.get_property(style, "overflow") {
            Some(StyleValue::String(s)) => {
                match s.to_lowercase().as_str() {
                    "hidden" => 1,
                    "scroll" => 2,
                    "auto" => 3,
                    _ => 0,
                }
            }
            Some(StyleValue::Auto) => 3,
            _ => 0,
        }
    }

    fn parse_length(&self, style: &ComputedStyle, property: &str, default: f32) -> f32 {
        match self.style_matcher.get_property(style, property) {
            Some(StyleValue::Length(v)) => *v,
            Some(StyleValue::Number(v)) => *v,
            _ => default,
        }
    }

    fn parse_number(&self, style: &ComputedStyle, property: &str, default: f32) -> f32 {
        match self.style_matcher.get_property(style, property) {
            Some(StyleValue::Number(v)) => *v,
            Some(StyleValue::Length(v)) => *v,
            _ => default,
        }
    }

    fn parse_margin(&self, style: &ComputedStyle) -> (f32, f32, f32, f32) {
        self.parse_sides(style, "margin", 0.0)
    }

    fn parse_padding(&self, style: &ComputedStyle) -> (f32, f32, f32, f32) {
        self.parse_sides(style, "padding", 0.0)
    }

    fn parse_sides(&self, style: &ComputedStyle, property: &str, default: f32) -> (f32, f32, f32, f32) {
        match self.style_matcher.get_property(style, property) {
            Some(StyleValue::Length(v)) => (*v, *v, *v, *v),
            Some(StyleValue::Number(v)) => (*v, *v, *v, *v),
            Some(StyleValue::String(s)) => {
                let values: Vec<f32> = s.split_whitespace()
                    .filter_map(|v| self.parse_css_length(v))
                    .collect();

                match values.len() {
                    1 => (values[0], values[0], values[0], values[0]),
                    2 => (values[0], values[1], values[0], values[1]),
                    3 => (values[0], values[1], values[2], values[1]),
                    4 => (values[0], values[1], values[2], values[3]),
                    _ => (default, default, default, default),
                }
            }
            _ => (default, default, default, default),
        }
    }

    fn parse_css_length(&self, value: &str) -> Option<f32> {
        let value = value.trim();
        if let Some(suffix) = value.strip_suffix("px") {
            suffix.trim().parse::<f32>().ok()
        } else {
            value.parse::<f32>().ok()
        }
    }

    fn parse_flow_type(&self, style: &ComputedStyle) -> u32 {
        match self.style_matcher.get_property(style, "display") {
            Some(StyleValue::String(s)) => {
                match s.as_str() {
                    "none" => 0,
                    "block" | "inline" | "inline-block" => 0,
                    "absolute" | "fixed" => 1,
                    "flex" => 2,
                    "grid" => 3,
                    _ => 0,
                }
            }
            _ => 0,
        }
    }

    fn parse_display_hide(&self, style: &ComputedStyle) -> bool {
        match self.style_matcher.get_property(style, "display") {
            Some(StyleValue::String(s)) => s == "none",
            _ => false,
        }
    }

    fn parse_flex_grow(&self, style: &ComputedStyle) -> f32 {
        match self.style_matcher.get_property(style, "flex-grow") {
            Some(StyleValue::Number(v)) => *v,
            Some(StyleValue::Length(v)) => *v,
            _ => 0.0,
        }
    }

    fn parse_flex_shrink(&self, style: &ComputedStyle) -> f32 {
        match self.style_matcher.get_property(style, "flex-shrink") {
            Some(StyleValue::Number(v)) => *v,
            Some(StyleValue::Length(v)) => *v,
            _ => 1.0,
        }
    }

    fn parse_border_width(&self, style: &ComputedStyle) -> (f32, f32, f32, f32) {
        self.parse_sides(style, "border-width", 0.0)
    }

    fn parse_border_color(&self, style: &ComputedStyle) -> (f32, f32, f32, f32) {
        match self.style_matcher.get_property(style, "border-color") {
            Some(StyleValue::Color(s)) => self.parse_color(&s),
            Some(StyleValue::String(s)) => self.parse_color(&s),
            _ => (0.0, 0.0, 0.0, 1.0),
        }
    }

    fn parse_position(&self, style: &ComputedStyle) -> (bool, f32, f32) {
        let position_type = match self.style_matcher.get_property(style, "position") {
            Some(StyleValue::String(s)) => s.as_str(),
            _ => "static",
        };

        let is_absolute = matches!(position_type, "absolute" | "fixed");

        if is_absolute {
            let top = self.parse_length(style, "top", 0.0);
            let left = self.parse_length(style, "left", 0.0);
            (true, left, top)
        } else {
            (false, 0.0, 0.0)
        }
    }

    fn parse_right_bottom(&self, style: &ComputedStyle, is_absolute: bool) -> (f32, f32) {
        if !is_absolute {
            return (-1.0, -1.0);
        }
        let right = self.parse_length(style, "right", -1.0);
        let bottom = self.parse_length(style, "bottom", -1.0);
        (right, bottom)
    }

    fn parse_background_color(&self, style: &ComputedStyle) -> (f32, f32, f32, f32) {
        match self.style_matcher.get_property(style, "background-color") {
            Some(StyleValue::Color(s)) => self.parse_color(&s),
            Some(StyleValue::String(s)) => self.parse_color(&s),
            _ => (0.0, 0.0, 0.0, 0.0),
        }
    }

    fn parse_color(&self, color_str: &str) -> (f32, f32, f32, f32) {
        let color_str = color_str.trim();

        if color_str.len() == 4 && color_str.starts_with('#') {
            let r = u8::from_str_radix(&color_str[1..2].repeat(2), 16).unwrap_or(0) as f32 / 255.0;
            let g = u8::from_str_radix(&color_str[2..3].repeat(2), 16).unwrap_or(0) as f32 / 255.0;
            let b = u8::from_str_radix(&color_str[3..4].repeat(2), 16).unwrap_or(0) as f32 / 255.0;
            return (r, g, b, 1.0);
        }

        if color_str.len() == 7 && color_str.starts_with('#') {
            let r = u8::from_str_radix(&color_str[1..3], 16).unwrap_or(0) as f32 / 255.0;
            let g = u8::from_str_radix(&color_str[3..5], 16).unwrap_or(0) as f32 / 255.0;
            let b = u8::from_str_radix(&color_str[5..7], 16).unwrap_or(0) as f32 / 255.0;
            return (r, g, b, 1.0);
        }

        if color_str.starts_with("rgb(") && color_str.ends_with(")") {
            let inner = &color_str[4..color_str.len()-1];
            let parts: Vec<&str> = inner.split(',').map(|s| s.trim()).collect();
            if parts.len() >= 3 {
                let r = parts[0].parse::<f32>().unwrap_or(0.0) / 255.0;
                let g = parts[1].parse::<f32>().unwrap_or(0.0) / 255.0;
                let b = parts[2].parse::<f32>().unwrap_or(0.0) / 255.0;
                return (r.clamp(0.0, 1.0), g.clamp(0.0, 1.0), b.clamp(0.0, 1.0), 1.0);
            }
        }

        if color_str.starts_with("rgba(") && color_str.ends_with(")") {
            let inner = &color_str[5..color_str.len()-1];
            let parts: Vec<&str> = inner.split(',').map(|s| s.trim()).collect();
            if parts.len() >= 4 {
                let r = parts[0].parse::<f32>().unwrap_or(0.0) / 255.0;
                let g = parts[1].parse::<f32>().unwrap_or(0.0) / 255.0;
                let b = parts[2].parse::<f32>().unwrap_or(0.0) / 255.0;
                let a = parts[3].parse::<f32>().unwrap_or(1.0);
                return (r.clamp(0.0, 1.0), g.clamp(0.0, 1.0), b.clamp(0.0, 1.0), a.clamp(0.0, 1.0));
            }
        }

        self.parse_color_name(color_str)
    }

    fn parse_color_name(&self, name: &str) -> (f32, f32, f32, f32) {
        match name.to_lowercase().as_str() {
            "red" => (1.0, 0.0, 0.0, 1.0),
            "green" => (0.0, 0.5, 0.0, 1.0),
            "blue" => (0.0, 0.0, 1.0, 1.0),
            "yellow" => (1.0, 1.0, 0.0, 1.0),
            "cyan" => (0.0, 1.0, 1.0, 1.0),
            "magenta" => (1.0, 0.0, 1.0, 1.0),
            "black" => (0.0, 0.0, 0.0, 1.0),
            "white" => (1.0, 1.0, 1.0, 1.0),
            "gray" | "grey" => (0.5, 0.5, 0.5, 1.0),
            "orange" => (1.0, 0.65, 0.0, 1.0),
            "purple" => (0.5, 0.0, 0.5, 1.0),
            "pink" => (1.0, 0.75, 0.8, 1.0),
            "transparent" => (0.0, 0.0, 0.0, 0.0),
            _ => (0.0, 0.0, 0.0, 0.0),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_convert_basic_dimensions() {
        let css = "div { width: 200px; height: 100px; }";
        let stylesheet = StyleSheet::parse(css);
        let matcher = StyleMatcher::new(stylesheet);
        let env = LayoutEnv::new(800.0, 600.0);
        let converter = LayoutConverter::new(matcher, env);

        let mut tree = DomTree::new();
        let div_id = tree.create_node("div".to_string());
        tree.add_node(None, div_id);

        let items = converter.convert_dom(&tree).unwrap();

        assert_eq!(items.len(), 1);
        assert_eq!(items[0].width(), 200.0);
        assert_eq!(items[0].height(), 100.0);
    }

    #[test]
    fn test_convert_with_margin() {
        let css = "div { margin: 10px; }";
        let stylesheet = StyleSheet::parse(css);
        let matcher = StyleMatcher::new(stylesheet);
        let env = LayoutEnv::new(800.0, 600.0);
        let converter = LayoutConverter::new(matcher, env);

        let mut tree = DomTree::new();
        let div_id = tree.create_node("div".to_string());
        tree.add_node(None, div_id);

        let items = converter.convert_dom(&tree).unwrap();

        assert_eq!(items[0].margin, [10.0, 10.0, 10.0, 10.0]);
    }

    #[test]
    fn test_convert_with_padding() {
        let css = "div { padding: 15px; }";
        let stylesheet = StyleSheet::parse(css);
        let matcher = StyleMatcher::new(stylesheet);
        let env = LayoutEnv::new(800.0, 600.0);
        let converter = LayoutConverter::new(matcher, env);

        let mut tree = DomTree::new();
        let div_id = tree.create_node("div".to_string());
        tree.add_node(None, div_id);

        let items = converter.convert_dom(&tree).unwrap();

        assert_eq!(items[0].padding, [15.0, 15.0, 15.0, 15.0]);
    }

    #[test]
    fn test_convert_display_none() {
        let css = "div { display: none; }";
        let stylesheet = StyleSheet::parse(css);
        let matcher = StyleMatcher::new(stylesheet);
        let env = LayoutEnv::new(800.0, 600.0);
        let converter = LayoutConverter::new(matcher, env);

        let mut tree = DomTree::new();
        let div_id = tree.create_node("div".to_string());
        tree.add_node(None, div_id);

        let items = converter.convert_dom(&tree).unwrap();

        assert_eq!(items[0].is_hide, 1);
        assert!(!items[0].is_visible());
    }

    #[test]
    fn test_convert_display_flex() {
        let css = "div { display: flex; }";
        let stylesheet = StyleSheet::parse(css);
        let matcher = StyleMatcher::new(stylesheet);
        let env = LayoutEnv::new(800.0, 600.0);
        let converter = LayoutConverter::new(matcher, env);

        let mut tree = DomTree::new();
        let div_id = tree.create_node("div".to_string());
        tree.add_node(None, div_id);

        let items = converter.convert_dom(&tree).unwrap();

        assert_eq!(items[0].flow_type, 2);
    }

    #[test]
    fn test_convert_nested_elements() {
        let css = "div { width: 100px; } p { width: 50px; }";
        let stylesheet = StyleSheet::parse(css);
        let matcher = StyleMatcher::new(stylesheet);
        let env = LayoutEnv::new(800.0, 600.0);
        let converter = LayoutConverter::new(matcher, env);

        let mut tree = DomTree::new();
        let div_id = tree.create_node("div".to_string());
        let p_id = tree.create_node("p".to_string());
        tree.add_node(None, div_id);
        tree.add_node(Some(div_id), p_id);

        let items = converter.convert_dom(&tree).unwrap();

        assert_eq!(items.len(), 2);
        assert_eq!(items[0].width(), 100.0);
        assert_eq!(items[1].width(), 50.0);
    }

    #[test]
    fn test_convert_opacity() {
        let css = "div { opacity: 0.5; }";
        let stylesheet = StyleSheet::parse(css);
        let matcher = StyleMatcher::new(stylesheet);
        let env = LayoutEnv::new(800.0, 600.0);
        let converter = LayoutConverter::new(matcher, env);

        let mut tree = DomTree::new();
        let div_id = tree.create_node("div".to_string());
        tree.add_node(None, div_id);

        let items = converter.convert_dom(&tree).unwrap();

        assert!((items[0].opacity - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_convert_opacity_default() {
        let stylesheet = StyleSheet::new();
        let matcher = StyleMatcher::new(stylesheet);
        let env = LayoutEnv::new(800.0, 600.0);
        let converter = LayoutConverter::new(matcher, env);

        let mut tree = DomTree::new();
        let div_id = tree.create_node("div".to_string());
        tree.add_node(None, div_id);

        let items = converter.convert_dom(&tree).unwrap();

        assert_eq!(items[0].opacity, 1.0);
    }

    #[test]
    fn test_convert_position_absolute() {
        let css = "div { position: absolute; top: 50px; left: 100px; }";
        let stylesheet = StyleSheet::parse(css);
        let matcher = StyleMatcher::new(stylesheet);
        let env = LayoutEnv::new(800.0, 600.0);
        let converter = LayoutConverter::new(matcher, env);

        let mut tree = DomTree::new();
        let div_id = tree.create_node("div".to_string());
        tree.add_node(None, div_id);

        let items = converter.convert_dom(&tree).unwrap();

        assert_eq!(items[0].flow_type, 1);
        assert_eq!(items[0].x(), 100.0);
        assert_eq!(items[0].y(), 50.0);
    }

    #[test]
    fn test_convert_background_color_hex() {
        let css = "div { background-color: #ff0000; }";
        let stylesheet = StyleSheet::parse(css);
        let matcher = StyleMatcher::new(stylesheet);
        let env = LayoutEnv::new(800.0, 600.0);
        let converter = LayoutConverter::new(matcher, env);

        let mut tree = DomTree::new();
        let div_id = tree.create_node("div".to_string());
        tree.add_node(None, div_id);

        let items = converter.convert_dom(&tree).unwrap();

        assert!((items[0].bg_color[0] - 1.0).abs() < 0.01);
        assert!((items[0].bg_color[1] - 0.0).abs() < 0.01);
        assert!((items[0].bg_color[2] - 0.0).abs() < 0.01);
        assert!((items[0].bg_color[3] - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_convert_background_color_name() {
        let css = "div { background-color: blue; }";
        let stylesheet = StyleSheet::parse(css);
        let matcher = StyleMatcher::new(stylesheet);
        let env = LayoutEnv::new(800.0, 600.0);
        let converter = LayoutConverter::new(matcher, env);

        let mut tree = DomTree::new();
        let div_id = tree.create_node("div".to_string());
        tree.add_node(None, div_id);

        let items = converter.convert_dom(&tree).unwrap();

        assert!((items[0].bg_color[2] - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_convert_border_width() {
        let css = "div { border-width: 2px; }";
        let stylesheet = StyleSheet::parse(css);
        let matcher = StyleMatcher::new(stylesheet);
        let env = LayoutEnv::new(800.0, 600.0);
        let converter = LayoutConverter::new(matcher, env);

        let mut tree = DomTree::new();
        let div_id = tree.create_node("div".to_string());
        tree.add_node(None, div_id);

        let items = converter.convert_dom(&tree).unwrap();

        assert_eq!(items[0].border, [2.0, 2.0, 2.0, 2.0]);
    }

    #[test]
    fn test_convert_overflow_hidden() {
        let css = "div { overflow: hidden; }";
        let stylesheet = StyleSheet::parse(css);
        let matcher = StyleMatcher::new(stylesheet);
        let env = LayoutEnv::new(800.0, 600.0);
        let converter = LayoutConverter::new(matcher, env);

        let mut tree = DomTree::new();
        let div_id = tree.create_node("div".to_string());
        tree.add_node(None, div_id);

        let items = converter.convert_dom(&tree).unwrap();

        assert_eq!(items[0].overflow, 1);
    }

    #[test]
    fn test_convert_overflow_default() {
        let stylesheet = StyleSheet::new();
        let matcher = StyleMatcher::new(stylesheet);
        let env = LayoutEnv::new(800.0, 600.0);
        let converter = LayoutConverter::new(matcher, env);

        let mut tree = DomTree::new();
        let div_id = tree.create_node("div".to_string());
        tree.add_node(None, div_id);

        let items = converter.convert_dom(&tree).unwrap();

        assert_eq!(items[0].overflow, 0);
    }

    #[test]
    fn test_convert_right_bottom() {
        let css = "div { position: absolute; top: 50px; left: 100px; right: 200px; bottom: 150px; }";
        let stylesheet = StyleSheet::parse(css);
        let matcher = StyleMatcher::new(stylesheet);
        let env = LayoutEnv::new(800.0, 600.0);
        let converter = LayoutConverter::new(matcher, env);

        let mut tree = DomTree::new();
        let div_id = tree.create_node("div".to_string());
        tree.add_node(None, div_id);

        let items = converter.convert_dom(&tree).unwrap();

        assert_eq!(items[0].size_constraint, [200.0, 150.0]);
    }

    #[test]
    fn test_convert_transform_translate() {
        let css = "div { transform: translate(10px, 20px); }";
        let stylesheet = StyleSheet::parse(css);
        let matcher = StyleMatcher::new(stylesheet);
        let env = LayoutEnv::new(800.0, 600.0);
        let converter = LayoutConverter::new(matcher, env);

        let mut tree = DomTree::new();
        let div_id = tree.create_node("div".to_string());
        tree.add_node(None, div_id);

        let items = converter.convert_dom(&tree).unwrap();

        assert_eq!(items[0].transform, [1.0, 0.0, 0.0, 1.0, 10.0, 20.0]);
    }

    #[test]
    fn test_convert_transform_rotate() {
        let css = "div { transform: rotate(90deg); }";
        let stylesheet = StyleSheet::parse(css);
        let matcher = StyleMatcher::new(stylesheet);
        let env = LayoutEnv::new(800.0, 600.0);
        let converter = LayoutConverter::new(matcher, env);

        let mut tree = DomTree::new();
        let div_id = tree.create_node("div".to_string());
        tree.add_node(None, div_id);

        let items = converter.convert_dom(&tree).unwrap();

        let expected = [0.0, 1.0, -1.0, 0.0, 0.0, 0.0];
        for i in 0..6 {
            assert!((items[0].transform[i] - expected[i]).abs() < 0.01);
        }
    }

    #[test]
    fn test_convert_transform_default() {
        let stylesheet = StyleSheet::new();
        let matcher = StyleMatcher::new(stylesheet);
        let env = LayoutEnv::new(800.0, 600.0);
        let converter = LayoutConverter::new(matcher, env);

        let mut tree = DomTree::new();
        let div_id = tree.create_node("div".to_string());
        tree.add_node(None, div_id);

        let items = converter.convert_dom(&tree).unwrap();

        assert_eq!(items[0].transform, [1.0, 0.0, 0.0, 1.0, 0.0, 0.0]);
    }

    #[test]
    fn test_convert_box_shadow() {
        let css = "div { box-shadow: 5px 5px 10px 0px rgba(0,0,0,0.5); }";
        let stylesheet = StyleSheet::parse(css);
        let matcher = StyleMatcher::new(stylesheet);
        let env = LayoutEnv::new(800.0, 600.0);
        let converter = LayoutConverter::new(matcher, env);

        let mut tree = DomTree::new();
        let div_id = tree.create_node("div".to_string());
        tree.add_node(None, div_id);

        let items = converter.convert_dom(&tree).unwrap();

        assert_eq!(items[0].has_shadow, 1);
        assert!((items[0].shadow_offset[0] - 5.0).abs() < 0.01);
        assert!((items[0].shadow_offset[1] - 5.0).abs() < 0.01);
        assert!((items[0].shadow_blur - 10.0).abs() < 0.01);
        assert!((items[0].shadow_spread - 0.0).abs() < 0.01);
    }

    #[test]
    fn test_convert_box_shadow_simple() {
        let css = "div { box-shadow: 3px 3px red; }";
        let stylesheet = StyleSheet::parse(css);
        let matcher = StyleMatcher::new(stylesheet);
        let env = LayoutEnv::new(800.0, 600.0);
        let converter = LayoutConverter::new(matcher, env);

        let mut tree = DomTree::new();
        let div_id = tree.create_node("div".to_string());
        tree.add_node(None, div_id);

        let items = converter.convert_dom(&tree).unwrap();

        assert_eq!(items[0].has_shadow, 1);
        assert!((items[0].shadow_offset[0] - 3.0).abs() < 0.01);
        assert!((items[0].shadow_offset[1] - 3.0).abs() < 0.01);
    }

    #[test]
    fn test_convert_box_shadow_default() {
        let stylesheet = StyleSheet::new();
        let matcher = StyleMatcher::new(stylesheet);
        let env = LayoutEnv::new(800.0, 600.0);
        let converter = LayoutConverter::new(matcher, env);

        let mut tree = DomTree::new();
        let div_id = tree.create_node("div".to_string());
        tree.add_node(None, div_id);

        let items = converter.convert_dom(&tree).unwrap();

        assert_eq!(items[0].has_shadow, 0);
    }

    #[test]
    fn test_convert_border_radius_single() {
        let css = "div { border-radius: 10px; }";
        let stylesheet = StyleSheet::parse(css);
        let matcher = StyleMatcher::new(stylesheet);
        let env = LayoutEnv::new(800.0, 600.0);
        let converter = LayoutConverter::new(matcher, env);

        let mut tree = DomTree::new();
        let div_id = tree.create_node("div".to_string());
        tree.add_node(None, div_id);

        let items = converter.convert_dom(&tree).unwrap();

        assert_eq!(items[0].border_radius, [10.0, 10.0, 10.0, 10.0]);
    }

    #[test]
    fn test_convert_border_radius_two_values() {
        let css = "div { border-radius: 10px 20px; }";
        let stylesheet = StyleSheet::parse(css);
        let matcher = StyleMatcher::new(stylesheet);
        let env = LayoutEnv::new(800.0, 600.0);
        let converter = LayoutConverter::new(matcher, env);

        let mut tree = DomTree::new();
        let div_id = tree.create_node("div".to_string());
        tree.add_node(None, div_id);

        let items = converter.convert_dom(&tree).unwrap();

        assert_eq!(items[0].border_radius, [10.0, 20.0, 10.0, 20.0]);
    }

    #[test]
    fn test_convert_border_radius_four_values() {
        let css = "div { border-radius: 5px 10px 15px 20px; }";
        let stylesheet = StyleSheet::parse(css);
        let matcher = StyleMatcher::new(stylesheet);
        let env = LayoutEnv::new(800.0, 600.0);
        let converter = LayoutConverter::new(matcher, env);

        let mut tree = DomTree::new();
        let div_id = tree.create_node("div".to_string());
        tree.add_node(None, div_id);

        let items = converter.convert_dom(&tree).unwrap();

        assert_eq!(items[0].border_radius, [5.0, 10.0, 15.0, 20.0]);
    }

    #[test]
    fn test_convert_border_radius_default() {
        let stylesheet = StyleSheet::new();
        let matcher = StyleMatcher::new(stylesheet);
        let env = LayoutEnv::new(800.0, 600.0);
        let converter = LayoutConverter::new(matcher, env);

        let mut tree = DomTree::new();
        let div_id = tree.create_node("div".to_string());
        tree.add_node(None, div_id);

        let items = converter.convert_dom(&tree).unwrap();

        assert_eq!(items[0].border_radius, [0.0, 0.0, 0.0, 0.0]);
    }

    #[test]
    fn test_convert_visibility_hidden() {
        let css = "div { visibility: hidden; }";
        let stylesheet = StyleSheet::parse(css);
        let matcher = StyleMatcher::new(stylesheet);
        let env = LayoutEnv::new(800.0, 600.0);
        let converter = LayoutConverter::new(matcher, env);

        let mut tree = DomTree::new();
        let div_id = tree.create_node("div".to_string());
        tree.add_node(None, div_id);

        let items = converter.convert_dom(&tree).unwrap();

        assert_eq!(items[0].visibility, 1);
    }

    #[test]
    fn test_convert_visibility_collapse() {
        let css = "div { visibility: collapse; }";
        let stylesheet = StyleSheet::parse(css);
        let matcher = StyleMatcher::new(stylesheet);
        let env = LayoutEnv::new(800.0, 600.0);
        let converter = LayoutConverter::new(matcher, env);

        let mut tree = DomTree::new();
        let div_id = tree.create_node("div".to_string());
        tree.add_node(None, div_id);

        let items = converter.convert_dom(&tree).unwrap();

        assert_eq!(items[0].visibility, 2);
    }

    #[test]
    fn test_convert_visibility_default() {
        let stylesheet = StyleSheet::new();
        let matcher = StyleMatcher::new(stylesheet);
        let env = LayoutEnv::new(800.0, 600.0);
        let converter = LayoutConverter::new(matcher, env);

        let mut tree = DomTree::new();
        let div_id = tree.create_node("div".to_string());
        tree.add_node(None, div_id);

        let items = converter.convert_dom(&tree).unwrap();

        assert_eq!(items[0].visibility, 0);
    }

    #[test]
    fn test_convert_flex_direction_row() {
        let css = "div { flex-direction: row; }";
        let stylesheet = StyleSheet::parse(css);
        let matcher = StyleMatcher::new(stylesheet);
        let env = LayoutEnv::new(800.0, 600.0);
        let converter = LayoutConverter::new(matcher, env);

        let mut tree = DomTree::new();
        let div_id = tree.create_node("div".to_string());
        tree.add_node(None, div_id);

        let items = converter.convert_dom(&tree).unwrap();

        assert_eq!(items[0].flex_direction, 0);
    }

    #[test]
    fn test_convert_flex_direction_column() {
        let css = "div { flex-direction: column; }";
        let stylesheet = StyleSheet::parse(css);
        let matcher = StyleMatcher::new(stylesheet);
        let env = LayoutEnv::new(800.0, 600.0);
        let converter = LayoutConverter::new(matcher, env);

        let mut tree = DomTree::new();
        let div_id = tree.create_node("div".to_string());
        tree.add_node(None, div_id);

        let items = converter.convert_dom(&tree).unwrap();

        assert_eq!(items[0].flex_direction, 2);
    }

    #[test]
    fn test_convert_flex_direction_row_reverse() {
        let css = "div { flex-direction: row-reverse; }";
        let stylesheet = StyleSheet::parse(css);
        let matcher = StyleMatcher::new(stylesheet);
        let env = LayoutEnv::new(800.0, 600.0);
        let converter = LayoutConverter::new(matcher, env);

        let mut tree = DomTree::new();
        let div_id = tree.create_node("div".to_string());
        tree.add_node(None, div_id);

        let items = converter.convert_dom(&tree).unwrap();

        assert_eq!(items[0].flex_direction, 1);
    }

    #[test]
    fn test_convert_flex_wrap_wrap() {
        let css = "div { flex-wrap: wrap; }";
        let stylesheet = StyleSheet::parse(css);
        let matcher = StyleMatcher::new(stylesheet);
        let env = LayoutEnv::new(800.0, 600.0);
        let converter = LayoutConverter::new(matcher, env);

        let mut tree = DomTree::new();
        let div_id = tree.create_node("div".to_string());
        tree.add_node(None, div_id);

        let items = converter.convert_dom(&tree).unwrap();

        assert_eq!(items[0].flex_wrap, 1);
    }

    #[test]
    fn test_convert_flex_wrap_default() {
        let stylesheet = StyleSheet::new();
        let matcher = StyleMatcher::new(stylesheet);
        let env = LayoutEnv::new(800.0, 600.0);
        let converter = LayoutConverter::new(matcher, env);

        let mut tree = DomTree::new();
        let div_id = tree.create_node("div".to_string());
        tree.add_node(None, div_id);

        let items = converter.convert_dom(&tree).unwrap();

        assert_eq!(items[0].flex_wrap, 0);
    }

    #[test]
    fn test_convert_align_items_center() {
        let css = "div { align-items: center; }";
        let stylesheet = StyleSheet::parse(css);
        let matcher = StyleMatcher::new(stylesheet);
        let env = LayoutEnv::new(800.0, 600.0);
        let converter = LayoutConverter::new(matcher, env);

        let mut tree = DomTree::new();
        let div_id = tree.create_node("div".to_string());
        tree.add_node(None, div_id);

        let items = converter.convert_dom(&tree).unwrap();

        assert_eq!(items[0].align_items, 3);
    }

    #[test]
    fn test_convert_align_items_flex_end() {
        let css = "div { align-items: flex-end; }";
        let stylesheet = StyleSheet::parse(css);
        let matcher = StyleMatcher::new(stylesheet);
        let env = LayoutEnv::new(800.0, 600.0);
        let converter = LayoutConverter::new(matcher, env);

        let mut tree = DomTree::new();
        let div_id = tree.create_node("div".to_string());
        tree.add_node(None, div_id);

        let items = converter.convert_dom(&tree).unwrap();

        assert_eq!(items[0].align_items, 2);
    }

    #[test]
    fn test_convert_align_items_default() {
        let stylesheet = StyleSheet::new();
        let matcher = StyleMatcher::new(stylesheet);
        let env = LayoutEnv::new(800.0, 600.0);
        let converter = LayoutConverter::new(matcher, env);

        let mut tree = DomTree::new();
        let div_id = tree.create_node("div".to_string());
        tree.add_node(None, div_id);

        let items = converter.convert_dom(&tree).unwrap();

        assert_eq!(items[0].align_items, 0);
    }

    #[test]
    fn test_convert_justify_content_center() {
        let css = "div { justify-content: center; }";
        let stylesheet = StyleSheet::parse(css);
        let matcher = StyleMatcher::new(stylesheet);
        let env = LayoutEnv::new(800.0, 600.0);
        let converter = LayoutConverter::new(matcher, env);

        let mut tree = DomTree::new();
        let div_id = tree.create_node("div".to_string());
        tree.add_node(None, div_id);

        let items = converter.convert_dom(&tree).unwrap();

        assert_eq!(items[0].justify_content, 2);
    }

    #[test]
    fn test_convert_justify_content_space_between() {
        let css = "div { justify-content: space-between; }";
        let stylesheet = StyleSheet::parse(css);
        let matcher = StyleMatcher::new(stylesheet);
        let env = LayoutEnv::new(800.0, 600.0);
        let converter = LayoutConverter::new(matcher, env);

        let mut tree = DomTree::new();
        let div_id = tree.create_node("div".to_string());
        tree.add_node(None, div_id);

        let items = converter.convert_dom(&tree).unwrap();

        assert_eq!(items[0].justify_content, 3);
    }

    #[test]
    fn test_convert_justify_content_default() {
        let stylesheet = StyleSheet::new();
        let matcher = StyleMatcher::new(stylesheet);
        let env = LayoutEnv::new(800.0, 600.0);
        let converter = LayoutConverter::new(matcher, env);

        let mut tree = DomTree::new();
        let div_id = tree.create_node("div".to_string());
        tree.add_node(None, div_id);

        let items = converter.convert_dom(&tree).unwrap();

        assert_eq!(items[0].justify_content, 0);
    }

    #[test]
    fn test_convert_align_content_center() {
        let css = "div { align-content: center; }";
        let stylesheet = StyleSheet::parse(css);
        let matcher = StyleMatcher::new(stylesheet);
        let env = LayoutEnv::new(800.0, 600.0);
        let converter = LayoutConverter::new(matcher, env);

        let mut tree = DomTree::new();
        let div_id = tree.create_node("div".to_string());
        tree.add_node(None, div_id);

        let items = converter.convert_dom(&tree).unwrap();

        assert_eq!(items[0].align_content, 3);
    }

    #[test]
    fn test_convert_align_content_default() {
        let stylesheet = StyleSheet::new();
        let matcher = StyleMatcher::new(stylesheet);
        let env = LayoutEnv::new(800.0, 600.0);
        let converter = LayoutConverter::new(matcher, env);

        let mut tree = DomTree::new();
        let div_id = tree.create_node("div".to_string());
        tree.add_node(None, div_id);

        let items = converter.convert_dom(&tree).unwrap();

        assert_eq!(items[0].align_content, 0);
    }

    #[test]
    fn test_convert_align_self_flex_end() {
        let css = "div { align-self: flex-end; }";
        let stylesheet = StyleSheet::parse(css);
        let matcher = StyleMatcher::new(stylesheet);
        let env = LayoutEnv::new(800.0, 600.0);
        let converter = LayoutConverter::new(matcher, env);

        let mut tree = DomTree::new();
        let div_id = tree.create_node("div".to_string());
        tree.add_node(None, div_id);

        let items = converter.convert_dom(&tree).unwrap();

        assert_eq!(items[0].align_self, 2);
    }

    #[test]
    fn test_convert_align_self_default() {
        let stylesheet = StyleSheet::new();
        let matcher = StyleMatcher::new(stylesheet);
        let env = LayoutEnv::new(800.0, 600.0);
        let converter = LayoutConverter::new(matcher, env);

        let mut tree = DomTree::new();
        let div_id = tree.create_node("div".to_string());
        tree.add_node(None, div_id);

        let items = converter.convert_dom(&tree).unwrap();

        assert_eq!(items[0].align_self, 0);
    }

    #[test]
    fn test_convert_order_positive() {
        let css = "div { order: 5; }";
        let stylesheet = StyleSheet::parse(css);
        let matcher = StyleMatcher::new(stylesheet);
        let env = LayoutEnv::new(800.0, 600.0);
        let converter = LayoutConverter::new(matcher, env);

        let mut tree = DomTree::new();
        let div_id = tree.create_node("div".to_string());
        tree.add_node(None, div_id);

        let items = converter.convert_dom(&tree).unwrap();

        assert_eq!(items[0].order, 5);
    }

    #[test]
    fn test_convert_order_negative() {
        let css = "div { order: -3; }";
        let stylesheet = StyleSheet::parse(css);
        let matcher = StyleMatcher::new(stylesheet);
        let env = LayoutEnv::new(800.0, 600.0);
        let converter = LayoutConverter::new(matcher, env);

        let mut tree = DomTree::new();
        let div_id = tree.create_node("div".to_string());
        tree.add_node(None, div_id);

        let items = converter.convert_dom(&tree).unwrap();

        assert_eq!(items[0].order, -3);
    }

    #[test]
    fn test_convert_order_default() {
        let stylesheet = StyleSheet::new();
        let matcher = StyleMatcher::new(stylesheet);
        let env = LayoutEnv::new(800.0, 600.0);
        let converter = LayoutConverter::new(matcher, env);

        let mut tree = DomTree::new();
        let div_id = tree.create_node("div".to_string());
        tree.add_node(None, div_id);

        let items = converter.convert_dom(&tree).unwrap();

        assert_eq!(items[0].order, 0);
    }

    #[test]
    fn test_convert_flex_basis_length() {
        let css = "div { flex-basis: 200px; }";
        let stylesheet = StyleSheet::parse(css);
        let matcher = StyleMatcher::new(stylesheet);
        let env = LayoutEnv::new(800.0, 600.0);
        let converter = LayoutConverter::new(matcher, env);

        let mut tree = DomTree::new();
        let div_id = tree.create_node("div".to_string());
        tree.add_node(None, div_id);

        let items = converter.convert_dom(&tree).unwrap();

        assert_eq!(items[0].flex_basis, 200.0);
    }

    #[test]
    fn test_convert_flex_basis_auto() {
        let css = "div { flex-basis: auto; }";
        let stylesheet = StyleSheet::parse(css);
        let matcher = StyleMatcher::new(stylesheet);
        let env = LayoutEnv::new(800.0, 600.0);
        let converter = LayoutConverter::new(matcher, env);

        let mut tree = DomTree::new();
        let div_id = tree.create_node("div".to_string());
        tree.add_node(None, div_id);

        let items = converter.convert_dom(&tree).unwrap();

        assert_eq!(items[0].flex_basis, 0.0);
    }

    #[test]
    fn test_convert_flex_basis_default() {
        let stylesheet = StyleSheet::new();
        let matcher = StyleMatcher::new(stylesheet);
        let env = LayoutEnv::new(800.0, 600.0);
        let converter = LayoutConverter::new(matcher, env);

        let mut tree = DomTree::new();
        let div_id = tree.create_node("div".to_string());
        tree.add_node(None, div_id);

        let items = converter.convert_dom(&tree).unwrap();

        assert_eq!(items[0].flex_basis, 0.0);
    }

    #[test]
    fn test_convert_gap_length() {
        let css = "div { gap: 10px; }";
        let stylesheet = StyleSheet::parse(css);
        let matcher = StyleMatcher::new(stylesheet);
        let env = LayoutEnv::new(800.0, 600.0);
        let converter = LayoutConverter::new(matcher, env);

        let mut tree = DomTree::new();
        let div_id = tree.create_node("div".to_string());
        tree.add_node(None, div_id);

        let items = converter.convert_dom(&tree).unwrap();

        assert_eq!(items[0].gap, 10.0);
    }

    #[test]
    fn test_convert_gap_default() {
        let stylesheet = StyleSheet::new();
        let matcher = StyleMatcher::new(stylesheet);
        let env = LayoutEnv::new(800.0, 600.0);
        let converter = LayoutConverter::new(matcher, env);

        let mut tree = DomTree::new();
        let div_id = tree.create_node("div".to_string());
        tree.add_node(None, div_id);

        let items = converter.convert_dom(&tree).unwrap();

        assert_eq!(items[0].gap, 0.0);
    }

    #[test]
    fn test_convert_outline_width() {
        let css = "div { outline-width: 2px; }";
        let stylesheet = StyleSheet::parse(css);
        let matcher = StyleMatcher::new(stylesheet);
        let env = LayoutEnv::new(800.0, 600.0);
        let converter = LayoutConverter::new(matcher, env);

        let mut tree = DomTree::new();
        let div_id = tree.create_node("div".to_string());
        tree.add_node(None, div_id);

        let items = converter.convert_dom(&tree).unwrap();

        assert_eq!(items[0].outline_width, 2.0);
    }

    #[test]
    fn test_convert_outline_color() {
        let css = "div { outline-color: #ff0000; }";
        let stylesheet = StyleSheet::parse(css);
        let matcher = StyleMatcher::new(stylesheet);
        let env = LayoutEnv::new(800.0, 600.0);
        let converter = LayoutConverter::new(matcher, env);

        let mut tree = DomTree::new();
        let div_id = tree.create_node("div".to_string());
        tree.add_node(None, div_id);

        let items = converter.convert_dom(&tree).unwrap();

        assert!((items[0].outline_color[0] - 1.0).abs() < 0.01);
        assert!((items[0].outline_color[1] - 0.0).abs() < 0.01);
        assert!((items[0].outline_color[2] - 0.0).abs() < 0.01);
    }

    #[test]
    fn test_convert_outline_default() {
        let stylesheet = StyleSheet::new();
        let matcher = StyleMatcher::new(stylesheet);
        let env = LayoutEnv::new(800.0, 600.0);
        let converter = LayoutConverter::new(matcher, env);

        let mut tree = DomTree::new();
        let div_id = tree.create_node("div".to_string());
        tree.add_node(None, div_id);

        let items = converter.convert_dom(&tree).unwrap();

        assert_eq!(items[0].outline_width, 0.0);
    }

    #[test]
    fn test_convert_cursor_pointer() {
        let css = "div { cursor: pointer; }";
        let stylesheet = StyleSheet::parse(css);
        let matcher = StyleMatcher::new(stylesheet);
        let env = LayoutEnv::new(800.0, 600.0);
        let converter = LayoutConverter::new(matcher, env);

        let mut tree = DomTree::new();
        let div_id = tree.create_node("div".to_string());
        tree.add_node(None, div_id);

        let items = converter.convert_dom(&tree).unwrap();

        assert_eq!(items[0].cursor, 1);
    }

    #[test]
    fn test_convert_cursor_text() {
        let css = "div { cursor: text; }";
        let stylesheet = StyleSheet::parse(css);
        let matcher = StyleMatcher::new(stylesheet);
        let env = LayoutEnv::new(800.0, 600.0);
        let converter = LayoutConverter::new(matcher, env);

        let mut tree = DomTree::new();
        let div_id = tree.create_node("div".to_string());
        tree.add_node(None, div_id);

        let items = converter.convert_dom(&tree).unwrap();

        assert_eq!(items[0].cursor, 2);
    }

    #[test]
    fn test_convert_cursor_default() {
        let stylesheet = StyleSheet::new();
        let matcher = StyleMatcher::new(stylesheet);
        let env = LayoutEnv::new(800.0, 600.0);
        let converter = LayoutConverter::new(matcher, env);

        let mut tree = DomTree::new();
        let div_id = tree.create_node("div".to_string());
        tree.add_node(None, div_id);

        let items = converter.convert_dom(&tree).unwrap();

        assert_eq!(items[0].cursor, 0);
    }

    #[test]
    fn test_convert_pointer_events_none() {
        let css = "div { pointer-events: none; }";
        let stylesheet = StyleSheet::parse(css);
        let matcher = StyleMatcher::new(stylesheet);
        let env = LayoutEnv::new(800.0, 600.0);
        let converter = LayoutConverter::new(matcher, env);

        let mut tree = DomTree::new();
        let div_id = tree.create_node("div".to_string());
        tree.add_node(None, div_id);

        let items = converter.convert_dom(&tree).unwrap();

        assert_eq!(items[0].pointer_events, 1);
    }

    #[test]
    fn test_convert_pointer_events_all() {
        let css = "div { pointer-events: all; }";
        let stylesheet = StyleSheet::parse(css);
        let matcher = StyleMatcher::new(stylesheet);
        let env = LayoutEnv::new(800.0, 600.0);
        let converter = LayoutConverter::new(matcher, env);

        let mut tree = DomTree::new();
        let div_id = tree.create_node("div".to_string());
        tree.add_node(None, div_id);

        let items = converter.convert_dom(&tree).unwrap();

        assert_eq!(items[0].pointer_events, 9);
    }

    #[test]
    fn test_convert_pointer_events_default() {
        let stylesheet = StyleSheet::new();
        let matcher = StyleMatcher::new(stylesheet);
        let env = LayoutEnv::new(800.0, 600.0);
        let converter = LayoutConverter::new(matcher, env);

        let mut tree = DomTree::new();
        let div_id = tree.create_node("div".to_string());
        tree.add_node(None, div_id);

        let items = converter.convert_dom(&tree).unwrap();

        assert_eq!(items[0].pointer_events, 0);
    }

    #[test]
    fn test_convert_float_left() {
        let css = "div { float: left; }";
        let stylesheet = StyleSheet::parse(css);
        let matcher = StyleMatcher::new(stylesheet);
        let env = LayoutEnv::new(800.0, 600.0);
        let converter = LayoutConverter::new(matcher, env);

        let mut tree = DomTree::new();
        let div_id = tree.create_node("div".to_string());
        tree.add_node(None, div_id);

        let items = converter.convert_dom(&tree).unwrap();

        assert_eq!(items[0].float, 1);
    }

    #[test]
    fn test_convert_float_right() {
        let css = "div { float: right; }";
        let stylesheet = StyleSheet::parse(css);
        let matcher = StyleMatcher::new(stylesheet);
        let env = LayoutEnv::new(800.0, 600.0);
        let converter = LayoutConverter::new(matcher, env);

        let mut tree = DomTree::new();
        let div_id = tree.create_node("div".to_string());
        tree.add_node(None, div_id);

        let items = converter.convert_dom(&tree).unwrap();

        assert_eq!(items[0].float, 2);
    }

    #[test]
    fn test_convert_float_default() {
        let stylesheet = StyleSheet::new();
        let matcher = StyleMatcher::new(stylesheet);
        let env = LayoutEnv::new(800.0, 600.0);
        let converter = LayoutConverter::new(matcher, env);

        let mut tree = DomTree::new();
        let div_id = tree.create_node("div".to_string());
        tree.add_node(None, div_id);

        let items = converter.convert_dom(&tree).unwrap();

        assert_eq!(items[0].float, 0);
    }

    #[test]
    fn test_convert_clear_both() {
        let css = "div { clear: both; }";
        let stylesheet = StyleSheet::parse(css);
        let matcher = StyleMatcher::new(stylesheet);
        let env = LayoutEnv::new(800.0, 600.0);
        let converter = LayoutConverter::new(matcher, env);

        let mut tree = DomTree::new();
        let div_id = tree.create_node("div".to_string());
        tree.add_node(None, div_id);

        let items = converter.convert_dom(&tree).unwrap();

        assert_eq!(items[0].clear, 3);
    }

    #[test]
    fn test_convert_clear_default() {
        let stylesheet = StyleSheet::new();
        let matcher = StyleMatcher::new(stylesheet);
        let env = LayoutEnv::new(800.0, 600.0);
        let converter = LayoutConverter::new(matcher, env);

        let mut tree = DomTree::new();
        let div_id = tree.create_node("div".to_string());
        tree.add_node(None, div_id);

        let items = converter.convert_dom(&tree).unwrap();

        assert_eq!(items[0].clear, 0);
    }

    #[test]
    fn test_convert_overflow_x_hidden() {
        let css = "div { overflow-x: hidden; }";
        let stylesheet = StyleSheet::parse(css);
        let matcher = StyleMatcher::new(stylesheet);
        let env = LayoutEnv::new(800.0, 600.0);
        let converter = LayoutConverter::new(matcher, env);

        let mut tree = DomTree::new();
        let div_id = tree.create_node("div".to_string());
        tree.add_node(None, div_id);

        let items = converter.convert_dom(&tree).unwrap();

        assert_eq!(items[0].overflow_x, 1);
    }

    #[test]
    fn test_convert_overflow_y_scroll() {
        let css = "div { overflow-y: scroll; }";
        let stylesheet = StyleSheet::parse(css);
        let matcher = StyleMatcher::new(stylesheet);
        let env = LayoutEnv::new(800.0, 600.0);
        let converter = LayoutConverter::new(matcher, env);

        let mut tree = DomTree::new();
        let div_id = tree.create_node("div".to_string());
        tree.add_node(None, div_id);

        let items = converter.convert_dom(&tree).unwrap();

        assert_eq!(items[0].overflow_y, 2);
    }

    #[test]
    fn test_convert_overflow_x_y_default() {
        let stylesheet = StyleSheet::new();
        let matcher = StyleMatcher::new(stylesheet);
        let env = LayoutEnv::new(800.0, 600.0);
        let converter = LayoutConverter::new(matcher, env);

        let mut tree = DomTree::new();
        let div_id = tree.create_node("div".to_string());
        tree.add_node(None, div_id);

        let items = converter.convert_dom(&tree).unwrap();

        assert_eq!(items[0].overflow_x, 0);
        assert_eq!(items[0].overflow_y, 0);
    }
}