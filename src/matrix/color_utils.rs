use embedded_graphics::{
    pixelcolor::{raw::ToBytes, Rgb888},
    prelude::RgbColor,
};
use rand::Rng;

pub fn generate_table() -> Vec<Rgb888> {
    let mut rng = rand::thread_rng();
    let amount = rng.gen_range(2..6);

    let colors = vec![0; amount]
        .into_iter()
        .map(|_| Rgb888::new(rand::random(), rand::random(), rand::random()))
        .collect::<Vec<_>>();

    let gradient_len = 95;
    let mut image = vec![0; 95 * 3];

    for x in 0..gradient_len {
        let t = x as f32 / (gradient_len - 1) as f32;
        let color = interpolate_colors(&colors, t);

        image[x * 3..x * 3 + 3].copy_from_slice(&color.to_be_bytes());
    }

    let mut table = vec![0; 3 * 2 * gradient_len];

    table[..95 * 3].copy_from_slice(&image);
    reverse_rgb_data(&mut image, 3);
    table[95 * 3..].copy_from_slice(&image);

    let rgb = table
        .chunks_exact(3)
        .map(|chunk| Rgb888::new(chunk[0], chunk[1], chunk[2]))
        .collect::<Vec<_>>();

    rgb
}

fn reverse_rgb_data(data: &mut Vec<u8>, bytes_per_pixel: usize) {
    // Ensure that the data length is a multiple of bytes_per_pixel
    if data.len() % bytes_per_pixel != 0 {
        panic!("Invalid data length");
    }

    let num_pixels = data.len() / bytes_per_pixel;

    for i in 0..num_pixels / 2 {
        let start_idx = i * bytes_per_pixel;
        let end_idx = (num_pixels - i - 1) * bytes_per_pixel;

        // Swap the RGB triples
        for j in 0..bytes_per_pixel {
            data.swap(start_idx + j, end_idx + j);
        }
    }
}

fn interpolate_colors(colors: &[Rgb888], t: f32) -> Rgb888 {
    let segment = t * (colors.len() - 1) as f32;
    let index = segment.floor() as usize;
    let t_segment = segment - index as f32;
    let color1 = colors[index];
    let color2 = colors[(index + 1) % colors.len()];
    let r = lerp(color1.r(), color2.r(), t_segment);
    let g = lerp(color1.g(), color2.g(), t_segment);
    let b = lerp(color1.b(), color2.b(), t_segment);

    Rgb888::new(r, g, b)
}

fn lerp(a: u8, b: u8, t: f32) -> u8 {
    ((1.0 - t) * f32::from(a) + t * f32::from(b)) as u8
}
