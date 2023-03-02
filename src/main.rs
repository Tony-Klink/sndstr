use salvo::listener::rustls::{Keycert, RustlsConfig};
use salvo::prelude::*;
use salvo::ws::{Message, WebSocketUpgrade};

use dotenvy::dotenv;

use nostr::ClientMessage;

#[handler]
async fn connect(req: &mut Request, res: &mut Response) -> Result<(), StatusError> {
    WebSocketUpgrade::new()
        .upgrade(req, res, |mut ws| async move {
            while let Some(msg) = ws.recv().await {
                let msg = if let Ok(msg) = msg {
                    if !msg.is_text() {
                        return;
                    }

                    msg
                } else {
                    // client disconnected
                    return;
                };

                let parsed_message = msg.to_str().unwrap();
                let client_message = ClientMessage::from_json(parsed_message).unwrap();
                let s = match client_message {
                    ClientMessage::Event(m) => Message::text(m.as_json().unwrap()),
                    ClientMessage::Req { subscription_id, filters } => Message::text("Unsupported for now"),
                    ClientMessage::Close(m) => {
                        Message::text(format!("Unsubscribed from: {}", m.to_string()))
                    },
                    _ => Message::text("".to_string()),
                };

                if ws.send(s).await.is_err() {
                    // client disconnected
                    return;
                }
            }
        })
        .await
}

#[handler]
async fn index(res: &mut Response) {
    res.render(Text::Html(INDEX_HTML));
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    tracing_subscriber::fmt().init();

    let router = Router::new()
        .get(index)
        .push(Router::with_path("ws").handle(connect));
    let config = RustlsConfig::new(
        Keycert::new()
            .with_cert(include_bytes!("../certs/cert.pem").as_ref())
            .with_key(include_bytes!("../certs/key.pem").as_ref()),
    );
    tracing::info!("Listening on https://127.0.0.1:7878");
    let listener = RustlsListener::with_config(config).bind("127.0.0.1:7878");
    Server::new(listener)
        .serve(router).await;
}

static INDEX_HTML: &str = r#"<!DOCTYPE html>
<html>
    <head>
        <title>SNDSTR (formerly CHKSTR)</title>
    </head>
    <body>
        <h1>UI Test</h1>
        <div id="status">
            <p><em>Connecting...</em></p>
        </div>
        <script>
            const status = document.getElementById('status');
            const msg = document.getElementById('msg');
            const submit = document.getElementById('submit');
            const ws = new WebSocket(`wss://${location.host}/ws`);

            ws.onopen = function() {
                status.innerHTML = '<p><em>Connected!</em></p>';
            };
        </script>
    </body>
</html>
"#;
