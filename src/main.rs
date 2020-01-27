use chunk_writer::Image;

fn main() {
    let mut colour_image = image::ColorImage::new(300, 300, image::RGB { red: 0, green: 0, blue: 0 });
    for i in 0..300 {
        for j in 0..300 {
            colour_image.set_pixel(i, j, image::RGB { red: 120, green: 0, blue: 150 });
        }
    }
    let writer = chunk_writer::gif::GifImage { image: colour_image };
    writer.write("test.gif").unwrap();
}
