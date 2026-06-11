use clap::Parser;
use crossterm::style::{Color, ResetColor, SetForegroundColor};
use image::{codecs::gif::GifEncoder, Delay, Frame, Rgba, RgbaImage};
use std::collections::HashMap;
use std::fs::File;
use std::io::{stdout, BufRead, BufReader, Write};
use crossterm::ExecutableCommand;


#[derive(Parser)]
#[command(name = "ua-vertical-bars")]
#[command(about = "Colorful vertical frequency bars for Zcash UA (alphabetical) with GIF export")]
struct Args {
    /// Single Zcash Unified Address
    ua: Option<String>,

    /// File containing multiple UAs (one per line)
    #[arg(long)]
    file: Option<String>,

    /// Generate animated GIF (requires --file with multiple UAs)
    #[arg(long)]
    gif: Option<String>,

    #[arg(long)]
    demo: bool,
}

fn hsv_to_rgb(h: f32, s: f32, v: f32) -> (u8, u8, u8) {
    let c = v * s;
    let h = h / 60.0;
    let x = c * (1.0 - ((h % 2.0) - 1.0).abs());
    let m = v - c;

    let (r, g, b) = match h as u32 {
        0 => (c, x, 0.0),
        1 => (x, c, 0.0),
        2 => (0.0, c, x),
        3 => (0.0, x, c),
        4 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };
    (
        ((r + m) * 255.0) as u8,
        ((g + m) * 255.0) as u8,
        ((b + m) * 255.0) as u8,
    )
}

fn get_gradient_color(count: usize, max_count: usize, level: usize) -> Color {
    if max_count == 0 {
        return Color::Reset;
    }
    let ratio = (count as f32 - 1.0) / (max_count as f32 - 1.0);
    let ratio = ratio.clamp(0.0, 1.0);
    let hue = ratio * 300.0;
    let brightness = 0.55 + (level as f32 / max_count as f32) * 0.45;
    let (r, g, b) = hsv_to_rgb(hue, 0.9, brightness);
    Color::Rgb { r, g, b }
}

fn print_histogram(ua: &str) {
    let ua = ua.to_lowercase();
    let mut freq: HashMap<char, usize> = HashMap::new();
    for c in ua.chars() {
        *freq.entry(c).or_insert(0) += 1;
    }

    let mut items: Vec<(char, usize)> = freq.into_iter().collect();
    items.sort_by_key(|(c, _)| *c);

    let max_count = items.iter().map(|(_, c)| *c).max().unwrap_or(0);
    let unique = items.len();
    let total = ua.len();
    let avg = total as f32 / unique as f32;

    println!("Max: {max_count} | Avg: {avg:.2} | Total unique: {unique}\n");

    let mut stdout = stdout();

    for level in (1..=max_count).rev() {
        write!(stdout, "{level:2} | ").unwrap();
        for &(_, count) in &items {
            if count >= level {
                let color = get_gradient_color(count, max_count, level);
                stdout.execute(SetForegroundColor(color)).unwrap();
                write!(stdout, "█ ").unwrap();
                stdout.execute(ResetColor).unwrap();
            } else {
                write!(stdout, "  ").unwrap();
            }
        }
        writeln!(stdout).unwrap();
    }

    write!(stdout, "   +-").unwrap();
    for _ in &items {
        write!(stdout, "--").unwrap();
    }
    writeln!(stdout).unwrap();

    write!(stdout, "    ").unwrap();
    for (c, _) in &items {
        write!(stdout, "{c} ").unwrap();
    }
    writeln!(stdout).unwrap();
    println!();
}

fn create_frame(items: &[(char, usize)], max_count: usize, width: u32, height: u32) -> RgbaImage {
    let mut img = RgbaImage::new(width, height);
    let num_bars = items.len() as u32;
    let margin_x = 50u32;
    let margin_top = 50u32;
    let bar_spacing = 5u32;
    let available_width = width.saturating_sub(margin_x * 2);
    let bar_width = (available_width / num_bars).saturating_sub(bar_spacing).max(10);

    for (i, &(_, count)) in items.iter().enumerate() {
        let x_start = margin_x + (i as u32 * (bar_width + bar_spacing));
        let bar_height = ((count as f32 / max_count as f32) * (height as f32 - margin_top as f32 - 60.0)) as u32;

        for y in 0..bar_height {
            let level = ((y as f32 / bar_height as f32) * max_count as f32) as usize + 1;
            let color = get_gradient_color(count, max_count, level);
            let rgb = match color {
                Color::Rgb { r, g, b } => [r, g, b, 255],
                _ => [100, 150, 255, 255],
            };

            for x in x_start..(x_start + bar_width).min(width) {
                if x < width && (height - 1 - y - 40) < height {
                    img.put_pixel(x, height - 1 - y - 40, Rgba(rgb));
                }
            }
        }
    }
    img
}

fn main() {
    let args = Args::parse();

    let uas: Vec<String> = if let Some(path) = args.file {
        let file = File::open(path).expect("Could not open UA file");
        BufReader::new(file)
            .lines()
            .filter_map(|line| line.ok())
            .filter(|line| !line.trim().is_empty())
            .collect()
    } else if args.demo || args.ua.is_none() {
        vec!["u19vfxe6q07psxewllvc78yuaytxxukwjjh5nwlhr7wj3jfzpg8xxfz8df78s6ghevex7uhvv2lc8pge7yddv5580yhye6tcwrvtwzvn25x92k9cdu6xjc2sha68jr6re7ws7dqpjhuq95uyhl2c0jw0umcxdm6m2ks5z5csagewaruy4rt97yluad728ft8ynw8g5xlnvr2gycs50cmq".to_string()]
    } else {
        vec![args.ua.unwrap()]
    };

    // Print histograms to terminal
    for (i, ua) in uas.iter().enumerate() {
        if uas.len() > 1 {
            println!("=== UA #{}/{} ===", i + 1, uas.len());
        }
        print_histogram(ua);
    }

    // Generate animated GIF
    if let Some(gif_path) = args.gif {
        if uas.len() < 2 {
            eprintln!("Error: --gif requires multiple UAs (use --file with more than one address)");
            return;
        }

        println!("Generating animated GIF with {} frames...", uas.len());

        // Calculate consistent dimensions
        let max_unique_chars = uas.iter()
            .map(|ua| {
                let mut freq: HashMap<char, usize> = HashMap::new();
                for c in ua.to_lowercase().chars() {
                    *freq.entry(c).or_insert(0) += 1;
                }
                freq.len()
            })
            .max()
            .unwrap_or(20);

        let width = (max_unique_chars as u32 * 30 + 140).max(1000);
        let height = 720u32;

        let mut frames = Vec::new();

        for (i, ua) in uas.iter().enumerate() {
            let ua_lower = ua.to_lowercase();
            let mut freq: HashMap<char, usize> = HashMap::new();
            for c in ua_lower.chars() {
                *freq.entry(c).or_insert(0) += 1;
            }
            let mut items: Vec<(char, usize)> = freq.into_iter().collect();
            items.sort_by_key(|(c, _)| *c);

            let max_count = items.iter().map(|(_, c)| *c).max().unwrap_or(1);
            let frame = create_frame(&items, max_count, width, height);

            frames.push(Frame::from_parts(
                frame,
                0,
                0,
                Delay::from_numer_denom_ms(700, 1),
            ));

            print!("\rProgress: {}/{}", i + 1, uas.len());
            std::io::stdout().flush().unwrap();
        }
        println!();

        let file = File::create(&gif_path).expect("Failed to create GIF file");
        let mut encoder = GifEncoder::new(file);
        encoder.set_repeat(image::codecs::gif::Repeat::Infinite).unwrap();
        encoder.encode_frames(frames).expect("Failed to encode GIF");

        println!("Animated GIF saved to: {}", gif_path);
    }
}