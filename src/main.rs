use image::GenericImageView;
use pdfium_render::prelude::*;
use std::env;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} <file_paths...>", args[0]);
        std::process::exit(1);
    }

    let pdfium = Pdfium::new(
        Pdfium::bind_to_library(Pdfium::pdfium_platform_library_name_at_path("./"))
            .or_else(|_| Pdfium::bind_to_system_library())
            .map_err(|e| {
                eprintln!("Ошибка: не найдена pdfium.dll. Положите ее рядом с исполняемым файлом");
                e
            })?,
    );

    let first_path = &args[1];

    if first_path.to_lowercase().ends_with(".pdf") {
        pdf_to_images(&pdfium, first_path)
    } else {
        images_to_pdf(&pdfium, &args[1..])
    }
}

fn pdf_to_images(pdfium: &Pdfium, pdf_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let document = pdfium.load_pdf_from_file(pdf_path, None)?;

    for (page_index, page) in document.pages().iter().enumerate() {
        let render_config = PdfRenderConfig::new()
            .set_target_width(2000)
            .set_maximum_height(2000);

        let bitmap = page.render_with_config(&render_config)?;
        let image = bitmap.as_image();

        image.save(format!("page_{:04}.png", page_index + 1))?;
    }

    Ok(())
}

fn images_to_pdf(
    pdfium: &Pdfium,
    image_paths: &[String],
) -> Result<(), Box<dyn std::error::Error>> {
    for path in image_paths {
        if !Path::new(path).exists() {
            eprintln!("Ошибка: файл не найден: {}", path);
            std::process::exit(1);
        }
    }

    let mut document = pdfium.create_new_pdf()?;

    for path in image_paths {
        let image = image::open(path).map_err(|e| {
            eprintln!("Ошибка открытия изображения {}: {}", path, e);
            e
        })?;

        let (width_px, height_px) = image.dimensions();

        let width = PdfPoints::new(width_px as f32);
        let height = PdfPoints::new(height_px as f32);

        let mut page = document
            .pages_mut()
            .create_page_at_end(PdfPagePaperSize::Custom(width, height))?;

        let mut image_object = PdfPageImageObject::new(&document, &image)?;
        image_object.scale(width.value, height.value)?;

        page.objects_mut()
            .add_object(PdfPageObject::Image(image_object))?;
    }

    document.save_to_file("output.pdf")?;
    Ok(())
}
