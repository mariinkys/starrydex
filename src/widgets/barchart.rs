// SPDX-License-Identifier: GPL-3.0-only

use cosmic::{
    iced::{
        Background, Border, Color, Element, Length, Padding, Point, Rectangle, Size, Vector,
        alignment::{Horizontal, Vertical},
        event::{self, Event},
        mouse, overlay,
    },
    iced_core::{
        Clipboard, Layout, Renderer as IcedRenderer, Shell, layout, renderer, text::Renderer,
        widget::Tree,
    },
    widget::{Operation, Widget},
};
use std::f32;

/// A bar chart widget that displays data as vertical bars
#[must_use]
pub struct BarChart<'a, Message> {
    /// The data points for the chart
    columns: Vec<ColumnData>,
    /// Sets the padding around the widget
    padding: Padding,
    /// Sets the space between each column
    column_spacing: f32,
    /// Sets the space between labels and bars
    row_spacing: f32,
    /// Sets the width of the chart
    width: Length,
    /// Sets the height of the chart
    height: Length,
    /// Sets the max width
    max_width: f32,
    /// Color scheme for bars
    bar_colors: Vec<Color>,
    /// Show values on top of bars
    show_values: bool,
    /// Show labels below bars
    show_labels: bool,
    /// Minimum bar height (for visual purposes)
    min_bar_height: f32,
    /// Chart title
    title: Option<String>,
    _phantom: std::marker::PhantomData<&'a Message>,
}

#[derive(Clone, Debug)]
struct ColumnData {
    label: String,
    value: f32,
}

impl<'a, Message> Default for BarChart<'a, Message> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a, Message> BarChart<'a, Message> {
    pub const fn new() -> Self {
        Self {
            columns: Vec::new(),
            padding: Padding::new(20.0),
            column_spacing: 10.0,
            row_spacing: 5.0,
            width: Length::Fill,
            height: Length::Fixed(300.0),
            max_width: f32::INFINITY,
            bar_colors: Vec::new(),
            show_values: true,
            show_labels: true,
            min_bar_height: 5.0,
            title: None,
            _phantom: std::marker::PhantomData,
        }
    }

    /// Add a column to the bar chart
    pub fn push(mut self, label: impl Into<String>, value: f32) -> Self {
        self.columns.push(ColumnData {
            label: label.into(),
            value,
        });
        self
    }

    /// Set the padding around the widget
    pub fn padding<P: Into<Padding>>(mut self, padding: P) -> Self {
        self.padding = padding.into();
        self
    }

    /// Set the spacing between columns
    pub fn column_spacing(mut self, spacing: f32) -> Self {
        self.column_spacing = spacing;
        self
    }

    /// Set the spacing between labels and bars
    pub fn row_spacing(mut self, spacing: f32) -> Self {
        self.row_spacing = spacing;
        self
    }

    /// Set the width of the chart
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    /// Set the height of the chart
    pub fn height(mut self, height: impl Into<Length>) -> Self {
        self.height = height.into();
        self
    }

    /// Set the maximum width
    pub fn max_width(mut self, max_width: f32) -> Self {
        self.max_width = max_width;
        self
    }

    /// Set custom colors for bars
    pub fn bar_colors(mut self, colors: Vec<Color>) -> Self {
        self.bar_colors = colors;
        self
    }

    /// Show/hide values on top of bars
    pub fn show_values(mut self, show: bool) -> Self {
        self.show_values = show;
        self
    }

    /// Show/hide labels below bars
    pub fn show_labels(mut self, show: bool) -> Self {
        self.show_labels = show;
        self
    }

    /// Set minimum bar height for visual purposes
    pub fn min_bar_height(mut self, height: f32) -> Self {
        self.min_bar_height = height;
        self
    }

    /// Set chart title
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Get default color for a bar at given index
    fn get_bar_color(&self, index: usize) -> Color {
        if !self.bar_colors.is_empty() {
            self.bar_colors[index % self.bar_colors.len()]
        } else {
            // Default color palette
            let default_colors = [
                Color::from_rgb8(160, 200, 120), // light green (hp)
                Color::from_rgb8(229, 80, 80),   // light red (attack)
                Color::from_rgb8(145, 200, 228), // light blue (defense)
                Color::from_rgb8(255, 144, 187), // light pink (spa)
                Color::from_rgb8(165, 148, 249), // light purple (spdef)
                Color::from_rgb8(255, 235, 85),  // light yellow (spd)
            ];
            default_colors[index % default_colors.len()]
        }
    }
}

impl<'a, Message: 'static + Clone> Widget<Message, cosmic::Theme, cosmic::Renderer>
    for BarChart<'a, Message>
{
    fn children(&self) -> Vec<Tree> {
        Vec::new()
    }

    fn diff(&mut self, _tree: &mut Tree) {
        // No children to diff
    }

    fn size(&self) -> Size<Length> {
        Size::new(self.width, self.height)
    }

    fn layout(
        &self,
        _tree: &mut Tree,
        _renderer: &cosmic::Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let size = self.size();
        let limits = limits
            .max_width(self.max_width)
            .width(size.width)
            .height(size.height);

        let size = limits.resolve(size.width, size.height, Size::ZERO);
        layout::Node::new(size)
    }

    fn operate(
        &self,
        _tree: &mut Tree,
        layout: Layout<'_>,
        _renderer: &cosmic::Renderer,
        operation: &mut dyn Operation<()>,
    ) {
        operation.container(None, layout.bounds(), &mut |_operation| {});
    }

    fn on_event(
        &mut self,
        _tree: &mut Tree,
        _event: Event,
        _layout: Layout<'_>,
        _cursor: mouse::Cursor,
        _renderer: &cosmic::Renderer,
        _clipboard: &mut dyn Clipboard,
        _shell: &mut Shell<'_, Message>,
        _viewport: &Rectangle,
    ) -> event::Status {
        event::Status::Ignored
    }

    fn mouse_interaction(
        &self,
        _tree: &Tree,
        _layout: Layout<'_>,
        _cursor: mouse::Cursor,
        _viewport: &Rectangle,
        _renderer: &cosmic::Renderer,
    ) -> mouse::Interaction {
        mouse::Interaction::default()
    }

    fn draw(
        &self,
        _tree: &Tree,
        renderer: &mut cosmic::Renderer,
        theme: &cosmic::Theme,
        _style: &renderer::Style,
        layout: Layout<'_>,
        _cursor: mouse::Cursor,
        _viewport: &Rectangle,
    ) {
        if self.columns.is_empty() {
            return;
        }

        let bounds = layout.bounds();
        let content_bounds = Rectangle {
            x: bounds.x + self.padding.left,
            y: bounds.y + self.padding.top,
            width: bounds.width - self.padding.horizontal(),
            height: bounds.height - self.padding.vertical(),
        };

        let title_height = if self.title.is_some() { 30.0 } else { 0.0 };
        let label_height = if self.show_labels { 25.0 } else { 0.0 };
        let value_height = if self.show_values { 20.0 } else { 0.0 };

        let chart_height = content_bounds.height
            - title_height
            - label_height
            - value_height
            - self.row_spacing * 2.0;
        let chart_y = content_bounds.y + title_height + value_height + self.row_spacing;

        // find max value for scaling
        let max_value = self
            .columns
            .iter()
            .map(|col| col.value)
            .fold(0.0f32, f32::max)
            .max(1.0);

        // column width
        let total_spacing = self.column_spacing * (self.columns.len() - 1) as f32;
        let available_width = content_bounds.width - total_spacing;
        let column_width = available_width / self.columns.len() as f32;

        // Draw title
        if let Some(ref title) = self.title {
            renderer.fill_text(
                cosmic::iced_core::text::Text {
                    content: title.to_owned(),
                    bounds: Size::new(content_bounds.width, title_height),
                    size: cosmic::iced::Pixels(16.0),
                    line_height: cosmic::iced_core::text::LineHeight::default(),
                    font: cosmic::font::Font::default(),
                    horizontal_alignment: Horizontal::Center,
                    vertical_alignment: Vertical::Center,
                    shaping: cosmic::iced::advanced::text::Shaping::Advanced,
                    wrapping: cosmic::iced_core::text::Wrapping::Word,
                },
                Point::new(content_bounds.x, content_bounds.y),
                theme.cosmic().accent_color().into(),
                bounds,
            );
        }

        // draw bars, values, and labels
        for (i, column) in self.columns.iter().enumerate() {
            let x = content_bounds.x + i as f32 * (column_width + self.column_spacing);

            // Calculate bar height (ensure minimum height for visual purposes)
            let normalized_height = (column.value / max_value) * chart_height;
            let bar_height = normalized_height.max(self.min_bar_height);
            let bar_y = chart_y + chart_height - bar_height;

            // bar
            let bar_rect = Rectangle {
                x,
                y: bar_y,
                width: column_width,
                height: bar_height,
            };

            renderer.fill_quad(
                cosmic::iced::advanced::renderer::Quad {
                    bounds: bar_rect,
                    border: Border::default().rounded(2.0),
                    shadow: cosmic::iced::Shadow::default(),
                },
                Background::Color(self.get_bar_color(i)),
            );

            // value on top of bar
            if self.show_values {
                let value_text = format!("{:.1}", column.value);
                renderer.fill_text(
                    cosmic::iced_core::text::Text {
                        content: value_text,
                        bounds: Size::new(column_width, value_height),
                        size: cosmic::iced::Pixels(12.0),
                        line_height: cosmic::iced_core::text::LineHeight::default(),
                        font: cosmic::font::Font::default(),
                        horizontal_alignment: Horizontal::Center,
                        vertical_alignment: Vertical::Center,
                        shaping: cosmic::iced::advanced::text::Shaping::Advanced,
                        wrapping: cosmic::iced_core::text::Wrapping::Word,
                    },
                    Point::new(x, bar_y - value_height - 2.0),
                    theme.cosmic().on_bg_color().into(),
                    bounds,
                );
            }

            // label below chart
            if self.show_labels {
                renderer.fill_text(
                    cosmic::iced_core::text::Text {
                        content: column.label.clone(),
                        bounds: Size::new(column_width, label_height),
                        size: cosmic::iced::Pixels(11.0),
                        line_height: cosmic::iced_core::text::LineHeight::default(),
                        font: cosmic::font::Font::default(),
                        horizontal_alignment: Horizontal::Center,
                        vertical_alignment: Vertical::Center,
                        shaping: cosmic::iced::advanced::text::Shaping::Advanced,
                        wrapping: cosmic::iced_core::text::Wrapping::Word,
                    },
                    Point::new(x, chart_y + chart_height + self.row_spacing),
                    theme.cosmic().on_bg_color().into(),
                    bounds,
                );
            }
        }
    }

    fn overlay<'b>(
        &'b mut self,
        _tree: &'b mut Tree,
        _layout: Layout<'_>,
        _renderer: &cosmic::Renderer,
        _translation: Vector,
    ) -> Option<overlay::Element<'b, Message, cosmic::Theme, cosmic::Renderer>> {
        None
    }
}

impl<'a, Message: 'static + Clone> From<BarChart<'a, Message>>
    for Element<'a, Message, cosmic::Theme, cosmic::Renderer>
{
    fn from(bar_chart: BarChart<'a, Message>) -> Self {
        Self::new(bar_chart)
    }
}
