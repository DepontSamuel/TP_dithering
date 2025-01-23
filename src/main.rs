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
    Blanchir(OptsBlanchir),
    Diffusion(OptsDiffusion),
    DiffusionPalette(OptsDiffusionPalette),
    DiffusionFloydSteinberg(OptsDiffusionFloydSteinberg),
    DiffusionMatrice(OptsDiffusionMatrice)
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
struct OptsBlanchir {}

#[derive(Debug, Clone, PartialEq, FromArgs)]
#[argh(subcommand, name="diffusion")]
/// Applique la diffusion d'erreur sur l'image
struct OptsDiffusion {}

#[derive(Debug, Clone, PartialEq, FromArgs)]
#[argh(subcommand, name="diffusion-palette")]
/// Applique la diffusion d'erreur sur l'image
struct OptsDiffusionPalette {}

#[derive(Debug, Clone, PartialEq, FromArgs)]
#[argh(subcommand, name="diffusion-floyd-steinberg")]
/// Applique la diffusion d'erreur sur l'image
struct OptsDiffusionFloydSteinberg {}

#[derive(Debug, Clone, PartialEq, FromArgs)]
#[argh(subcommand, name="diffusion-matrice")]
/// Applique une diffusion d'erreur avec une matrice personnalisée.
struct OptsDiffusionMatrice {
    /// nom de la matrice : "jarvis", "atkinson" ou "floyd"
    #[argh(option, default = "String::from(\"floyd\")")]
    matrice: String,
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
        },
        Mode::Diffusion(_) => "_diffusion".to_string(),
        Mode::DiffusionPalette(_) => "_diffusion_palette".to_string(),
        Mode::DiffusionFloydSteinberg(_) => "_diffusion_floyd_steinberg".to_string(),
        Mode::DiffusionMatrice(_) => "_diffusion_matrice".to_string()
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

fn apply_diffusion_erreur(image: &mut RgbImage) {
    let (width, height) = image.dimensions();
    let mut erreur_image: Vec<Vec<f32>> = vec![vec![0.0; width as usize]; height as usize];

    for y in 0..height {
        for x in 0..width {
            // Obtenir la luminosité du pixel courant et ajouter l'erreur accumulée
            let pixel = image.get_pixel(x, y);
            let lum = luminosite(pixel) as f32 / 255.0; // Normalisé entre 0 et 1
            let lum_corrigee = (lum + erreur_image[y as usize][x as usize]).clamp(0.0, 1.0);

            // Déterminer la nouvelle couleur (noir ou blanc)
            let nouvelle_couleur = if lum_corrigee > 0.5 {
                Rgb([255, 255, 255]) // Blanc
            } else {
                Rgb([0, 0, 0]) // Noir
            };

            // Calculer l'erreur entre la luminosité réelle et celle choisie
            let erreur = lum_corrigee - if nouvelle_couleur == Rgb([255, 255, 255]) { 1.0 } else { 0.0 };

            // Appliquer la nouvelle couleur au pixel
            image.put_pixel(x, y, nouvelle_couleur);

            // Diffuser l'erreur aux pixels voisins
            if x + 1 < width {
                erreur_image[y as usize][(x + 1) as usize] += erreur * 0.5;
            }
            if y + 1 < height {
                erreur_image[(y + 1) as usize][x as usize] += erreur * 0.5;
            }
        }
    }
}

fn apply_diffusion_erreur_palette(image: &mut RgbImage, palette: &[Rgb<u8>]) {
    let (width, height) = image.dimensions();
    let mut erreur_image: Vec<Vec<[f32; 3]>> = vec![vec![[0.0, 0.0, 0.0]; width as usize]; height as usize];

    for y in 0..height {
        for x in 0..width {
            // Ajouter l'erreur accumulée à la couleur du pixel
            let pixel = image.get_pixel(x, y);
            let corrected_color = [
                (pixel[0] as f32 + erreur_image[y as usize][x as usize][0]).clamp(0.0, 255.0),
                (pixel[1] as f32 + erreur_image[y as usize][x as usize][1]).clamp(0.0, 255.0),
                (pixel[2] as f32 + erreur_image[y as usize][x as usize][2]).clamp(0.0, 255.0),
            ];

            // Trouver la couleur la plus proche dans la palette
            let quantized_color = palette.iter().min_by_key(|&&p| {
                let diff_r = p[0] as f32 - corrected_color[0];
                let diff_g = p[1] as f32 - corrected_color[1];
                let diff_b = p[2] as f32 - corrected_color[2];
                let distance = diff_r * diff_r + diff_g * diff_g + diff_b * diff_b;
                distance as u32
            }).unwrap();

            // Calculer l'erreur de quantification
            let error = [
                corrected_color[0] - quantized_color[0] as f32,
                corrected_color[1] - quantized_color[1] as f32,
                corrected_color[2] - quantized_color[2] as f32,
            ];

            // Appliquer la couleur quantifiée au pixel
            image.put_pixel(x, y, *quantized_color);

            // Diffuser l'erreur aux voisins
            if x + 1 < width {
                for c in 0..3 {
                    erreur_image[y as usize][(x + 1) as usize][c] += error[c] * 0.5;
                }
            }
            if y + 1 < height {
                for c in 0..3 {
                    erreur_image[(y + 1) as usize][x as usize][c] += error[c] * 0.5;
                }
            }
        }
    }
}

fn apply_diffusion_erreur_floyd_steinberg(image: &mut RgbImage, palette: &[Rgb<u8>]) {
    let (width, height) = image.dimensions();
    let mut erreur_image: Vec<Vec<[f32; 3]>> = vec![vec![[0.0, 0.0, 0.0]; width as usize]; height as usize];

    for y in 0..height {
        for x in 0..width {
            // Ajouter l'erreur accumulée à la couleur du pixel
            let pixel = image.get_pixel(x, y);
            let corrected_color = [
                (pixel[0] as f32 + erreur_image[y as usize][x as usize][0]).clamp(0.0, 255.0),
                (pixel[1] as f32 + erreur_image[y as usize][x as usize][1]).clamp(0.0, 255.0),
                (pixel[2] as f32 + erreur_image[y as usize][x as usize][2]).clamp(0.0, 255.0),
            ];

            // Trouver la couleur la plus proche dans la palette
            let quantized_color = palette.iter().min_by_key(|&&p| {
                let diff_r = p[0] as f32 - corrected_color[0];
                let diff_g = p[1] as f32 - corrected_color[1];
                let diff_b = p[2] as f32 - corrected_color[2];
                let distance = diff_r * diff_r + diff_g * diff_g + diff_b * diff_b;
                distance as u32
            }).unwrap();

            // Calculer l'erreur de quantification
            let error = [
                corrected_color[0] - quantized_color[0] as f32,
                corrected_color[1] - quantized_color[1] as f32,
                corrected_color[2] - quantized_color[2] as f32,
            ];

            // Appliquer la couleur quantifiée au pixel
            image.put_pixel(x, y, *quantized_color);

            // Diffuser l'erreur selon la matrice Floyd-Steinberg
            if x + 1 < width {
                for c in 0..3 {
                    erreur_image[y as usize][(x + 1) as usize][c] += error[c] * 7.0 / 16.0;
                }
            }
            if y + 1 < height {
                if x > 0 {
                    for c in 0..3 {
                        erreur_image[(y + 1) as usize][(x - 1) as usize][c] += error[c] * 3.0 / 16.0;
                    }
                }
                for c in 0..3 {
                    erreur_image[(y + 1) as usize][x as usize][c] += error[c] * 5.0 / 16.0;
                }
                if x + 1 < width {
                    for c in 0..3 {
                        erreur_image[(y + 1) as usize][(x + 1) as usize][c] += error[c] * 1.0 / 16.0;
                    }
                }
            }
        }
    }
}

fn apply_diffusion_matrice(image: &mut RgbImage, palette: &[Rgb<u8>], matrix: &[(i32, i32, f32)]) {
    let (width, height) = image.dimensions();
    let mut erreur_image: Vec<Vec<[f32; 3]>> = vec![vec![[0.0, 0.0, 0.0]; width as usize]; height as usize];

    for y in 0..height {
        for x in 0..width {
            // Ajouter l'erreur accumulée à la couleur du pixel
            let pixel = image.get_pixel(x, y);
            let corrected_color = [
                (pixel[0] as f32 + erreur_image[y as usize][x as usize][0]).clamp(0.0, 255.0),
                (pixel[1] as f32 + erreur_image[y as usize][x as usize][1]).clamp(0.0, 255.0),
                (pixel[2] as f32 + erreur_image[y as usize][x as usize][2]).clamp(0.0, 255.0),
            ];

            // Trouver la couleur la plus proche dans la palette
            let quantized_color = palette.iter().min_by_key(|&&p| {
                let diff_r = p[0] as f32 - corrected_color[0];
                let diff_g = p[1] as f32 - corrected_color[1];
                let diff_b = p[2] as f32 - corrected_color[2];
                let distance = diff_r * diff_r + diff_g * diff_g + diff_b * diff_b;
                distance as u32
            }).unwrap();

            // Calculer l'erreur de quantification
            let error = [
                corrected_color[0] - quantized_color[0] as f32,
                corrected_color[1] - quantized_color[1] as f32,
                corrected_color[2] - quantized_color[2] as f32,
            ];

            // Appliquer la couleur quantifiée au pixel
            image.put_pixel(x, y, *quantized_color);

            // Diffuser l'erreur selon la matrice
            for &(dx, dy, weight) in matrix {
                let nx = x as i32 + dx;
                let ny = y as i32 + dy;
                if nx >= 0 && nx < width as i32 && ny >= 0 && ny < height as i32 {
                    let nx = nx as usize;
                    let ny = ny as usize;
                    for c in 0..3 {
                        erreur_image[ny][nx][c] += error[c] * weight;
                    }
                }
            }
        }
    }
}

fn apply_palette(image: &mut RgbImage, n_couleurs: usize){
    for pixel in image.pixels_mut() {
        *pixel = plus_proche_couleur(*pixel, &PALETTE[..n_couleurs]);
    }
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
        Mode::Palette(opts) => {
            apply_palette(&mut rgb_img, opts.n_couleurs);
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
        Mode::Diffusion(_) => {
            apply_diffusion_erreur(&mut rgb_img);
            println!("Traitement en mode diffusion d'erreur terminé.");
        },
        Mode::DiffusionPalette(_) => {
            apply_diffusion_erreur_palette(&mut rgb_img, &PALETTE);
            println!("Traitement en mode diffusion d'erreur avec palettisation terminé.");
        },
        Mode::DiffusionFloydSteinberg(_) => {
            apply_diffusion_erreur_floyd_steinberg(&mut rgb_img, &PALETTE);
            println!("Traitement en mode diffusion d'erreur avec palettisation et floyd steinberg terminé.");
        },
        Mode::DiffusionMatrice(opts) => {
            let matrice = match opts.matrice.as_str() {
                "jarvis" => vec![
                    (1, 0, 7.0 / 48.0), (2, 0, 5.0 / 48.0),
                    (-2, 1, 3.0 / 48.0), (-1, 1, 5.0 / 48.0), (0, 1, 7.0 / 48.0), (1, 1, 5.0 / 48.0), (2, 1, 3.0 / 48.0),
                    (-2, 2, 1.0 / 48.0), (-1, 2, 3.0 / 48.0), (0, 2, 5.0 / 48.0), (1, 2, 3.0 / 48.0), (2, 2, 1.0 / 48.0),
                ],
                "atkinson" => vec![
                    (1, 0, 1.0 / 8.0), (2, 0, 1.0 / 8.0),
                    (-1, 1, 1.0 / 8.0), (0, 1, 1.0 / 8.0), (1, 1, 1.0 / 8.0),
                    (0, 2, 1.0 / 8.0),
                ],
                _ => vec![
                    (1, 0, 7.0 / 16.0),
                    (-1, 1, 3.0 / 16.0), (0, 1, 5.0 / 16.0), (1, 1, 1.0 / 16.0),
                ], // Par défaut Floyd-Steinberg
            };
            apply_diffusion_matrice(&mut rgb_img, &PALETTE, &matrice);
            println!("Diffusion d'erreur avec la matrice '{}' appliquée.", opts.matrice);
        }
        
    }

    // Sauvegarder l'image dans le dossier images/output/
    rgb_img.save(&output_path)?;
    println!("Image sauvegardée sous : {}", output_path.display());

    Ok(())
}