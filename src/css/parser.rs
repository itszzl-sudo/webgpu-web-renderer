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
    /// 伪类选择器，如 ":hover", ":nth-child(2)"
    PseudoClass(String),
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

        // 单选择器解析（支持复合选择器如 div:first-child）
        result = Self::parse_compound(selector);
        result
    }

    /// 解析后代选择器
    fn parse_descendant(parts: &[&str]) -> Option<Selector> {
        if parts.len() < 2 {
            return None;
        }

        // 每个 part 可能是复合选择器，取第一个
        let ancestor = Self::parse_compound(parts[0]).into_iter().next()
            .unwrap_or(Selector::Universal);
        let descendant = Self::parse_compound(parts[parts.len() - 1]).into_iter().next()
            .unwrap_or(Selector::Universal);

        Some(Selector::Descendant(
            Box::new(ancestor),
            Box::new(descendant),
        ))
    }

    /// 解析复合选择器 (如 "div:first-child" -> [Tag("div"), PseudoClass("first-child")])
    fn parse_compound(selector: &str) -> Vec<Selector> {
        let selector = selector.trim();
        if selector.is_empty() || selector == "*" {
            return vec![Selector::Universal];
        }

        let mut result = Vec::new();
        let chars: Vec<char> = selector.chars().collect();
        let len = chars.len();
        let mut i = 0;

        // 解析第一个部分
        if i < len {
            let c = chars[i];
            match c {
                '#' => {
                    i += 1;
                    let id = Self::extract_selector_part(&chars, &mut i);
                    result.push(Selector::Id(id));
                }
                '.' => {
                    i += 1;
                    let class_name = Self::extract_selector_part(&chars, &mut i);
                    result.push(Selector::Class(class_name));
                }
                ':' => {
                    i += 1;
                    let pseudo = Self::extract_selector_part(&chars, &mut i);
                    result.push(Selector::PseudoClass(pseudo));
                }
                '[' => {
                    if let Some(end) = selector[i..].find(']') {
                        let inner = &selector[i+1..i+end];
                        i += end + 1;
                        if let Some(eq) = inner.find('=') {
                            let attr = inner[..eq].to_string();
                            let value = inner[eq + 1..].trim_matches(|c| c == '\'' || c == '"');
                            result.push(Selector::Attribute(attr, value.to_string()));
                        } else {
                            result.push(Selector::Attribute(inner.to_string(), String::new()));
                        }
                    }
                }
                _ => {
                    // tag 名开头
                    let tag = Self::extract_selector_part(&chars, &mut i);
                    result.push(Selector::Tag(tag));
                }
            }
        }

        // 继续解析剩余部分
        while i < len {
            let c = chars[i];
            match c {
                '.' => {
                    i += 1;
                    let class_name = Self::extract_selector_part(&chars, &mut i);
                    result.push(Selector::Class(class_name));
                }
                '#' => {
                    i += 1;
                    let id = Self::extract_selector_part(&chars, &mut i);
                    result.push(Selector::Id(id));
                }
                ':' => {
                    i += 1;
                    let pseudo = Self::extract_selector_part(&chars, &mut i);
                    result.push(Selector::PseudoClass(pseudo));
                }
                '[' => {
                    if let Some(end) = selector[i..].find(']') {
                        let inner = &selector[i+1..i+end];
                        i += end + 1;
                        if let Some(eq) = inner.find('=') {
                            let attr = inner[..eq].to_string();
                            let value = inner[eq + 1..].trim_matches(|c| c == '\'' || c == '"');
                            result.push(Selector::Attribute(attr, value.to_string()));
                        } else {
                            result.push(Selector::Attribute(inner.to_string(), String::new()));
                        }
                    } else {
                        break;
                    }
                }
                _ => i += 1, // 跳过空白或其他字符
            }
        }

        result
    }

    /// 提取选择器的一部分 (从当前位置到下一个分隔符)
    fn extract_selector_part(chars: &[char], i: &mut usize) -> String {
        let start = *i;
        while *i < chars.len() {
            let c = chars[*i];
            if c == '.' || c == '#' || c == ':' || c == '[' {
                break;
            }
            *i += 1;
        }
        chars[start..*i].iter().collect()
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
            Selector::Class(_) | Selector::Attribute(_, _) | Selector::PseudoClass(_) => *class_count += 1,
            Selector::Tag(_) => *tag_count += 1,
            Selector::Universal => {}
            Selector::Descendant(ancestor, descendant) => {
                Self::count_specificity(ancestor, id_count, class_count, tag_count);
                Self::count_specificity(descendant, id_count, class_count, tag_count);
            }
        }
    }
}

/// CSS 关键帧
#[derive(Debug, Clone)]
pub struct Keyframe {
    /// 关键帧选择器 (0.0~1.0, from=0.0, to=1.0)
    pub selector: f32,
    /// 关键帧声明
    pub declarations: Vec<Declaration>,
}

/// CSS 样式表
#[derive(Debug, Clone)]
pub struct StyleSheet {
    pub rules: Vec<StyleRule>,
    /// @keyframes 动画定义 (name → keyframes)
    pub keyframes: HashMap<String, Vec<Keyframe>>,
}

impl StyleSheet {
    pub fn new() -> Self {
        StyleSheet {
            rules: Vec::new(),
            keyframes: HashMap::new(),
        }
    }

    /// 添加规则
    pub fn add_rule(&mut self, rule: StyleRule) {
        self.rules.push(rule);
    }

/// 解析 CSS 文本
    pub fn parse(css_text: &str) -> Self {
        let mut stylesheet = StyleSheet::new();

        let blocks = Self::split_rules(css_text);

        for block in blocks {
            // 检测 @keyframes 规则
            if block.trim_start().starts_with("@keyframes") {
                if let Some((name, kf_list)) = Self::parse_keyframes(&block) {
                    stylesheet.keyframes.insert(name, kf_list);
                }
                continue;
            }

            if let Some((selectors_part, declarations_part)) = Self::parse_rule_block(&block) {
                let selector_groups = Self::parse_selectors(&selectors_part);
                let declarations = Self::parse_declarations(&declarations_part);

                if !declarations.is_empty() {
                    for (selectors, _specificity) in selector_groups {
                        if !selectors.is_empty() {
                            let rule = StyleRule::new(selectors, declarations.clone());
                            stylesheet.add_rule(rule);
                        }
                    }
                }
            }
        }

        stylesheet
    }

    /// 解析 @keyframes 规则
    /// 输入: "@keyframes slide { 0% { left: 0px; } 100% { left: 100px; } }"
    /// 输出: ("slide", [Keyframe{selector:0.0, ...}, Keyframe{selector:1.0, ...}])
    fn parse_keyframes(block: &str) -> Option<(String, Vec<Keyframe>)> {
        let block = block.trim();
        // 提取名称: "@keyframes name {" → 去掉 "@keyframes " 前缀
        let after_at = block.strip_prefix("@keyframes ")?;
        let name_end = after_at.find('{')?;
        let name = after_at[..name_end].trim().to_string();

        if name.is_empty() {
            return None;
        }

        // 提取 {} 内的所有内容
        let body_start = after_at.find('{')?;
        let body_end = after_at.rfind('}')?;
        let body = &after_at[body_start + 1..body_end].trim();

        if body.is_empty() {
            return Some((name, Vec::new()));
        }

        // 解析内部的关键帧选择器块: "0% { left: 0px; } 100% { left: 100px; }"
        // 用 split_rules 的相同逻辑拆分内部块
        let mut kf_list = Vec::new();
        let mut current = String::new();
        let mut depth = 0u32;

        for ch in body.chars() {
            current.push(ch);
            if ch == '{' {
                depth += 1;
            } else if ch == '}' {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    // 提取选择器和声明
                    let brace_start = current.find('{')?;
                    let selector_str = current[..brace_start].trim();
                    let decl_str = current[brace_start + 1..].trim_end_matches('}').trim();

                    let selector = if selector_str == "from" {
                        0.0
                    } else if selector_str == "to" {
                        1.0
                    } else if let Some(pct) = selector_str.strip_suffix('%') {
                        pct.trim().parse::<f32>().unwrap_or(0.0) / 100.0
                    } else {
                        selector_str.parse::<f32>().unwrap_or(0.0)
                    };

                    let decls = Self::parse_declarations(decl_str);
                    kf_list.push(Keyframe {
                        selector: selector.clamp(0.0, 1.0),
                        declarations: decls,
                    });

                    current.clear();
                }
            }
        }

        // 按选择器排序
        kf_list.sort_by(|a, b| a.selector.partial_cmp(&b.selector).unwrap_or(std::cmp::Ordering::Equal));

Some((name, kf_list))
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
    fn parse_selectors(selectors_part: &str) -> Vec<(Vec<Selector>, u32)> {
        selectors_part.split(',')
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .map(|s| {
                let selectors = Selector::parse(s);
                let specificity = StyleRule::calculate_specificity(&selectors);
                (selectors, specificity)
            })
            .collect()
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
            Selector::Attribute(_attr_name, _attr_value) => {
                false
            }
            Selector::PseudoClass(_pseudo) => {
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

    #[test]
    fn test_selector_parse_pseudo_class() {
        let selectors = Selector::parse(":hover");
        assert_eq!(selectors, vec![Selector::PseudoClass("hover".to_string())]);
    }

    #[test]
    fn test_selector_parse_nth_child() {
        let selectors = Selector::parse(":nth-child(2)");
        assert_eq!(selectors, vec![Selector::PseudoClass("nth-child(2)".to_string())]);
    }

    #[test]
    fn test_pseudo_class_specificity() {
        let pseudo_rule = StyleRule::new(vec![Selector::PseudoClass("hover".to_string())], vec![]);
        let tag_rule = StyleRule::new(vec![Selector::Tag("div".to_string())], vec![]);
        let class_rule = StyleRule::new(vec![Selector::Class("container".to_string())], vec![]);

        // 伪类优先级等于类选择器
        assert_eq!(pseudo_rule.specificity, class_rule.specificity);
        // 伪类优先级高于标签选择器
        assert!(pseudo_rule.specificity > tag_rule.specificity);
    }

    #[test]
    fn test_parse_keyframes() {
        let css = r#"
            @keyframes slide {
                0% { left: 0px; opacity: 0; }
                100% { left: 100px; opacity: 1; }
            }
            div { width: 200px; }
        "#;
        let stylesheet = StyleSheet::parse(css);

        // 应解析出 keyframes
        assert!(stylesheet.keyframes.contains_key("slide"));
        let kf = stylesheet.keyframes.get("slide").unwrap();
        assert_eq!(kf.len(), 2);
        assert_eq!(kf[0].selector, 0.0);
        assert_eq!(kf[1].selector, 1.0);

        // 普通规则也应解析
        assert_eq!(stylesheet.rules.len(), 1);
        assert_eq!(stylesheet.rules[0].selectors.len(), 1);

        // from/to 语法
        let css2 = "@keyframes fade { from { opacity: 0; } to { opacity: 1; } }";
        let ss2 = StyleSheet::parse(css2);
        let kf2 = ss2.keyframes.get("fade").unwrap();
        assert_eq!(kf2[0].selector, 0.0); // from = 0%
        assert_eq!(kf2[1].selector, 1.0); // to = 100%
    }
}