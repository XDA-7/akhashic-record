use chunk_writer::Image;

fn main() {
    let colour_image = image::ColorImage::new(300, 150, image::RGB { red: 0, green: 0, blue: 0 });
    let writer = chunk_writer::gif::GifImage { image: colour_image };
    writer.write("test.gif").unwrap();
}
