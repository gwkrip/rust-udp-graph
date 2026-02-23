use actix::ActorContext;
use actix::AsyncContext;
use actix_web::{get, web, App, HttpServer, HttpResponse, Responder};
use actix_web::web::Data;
use actix_web_actors::ws;
use serde::{Serialize, Deserialize};
use std::sync::{Arc, Mutex};
use tera::Tera;
use tokio::net::UdpSocket;
use std::str;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct VryptMetrics {
    pub rps: u64,
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub active_connections: u64,
    pub total_accepted: u64,
    pub conn_timeouts: u64,
    pub dropped_token_exhausted: u64,
    pub dropped_buf_exhausted: u64,
    pub errors_read: u64,
    pub errors_write: u64,
    pub errors_request_too_large: u64,
}

type SharedData = Arc<Mutex<VryptMetrics>>;

fn parse_statsd(msg: &str) -> VryptMetrics {
    let mut m = VryptMetrics::default();
    for line in msg.lines() {
        let line = line.trim();
        if line.is_empty() { continue; }
        let parts: Vec<&str> = line.splitn(2, ':').collect();
        if parts.len() != 2 { continue; }
        let name = parts[0];
        let value_str = parts[1].split('|').next().unwrap_or("0");
        let value: u64 = value_str.parse().unwrap_or(0);

        match name {
            "vrypt.rps"                        => m.rps = value,
            "vrypt.bytes_sent"                 => m.bytes_sent = value,
            "vrypt.bytes_received"             => m.bytes_received = value,
            "vrypt.active_connections"         => m.active_connections = value,
            "vrypt.total_accepted"             => m.total_accepted = value,
            "vrypt.conn_timeouts"              => m.conn_timeouts = value,
            "vrypt.dropped.token_exhausted"    => m.dropped_token_exhausted = value,
            "vrypt.dropped.buf_exhausted"      => m.dropped_buf_exhausted = value,
            "vrypt.errors.read"                => m.errors_read = value,
            "vrypt.errors.write"               => m.errors_write = value,
            "vrypt.errors.request_too_large"   => m.errors_request_too_large = value,
            _ => {}
        }
    }
    m
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let data: SharedData = Arc::new(Mutex::new(VryptMetrics::default()));

    let udp_data = data.clone();
    tokio::spawn(async move {
        let socket = UdpSocket::bind("vrypt-server.railway.internal:8125").await.unwrap();
        let mut buf = [0u8; 4096];

        loop {
            if let Ok((len, _addr)) = socket.recv_from(&mut buf).await {
                if let Ok(msg) = str::from_utf8(&buf[..len]) {
                    let metrics = parse_statsd(msg);
                    let mut shared = udp_data.lock().unwrap();
                    *shared = metrics;
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
        ctx.run_interval(std::time::Duration::from_secs(1), |act, ctx: &mut ws::WebsocketContext<Self>| {
            let shared = act.data.lock().unwrap();
            let msg = serde_json::to_string(&*shared).unwrap();
            ctx.text(msg);
        });
    }
}
