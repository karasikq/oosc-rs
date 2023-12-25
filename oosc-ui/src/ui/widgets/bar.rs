use ratatui::widgets::Widget;

pub struct Bar {
    pub y_bounds: (f32, f32),
    pub y_from: f32,
    pub value: f32,
}

impl Bar {
    pub fn get_point(&self, resolution: f32, value: f32) -> Option<usize> {
        let top = self.y_bounds.1;
        let bottom = self.y_bounds.0;
        if value < bottom || value > top {
            return None;
        }
        let height = (self.y_bounds.1 - self.y_bounds.0).abs();
        if height == 0.0 {
            return None;
        }
        let y = ((top - value) * (resolution) / height) as usize;
        Some(y)
    }
}

impl Widget for Bar {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer) {
        let y1 = self.get_point(area.height as f32 - 1.0, self.value);
        let y2 = self.get_point(area.height as f32 - 1.0, self.y_from);
        if y1.is_none() || y2.is_none() {
            return;
        }
        let y1 = y1.unwrap();
        let y2 = y2.unwrap();
        let (dy, y_range) = if y2 >= y1 {
            (y2 - y1, y1..=y2)
        } else {
            (y1 - y2, y2..=y1)
        };
        for x in 0..area.width {
            y_range.clone().for_each(|y| {
                let (x, y) = (x + area.x, y as u16 + area.y);
                buf.get_mut(x, y).set_char('#');
            });
        }
    }
}
