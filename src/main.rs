use chunk_writer::Image;

fn main() {
    let mut colour_image = image::ColorImage::new(256, 256, image::RGB { red: 0, green: 0, blue: 0 });
    for i in 0..256 {
        for j in 0..256 {
            colour_image.set_pixel(i, j, image::RGB { red: ((i / 25) * 25) as u8, green: 0, blue: ((j / 25) * 25) as u8 });
        }
    }

    println!("encoding...");
    let writer = chunk_writer::gif::GifImage { image: colour_image };
    writer.write("test.gif").unwrap();
}
