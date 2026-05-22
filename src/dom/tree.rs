use std::collections::HashMap;

/// DOM 节点
#[derive(Debug, Clone)]
pub struct DomNode {
    pub id: usize,
    pub tag_name: String,
    pub attributes: HashMap<String, String>,
    pub children: Vec<usize>,
    pub parent: Option<usize>,
    pub text_content: Option<String>,
}

impl DomNode {
    /// 创建新节点
    pub fn new(id: usize, tag_name: String) -> Self {
        DomNode {
            id,
            tag_name,
            attributes: HashMap::new(),
            children: Vec::new(),
            parent: None,
            text_content: None,
        }
    }

    /// 设置属性
    pub fn set_attr(&mut self, name: String, value: String) {
        self.attributes.insert(name, value);
    }

    /// 获取属性
    pub fn get_attr(&self, name: &str) -> Option<&String> {
        self.attributes.get(name)
    }

    /// 添加子节点
    pub fn add_child(&mut self, child_id: usize) {
        self.children.push(child_id);
    }

    /// 设置父节点
    pub fn set_parent(&mut self, parent_id: usize) {
        self.parent = Some(parent_id);
    }

    /// 设置文本内容
    pub fn set_text(&mut self, text: String) {
        self.text_content = Some(text);
    }
}

/// DOM 树管理器
#[derive(Debug)]
pub struct DomTree {
    nodes: HashMap<usize, DomNode>,
    next_id: usize,
    root: Option<usize>,
}

impl DomTree {
    /// 创建新的 DOM 树
    pub fn new() -> Self {
        DomTree {
            nodes: HashMap::new(),
            next_id: 1,
            root: None,
        }
    }

    /// 创建新节点
    pub fn create_node(&mut self, tag_name: String) -> usize {
        let id = self.next_id;
        self.next_id += 1;

        let node = DomNode::new(id, tag_name);
        self.nodes.insert(id, node);

        id
    }

    /// 添加节点到树中
    pub fn add_node(&mut self, parent_id: Option<usize>, node_id: usize) {
        if let Some(parent_id) = parent_id {
            if let Some(parent) = self.nodes.get_mut(&parent_id) {
                parent.add_child(node_id);
            }

            if let Some(node) = self.nodes.get_mut(&node_id) {
                node.set_parent(parent_id);
            }
        } else {
            // 没有父节点，作为根节点
            self.root = Some(node_id);
        }
    }

    /// 获取节点
    pub fn get_node(&self, id: usize) -> Option<&DomNode> {
        self.nodes.get(&id)
    }

    /// 获取节点（可变）
    pub fn get_node_mut(&mut self, id: usize) -> Option<&mut DomNode> {
        self.nodes.get_mut(&id)
    }

    /// 获取根节点
    pub fn get_root(&self) -> Option<usize> {
        self.root
    }

    /// 按 ID 选择器查找节点
    pub fn query_by_id(&self, id: &str) -> Option<usize> {
        // 去掉 #
        let id = id.trim_start_matches('#');

        for (&node_id, node) in &self.nodes {
            if let Some(node_id_attr) = node.get_attr("id") {
                if node_id_attr == id {
                    return Some(node_id);
                }
            }
        }

        None
    }

    /// 按标签名查找所有节点
    pub fn query_by_tag(&self, tag: &str) -> Vec<usize> {
        let mut result = Vec::new();

        for (&node_id, node) in &self.nodes {
            if node.tag_name == tag {
                result.push(node_id);
            }
        }

        result
    }

    /// 按类名查找所有节点
    pub fn query_by_class(&self, class: &str) -> Vec<usize> {
        let mut result = Vec::new();
        // 去掉 .
        let class = class.trim_start_matches('.');

        for (&node_id, node) in &self.nodes {
            if let Some(class_attr) = node.get_attr("class") {
                let classes: Vec<&str> = class_attr.split_whitespace().collect();
                if classes.contains(&class) {
                    result.push(node_id);
                }
            }
        }

        result
    }

    /// 清空 DOM 树
    pub fn clear(&mut self) {
        self.nodes.clear();
        self.next_id = 1;
        self.root = None;
    }

    /// 获取节点数量
    pub fn len(&self) -> usize {
        self.nodes.len()
    }
    
    /// 获取所有节点 ID
    pub fn get_all_nodes(&self) -> Vec<usize> {
        self.nodes.keys().cloned().collect()
    }

    /// 检查是否为空
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }
}

impl Default for DomTree {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_node() {
        let mut tree = DomTree::new();

        let node_id = tree.create_node("div".to_string());
        assert_eq!(node_id, 1);

        let node = tree.get_node(node_id);
        assert!(node.is_some());
        assert_eq!(node.unwrap().tag_name, "div");
    }

    #[test]
    fn test_add_child() {
        let mut tree = DomTree::new();

        let parent_id = tree.create_node("div".to_string());
        let child_id = tree.create_node("p".to_string());

        tree.add_node(None, parent_id); // 根节点
        tree.add_node(Some(parent_id), child_id);

        let parent = tree.get_node(parent_id).unwrap();
        assert_eq!(parent.children, vec![child_id]);

        let child = tree.get_node(child_id).unwrap();
        assert_eq!(child.parent, Some(parent_id));
    }

    #[test]
    fn test_query_by_id() {
        let mut tree = DomTree::new();

        let node_id = tree.create_node("div".to_string());
        let node = tree.get_node_mut(node_id).unwrap();
        node.set_attr("id".to_string(), "test-id".to_string());

        let result = tree.query_by_id("#test-id");
        assert_eq!(result, Some(node_id));

        let result = tree.query_by_id("test-id");
        assert_eq!(result, Some(node_id));
    }

    #[test]
    fn test_query_by_class() {
        let mut tree = DomTree::new();

        let node_id = tree.create_node("div".to_string());
        let node = tree.get_node_mut(node_id).unwrap();
        node.set_attr("class".to_string(), "container main".to_string());

        let result = tree.query_by_class(".main");
        assert_eq!(result, vec![node_id]);

        let result = tree.query_by_class("main");
        assert_eq!(result, vec![node_id]);
    }

    #[test]
    fn test_query_by_tag() {
        let mut tree = DomTree::new();

        let div_id = tree.create_node("div".to_string());
        let p_id = tree.create_node("p".to_string());

        let divs = tree.query_by_tag("div");
        assert_eq!(divs, vec![div_id]);

        let ps = tree.query_by_tag("p");
        assert_eq!(ps, vec![p_id]);
    }
}