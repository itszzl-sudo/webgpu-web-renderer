/// 布局项 - Rust 侧表示
#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct LayoutItem {
    pub dom_id: u32,              // DOM 节点 ID
    pub size: [f32; 2],        // [width, height]
    pub margin: [f32; 4],      // [top, right, bottom, left]
    pub padding: [f32; 4],     // [top, right, bottom, left]
    pub pos: [f32; 2],         // [x, y]
    pub size_constraint: [f32; 2], // [right, bottom] - 绝对定位时用right/bottom约束尺寸
    pub flow_type: u32,        // 0=文档流, 1=绝对定位, 2=Flex, 3=Grid
    pub weight: f32,           // flex-grow
    pub flex_shrink: f32,      // flex-shrink
    pub z_index: f32,          // 层级
    pub tex_idx: u32,          // 纹理索引
    pub is_valid: u32,         // 是否启用 (0=禁用, 1=启用)
    pub is_hide: u32,          // 是否隐藏 (0=显示, 1=隐藏)
    pub bg_color: [f32; 4],    // 背景色 [r, g, b, a]
    pub border: [f32; 4],      // 边框宽度 [top, right, bottom, left]
    pub border_color: [f32; 4], // 边框颜色 [r, g, b, a]
    pub opacity: f32,           // 不透明度 0.0-1.0
    pub overflow: u32,          // 溢出处理: 0=visible, 1=hidden, 2=scroll, 3=auto
    pub transform: [f32; 6],     // 2D变换矩阵 [a, b, c, d, tx, ty] = | a c tx |
                                //                                   | b d ty |
                                //                                   | 0 0 1  |
    pub shadow_color: [f32; 4],  // 阴影颜色 [r, g, b, a]
    pub shadow_offset: [f32; 2], // 阴影偏移 [x, y]
    pub shadow_blur: f32,         // 阴影模糊半径
    pub shadow_spread: f32,       // 阴影扩散
    pub has_shadow: u32,          // 是否有阴影 (0=无, 1=有)
    pub border_radius: [f32; 4],  // 圆角 [top-left, top-right, bottom-right, bottom-left]
    pub visibility: u32,          // 可见性: 0=visible, 1=hidden, 2=collapse
    pub flex_direction: u32,      // flex方向: 0=row, 1=row-reverse, 2=column, 3=column-reverse
    pub flex_wrap: u32,           // flex换行: 0=nowrap, 1=wrap, 2=wrap-reverse
    pub align_items: u32,          // 交叉轴对齐: 0=stretch, 1=flex-start, 2=flex-end, 3=center, 4=baseline
    pub justify_content: u32,     // 主轴对齐: 0=flex-start, 1=flex-end, 2=center, 3=space-between, 4=space-around, 5=space-evenly
    pub align_content: u32,       // 多行对齐: 0=stretch, 1=flex-start, 2=flex-end, 3=center, 4=space-between, 5=space-around
    pub align_self: u32,          // 自身对齐: 0=auto, 1=flex-start, 2=flex-end, 3=center, 4=baseline, 5=stretch
    pub order: i32,               // 排序顺序 (可以是负数)
    pub flex_basis: f32,          // flex基础尺寸
    pub gap: f32,                 // 间距 (row-gap 和 column-gap 相同)
    pub outline_width: f32,       // 轮廓宽度
    pub outline_color: [f32; 4],  // 轮廓颜色 [r, g, b, a]
    pub cursor: u32,              // 光标类型: 0=default, 1=pointer, 2=text, 3=wait, 4=crosshair, 5=move, 6=resize
    pub pointer_events: u32,      // 指针事件: 0=all, 1=none, 2=visiblePainted, 3=visibleFill, 4=visibleStroke, 5=visible, 6=painted, 7=fill, 8=stroke, 9=all
    pub float: u32,              // 浮动: 0=none, 1=left, 2=right
    pub clear: u32,               // 清除浮动: 0=none, 1=left, 2=right, 3=both
    pub overflow_x: u32,          // 水平溢出: 0=visible, 1=hidden, 2=scroll, 3=auto
    pub overflow_y: u32,          // 垂直溢出: 0=visible, 1=hidden, 2=scroll, 3=auto
    pub line_height: f32,         // 行高
    pub letter_spacing: f32,      // 字间距
    pub word_spacing: f32,         // 单词间距
    pub text_indent: f32,          // 文本缩进
    pub white_space: u32,          // 空白处理: 0=normal, 1=nowrap, 2=pre, 3=pre-wrap, 4=pre-line
    pub max_width: f32,            // 最大宽度
    pub min_width: f32,            // 最小宽度
    pub max_height: f32,           // 最大高度
    pub min_height: f32,           // 最小高度
}

impl LayoutItem {
    pub const SIZE: usize = std::mem::size_of::<LayoutItem>();

    pub fn new() -> Self {
        LayoutItem {
            dom_id: 0,
            size: [0.0, 0.0],
            margin: [0.0, 0.0, 0.0, 0.0],
            padding: [0.0, 0.0, 0.0, 0.0],
            pos: [0.0, 0.0],
            size_constraint: [-1.0, -1.0], // -1 表示未设置
            flow_type: 0,  // 默认文档流
            weight: 0.0,
            flex_shrink: 1.0,  // 默认 flex-shrink: 1
            z_index: 0.0,
            tex_idx: 0,
            is_valid: 1,
            is_hide: 0,
            bg_color: [0.0, 0.0, 0.0, 0.0], // 默认透明
            border: [0.0, 0.0, 0.0, 0.0],    // 默认无边框
            border_color: [0.0, 0.0, 0.0, 1.0], // 默认黑色边框
            opacity: 1.0,       // 默认不透明
            overflow: 0,        // 默认 visible
            transform: [1.0, 0.0, 0.0, 1.0, 0.0, 0.0], // 默认单位矩阵（无变换）
            shadow_color: [0.0, 0.0, 0.0, 0.0], // 默认无阴影
            shadow_offset: [0.0, 0.0],
            shadow_blur: 0.0,
            shadow_spread: 0.0,
            has_shadow: 0,
            border_radius: [0.0, 0.0, 0.0, 0.0], // 默认无圆角
            visibility: 0, // 默认 visible
            flex_direction: 0, // 默认 row
            flex_wrap: 0, // 默认 nowrap
            align_items: 0, // 默认 stretch
            justify_content: 0, // 默认 flex-start
            align_content: 0, // 默认 stretch
            align_self: 0, // 默认 auto
            order: 0, // 默认 0
            flex_basis: 0.0, // 默认 0 (auto)
            gap: 0.0, // 默认 0
            outline_width: 0.0, // 默认无轮廓
            outline_color: [0.0, 0.0, 0.0, 0.0], // 默认透明
            cursor: 0, // 默认 default
            pointer_events: 0, // 默认 all
            float: 0, // 默认 none
            clear: 0, // 默认 none
            overflow_x: 0, // 默认 visible
            overflow_y: 0, // 默认 visible
            line_height: 0.0, // 默认 0 (normal)
            letter_spacing: 0.0, // 默认 normal
            word_spacing: 0.0, // 默认 normal
            text_indent: 0.0, // 默认 0
            white_space: 0, // 默认 normal
            max_width: 0.0, // 默认 0 (none)
            min_width: 0.0, // 默认 0
            max_height: 0.0, // 默认 0 (none)
            min_height: 0.0, // 默认 0
        }
    }

    pub fn with_size(mut self, width: f32, height: f32) -> Self {
        self.size = [width, height];
        self
    }

    pub fn with_dom_id(mut self, dom_id: u32) -> Self {
        self.dom_id = dom_id;
        self
    }

    pub fn with_pos(mut self, x: f32, y: f32) -> Self {
        self.pos = [x, y];
        self
    }

    pub fn with_size_constraint(mut self, right: f32, bottom: f32) -> Self {
        self.size_constraint = [right, bottom];
        self
    }

    pub fn with_margin(mut self, top: f32, right: f32, bottom: f32, left: f32) -> Self {
        self.margin = [top, right, bottom, left];
        self
    }

    pub fn with_padding(mut self, top: f32, right: f32, bottom: f32, left: f32) -> Self {
        self.padding = [top, right, bottom, left];
        self
    }

    pub fn with_flow_type(mut self, flow_type: u32) -> Self {
        self.flow_type = flow_type;
        self
    }

    pub fn with_weight(mut self, weight: f32) -> Self {
        self.weight = weight;
        self
    }

    pub fn with_flex_shrink(mut self, flex_shrink: f32) -> Self {
        self.flex_shrink = flex_shrink;
        self
    }

    pub fn with_z_index(mut self, z_index: f32) -> Self {
        self.z_index = z_index;
        self
    }

    pub fn with_texture(mut self, tex_idx: u32) -> Self {
        self.tex_idx = tex_idx;
        self
    }

    pub fn with_bg_color(mut self, r: f32, g: f32, b: f32, a: f32) -> Self {
        self.bg_color = [r, g, b, a];
        self
    }

    pub fn with_border(mut self, top: f32, right: f32, bottom: f32, left: f32) -> Self {
        self.border = [top, right, bottom, left];
        self
    }

    pub fn with_border_color(mut self, r: f32, g: f32, b: f32, a: f32) -> Self {
        self.border_color = [r, g, b, a];
        self
    }

    pub fn with_opacity(mut self, opacity: f32) -> Self {
        self.opacity = opacity.clamp(0.0, 1.0);
        self
    }

    pub fn with_overflow(mut self, overflow: u32) -> Self {
        self.overflow = overflow.clamp(0, 3);
        self
    }

    pub fn with_transform(mut self, a: f32, b: f32, c: f32, d: f32, tx: f32, ty: f32) -> Self {
        self.transform = [a, b, c, d, tx, ty];
        self
    }

    pub fn with_shadow(mut self, color: [f32; 4], offset: [f32; 2], blur: f32, spread: f32) -> Self {
        self.shadow_color = color;
        self.shadow_offset = offset;
        self.shadow_blur = blur;
        self.shadow_spread = spread;
        self.has_shadow = 1;
        self
    }

    pub fn with_shadow_if(mut self, enabled: bool, color: [f32; 4], offset: [f32; 2], blur: f32, spread: f32) -> Self {
        self.shadow_color = color;
        self.shadow_offset = offset;
        self.shadow_blur = blur;
        self.shadow_spread = spread;
        self.has_shadow = if enabled { 1 } else { 0 };
        self
    }

    pub fn with_border_radius(mut self, tl: f32, tr: f32, br: f32, bl: f32) -> Self {
        self.border_radius = [tl, tr, br, bl];
        self
    }

    pub fn with_visibility(mut self, visibility: u32) -> Self {
        self.visibility = visibility.clamp(0, 2);
        self
    }

    pub fn with_flex_direction(mut self, direction: u32) -> Self {
        self.flex_direction = direction.clamp(0, 3);
        self
    }

    pub fn with_flex_wrap(mut self, wrap: u32) -> Self {
        self.flex_wrap = wrap.clamp(0, 2);
        self
    }

    pub fn with_align_items(mut self, align: u32) -> Self {
        self.align_items = align.clamp(0, 4);
        self
    }

    pub fn with_justify_content(mut self, justify: u32) -> Self {
        self.justify_content = justify.clamp(0, 5);
        self
    }

    pub fn with_align_content(mut self, align: u32) -> Self {
        self.align_content = align.clamp(0, 5);
        self
    }

    pub fn with_align_self(mut self, align: u32) -> Self {
        self.align_self = align.clamp(0, 5);
        self
    }

    pub fn with_order(mut self, order: i32) -> Self {
        self.order = order;
        self
    }

    pub fn with_flex_basis(mut self, basis: f32) -> Self {
        self.flex_basis = basis;
        self
    }

    pub fn with_gap(mut self, gap: f32) -> Self {
        self.gap = gap;
        self
    }

    pub fn with_outline_width(mut self, width: f32) -> Self {
        self.outline_width = width;
        self
    }

    pub fn with_outline_color(mut self, r: f32, g: f32, b: f32, a: f32) -> Self {
        self.outline_color = [r, g, b, a];
        self
    }

    pub fn with_cursor(mut self, cursor: u32) -> Self {
        self.cursor = cursor.clamp(0, 6);
        self
    }

    pub fn with_pointer_events(mut self, events: u32) -> Self {
        self.pointer_events = events.clamp(0, 9);
        self
    }

    pub fn with_float(mut self, float: u32) -> Self {
        self.float = float.clamp(0, 2);
        self
    }

    pub fn with_clear(mut self, clear: u32) -> Self {
        self.clear = clear.clamp(0, 3);
        self
    }

    pub fn with_overflow_x(mut self, overflow: u32) -> Self {
        self.overflow_x = overflow.clamp(0, 3);
        self
    }

    pub fn with_overflow_y(mut self, overflow: u32) -> Self {
        self.overflow_y = overflow.clamp(0, 3);
        self
    }

    pub fn with_line_height(mut self, height: f32) -> Self {
        self.line_height = height;
        self
    }

    pub fn with_letter_spacing(mut self, spacing: f32) -> Self {
        self.letter_spacing = spacing;
        self
    }

    pub fn with_word_spacing(mut self, spacing: f32) -> Self {
        self.word_spacing = spacing;
        self
    }

    pub fn with_text_indent(mut self, indent: f32) -> Self {
        self.text_indent = indent;
        self
    }

    pub fn with_white_space(mut self, space: u32) -> Self {
        self.white_space = space.clamp(0, 4);
        self
    }

    pub fn with_max_width(mut self, width: f32) -> Self {
        self.max_width = width;
        self
    }

    pub fn with_min_width(mut self, width: f32) -> Self {
        self.min_width = width;
        self
    }

    pub fn with_max_height(mut self, height: f32) -> Self {
        self.max_height = height;
        self
    }

    pub fn with_min_height(mut self, height: f32) -> Self {
        self.min_height = height;
        self
    }

    pub fn hide(mut self) -> Self {
        self.is_hide = 1;
        self
    }

    pub fn with_hide_if(mut self, hide: bool) -> Self {
        self.is_hide = if hide { 1 } else { 0 };
        self
    }

    pub fn disable(mut self) -> Self {
        self.is_valid = 0;
        self
    }

    pub fn is_visible(&self) -> bool {
        self.is_valid == 1 && self.is_hide == 0
    }

    pub fn width(&self) -> f32 {
        self.size[0]
    }

    pub fn height(&self) -> f32 {
        self.size[1]
    }

    pub fn x(&self) -> f32 {
        self.pos[0]
    }

    pub fn y(&self) -> f32 {
        self.pos[1]
    }

    /// 点击测试 - 检查点是否在元素范围内
    pub fn hit_test(&self, x: f32, y: f32) -> bool {
        // 检查元素是否有效且可见
        if self.is_valid == 0 || self.is_hide == 1 {
            return false;
        }

        // 检查 pointer_events
        if self.pointer_events == 1 {
            // none - 不响应点击
            return false;
        }

        // 检查 visibility
        if self.visibility == 1 {
            // hidden - 不响应点击
            return false;
        }

        // AABB 碰撞检测
        let left = self.pos[0];
        let right = self.pos[0] + self.size[0];
        let top = self.pos[1];
        let bottom = self.pos[1] + self.size[1];

        x >= left && x <= right && y >= top && y <= bottom
    }

    /// 获取元素的边界矩形
    pub fn bounds(&self) -> (f32, f32, f32, f32) {
        (self.pos[0], self.pos[1], self.size[0], self.size[1])
    }
}

impl Default for LayoutItem {
    fn default() -> Self {
        Self::new()
    }
}

/// 布局环境 - Rust 侧表示
#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct LayoutEnv {
    pub view_size: [f32; 2],  // [width, height]
    pub flow_dir: u32,        // 0=垂直, 1=水平
    pub gap: [f32; 2],        // [row_gap, column_gap]
}

impl LayoutEnv {
    pub const SIZE: usize = std::mem::size_of::<LayoutEnv>();

    pub fn new(width: f32, height: f32) -> Self {
        LayoutEnv {
            view_size: [width, height],
            flow_dir: 0,
            gap: [0.0, 0.0],
        }
    }

    pub fn view_width(&self) -> f32 {
        self.view_size[0]
    }

    pub fn view_height(&self) -> f32 {
        self.view_size[1]
    }
}

impl Default for LayoutEnv {
    fn default() -> Self {
        Self::new(800.0, 600.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_layout_item_new() {
        let item = LayoutItem::new();
        assert_eq!(item.width(), 0.0);
        assert_eq!(item.height(), 0.0);
        assert_eq!(item.x(), 0.0);
        assert_eq!(item.y(), 0.0);
        assert!(item.is_visible());
    }

    #[test]
    fn test_layout_item_builders() {
        let item = LayoutItem::new()
            .with_size(100.0, 50.0)
            .with_pos(10.0, 20.0)
            .with_flow_type(1);

        assert_eq!(item.width(), 100.0);
        assert_eq!(item.height(), 50.0);
        assert_eq!(item.x(), 10.0);
        assert_eq!(item.y(), 20.0);
        assert_eq!(item.flow_type, 1);
    }

    #[test]
    fn test_layout_item_visibility() {
        let item = LayoutItem::new();
        assert!(item.is_visible());

        let hidden = item.hide();
        assert!(!hidden.is_visible());

        let disabled = item.disable();
        assert!(!disabled.is_visible());
    }

    #[test]
    fn test_layout_env() {
        let env = LayoutEnv::new(800.0, 600.0);
        assert_eq!(env.view_width(), 800.0);
        assert_eq!(env.view_height(), 600.0);
    }

    #[test]
    fn test_memory_layout() {
        // LayoutItem 大小取决于字段对齐
        // size[2] + margin[4] + padding[4] + pos[2] + size_constraint[2] + flow_type + weight + flex_shrink + z_index + tex_idx + is_valid + is_hide + bg_color[4] + border[4] + border_color[4] + opacity + overflow + transform[6] + shadow_color[4] + shadow_offset[2] + shadow_blur + shadow_spread + has_shadow + border_radius[4] + visibility
        // = 8 + 16 + 16 + 8 + 8 + 4*13 + 4*11 + 4*5 + 4*4 + 4*3 + 4 + 8 + 16 + 24 + 16 + 8 = 272 bytes (实际可能因对齐更大)
        // size[2] + margin[4] + padding[4] + pos[2] + size_constraint[2] + (u32)*13 + (f32)*11 + (f32)*5 + (f32)*4 + (f32)*3 + i32 + bg_color[4] + transform[6] + shadow_color[4]
        assert!(LayoutItem::SIZE >= 272);
        assert_eq!(LayoutEnv::SIZE, 20);
    }

    #[test]
    fn test_hit_test_inside() {
        let item = LayoutItem::new()
            .with_pos(10.0, 20.0)
            .with_size(100.0, 50.0);

        assert!(item.hit_test(50.0, 40.0));  // 中心点
        assert!(item.hit_test(10.0, 20.0));  // 左上角
        assert!(item.hit_test(110.0, 70.0)); // 右下角
    }

    #[test]
    fn test_hit_test_outside() {
        let item = LayoutItem::new()
            .with_pos(10.0, 20.0)
            .with_size(100.0, 50.0);

        assert!(!item.hit_test(5.0, 40.0));   // 左边外
        assert!(!item.hit_test(115.0, 40.0)); // 右边外
        assert!(!item.hit_test(50.0, 15.0));   // 上边外
        assert!(!item.hit_test(50.0, 75.0));   // 下边外
    }

    #[test]
    fn test_hit_test_hidden_item() {
        let item = LayoutItem::new()
            .with_pos(10.0, 20.0)
            .with_size(100.0, 50.0)
            .hide();

        assert!(!item.hit_test(50.0, 40.0)); // 隐藏项不响应点击
    }

    #[test]
    fn test_hit_test_disabled_item() {
        let item = LayoutItem::new()
            .with_pos(10.0, 20.0)
            .with_size(100.0, 50.0)
            .disable();

        assert!(!item.hit_test(50.0, 40.0)); // 禁用项不响应点击
    }

    #[test]
    fn test_hit_test_pointer_events_none() {
        let item = LayoutItem::new()
            .with_pos(10.0, 20.0)
            .with_size(100.0, 50.0)
            .with_pointer_events(1); // pointer-events: none

        assert!(!item.hit_test(50.0, 40.0)); // 不响应点击
    }

    #[test]
    fn test_hit_test_visibility_hidden() {
        let item = LayoutItem::new()
            .with_pos(10.0, 20.0)
            .with_size(100.0, 50.0)
            .with_visibility(1); // visibility: hidden

        assert!(!item.hit_test(50.0, 40.0)); // 不响应点击
    }

    #[test]
    fn test_bounds() {
        let item = LayoutItem::new()
            .with_pos(10.0, 20.0)
            .with_size(100.0, 50.0);

        let (x, y, w, h) = item.bounds();
        assert_eq!(x, 10.0);
        assert_eq!(y, 20.0);
        assert_eq!(w, 100.0);
        assert_eq!(h, 50.0);
    }
}