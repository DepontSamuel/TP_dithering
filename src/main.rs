use argh::FromArgs;
use std::path::Path;
use image::{ImageError, Rgb, RgbImage};

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
/// Rendu de l’image par seuillage monochrome.
struct OptsSeuil {
}


#[derive(Debug, Clone, PartialEq, FromArgs)]
#[argh(subcommand, name="palette")]
/// Rendu de l’image avec une palette contenant un nombre limité de couleurs
struct OptsPalette {

    /// le nombre de couleurs à utiliser, dans la liste [NOIR, BLANC, ROUGE, VERT, BLEU, JAUNE, CYAN, MAGENTA]
    #[argh(option)]
    n_couleurs: usize
}
 
// const WHITE: image::Rgb<u8> = image::Rgb([255, 255, 255]);
// const GREY: image::Rgb<u8> = image::Rgb([127, 127, 127]);
// const BLACK: image::Rgb<u8> = image::Rgb([0, 0, 0]);
// const BLUE: image::Rgb<u8> = image::Rgb([0, 0, 255]);
// const RED: image::Rgb<u8> = image::Rgb([255, 0, 0]);
// const GREEN: image::Rgb<u8> = image::Rgb([0, 255, 0]);
// const YELLOW: image::Rgb<u8> = image::Rgb([255, 255, 0]);
// const MAGENTA: image::Rgb<u8> = image::Rgb([255, 0, 255]);
// const CYAN: image::Rgb<u8> = image::Rgb([0, 255, 255]);


// Fonction pour calculer la luminosité d'un pixel RGB
fn luminosite(pixel: &Rgb<u8>) -> u8 {
    let Rgb(data) = *pixel;
    // Calcul de la luminosité : Luminosité = 0.2126*R + 0.7152*G + 0.0722*B
    (0.2126 * data[0] as f32 + 0.7152 * data[1] as f32 + 0.0722 * data[2] as f32) as u8
}

// Fonction pour appliquer le seuillage monochrome
fn apply_seuil(image: &mut RgbImage) {
    let (width, height) = image.dimensions();
    
    for y in 0..height {
        for x in 0..width {
            let pixel = image.get_pixel(x, y);
            let lum = luminosite(pixel);
            
            // Si la luminosité est supérieure ou égale à 128 (50%), le pixel devient blanc
            let new_pixel = if lum >= 128 {
                Rgb([255, 255, 255])  // Blanc
            } else {
                Rgb([0, 0, 0])  // Noir
            };
            
            image.put_pixel(x, y, new_pixel);
        }
    }
}

fn main() -> Result<(), ImageError> {
    let args: DitherArgs = argh::from_env();
    let path_in = args.input;
    let path_out = args.output.unwrap_or_else(|| "image/output.png".to_string());

    // Charger l'image d'entrée
    let img = image::open(&Path::new(&path_in))?;

    // Convertir l'image en format RGB8
    let mut rgb_img: RgbImage = img.to_rgb8();

    // Appliquer le mode "seuil" si spécifié
    match args.mode {
        Mode::Seuil(_) => {
            apply_seuil(&mut rgb_img);
            println!("Traitement en mode seuil appliqué.");
        },
        Mode::Palette(_) => {
            // Pour l'instant, nous n'implémentons rien pour le mode Palette
        },
    }

    // Sauvegarder l'image modifiée
    rgb_img.save(&Path::new(&path_out))?;

    println!("Image sauvegardée sous : {}", path_out);

    Ok(())
}