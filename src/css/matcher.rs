use crate::css::parser::{StyleRule, StyleSheet, Declaration, Selector};
use crate::dom::tree::{DomTree, DomNode};
use std::collections::HashMap;

/// 计算的样式
#[derive(Debug, Clone, PartialEq)]
pub struct ComputedStyle {
    pub properties: HashMap<String, StyleValue>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum StyleValue {
    String(String),
    Color(String),
    Length(f32),
    Percentage(f32),
    Number(f32),
    Boolean(bool),
    Auto,
    Inherit,
}

impl StyleValue {
    pub fn as_string(&self) -> Option<&str> {
        match self {
            StyleValue::String(s) => Some(s.as_str()),
            StyleValue::Color(s) => Some(s.as_str()),
            _ => None,
        }
    }

    pub fn as_length(&self) -> Option<f32> {
        match self {
            StyleValue::Length(v) => Some(*v),
            _ => None,
        }
    }

    pub fn as_number(&self) -> Option<f32> {
        match self {
            StyleValue::Number(v) => Some(*v),
            _ => None,
        }
    }
}

/// 样式匹配器
pub struct StyleMatcher {
    stylesheet: StyleSheet,
}

impl StyleMatcher {
    pub fn new(stylesheet: StyleSheet) -> Self {
        StyleMatcher { stylesheet }
    }

    /// 为指定节点计算样式
    pub fn compute_style(&self, tree: &DomTree, node_id: usize) -> ComputedStyle {
        let node = match tree.get_node(node_id) {
            Some(n) => n,
            None => return ComputedStyle {
                properties: HashMap::new(),
            },
        };

        let mut style = ComputedStyle {
            properties: HashMap::new(),
        };

        // 1. 收集匹配的规则
        let mut matching_rules = self.find_matching_rules(tree, node_id);

        // 2. 按优先级排序（低优先级在前，高优先级在后，这样后面的会覆盖前面的）
        matching_rules.sort_by(|a, b| {
            a.specificity
                .cmp(&b.specificity)
                .then_with(|| a.declarations.len().cmp(&b.declarations.len()))
        });

        // 3. 应用样式
        for rule in matching_rules {
            for decl in &rule.declarations {
                let value = self.parse_style_value(&decl.value);
                style.properties.insert(decl.property.clone(), value);
            }
        }

        // 4. 应用内联样式（最高优先级）
        if let Some(style_attr) = node.get_attr("style") {
            if let Some(inline_decls) = self.parse_inline_style(style_attr) {
                for (property, value) in inline_decls {
                    style.properties.insert(property, value);
                }
            }
        }

        // 5. 样式继承
        if let Some(parent_id) = node.parent {
            let parent_style = self.compute_style(tree, parent_id);
            self.apply_inheritance(&mut style, &parent_style);
        }

        style
    }

    /// 查找匹配节点的规则
    fn find_matching_rules(&self, tree: &DomTree, node_id: usize) -> Vec<&StyleRule> {
        let mut matching = Vec::new();

        for rule in &self.stylesheet.rules {
            if self.rule_matches(tree, node_id, rule) {
                matching.push(rule);
            }
        }

        matching
    }

    /// 检查规则是否匹配节点
    fn rule_matches(&self, tree: &DomTree, node_id: usize, rule: &StyleRule) -> bool {
        for selector in &rule.selectors {
            if self.selector_matches(tree, node_id, selector) {
                return true;
            }
        }

        false
    }

    /// 检查选择器是否匹配
    fn selector_matches(
        &self,
        tree: &DomTree,
        node_id: usize,
        selector: &Selector,
    ) -> bool {
        let node = match tree.get_node(node_id) {
            Some(n) => n,
            None => return false,
        };

        let tag = Some(node.tag_name.as_str());
        let id = node.get_attr("id").map(|s| s.as_str());
        let default_class = String::new();
        let class_attr = node.get_attr("class").unwrap_or(&default_class);
        let classes: Vec<&str> = class_attr.split_whitespace().collect();

        match selector {
            Selector::Universal => true,
            Selector::Tag(selector_tag) => tag.map_or(false, |t| t == selector_tag),
            Selector::Id(selector_id) => id.map_or(false, |i| i == selector_id),
            Selector::Class(selector_class) => classes.contains(&selector_class.as_str()),
            Selector::Attribute(attr_name, attr_value) => {
                node.get_attr(attr_name).map_or(false, |v| v == attr_value)
            }
            Selector::Descendant(ancestor_selector, descendant_selector) => {
                // 首先检查当前节点是否匹配后代部分
                if !self.selector_matches(tree, node_id, descendant_selector) {
                    return false;
                }
                // 然后检查祖先链中是否有匹配祖先部分的节点
                self.has_ancestor_matching(tree, node_id, ancestor_selector)
            }
        }
    }

    /// 检查节点是否有匹配的祖先
    fn has_ancestor_matching(
        &self,
        tree: &DomTree,
        node_id: usize,
        ancestor_selector: &Selector,
    ) -> bool {
        let mut current_id = node_id;

        while let Some(node) = tree.get_node(current_id) {
            if let Some(parent_id) = node.parent {
                if self.selector_matches_simple(tree, parent_id, ancestor_selector) {
                    return true;
                }
                current_id = parent_id;
            } else {
                break;
            }
        }

        false
    }

    /// 简化的选择器匹配（不递归处理后代选择器）
    fn selector_matches_simple(
        &self,
        tree: &DomTree,
        node_id: usize,
        selector: &Selector,
    ) -> bool {
        let node = match tree.get_node(node_id) {
            Some(n) => n,
            None => return false,
        };

        let tag = Some(node.tag_name.as_str());
        let id = node.get_attr("id").map(|s| s.as_str());
        let default_class = String::new();
        let class_attr = node.get_attr("class").unwrap_or(&default_class);
        let classes: Vec<&str> = class_attr.split_whitespace().collect();

        match selector {
            Selector::Universal => true,
            Selector::Tag(selector_tag) => tag.map_or(false, |t| t == selector_tag),
            Selector::Id(selector_id) => id.map_or(false, |i| i == selector_id),
            Selector::Class(selector_class) => classes.contains(&selector_class.as_str()),
            Selector::Attribute(attr_name, attr_value) => {
                node.get_attr(attr_name).map_or(false, |v| v == attr_value)
            }
            Selector::Descendant(_, _) => {
                // 简化处理：后代选择器在简单匹配中只匹配后代部分
                true
            }
        }
    }

    /// 解析样式值
    fn parse_style_value(&self, value: &str) -> StyleValue {
        let value = value.trim();

        // 特殊值
        if value == "auto" {
            return StyleValue::Auto;
        }
        if value == "inherit" {
            return StyleValue::Inherit;
        }
        if value == "true" {
            return StyleValue::Boolean(true);
        }
        if value == "false" {
            return StyleValue::Boolean(false);
        }

        // 颜色值
        if value.starts_with('#') || value.starts_with("rgb") || value.starts_with("rgba") {
            return StyleValue::Color(value.to_string());
        }

        // 颜色名称
        let color_names = [
            "red", "blue", "green", "yellow", "orange", "purple", "pink", "black",
            "white", "gray", "grey", "brown", "cyan", "magenta", "lime", "maroon",
            "navy", "olive", "silver", "teal", "aqua", "fuchsia", "transparent",
        ];
        if color_names.contains(&value.to_lowercase().as_str()) {
            return StyleValue::Color(value.to_string());
        }

        // 百分比
        if value.ends_with('%') {
            let num = value[..value.len() - 1].parse::<f32>();
            return num.map_or(StyleValue::String(value.to_string()), StyleValue::Percentage);
        }

        // 单位值
        if let Some(suffix) = value.strip_suffix("px") {
            let num = suffix.trim().parse::<f32>();
            return num.map_or(StyleValue::String(value.to_string()), StyleValue::Length);
        }

        if let Some(suffix) = value.strip_suffix("em") {
            let num = suffix.trim().parse::<f32>();
            return num.map_or(StyleValue::String(value.to_string()), StyleValue::Length);
        }

        if let Some(suffix) = value.strip_suffix("rem") {
            let num = suffix.trim().parse::<f32>();
            return num.map_or(StyleValue::String(value.to_string()), StyleValue::Length);
        }

        // 纯数字
        if let Ok(num) = value.parse::<f32>() {
            return StyleValue::Number(num);
        }

        // 默认为字符串
        StyleValue::String(value.to_string())
    }

    /// 解析内联样式
    fn parse_inline_style(&self, style_attr: &str) -> Option<HashMap<String, StyleValue>> {
        let mut declarations = HashMap::new();

        for decl in style_attr.split(';') {
            let decl = decl.trim();
            if decl.is_empty() {
                continue;
            }

            if let Some(colon) = decl.find(':') {
                let property = decl[..colon].trim().to_string();
                let value = decl[colon + 1..].trim();
                let style_value = self.parse_style_value(value);
                declarations.insert(property, style_value);
            }
        }

        if declarations.is_empty() {
            None
        } else {
            Some(declarations)
        }
    }

    /// 应用样式继承
    fn apply_inheritance(&self, style: &mut ComputedStyle, parent_style: &ComputedStyle) {
        // 可继承的属性
        let inheritable_properties = [
            "color",
            "font-family",
            "font-size",
            "font-weight",
            "font-style",
            "line-height",
            "text-align",
            "text-decoration",
            "letter-spacing",
            "word-spacing",
            "white-space",
            "visibility",
        ];

        for prop in inheritable_properties {
            if !style.properties.contains_key(prop) {
                if let Some(parent_value) = parent_style.properties.get(prop) {
                    style.properties.insert(prop.to_string(), parent_value.clone());
                }
            }
        }
    }

    /// 获取样式值
    pub fn get_property<'a>(&self, style: &'a ComputedStyle, property: &str) -> Option<&'a StyleValue> {
        style.properties.get(property)
    }

    /// 获取样式值（默认值）
    pub fn get_property_or_default(&self, style: &ComputedStyle, property: &str, default: &str) -> String {
        match style.properties.get(property) {
            Some(StyleValue::String(s)) => s.clone(),
            Some(StyleValue::Color(s)) => s.clone(),
            Some(StyleValue::Length(n)) => format!("{}px", n),
            Some(StyleValue::Percentage(n)) => format!("{}%", n),
            Some(StyleValue::Number(n)) => n.to_string(),
            Some(StyleValue::Boolean(b)) => b.to_string(),
            Some(StyleValue::Auto) => "auto".to_string(),
            Some(StyleValue::Inherit) => "inherit".to_string(),
            None => default.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_style_value_parse_color() {
        let matcher = StyleMatcher::new(StyleSheet::new());

        assert!(matches!(
            matcher.parse_style_value("#ff0000"),
            StyleValue::Color(_)
        ));
        assert!(matches!(
            matcher.parse_style_value("rgb(255, 0, 0)"),
            StyleValue::Color(_)
        ));
    }

    #[test]
    fn test_style_value_parse_length() {
        let matcher = StyleMatcher::new(StyleSheet::new());

        match matcher.parse_style_value("100px") {
            StyleValue::Length(v) => assert_eq!(v, 100.0),
            _ => panic!("Expected Length"),
        }

        match matcher.parse_style_value("2em") {
            StyleValue::Length(v) => assert_eq!(v, 2.0),
            _ => panic!("Expected Length"),
        }
    }

    #[test]
    fn test_style_value_parse_percentage() {
        let matcher = StyleMatcher::new(StyleSheet::new());

        match matcher.parse_style_value("50%") {
            StyleValue::Percentage(v) => assert_eq!(v, 50.0),
            _ => panic!("Expected Percentage"),
        }
    }

    #[test]
    fn test_compute_style_simple() {
        let css = "div { color: red; }";
        let stylesheet = StyleSheet::parse(css);
        let matcher = StyleMatcher::new(stylesheet);

        let mut tree = DomTree::new();
        let node_id = tree.create_node("div".to_string());

        let style = matcher.compute_style(&tree, node_id);

        assert!(style.properties.contains_key("color"));
    }

    #[test]
    fn test_selector_specificity() {
        let css = "div { color: blue; } #test { color: red; }";
        let stylesheet = StyleSheet::parse(css);
        let matcher = StyleMatcher::new(stylesheet);

        let mut tree = DomTree::new();
        let node_id = tree.create_node("div".to_string());
        tree.get_node_mut(node_id).unwrap().set_attr("id".to_string(), "test".to_string());

        let style = matcher.compute_style(&tree, node_id);

        let color = style.properties.get("color").unwrap();
        if let StyleValue::Color(c) = color {
            assert_eq!(c, "red"); // ID 选择器优先级更高
        } else {
            panic!("Expected Color");
        }
    }

    #[test]
    fn test_inline_style_highest_priority() {
        let css = "div { color: blue; }";
        let stylesheet = StyleSheet::parse(css);
        let matcher = StyleMatcher::new(stylesheet);

        let mut tree = DomTree::new();
        let node_id = tree.create_node("div".to_string());
        tree.get_node_mut(node_id)
            .unwrap()
            .set_attr("style".to_string(), "color: red".to_string());

        let style = matcher.compute_style(&tree, node_id);

        let color = style.properties.get("color").unwrap();
        if let StyleValue::Color(c) = color {
            assert_eq!(c, "red"); // 内联样式优先级最高
        } else {
            panic!("Expected Color");
        }
    }

    #[test]
    fn test_descendant_selector() {
        let css = "div p { color: red; }";
        let stylesheet = StyleSheet::parse(css);
        let matcher = StyleMatcher::new(stylesheet);

        let mut tree = DomTree::new();
        let div_id = tree.create_node("div".to_string());
        let p_id = tree.create_node("p".to_string());
        
        // 建立父子关系
        tree.get_node_mut(p_id).unwrap().parent = Some(div_id);

        // p 在 div 下，应该匹配后代选择器
        let style = matcher.compute_style(&tree, p_id);
        let color = style.properties.get("color").unwrap();
        if let StyleValue::Color(c) = color {
            assert_eq!(c, "red");
        } else {
            panic!("Expected Color");
        }
    }

    #[test]
    fn test_attribute_selector() {
        let css = "[type='text'] { color: blue; }";
        let stylesheet = StyleSheet::parse(css);
        let matcher = StyleMatcher::new(stylesheet);

        let mut tree = DomTree::new();
        let input_id = tree.create_node("input".to_string());
        tree.get_node_mut(input_id)
            .unwrap()
            .set_attr("type".to_string(), "text".to_string());

        let style = matcher.compute_style(&tree, input_id);
        let color = style.properties.get("color").unwrap();
        if let StyleValue::Color(c) = color {
            assert_eq!(c, "blue");
        } else {
            panic!("Expected Color");
        }
    }
}