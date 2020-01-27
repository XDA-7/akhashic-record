#[derive(Clone,Copy,Default,PartialEq,Eq,Hash,Debug)]
pub struct RGB {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

pub struct Canvas<T> {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<T>,
}
impl<T> Canvas<T> where T : Default + Clone {
    pub fn new(width: u32, height: u32) -> Self {
        Canvas {
            width,
            height,
            pixels: vec![T::default(); (height * width) as usize],
        }
    }
    pub fn pixel(&self, x: u32, y: u32) -> &T {
        &self.pixels[(y * self.width + x) as usize]
    }
    pub fn set_pixel(&mut self, x: u32, y: u32, value: T) {
        self.pixels[(y * self.width + x) as usize] = value;
    }
}

struct GreyScaleImage {
    canvas: Canvas<u8>,
}
impl GreyScaleImage {
    fn new(width: u32, height: u32) -> Self {
        GreyScaleImage {
            canvas: Canvas::new(width, height),
        }
    }
    fn pixel(&self, x: u32, y: u32) -> u8 {
        *self.canvas.pixel(x, y)
    }
    fn set_pixel(&mut self, x: u32, y: u32, value: u8) {
        self.canvas.set_pixel(x, y, value);
    }
}

pub struct Palette {
    color_index_map: std::collections::HashMap<RGB,usize>,
    index_color_map: std::collections::HashMap<usize,RGB>,
}
impl Palette {
    fn new() -> Self {
        Palette {
            color_index_map: std::collections::HashMap::new(),
            index_color_map: std::collections::HashMap::new(),
        }
    }
    fn index(&mut self, rgb: RGB) -> usize {
        match self.color_index_map.get(&rgb) {
            Some(idx) => *idx,
            None => {
                let idx = self.color_index_map.len();
                self.color_index_map.insert(rgb, idx);
                self.index_color_map.insert(idx, rgb);
                idx
            }
        }
    }
    pub fn color(&self, index: usize) -> Option<&RGB> {
        self.index_color_map.get(&index)
    }
}

pub struct ColorImage {
    pub palette: Palette,
    pub canvas: Canvas<usize>,
}
impl ColorImage {
    pub fn new(width: u32, height: u32, initial_color: RGB) -> Self {
        let mut palette = Palette::new();
        palette.index(initial_color);
        ColorImage {
            palette,
            canvas: Canvas::new(width, height),
        }
    }
    pub fn set_pixel(&mut self, x: u32, y: u32, value: RGB) {
        self.canvas.set_pixel(x, y, self.palette.index(value));
    }
    pub fn pixel(&self, x: u32, y: u32) -> RGB {
        let index = self.canvas.pixel(x, y);
        *self.palette.color(*index).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn canvas_remembers_values() {
        let mut canvas = Canvas::new(60, 60);
        canvas.set_pixel(13, 25, RGB { red: 20, green: 50, blue: 35 });
        assert_eq!(canvas.pixel(13, 25), &RGB { red: 20, green: 50, blue: 35 });
    }
    #[test]
    fn canvas_uses_default_values() {
        let canvas: Canvas<u8> = Canvas::new(60, 60);
        assert_eq!(canvas.pixel(13, 25), &0);
    }
    #[test]
    fn canvas_does_not_overlap_values() {
        let mut canvas = Canvas::new(60, 90);
        for i in 0..60 {
            for j in 0..90 {
                canvas.set_pixel(i, j, RGB {red: i as u8, green: j as u8, blue: 250});
            }
        }
        for i in 0..60 {
            for j in 0..90 {
                assert_eq!(canvas.pixel(i, j), &RGB {red: i as u8, green: j as u8, blue: 250});
            }
        }
    }

    #[test]
    fn palette_returns_same_index_for_same_rgb() {
        let mut palette = Palette::new();
        let colors = [
            RGB {red: 5, green: 2, blue: 3},
            RGB {red: 6, green: 2, blue: 30},
            RGB {red: 36, green: 21, blue: 11},
            RGB {red: 6, green: 2, blue: 30},
            RGB {red: 36, green: 21, blue: 11}
        ];
        let indexes: Vec<usize> = colors.into_iter().map(|color| palette.index(*color)).collect();
        assert_ne!(indexes[0],indexes[1]);
        assert_ne!(indexes[0],indexes[2]);
        assert_ne!(indexes[1],indexes[2]);
        assert_eq!(indexes[1],indexes[3]);
        assert_eq!(indexes[2],indexes[4]);
    }
    #[test]
    fn palette_returns_correct_color_for_index() {
        let mut palette = Palette::new();
        let colors = [
            RGB {red: 5, green: 2, blue: 3},
            RGB {red: 6, green: 2, blue: 30},
            RGB {red: 36, green: 21, blue: 11},
            RGB {red: 6, green: 2, blue: 30},
            RGB {red: 36, green: 21, blue: 11}
        ];
        let indexes: Vec<usize> = colors.into_iter().map(|color| palette.index(*color)).collect();
        let returned_colors: Vec<RGB> = indexes.into_iter().map(|index| *palette.color(index).unwrap()).collect();
        for i in 0..5 {
            assert_eq!(colors[i], returned_colors[i]);
        }
    }

    #[test]
    fn color_image_uses_default_color() {
        let image = ColorImage::new(60, 90, RGB { red: 60, green: 30, blue: 45});
        for i in 0..60 {
            for j in 0..90 {
                assert_eq!(image.pixel(i, j), RGB { red: 60, green: 30, blue: 45});
            }
        }
    }
    #[test]
    fn color_image_remembers_colors_correctly() {
        let mut image = ColorImage::new(60, 90, RGB { red: 0, green: 0, blue: 0 });
        for i in 0..60 {
            for j in 0..90 {
                image.set_pixel(i, j, RGB { red: (i % 20) as u8, green: (j % 10) as u8, blue: 12 });
            }
        }
        for i in 0..60 {
            for j in 0..90 {
                assert_eq!(image.pixel(i, j), RGB { red: (i % 20) as u8, green: (j % 10) as u8, blue: 12 });
            }
        }
    }
}
