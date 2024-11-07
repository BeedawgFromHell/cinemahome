use std::env;
use std::fmt::format;
use actix_files::NamedFile;
use actix_web::{get, App, HttpRequest, HttpResponse, HttpServer, Responder};
use actix_web::dev::Path;
use actix_web::web::Data;
use tera::{Context, Tera};

#[get("/")]
async fn index() -> impl Responder {
    match get_video_list() {
        Ok(files) => {
            let file_links: Vec<String> = files.iter()
                .map(|file| format!("<a href=\"/player/{}\">{}</a>", file, file))
                .collect();
            let html_body = file_links.join("<br>");
            HttpResponse::Ok()
                .content_type("text/html")
                .body(html_body)
        }
        Err(err) => {
            panic!("{}", err)
        }
    }
}

#[get("/player/{filename}")]
async fn player(
    tmpl: Data<Tera>,
    path: actix_web::web::Path<String>,
) -> actix_web::Result<HttpResponse> {
    let filename = path.into_inner();
    let mut ctx = Context::new();
    ctx.insert("filename", &filename);

    let rendered = tmpl.render("player.html", &ctx)
        .map_err(|_| actix_web::error::ErrorInternalServerError("Ошибка рендеринга шаблона"))?;

    Ok(HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(rendered))
}

#[get("/video/{filename}")]
async fn video(req: HttpRequest, path: actix_web::web::Path<(String,)>) -> actix_web::Result<HttpResponse> {
    let filename = path.into_inner().0;
    let video_dir = env::var("VIDEO_DIR").unwrap_or("./videos".to_string());
    let filepath = std::path::Path::new(&video_dir).join(filename);

    if filepath.exists() && filepath.is_file() {
        let file = NamedFile::open(filepath)?;
        let content_type = match file.path().extension().and_then(|ext| ext.to_str()) {
            Some("mp4") => "video/mp4",
            Some("webm") => "video/webm",
            Some("mkv") => "video/x-matroska",
            _ => "application/octet-stream",
        };

        Ok(file
            .use_last_modified(true)
            .set_content_type(content_type.parse().unwrap())
            .disable_content_disposition()
            .into_response(&req)
        )
    } else {
        Err(actix_web::error::ErrorNotFound("Файл не найден"))
    }
}

fn get_video_list() -> std::io::Result<Vec<String>> {
    let video_dir = env::var("VIDEO_DIR").unwrap_or_else(|_| "./videos".to_string());

    let mut video_files = Vec::new();
    for entry in std::fs::read_dir(video_dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() {
            if let Some(extension) = path.extension() {
                if extension == "mp4" || extension == "avi" || extension == "webm" || extension == "mkv" {
                    if let Some(filename) = path.file_name() {
                        video_files.push(filename.to_string_lossy().into_owned());
                    }
                }
            }
        }
    }
    Ok(video_files)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();

    HttpServer::new(|| {
        App::new()
            .app_data(Data::new(Tera::new("templates/**/*").expect("Ошибка загрузки шаблонов")))
            .service(index)
            .service(video)
            .service(player)
    })
        .bind("0.0.0.0:8080")?
        .run()
        .await
}



