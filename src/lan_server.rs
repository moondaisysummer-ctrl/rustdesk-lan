use hbb_common::{
    allow_err,
    config::{option2bool, Config, RENDEZVOUS_PORT},
    log, sleep, tokio,
};

use base::config::keys::*;

use crate::server::{check_zombie, new as new_server, ServerPtr};

const DEFAULT_LAN_PASSWORD: &str = "12qwaszx";

fn ensure_default_password() {
    if !Config::has_permanent_password() && Config::set_permanent_password(DEFAULT_LAN_PASSWORD) {
        Config::set_option(
            "lan-password-plain".to_string(),
            DEFAULT_LAN_PASSWORD.to_string(),
        );
        log::info!("Default permanent password applied");
    }
    if Config::get_option("verification-method").is_empty() {
        Config::set_option("verification-method".to_string(), "use-permanent-password".to_string());
    }
}

pub async fn start_all() {
    check_zombie();
    ensure_default_password();
    let server = new_server();
    let server_cloned = server.clone();
    let handle = tokio::spawn(async move {
        direct_server(server_cloned).await;
    });
    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    if crate::platform::is_installed() {
        std::thread::spawn(move || {
            allow_err!(crate::lan::start_listening());
        });
    }
    scrap::codec::test_av1();
    // keep --server alive, as RendezvousMediator::start_all() did upstream
    let _ = handle.await;
}

pub fn get_direct_port() -> i32 {
    let mut port = Config::get_option("direct-access-port")
        .parse::<i32>()
        .unwrap_or(0);
    if port <= 0 {
        port = RENDEZVOUS_PORT + 2;
    }
    port
}

async fn direct_server(server: ServerPtr) {
    let mut listener = None;
    let mut port = 0;
    loop {
        let disabled = !option2bool(
            OPTION_DIRECT_SERVER,
            &Config::get_option(OPTION_DIRECT_SERVER),
        ) || option2bool("stop-service", &Config::get_option("stop-service"));
        if !disabled && listener.is_none() {
            port = get_direct_port();
            match hbb_common::tcp::listen_any(port as _).await {
                Ok(l) => {
                    listener = Some(l);
                    log::info!(
                        "Direct server listening on: {:?}",
                        listener.as_ref().map(|l| l.local_addr())
                    );
                }
                Err(err) => {
                    // to-do: pass to ui
                    log::error!(
                        "Failed to start direct server on port: {}, error: {}",
                        port,
                        err
                    );
                    loop {
                        if port != get_direct_port() {
                            break;
                        }
                        sleep(1.).await;
                    }
                }
            }
        }
        if let Some(l) = listener.as_mut() {
            if disabled || port != get_direct_port() {
                log::info!("Exit direct access listen");
                listener = None;
                server.write().unwrap().close_connections();
                continue;
            }
            if let Ok(Ok((stream, addr))) = hbb_common::timeout(1000, l.accept()).await {
                stream.set_nodelay(true).ok();
                log::info!("direct access from {}", addr);
                let local_addr = stream
                    .local_addr()
                    .unwrap_or(Config::get_any_listen_addr(true));
                let server = server.clone();
                tokio::spawn(async move {
                    allow_err!(
                        crate::server::create_tcp_connection(
                            server,
                            hbb_common::Stream::from(stream, local_addr),
                            addr,
                            false,
                        )
                        .await
                    );
                });
            } else {
                sleep(0.1).await;
            }
        } else {
            sleep(1.).await;
        }
    }
}
