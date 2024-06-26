use actix_files::NamedFile;
use actix_proxy::IntoHttpResponse;
use actix_web::{
    web::{self, Redirect},
    App, HttpRequest, HttpResponse, HttpServer, Responder,
};
use base64::prelude::*;

#[derive(serde::Deserialize)]
struct LinkForm {
    url: String,
}

struct State {
    secret_key: orion::aead::SecretKey,
    tera: tera::Tera,
}

#[actix_web::get("/url/{secret}")]
async fn paywall_page(secret: web::Path<String>, state: web::Data<State>) -> impl Responder {
    let mut context = tera::Context::new();
    context.insert(
        "url",
        &format!("/proxy/{}/", urlencoding::encode(&secret.into_inner())),
    );
    let rendered = state.tera.render("paywall.html", &context).unwrap();

    actix_web::HttpResponse::Ok()
        .content_type(actix_web::http::header::ContentType::html())
        .body(rendered)
}

async fn do_proxy_request(url: String) -> Result<HttpResponse, actix_proxy::SendRequestError> {
    let client = awc::Client::new();
    let mut response = client.get(&url).send().await?.into_http_response();

    let content_type = response.headers_mut().remove(actix_web::http::header::CONTENT_TYPE).next().unwrap().clone();
    response.headers_mut().clear();
    response.headers_mut().insert(actix_web::http::header::CONTENT_TYPE, content_type);

    // let scrub_headers = [
    //     actix_web::http::header::CONTENT_SECURITY_POLICY,
    //     actix_web::http::header::SET_COOKIE,
    //     actix_web::http::header::X_FRAME_OPTIONS,
    //     actix_web::http::header::STRICT_TRANSPORT_SECURITY,
    // ];
    // for header in scrub_headers {
    //     response.headers_mut().remove(header);
    // }

    Ok(response)
}

#[actix_web::get("/proxy/{secret}/{proxied:.*}")]
async fn proxy_relative(
    _request: HttpRequest,
    path: web::Path<(String, String)>,
    state: web::Data<State>,
) -> Result<actix_web::HttpResponse, actix_proxy::SendRequestError> {
    let (secret, _proxied) = path.into_inner();
    let url = String::from_utf8(
        orion::aead::open(
            &state.secret_key,
            &BASE64_STANDARD.decode(secret.as_bytes()).unwrap(),
        )
        .unwrap(),
    )
    .unwrap();

    // TODO: if proxied is not empty, we should generate a relative URL proxy request

    do_proxy_request(url).await
}

#[actix_web::get("/absolute_proxy/{secret}/{proxied:.*}")]
async fn proxy_absolute(
    _request: HttpRequest,
    path: web::Path<(String, String)>,
    state: web::Data<State>,
) -> Result<actix_web::HttpResponse, actix_proxy::SendRequestError> {
    let (secret, proxied) = path.into_inner();
    let secret = urlencoding::decode(&secret).unwrap();

    let url = String::from_utf8(
        orion::aead::open(
            &state.secret_key,
            &BASE64_STANDARD.decode(secret.as_bytes()).unwrap(),
        )
        .unwrap(),
    )
    .unwrap();

    let mut url = url::Url::parse(&url).unwrap();
    url.set_path(&proxied);

    do_proxy_request(url.to_string()).await
}

#[actix_web::get("/{proxied:.*}")]
async fn proxy_absolute_redirect(
    request: HttpRequest,
    proxied: web::Path<String>,
) -> Result<actix_web::HttpResponse, actix_web::error::Error> {
    let referer = request
        .headers()
        .get("Referer")
        .ok_or(actix_web::error::ErrorNotFound("missing referer"))
        .and_then(|r| {
            r.to_str()
                .map_err(|_| actix_web::error::ErrorBadRequest("referer not string-convertible"))
        })?;

    let url = url::Url::parse(referer)
        .map_err(|_| actix_web::error::ErrorBadRequest("referer not a url"))?;

    let secret = url
        .path()
        .strip_prefix("/proxy/")
        .ok_or(actix_web::error::ErrorNotFound("referer not from /proxy/"))?
        .trim_end_matches("/");

    Ok(HttpResponse::TemporaryRedirect()
        .insert_header((
            "location",
            format!("/absolute_proxy/{}/{}", secret, proxied.into_inner()),
        ))
        .finish())
}

#[actix_web::post("/link-result")]
async fn get_link_result(data: web::Form<LinkForm>, state: web::Data<State>) -> impl Responder {
    let encrypted_url = orion::aead::seal(&state.secret_key, data.url.as_bytes()).unwrap();
    let mut text = String::new();
    BASE64_STANDARD.encode_string(encrypted_url, &mut text);
    let encoded_encrypted_url = urlencoding::encode(&text);
    let encoded_encrypted_url = format!("/url/{encoded_encrypted_url}");

    let mut context = tera::Context::new();
    context.insert("url", &encoded_encrypted_url);
    let rendered = state.tera.render("link-result.html", &context).unwrap();

    actix_web::HttpResponse::Ok()
        .content_type(actix_web::http::header::ContentType::html())
        .body(rendered)
}

#[actix_web::get("/")]
async fn index_html() -> impl Responder {
    NamedFile::open_async("./static/index.html").await.unwrap()
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let tera = tera::Tera::new("./templates/*").unwrap();

    let state = web::Data::new(State {
        secret_key: orion::aead::SecretKey::default(),
        tera,
    });

    HttpServer::new(move || {
        App::new()
            .app_data(state.clone())
            .service(index_html)
            .service(get_link_result)
            .service(paywall_page)
            .service(proxy_relative)
            .service(proxy_absolute)
            .service(
                actix_files::Files::new("/xxxpaywallxxx/", "./static").index_file("index.html"),
            )
            .service(proxy_absolute_redirect)
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}
