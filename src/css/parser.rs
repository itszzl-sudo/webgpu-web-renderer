use std::collections::HashMap;

/// CSS 声明
#[derive(Debug, Clone, PartialEq)]
pub struct Declaration {
    pub property: String,
    pub value: String,
    pub important: bool,
}

impl Declaration {
    pub fn new(property: String, value: String) -> Self {
        Declaration {
            property,
            value,
            important: false,
        }
    }

    pub fn with_important(mut self) -> Self {
        self.important = true;
        self
    }
}

/// CSS 选择器类型
#[derive(Debug, Clone, PartialEq)]
pub enum Selector {
    /// 标签选择器，如 "div"
    Tag(String),
    /// ID 选择器，如 "#header"
    Id(String),
    /// 类选择器，如 ".container"
    Class(String),
    /// 属性选择器，如 "[type='text']"
    Attribute(String, String),
    /// 通用选择器，如 "*"
    Universal,
    /// 后代选择器，如 "div p" (祖先, 后代)
    Descendant(Box<Selector>, Box<Selector>),
}

impl Selector {
    /// 解析选择器
    pub fn parse(selector: &str) -> Vec<Self> {
        let mut result = Vec::new();

        let selector = selector.trim();

        if selector.is_empty() {
            return result;
        }

        // 检查是否是后代选择器（包含空格）
        let parts: Vec<&str> = selector.split_whitespace().collect();
        if parts.len() >= 2 {
            // 解析后代选择器
            if let Some(descendant) = Self::parse_descendant(&parts) {
                result.push(descendant);
                return result;
            }
        }

        // 单选择器解析
        result.push(Self::parse_single(selector));
        result
    }

    /// 解析后代选择器
    fn parse_descendant(parts: &[&str]) -> Option<Selector> {
        if parts.len() < 2 {
            return None;
        }

        let ancestor = Self::parse_single(parts[0]);
        let descendant = Self::parse_single(parts[parts.len() - 1]);

        Some(Selector::Descendant(
            Box::new(ancestor),
            Box::new(descendant),
        ))
    }

    /// 解析单个选择器
    fn parse_single(selector: &str) -> Self {
        let selector = selector.trim();

        if selector == "*" {
            Selector::Universal
        } else if selector.starts_with('#') {
            Selector::Id(selector[1..].to_string())
        } else if selector.starts_with('.') {
            Selector::Class(selector[1..].to_string())
        } else if selector.starts_with('[') {
            // 简化的属性选择器解析 [attr='value']
            if let Some(end) = selector.find(']') {
                let inner = &selector[1..end];
                if let Some(eq) = inner.find('=') {
                    let attr = inner[..eq].to_string();
                    let value = inner[eq + 1..].trim_matches(|c| c == '\'' || c == '"');
                    return Selector::Attribute(attr, value.to_string());
                }
            }
            Selector::Universal
        } else {
            // 标签选择器
            Selector::Tag(selector.to_string())
        }
    }
}

/// CSS 规则
#[derive(Debug, Clone)]
pub struct StyleRule {
    pub selectors: Vec<Selector>,
    pub declarations: Vec<Declaration>,
    pub specificity: u32,
}

impl StyleRule {
    pub fn new(selectors: Vec<Selector>, declarations: Vec<Declaration>) -> Self {
        let specificity = Self::calculate_specificity(&selectors);

        StyleRule {
            selectors,
            declarations,
            specificity,
        }
    }

    /// 计算选择器优先级
    /// (id, class, tag) -> (id * 256 + class) * 256 + tag
    fn calculate_specificity(selectors: &[Selector]) -> u32 {
        let mut id_count = 0;
        let mut class_count = 0;
        let mut tag_count = 0;

        for selector in selectors {
            Self::count_specificity(selector, &mut id_count, &mut class_count, &mut tag_count);
        }

        // 优先级计算：ID 最重要，类次之，标签最后
        (id_count * 256 + class_count) * 256 + tag_count
    }

    /// 递归计算单个选择器的优先级
    fn count_specificity(
        selector: &Selector,
        id_count: &mut u32,
        class_count: &mut u32,
        tag_count: &mut u32,
    ) {
        match selector {
            Selector::Id(_) => *id_count += 1,
            Selector::Class(_) | Selector::Attribute(_, _) => *class_count += 1,
            Selector::Tag(_) => *tag_count += 1,
            Selector::Universal => {}
            Selector::Descendant(ancestor, descendant) => {
                Self::count_specificity(ancestor, id_count, class_count, tag_count);
                Self::count_specificity(descendant, id_count, class_count, tag_count);
            }
        }
    }
}

/// CSS 样式表
#[derive(Debug, Clone)]
pub struct StyleSheet {
    pub rules: Vec<StyleRule>,
}

impl StyleSheet {
    pub fn new() -> Self {
        StyleSheet {
            rules: Vec::new(),
        }
    }

    /// 添加规则
    pub fn add_rule(&mut self, rule: StyleRule) {
        self.rules.push(rule);
    }

    /// 解析 CSS 文本
    pub fn parse(css_text: &str) -> Self {
        let mut stylesheet = StyleSheet::new();

        // 简化的 CSS 解析
        let blocks = Self::split_rules(css_text);

        for block in blocks {
            if let Some((selectors_part, declarations_part)) = Self::parse_rule_block(&block) {
                let selectors = Self::parse_selectors(&selectors_part);
                let declarations = Self::parse_declarations(&declarations_part);

                if !selectors.is_empty() && !declarations.is_empty() {
                    let rule = StyleRule::new(selectors, declarations);
                    stylesheet.add_rule(rule);
                }
            }
        }

        stylesheet
    }

    /// 分离 CSS 规则块
    fn split_rules(css_text: &str) -> Vec<String> {
        let mut rules = Vec::new();
        let mut current = String::new();
        let mut brace_depth = 0;

        for ch in css_text.chars() {
            current.push(ch);

            if ch == '{' {
                brace_depth += 1;
            } else if ch == '}' {
                brace_depth -= 1;

                if brace_depth == 0 {
                    rules.push(current.clone());
                    current.clear();
                }
            }
        }

        rules
    }

    /// 解析单个规则块
    fn parse_rule_block(block: &str) -> Option<(String, String)> {
        let brace_start = block.find('{')?;
        let brace_end = block.rfind('}')?;

        let selectors_part = block[..brace_start].trim().to_string();
        let declarations_part = block[brace_start + 1..brace_end].trim().to_string();

        Some((selectors_part, declarations_part))
    }

    /// 解析选择器
    fn parse_selectors(selectors_part: &str) -> Vec<Selector> {
        // 简化：只支持单选择器，逗号分隔的多个选择器只取第一个
        let selector = selectors_part.split(',').next().unwrap_or("").trim();

        if selector.is_empty() {
            Vec::new()
        } else {
            Selector::parse(selector)
        }
    }

    /// 解析声明
    fn parse_declarations(declarations_part: &str) -> Vec<Declaration> {
        let mut declarations = Vec::new();

        for decl in declarations_part.split(';') {
            let decl = decl.trim();
            if decl.is_empty() {
                continue;
            }

            if let Some(colon) = decl.find(':') {
                let property = decl[..colon].trim().to_string();
                let value = decl[colon + 1..].trim().to_string();

                let mut declaration = Declaration::new(property.clone(), value.clone());

                // 检查 !important
                if value.to_lowercase().ends_with("!important") {
                    declaration.important = true;
                    declaration.value = value[..value.len() - 10].trim().to_string();
                }

                declarations.push(declaration);
            }
        }

        declarations
    }

    /// 查找匹配特定选择器的规则（简化版本，不考虑DOM树结构）
    pub fn find_matching_rules(&self, tag: Option<&str>, id: Option<&str>, classes: &[&str]) -> Vec<&StyleRule> {
        let mut matching = Vec::new();

        for rule in &self.rules {
            for selector in &rule.selectors {
                let matches = Self::selector_matches(selector, tag, id, classes);

                if matches {
                    matching.push(rule);
                    break; // 规则中有一个选择器匹配即可
                }
            }
        }

        matching
    }

    /// 检查选择器是否匹配（静态匹配，不考虑后代关系）
    fn selector_matches(selector: &Selector, tag: Option<&str>, id: Option<&str>, classes: &[&str]) -> bool {
        match selector {
            Selector::Universal => true,
            Selector::Tag(selector_tag) => tag.map_or(false, |t| t == selector_tag),
            Selector::Id(selector_id) => id.map_or(false, |i| i == selector_id),
            Selector::Class(selector_class) => classes.contains(&selector_class.as_str()),
            Selector::Attribute(attr_name, attr_value) => {
                // 简化实现：属性选择器匹配
                // 实际应该检查节点的属性，这里暂时返回false
                false
            }
            Selector::Descendant(_, descendant) => {
                // 后代选择器的后代部分匹配当前节点
                Self::selector_matches(descendant, tag, id, classes)
            }
        }
    }
}

impl Default for StyleSheet {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_selector_parse_tag() {
        let selectors = Selector::parse("div");
        assert_eq!(selectors, vec![Selector::Tag("div".to_string())]);
    }

    #[test]
    fn test_selector_parse_id() {
        let selectors = Selector::parse("#header");
        assert_eq!(selectors, vec![Selector::Id("header".to_string())]);
    }

    #[test]
    fn test_selector_parse_class() {
        let selectors = Selector::parse(".container");
        assert_eq!(selectors, vec![Selector::Class("container".to_string())]);
    }

    #[test]
    fn test_declaration() {
        let decl = Declaration::new("color".to_string(), "red".to_string());
        assert_eq!(decl.property, "color");
        assert_eq!(decl.value, "red");
        assert!(!decl.important);
    }

    #[test]
    fn test_declaration_important() {
        let decl = Declaration::new("color".to_string(), "red !important".to_string());
        let decl = decl.with_important();
        assert!(decl.important);
    }

    #[test]
    fn test_stylesheet_parse_simple() {
        let css = "div { color: red; }";
        let stylesheet = StyleSheet::parse(css);

        assert_eq!(stylesheet.rules.len(), 1);
        assert_eq!(stylesheet.rules[0].selectors.len(), 1);
    }

    #[test]
    fn test_stylesheet_parse_multiple() {
        let css = "div { color: red; } p { font-size: 16px; }";
        let stylesheet = StyleSheet::parse(css);

        assert_eq!(stylesheet.rules.len(), 2);
    }

    #[test]
    fn test_specificity() {
        let tag_rule = StyleRule::new(vec![Selector::Tag("div".to_string())], vec![]);
        let class_rule = StyleRule::new(vec![Selector::Class("container".to_string())], vec![]);
        let id_rule = StyleRule::new(vec![Selector::Id("header".to_string())], vec![]);

        assert!(id_rule.specificity > class_rule.specificity);
        assert!(class_rule.specificity > tag_rule.specificity);
    }
}