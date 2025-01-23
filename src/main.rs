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
    let path_out = args.output.unwrap_or_else(|| "image/outputaaaa.png".to_string());

    // Charger l'image d'entrée
    let img = image::open(&Path::new(&path_in))?;

    // Convertir en format RGB8
    let rgb_img: RgbImage = img.to_rgb8();

    let processed_img = match args.mode {
        Mode::Seuil(_) => apply_seuil(rgb_img),
        Mode::Palette(opts) => apply_palette(rgb_img, opts.n_couleurs),

    let b3 = bayer_dithering(3, 3, &bayer);
    println!("{:?}", b3);


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
    let mut new_img = img.clone();
    for pixel in new_img.pixels_mut() {
        *pixel = plus_proche_couleur(*pixel, &PALETTE[..n_couleurs]);
    }
    new_img
}


fn distance_euclidienne(c1: Rgb<u8>, c2: Rgb<u8>) -> f64 {
    let r = c1[0] as f64 - c2[0] as f64;
    let g = c1[1] as f64 - c2[1] as f64;
    let b = c1[2] as f64 - c2[2] as f64;
    (r * r + g * g + b * b).sqrt()
}

fn plus_proche_couleur(c: Rgb<u8>, palette: &[Rgb<u8>]) -> Rgb<u8> {
    let mut min_distance = f64::INFINITY;
    let mut plus_proche = palette[0];
    for couleur in palette {
        let distance = distance_euclidienne(c, *couleur);
        if distance < min_distance {
            min_distance = distance;
            plus_proche = *couleur;
        }
    }
    plus_proche
}

fn luminosite(pixel: &Rgb<u8>) -> u8 {
    (0.299 * pixel[0] as f32 + 0.587 * pixel[1] as f32 + 0.114 * pixel[2] as f32) as u8
}

fn bayer_dithering(image: &RgbImage, bayer: &[[u8; 8]; 8]) -> RgbImage {
    let mut new_img = image.clone();
    let (width, height) = image.dimensions();
    for y in 0..height {
        for x in 0..width {
            let pixel = image.get_pixel(x, y);
            let lum = luminosite(&pixel);
            let seuil = bayer[y as usize % 8][x as usize % 8];
            let new_pixel = if lum > seuil {
                Rgb([255, 255, 255])
            } else {
                Rgb([0, 0, 0])
            };
            new_img.put_pixel(x, y, new_pixel);
        }
    }
    new_img
}