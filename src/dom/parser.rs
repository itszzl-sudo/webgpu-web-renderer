use crate::dom::tree::DomTree;
use std::iter::Peekable;
use std::str::Chars;

/// HTML 解析器
pub struct HtmlParser<'a> {
    input: Peekable<Chars<'a>>,
}

impl<'a> HtmlParser<'a> {
    /// 创建解析器
    pub fn new(html: &'a str) -> Self {
        HtmlParser {
            input: html.chars().peekable(),
        }
    }

/// 解析 HTML，构建 DOM 树
    pub fn parse(&mut self) -> Result<DomTree, String> {
        let mut dom_tree = DomTree::new();
        let mut parent_stack: Vec<usize> = Vec::new();

        // 跳过空白字符
        self.skip_whitespace();

        while let Some(&ch) = self.input.peek() {
            if ch == '<' {
                // 标签开始
                self.parse_tag(&mut dom_tree, &mut parent_stack)?;
            } else {
                // 文本内容
                if let Some(&parent_id) = parent_stack.last() {
                    let text = self.parse_text();
                    if !text.is_empty() {
                        if let Some(node) = dom_tree.get_node_mut(parent_id) {
                            node.set_text(text);
                        }
                    }
                } else {
                    self.skip_text();
                }
            }

            self.skip_whitespace();
        }

        Ok(dom_tree)
    }

    /// 解析标签
    fn parse_tag(
        &mut self,
        dom_tree: &mut DomTree,
        parent_stack: &mut Vec<usize>,
    ) -> Result<(), String> {
        // 消费 '<'
        self.input.next();

        // 检查是否是注释
        if self.check_string("!--") {
            self.skip_comment();
            return Ok(());
        }

        // 检查是否是结束标签
        if let Some(&'/') = self.input.peek() {
            self.input.next(); // 消费 '/'
            let tag_name = self.parse_tag_name();
            self.skip_whitespace();
            self.expect('>')?;

            // 弹出匹配的父节点
            if let Some(&parent_id) = parent_stack.last() {
                if let Some(node) = dom_tree.get_node(parent_id) {
                    if node.tag_name == tag_name {
                        parent_stack.pop();
                    }
                }
            }

            return Ok(());
        }

        // 解析开始标签
        let tag_name = self.parse_tag_name();

        if tag_name.is_empty() {
            return Err("Empty tag name".to_string());
        }

        let node_id = dom_tree.create_node(tag_name.clone());

        // 解析属性
        loop {
            self.skip_whitespace();

            match self.input.peek() {
                Some(&'>') => {
                    self.input.next(); // 消费 '>'
                    break;
                }
                Some(&'/') => {
                    self.input.next(); // 消费 '/'
                    self.expect('>')?;
                    // 自闭合标签，不加入栈
                    break;
                }
                Some(_) => {
                    // 解析属性
                    if let Some((name, value)) = self.parse_attribute()? {
                        if let Some(node) = dom_tree.get_node_mut(node_id) {
                            node.set_attr(name, value);
                        }
                    }
                }
                None => {
                    return Err("Unexpected end of input in tag".to_string());
                }
            }
        }

        // 将节点添加到父节点
        if let Some(parent_id) = parent_stack.last() {
            dom_tree.add_node(Some(*parent_id), node_id);
        } else {
            dom_tree.add_node(None, node_id);
        }

        // 检查是否是自闭合标签
        let is_void_tag = matches!(
            tag_name.as_str(),
            "area" | "base" | "br" | "col" | "embed" | "hr" | "img" | "input" |
            "link" | "meta" | "param" | "source" | "track" | "wbr"
        );

        if !is_void_tag {
            // 非自闭合标签，加入栈
            parent_stack.push(node_id);
        }

        Ok(())
    }

    /// 解析标签名
    fn parse_tag_name(&mut self) -> String {
        let mut name = String::new();

        while let Some(&ch) = self.input.peek() {
            if ch.is_alphanumeric() || ch == '-' || ch == '_' {
                name.push(ch);
                self.input.next();
            } else {
                break;
            }
        }

        name
    }

    /// 解析属性
    fn parse_attribute(&mut self) -> Result<Option<(String, String)>, String> {
        let mut name = String::new();

        // 解析属性名
        while let Some(&ch) = self.input.peek() {
            if ch.is_alphanumeric() || ch == '-' || ch == '_' || ch == ':' {
                name.push(ch);
                self.input.next();
            } else {
                break;
            }
        }

        if name.is_empty() {
            return Ok(None);
        }

        self.skip_whitespace();

        // 检查是否有值
        if let Some(&'=') = self.input.peek() {
            self.input.next(); // 消费 '='
            self.skip_whitespace();

            // 解析属性值
            let value = match self.input.peek() {
                Some(&'"') | Some(&'\'') => self.parse_quoted_string(),
                _ => self.parse_unquoted_value(),
            };

            Ok(Some((name, value)))
        } else {
            // 布尔属性
            Ok(Some((name, String::new())))
        }
    }

    /// 解析引号字符串
    fn parse_quoted_string(&mut self) -> String {
        let quote = self.input.next().unwrap(); // 消费引号
        let mut value = String::new();

        while let Some(&ch) = self.input.peek() {
            if ch == quote {
                self.input.next(); // 消费结束引号
                break;
            } else {
                value.push(ch);
                self.input.next();
            }
        }

        value
    }

    /// 解析未加引号的值
    fn parse_unquoted_value(&mut self) -> String {
        let mut value = String::new();

        while let Some(&ch) = self.input.peek() {
            if ch != '>' && ch != '/' && !ch.is_whitespace() {
                value.push(ch);
                self.input.next();
            } else {
                break;
            }
        }

        value
    }

    /// 解析文本
    fn parse_text(&mut self) -> String {
        let mut text = String::new();

        while let Some(&ch) = self.input.peek() {
            if ch != '<' {
                text.push(ch);
                self.input.next();
            } else {
                break;
            }
        }

        text.trim().to_string()
    }

    /// 跳过文本
    fn skip_text(&mut self) {
        while let Some(&ch) = self.input.peek() {
            if ch != '<' {
                self.input.next();
            } else {
                break;
            }
        }
    }

    /// 跳过空白字符
    fn skip_whitespace(&mut self) {
        while let Some(&ch) = self.input.peek() {
            if ch.is_whitespace() {
                self.input.next();
            } else {
                break;
            }
        }
    }

    /// 跳过注释
    fn skip_comment(&mut self) {
        // 已经确认是 <!--，消费这4个字符
        for _ in 0..4 {
            self.input.next();
        }

        let mut depth = 1;

        while depth > 0 {
            if self.check_string("<!--") {
                depth += 1;
                for _ in 0..4 {
                    self.input.next();
                }
            } else if self.check_string("-->") {
                depth -= 1;
                for _ in 0..3 {
                    self.input.next();
                }
            } else {
                self.input.next();
            }
        }
    }

    /// 检查字符串（不消费）
    fn check_string(&self, s: &str) -> bool {
        let mut temp_input = self.input.clone();
        
        for ch in s.chars() {
            if let Some(next_ch) = temp_input.next() {
                if next_ch != ch {
                    return false;
                }
            } else {
                return false;
            }
        }
        
        true
    }
    
    /// 检查并消费字符串
    fn peek_string(&mut self, s: &str) -> bool {
        if !self.check_string(s) {
            return false;
        }
        
        // 确认匹配，消费字符
        for _ in s.chars() {
            self.input.next();
        }
        
        true
    }

    /// 期望特定字符
    fn expect(&mut self, ch: char) -> Result<(), String> {
        match self.input.next() {
            Some(c) if c == ch => Ok(()),
            Some(c) => Err(format!("Expected '{}', got '{}'", ch, c)),
            None => Err(format!("Expected '{}', found end of input", ch)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple() {
        let html = "<div></div>";
        let mut parser = HtmlParser::new(html);
        let result = parser.parse();

        assert!(result.is_ok());
        let tree = result.unwrap();
        assert_eq!(tree.len(), 1);
    }

    #[test]
    fn test_parse_with_text() {
        let html = "<p>Hello World</p>";
        let mut parser = HtmlParser::new(html);
        let result = parser.parse();

        assert!(result.is_ok());
        let tree = result.unwrap();
        assert_eq!(tree.len(), 1);

        let node = tree.get_node(1).unwrap();
        assert_eq!(node.text_content, Some("Hello World".to_string()));
    }

    #[test]
    fn test_parse_with_attributes() {
        let html = r#"<div id="container" class="main"></div>"#;
        let mut parser = HtmlParser::new(html);
        let result = parser.parse();

        assert!(result.is_ok());
        let tree = result.unwrap();

        let node = tree.get_node(1).unwrap();
        assert_eq!(node.get_attr("id"), Some(&"container".to_string()));
        assert_eq!(node.get_attr("class"), Some(&"main".to_string()));
    }

    #[test]
    fn test_parse_nested() {
        let html = "<div><p></p></div>";
        let mut parser = HtmlParser::new(html);
        let result = parser.parse();

        assert!(result.is_ok());
        let tree = result.unwrap();
        assert_eq!(tree.len(), 2);

        let div = tree.get_node(1).unwrap();
        assert_eq!(div.children, vec![2]);

        let p = tree.get_node(2).unwrap();
        assert_eq!(p.parent, Some(1));
    }

    #[test]
    fn test_parse_void_tag() {
        let html = "<img src=\"test.jpg\" />";
        let mut parser = HtmlParser::new(html);
        let result = parser.parse();

        assert!(result.is_ok());
        let tree = result.unwrap();
        assert_eq!(tree.len(), 1);

        let img = tree.get_node(1).unwrap();
        assert_eq!(img.tag_name, "img");
        assert_eq!(img.get_attr("src"), Some(&"test.jpg".to_string()));
    }

    #[test]
    fn test_parse_comment() {
        let html = "<div><!-- comment --><p></p></div>";
        let mut parser = HtmlParser::new(html);
        let result = parser.parse();

        assert!(result.is_ok());
        let tree = result.unwrap();
        assert_eq!(tree.len(), 2); // div 和 p，注释不算节点
    }
}