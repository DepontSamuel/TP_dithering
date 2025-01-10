use argh::FromArgs;
use image::{ImageError, Rgb, RgbImage};
use std::path::Path;

#[derive(Debug, Clone, PartialEq, FromArgs)]
/// Convertit une image en monochrome ou vers une palette réduite de couleurs.
struct DitherArgs {
    /// le fichier d’entrée
    #[argh(positional)]
    input: String,

    /// le fichier de sortie (optionnel)
    #[argh(positional)]
    output: Option<String>,

    /// le mode d’opération
    #[argh(subcommand)]
    mode: Mode,
}

#[derive(Debug, Clone, PartialEq, FromArgs)]
#[argh(subcommand)]
enum Mode {
    Seuil(OptsSeuil),
    Palette(OptsPalette),
}

#[derive(Debug, Clone, PartialEq, FromArgs)]
#[argh(subcommand, name = "seuil")]
/// Rendu de l’image par seuillage monochrome.
struct OptsSeuil {}

#[derive(Debug, Clone, PartialEq, FromArgs)]
#[argh(subcommand, name = "palette")]
/// Rendu de l’image avec une palette contenant un nombre limité de couleurs
struct OptsPalette {
    /// le nombre de couleurs à utiliser, dans la liste [NOIR, BLANC, ROUGE, VERT, BLEU, JAUNE, CYAN, MAGENTA]
    #[argh(option)]
    n_couleurs: usize,
}

const PALETTE: [Rgb<u8>; 8] = [
    Rgb([0, 0, 0]),
    Rgb([255, 255, 255]),
    Rgb([255, 0, 0]),
    Rgb([0, 255, 0]),
    Rgb([0, 0, 255]),
    Rgb([255, 255, 0]),
    Rgb([0, 255, 255]),
    Rgb([255, 0, 255]),
];

fn main() -> Result<(), ImageError> {
    let args: DitherArgs = argh::from_env();
    let path_in = args.input;
    let path_out = args.output.unwrap_or_else(|| "image/output.png".to_string());

    // Charger l'image d'entrée
    let img = image::open(&Path::new(&path_in))?;

    // Convertir en format RGB8
    let rgb_img: RgbImage = img.to_rgb8();

    let processed_img = match args.mode {
        Mode::Seuil(_) => apply_seuil(rgb_img),
        Mode::Palette(opts) => apply_palette(rgb_img, opts.n_couleurs),
    };

    // Sauvegarder l'image au format PNG
    processed_img.save(&Path::new(&path_out))?;

    println!("Image sauvegardée sous : {}", path_out);

    Ok(())
}

fn apply_seuil(img: RgbImage) -> RgbImage {
    // Implémentation du seuillage monochrome
    img // Placeholder
}

fn apply_palette(img: RgbImage, n_couleurs: usize) -> RgbImage {
    let palette = &PALETTE[..n_couleurs.min(PALETTE.len())];
    let mut new_img = RgbImage::new(img.width(), img.height());

    for (x, y, pixel) in img.enumerate_pixels() {
        let closest_color = find_closest_color(pixel, palette);
        new_img.put_pixel(x, y, closest_color);
    }

    new_img
}

fn find_closest_color(pixel: &image::Rgb<u8>, palette: &[image::Rgb<u8>]) -> image::Rgb<u8> {
    palette.iter()
        .min_by(|&color1, &color2| {
            let dist1 = color_distance(pixel, color1);
            let dist2 = color_distance(pixel, color2);
            dist1.partial_cmp(&dist2).unwrap()
        })
        .cloned()
        .unwrap_or(image::Rgb([0, 0, 0]))
}
fn color_distance(c1: &Rgb<u8>, c2: &Rgb<u8>) -> u32 {
    let r_diff = c1[0] as i32 - c2[0] as i32;
    let g_diff = c1[1] as i32 - c2[1] as i32;
    let b_diff = c1[2] as i32 - c2[2] as i32;
    (r_diff * r_diff + g_diff * g_diff + b_diff * b_diff) as u32
}
