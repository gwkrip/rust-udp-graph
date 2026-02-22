use actix_web::{get, web, App, HttpServer, HttpResponse, Responder};
use actix_web::web::Data;
use actix_web_actors::ws;
use std::sync::{Arc, Mutex};
use tera::Tera;
use tokio::net::UdpSocket;
use std::str;

type SharedData = Arc<Mutex<Vec<u32>>>;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let data: SharedData = Arc::new(Mutex::new(vec![]));

    let udp_data = data.clone();
    tokio::spawn(async move {
        let socket = UdpSocket::bind("0.0.0.0:8125").await.unwrap();
        let mut buf = [0u8; 1024];

        loop {
            if let Ok((len, _addr)) = socket.recv_from(&mut buf).await {
                if let Ok(msg) = str::from_utf8(&buf[..len]) {
                    let nums: Vec<u32> = msg.split('|')
                        .filter_map(|s| s.split(':').nth(1))
                        .filter_map(|v| v.parse().ok())
                        .collect();

                    let mut shared = udp_data.lock().unwrap();
                    *shared = nums;
                }
            }
        }
    });

    let tera = Tera::new("templates/**/*").unwrap();

    HttpServer::new(move || {
        App::new()
            .app_data(Data::new(data.clone()))
            .app_data(Data::new(tera.clone()))
            .service(index)
            .route("/ws/", web::get().to(ws_index))
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}

#[get("/")]
async fn index(data: Data<SharedData>, tmpl: Data<Tera>) -> impl Responder {
    let shared = data.lock().unwrap();
    let mut ctx = tera::Context::new();
    ctx.insert("data", &*shared);
    let rendered = tmpl.render("index.html", &ctx).unwrap();
    HttpResponse::Ok().body(rendered)
}

async fn ws_index(
    req: actix_web::HttpRequest,
    stream: web::Payload,
    data: Data<SharedData>,
) -> impl Responder {
    ws::start(MyWs { data: data.get_ref().clone() }, &req, stream)
}

struct MyWs {
    data: SharedData,
}

impl actix::StreamHandler<Result<ws::Message, ws::ProtocolError>> for MyWs {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Ping(msg)) => ctx.pong(&msg),
            Ok(ws::Message::Close(reason)) => {
                ctx.close(reason);
                ctx.stop();
            }
            _ => {}
        }
    }
}

impl actix::Actor for MyWs {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        ctx.run_interval(std::time::Duration::from_secs(1), |act, ctx| {
            let shared = act.data.lock().unwrap();
            let msg = serde_json::to_string(&*shared).unwrap();
            ctx.text(msg);
        });
    }
}
