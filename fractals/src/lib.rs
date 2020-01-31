use std::collections::HashSet;
use image::ColorImage;
use image::RGB;

#[derive(Debug,Clone,Copy,Hash,PartialEq,Eq)]
struct Coord {
    pub x: i32,
    pub y: i32,
}
impl Coord {
    fn new(x: i32, y: i32) -> Self {
        Coord { x, y }
    }
}
struct Triangle {
    a: Coord,
    b: Coord,
    c: Coord,
}
pub fn sierpinski_triangle(image: &mut ColorImage, iterations: usize, width: u32, height: u32, triangle_color: RGB, background_color: RGB) {
    let width = width as i32;
    let height = height as i32;
    let initial_triangle = Triangle {
        a: Coord::new(width / 2, height),
        b: Coord::new(0, 0),
        c: Coord::new(width, 0),
    };
    draw_triangle(image, initial_triangle.a, initial_triangle.b, initial_triangle.c, triangle_color);
    sierpinski_triangle_rec(image, initial_triangle, iterations, background_color);
}
fn sierpinski_triangle_rec(image: &mut ColorImage, triangle: Triangle, iterations: usize, remove_color: RGB) {
    match iterations {
        0 => (),
        _ => {
            let line_midpoints = [
                Coord::new((triangle.a.x + triangle.b.x) / 2, (triangle.a.y + triangle.b.y) / 2),
                Coord::new((triangle.b.x + triangle.c.x) / 2, (triangle.b.y + triangle.c.y) / 2),
                Coord::new((triangle.c.x + triangle.a.x) / 2, (triangle.c.y + triangle.a.y) / 2),
            ];
            draw_triangle(image, line_midpoints[0], line_midpoints[1], line_midpoints[2], remove_color);
            let rec_triangles = vec![
                Triangle { a: triangle.a, b: line_midpoints[0], c: line_midpoints[2] },
                Triangle { a: triangle.b, b: line_midpoints[0], c: line_midpoints[1] },
                Triangle { a: triangle.c, b: line_midpoints[1], c: line_midpoints[2] },
            ];
            for triangle in rec_triangles {
                sierpinski_triangle_rec(image, triangle, iterations - 1, remove_color);
            }
        },
    }
}
pub fn draw_test() -> ColorImage {
    let mut image = ColorImage::new(900, 900, RGB { red: 0, green: 0, blue: 0 });
    draw_triangle(&mut image, Coord::new(200, 200), Coord::new(600, 350), Coord::new(400, 600), RGB { red: 180, green: 0, blue: 120 });
    draw_triangle(&mut image, Coord::new(250, 250), Coord::new(550, 300), Coord::new(350, 550), RGB { red: 0, green: 50, blue: 220 });
    image
}
fn get_line_coord_set(set: &mut HashSet<Coord>, a: Coord, b: Coord) {
    let dist_x = b.x - a.x;
    let dist_y = b.y - a.y;
    let line_length = {
        let dist_x = if dist_x < 0 { -dist_x } else { dist_x };
        let dist_y = if dist_y < 0 { -dist_y } else { dist_y };
        if dist_x > dist_y { dist_x } else { dist_y }
    };
    let delta_x = dist_x as f64 / line_length as f64;
    let delta_y = dist_y as f64 / line_length as f64;
    for i in 0..=line_length {
        let i = i as f64;
        set.insert(Coord::new(a.x + (delta_x * i) as i32, a.y + (delta_y * i) as i32));
    }
}
fn get_triangle_coord_set(set: &mut HashSet<Coord>, triangle: &Triangle) {
    get_line_coord_set(set, triangle.a, triangle.b);
    get_line_coord_set(set, triangle.a, triangle.c);
    get_line_coord_set(set, triangle.b, triangle.c);
    let initial_point = get_triangle_midpoint(&triangle);
    flood_fill_coord_set(set, initial_point);
}
fn get_triangle_midpoint(triangle: &Triangle) -> Coord {
    let line_midpoints = (
        ((triangle.a.x + triangle.b.x) as f64 / 2.0, (triangle.a.y + triangle.b.y) as f64 / 2.0),
        ((triangle.a.x + triangle.c.x) as f64 / 2.0, (triangle.a.y + triangle.c.y) as f64 / 2.0),
    );
    let x = f64::round(((line_midpoints.0).0 + (line_midpoints.1).0) / 2.0);
    let y = f64::round(((line_midpoints.0).1 + (line_midpoints.1).1) / 2.0);
    Coord { x: x as i32, y: y as i32 }
}
fn flood_fill_coord_set(set: &mut HashSet<Coord>, initial_point: Coord) {
    let mut coords = vec![initial_point];
    while let Some(coord) = coords.pop() {
        let surrounding_coords = vec![
            Coord::new(coord.x + 1, coord.y),
            Coord::new(coord.x - 1, coord.y),
            Coord::new(coord.x, coord.y + 1),
            Coord::new(coord.x, coord.y - 1),
        ];
        for coord in surrounding_coords {
            if set.insert(coord) {
                coords.push(coord);
            }
        }
    }
}
fn draw_line(image: &mut ColorImage, a: Coord, b: Coord, color: RGB) {
    let mut set = HashSet::new();
    get_line_coord_set(&mut set, a, b);
    set_image_pixels_from_coords(image, &mut set, color);
}
fn draw_triangle(image: &mut ColorImage, a: Coord, b: Coord, c: Coord, color: RGB) {
    let mut set = HashSet::new();
    get_triangle_coord_set(&mut set, &Triangle { a, b, c });
    set_image_pixels_from_coords(image, &mut set, color);
}
fn set_image_pixels_from_coords(image: &mut ColorImage, set: &HashSet<Coord>, color: RGB) {
    let image_width = image.canvas.width as i32;
    let image_height = image.canvas.height as i32;
    for coord in set.iter() {
        if coord.x >= 0 && coord.x < image_width && coord.y >= 0 && coord.y < image_height {
            image.set_pixel(coord.x as u32, coord.y as u32, color);
        }
    }
}