use argh::FromArgs;
use std::path::{Path};
use image::{Rgb, RgbImage};
use rand::Rng; // Nous aurons besoin de la bibliothèque `rand`
use std::fs; // Pour vérifier et créer des répertoires

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
    Tramage(OptsTramage), // Ajout de l'option Tramage
    Blanchir(OptsBlanchir)
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

#[derive(Debug, Clone, PartialEq, FromArgs)]
#[argh(subcommand, name="tramage")]
/// Applique un tramage aléatoire sur l'image
struct OptsTramage {
    /// seuil de tramage : une valeur entre 0 et 1 (ex : 0.5)
    #[argh(option, default = "0.5")]
    seuil: f32,
}

#[derive(Debug, Clone, PartialEq, FromArgs)]
#[argh(subcommand, name="blanchir")]
/// Applique un tramage aléatoire sur l'image
struct OptsBlanchir {
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

// applique un seuillage monochrome sur une image
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

// applique un tramage aléatoire sur l'image
fn apply_tramage(image: &mut RgbImage, seuil: f32) {
    let (width, height) = image.dimensions();
    let mut rng = rand::thread_rng(); // Générateur de nombres aléatoires
    for y in 0..height {
        for x in 0..width {
            let pixel = image.get_pixel(x, y);
            let lum = luminosite(pixel) as f32 / 255.0; // Luminosité normalisée entre 0 et 1
            let random_threshold: f32 = rng.gen(); // Tirer un seuil aléatoire entre 0 et 1
            let new_pixel = if lum > random_threshold * seuil {
                Rgb([255, 255, 255]) // Pixel blanc
            } else {
                Rgb([0, 0, 0]) // Pixel noir
            };
            image.put_pixel(x, y, new_pixel);
        }
    }
}

// Génère un nom de fichier de sortie basé sur l'entrée et le mode
fn generate_output_filename(input: &str, mode: &Mode, seuil: Option<f32>) -> String {
    let path = Path::new(input);
    let stem = path.file_stem().unwrap().to_str().unwrap();

    let mode_suffix = match mode {
        Mode::Seuil(_) => "_seuil".to_string(),
        Mode::Palette(_) => "_palette".to_string(),
        Mode::Blanchir(_) => "_blanchir".to_string(),
        Mode::Tramage(_) => {
            if let Some(seuil_value) = seuil {
                format!("_tramage_{:.1}", seuil_value)
            } else {
                "_tramage".to_string()
            }
        }
    };

    let extension = path.extension().unwrap_or_default().to_str().unwrap_or("png");
    format!("{}{}.{extension}", stem, mode_suffix, extension = extension)
}
fn passer_pixel_sur_deux_en_blanc(image: &mut RgbImage) {
    let (width, height) = image.dimensions();
    
    for y in 0..height {
        for x in 0..width {
            // On passe un pixel sur deux en blanc
            if (x + y) % 2 == 0 {
                image.put_pixel(x, y, Rgb([255, 255, 255])); // Pixel blanc
            }
        }
    }
}



fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: DitherArgs = argh::from_env();
    let path_in = args.input;
    
    // Si un output est spécifié, on l'utilise, sinon on génère un nom basé sur l'entrée et le mode
    let path_out = args.output.unwrap_or_else(|| {
        generate_output_filename(&path_in, &args.mode, match &args.mode {
            Mode::Tramage(opts) => Some(opts.seuil),
            _ => None,
        })
    });

    // Créer le répertoire images/output/ si nécessaire
    let output_dir = Path::new("images/output");
    if !output_dir.exists() {
        fs::create_dir_all(output_dir)?;
    }

    // Construire le chemin complet vers le fichier de sortie
    let output_path = output_dir.join(path_out);

    // Ouvrir l'image et appliquer le traitement
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
        Mode::Tramage(opts) => {
            apply_tramage(&mut rgb_img, opts.seuil);
            println!("Traitement en mode tramage aléatoire appliqué avec le seuil: {}", opts.seuil);
        },
        Mode::Blanchir(_) => {
            passer_pixel_sur_deux_en_blanc(&mut rgb_img);
            println!("Traitement en mode blanchiment appliqué");
        },
    }

    // Sauvegarder l'image dans le dossier images/output/
    rgb_img.save(&output_path)?;
    println!("Image sauvegardée sous : {}", output_path.display());

    Ok(())
}
