use argh::FromArgs;
use std::path::Path;
use image::{ImageError, Rgb, RgbImage};

#[derive(Debug, Clone, PartialEq, FromArgs)]
/// Convertit une image en monochrome ou vers une palette réduite de couleurs.
struct DitherArgs {
    /// le fichier d'entrée
    #[argh(positional)]
    input: String,
    
    /// le fichier de sortie (optionnel)
    #[argh(positional)]
    output: Option<String>,
    
    /// le mode d'opération
    #[argh(subcommand)]
    mode: Mode
}

#[derive(Debug, Clone, PartialEq, FromArgs)]
#[argh(subcommand)]
enum Mode {
    Seuil(OptsSeuil),
    Palette(OptsPalette),
}

#[derive(Debug, Clone, PartialEq, FromArgs)]
#[argh(subcommand, name="seuil")]
/// Rendu de l'image par seuillage avec deux couleurs personnalisables.
struct OptsSeuil {
    /// couleur claire (format R,G,B, ex: "255,0,0" pour rouge)
    #[argh(option, default = "String::from(\"255,255,255\")")]
    couleur_claire: String,
    
    /// couleur foncée (format R,G,B, ex: "0,0,255" pour bleu)
    #[argh(option, default = "String::from(\"0,0,0\")")]
    couleur_foncee: String,
}

#[derive(Debug, Clone, PartialEq, FromArgs)]
#[argh(subcommand, name="palette")]
/// Rendu de l'image avec une palette contenant un nombre limité de couleurs
struct OptsPalette {
    /// le nombre de couleurs à utiliser, dans la liste [NOIR, BLANC, ROUGE, VERT, BLEU, JAUNE, CYAN, MAGENTA]
    #[argh(option)]
    n_couleurs: usize
}

// renvoie une couleur a partie d'un string "R,G,B"
fn parse_color(color_str: &str) -> Result<Rgb<u8>, String> {
    let values: Result<Vec<u8>, _> = color_str
        .split(',')
        .map(|s| s.trim().parse::<u8>())
        .collect();
    
    match values {
        Ok(rgb) if rgb.len() == 3 => Ok(Rgb([rgb[0], rgb[1], rgb[2]])),
        _ => Err("Format de couleur invalide. Utilisez R,G,B (ex: 255,0,0)".to_string())
    }
}

// renvoie la luminosité d'un pixel
fn luminosite(pixel: &Rgb<u8>) -> u8 {
    let Rgb(data) = *pixel;
    (0.2126 * data[0] as f32 + 0.7152 * data[1] as f32 + 0.0722 * data[2] as f32) as u8
}

//applie un seuillage monochrome sur une image
fn apply_seuil(image: &mut RgbImage, couleur_claire: Rgb<u8>, couleur_foncee: Rgb<u8>) {
    let (width, height) = image.dimensions();
    for y in 0..height {
        for x in 0..width {
            let pixel = image.get_pixel(x, y);
            let lum = luminosite(pixel);
            let new_pixel = if lum >= 128 {
                couleur_claire
            } else {
                couleur_foncee
            };
            image.put_pixel(x, y, new_pixel);
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: DitherArgs = argh::from_env();
    let path_in = args.input;
    let path_out = args.output.unwrap_or_else(|| "output.png".to_string());

    let mut rgb_img = image::open(&Path::new(&path_in))?.to_rgb8();

    match args.mode {
        Mode::Seuil(opts) => {
            let couleur_claire = parse_color(&opts.couleur_claire)?;
            let couleur_foncee = parse_color(&opts.couleur_foncee)?;
            apply_seuil(&mut rgb_img, couleur_claire, couleur_foncee);
            println!("Traitement en mode seuil appliqué avec les couleurs personnalisées.");
        },
        Mode::Palette(_) => {
            println!("Mode palette non implémenté.");
        },
    }

    rgb_img.save(&Path::new(&path_out))?;
    println!("Image sauvegardée sous : {}", path_out);
    Ok(())
}
