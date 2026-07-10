/*
 * Copyright (c) 2026.
 * Created by Andy Pangaribuan (andypangaribuan@treasury.id)
 *
 * All Rights Reserved.
 */

use base64::{Engine as _, engine::general_purpose::STANDARD};
use rmod::{
    log,
    tokio::{
        io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
        net::{TcpListener, TcpStream},
    },
};
use std::sync::Arc;

pub async fn start() {
    let port = crate::app::env::smtp_listen_port();
    let addr = format!("0.0.0.0:{}", port);

    let listener = match TcpListener::bind(&addr).await {
        Ok(l) => l,
        Err(e) => {
            log!("❌ failed to bind SMTP proxy to {}: {:?}", addr, e);
            return;
        }
    };

    log!("🔥 SMTP proxy listening on {}...", addr);

    loop {
        match listener.accept().await {
            Ok((stream, peer_addr)) => {
                let client_ip = peer_addr.ip().to_string();
                if let Some(allowed_ips) = crate::app::env::smtp_allowed_ips() {
                    if !allowed_ips.contains(&client_ip) {
                        log!("🚫 client IP blocked: {}", client_ip);
                        continue;
                    }
                }

                log!("🔌 client connected: {}", peer_addr);
                rmod::tokio::spawn(async move {
                    if let Err(e) = handle_connection(stream).await {
                        log!("❌ error handling client {}: {:?}", peer_addr, e);
                    }
                    log!("🔌 client disconnected: {}", peer_addr);
                });
            }
            Err(e) => {
                log!("❌ failed to accept connection: {:?}", e);
            }
        }
    }
}

async fn handle_connection(client_stream: TcpStream) -> Result<(), Box<dyn std::error::Error>> {
    let mut client_reader = BufReader::new(client_stream);

    // 1. Send greeting to client
    client_reader.get_mut().write_all(b"220 notify-api SMTP Proxy Ready\r\n").await?;
    client_reader.get_mut().flush().await?;

    // Load local authentication credentials
    let (expected_user, expected_pass) = crate::app::env::local_credentials();

    // Loop for handshake and authentication
    loop {
        let mut line = String::new();
        let bytes_read = client_reader.read_line(&mut line).await?;
        if bytes_read == 0 {
            return Ok(()); // Client disconnected
        }

        let cmd = line.trim();
        if cmd.is_empty() {
            continue;
        }

        let upper_cmd = cmd.to_uppercase();

        if upper_cmd.starts_with("EHLO") || upper_cmd.starts_with("HELO") {
            client_reader.get_mut().write_all(b"250-notify-api\r\n250-AUTH LOGIN PLAIN\r\n250 8BITMIME\r\n").await?;
            client_reader.get_mut().flush().await?;
        } else if upper_cmd == "AUTH LOGIN" {
            // Prompt for Username
            client_reader
                .get_mut()
                .write_all(b"334 VXNlcm5hbWU6\r\n") // "Username:" in base64
                .await?;
            client_reader.get_mut().flush().await?;

            // Read Username
            let mut user_line = String::new();
            if client_reader.read_line(&mut user_line).await? == 0 {
                return Ok(());
            }

            // Prompt for Password
            client_reader
                .get_mut()
                .write_all(b"334 UGFzc3dvcmQ6\r\n") // "Password:" in base64
                .await?;
            client_reader.get_mut().flush().await?;

            // Read Password
            let mut pass_line = String::new();
            if client_reader.read_line(&mut pass_line).await? == 0 {
                return Ok(());
            }

            // Decode credentials
            let decoded_user = match STANDARD.decode(user_line.trim()) {
                Ok(bytes) => String::from_utf8_lossy(&bytes).into_owned(),
                Err(_) => {
                    client_reader.get_mut().write_all(b"535 Authentication credentials invalid\r\n").await?;
                    continue;
                }
            };

            let decoded_pass = match STANDARD.decode(pass_line.trim()) {
                Ok(bytes) => String::from_utf8_lossy(&bytes).into_owned(),
                Err(_) => {
                    client_reader.get_mut().write_all(b"535 Authentication credentials invalid\r\n").await?;
                    continue;
                }
            };

            if decoded_user == expected_user && decoded_pass == expected_pass {
                break;
            } else {
                client_reader.get_mut().write_all(b"535 Authentication failed\r\n").await?;
                client_reader.get_mut().flush().await?;
            }
        } else if upper_cmd.starts_with("AUTH PLAIN") {
            let payload_b64 = if upper_cmd == "AUTH PLAIN" {
                // Prompt for payload
                client_reader.get_mut().write_all(b"334 \r\n").await?;
                client_reader.get_mut().flush().await?;

                let mut payload_line = String::new();
                if client_reader.read_line(&mut payload_line).await? == 0 {
                    return Ok(());
                }
                payload_line.trim().to_string()
            } else {
                cmd[10..].trim().to_string()
            };

            let decoded_bytes = match STANDARD.decode(&payload_b64) {
                Ok(bytes) => bytes,
                Err(_) => {
                    client_reader.get_mut().write_all(b"535 Authentication credentials invalid\r\n").await?;
                    continue;
                }
            };

            // Format of AUTH PLAIN decoded string is: \0username\0password
            let parts: Vec<&[u8]> = decoded_bytes.split(|&b| b == 0).collect();
            if parts.len() >= 3 {
                let user_str = String::from_utf8_lossy(parts[1]);
                let pass_str = String::from_utf8_lossy(parts[2]);

                if user_str == expected_user && pass_str == expected_pass {
                    break;
                }
            }

            client_reader.get_mut().write_all(b"535 Authentication failed\r\n").await?;
            client_reader.get_mut().flush().await?;
        } else if upper_cmd == "QUIT" {
            client_reader.get_mut().write_all(b"221 Bye\r\n").await?;
            client_reader.get_mut().flush().await?;
            return Ok(());
        } else {
            client_reader.get_mut().write_all(b"530 5.7.0 Must authenticate first\r\n").await?;
            client_reader.get_mut().flush().await?;
        }
    }

    // 2. Check and reserve rate limit
    let reserve_key = match crate::svc::rate_limit::check_and_reserve() {
        Ok(key) => key,
        Err(err_msg) => {
            log!("🚫 Rate limit check failed: {}", err_msg);
            if err_msg.contains("blocked") {
                client_reader.get_mut().write_all(b"554 5.7.1 Sending not allowed by rate limit config\r\n").await?;
            } else {
                client_reader.get_mut().write_all(b"451 4.7.1 Rate limit exceeded. Try again later\r\n").await?;
            }
            client_reader.get_mut().flush().await?;
            return Ok(());
        }
    };

    // 3. Relay connection
    if let Err(e) = relay_connection(client_reader).await {
        log!("❌ Relay failed: {:?}", e);
        if let Some(ref key) = reserve_key {
            crate::svc::rate_limit::refund_reserve(key);
        }
        return Err(e);
    }

    Ok(())
}

async fn relay_connection(
    mut client_reader: BufReader<TcpStream>,
) -> Result<(), Box<dyn std::error::Error>> {
    let provider = crate::app::env::email_provider();
    log!("🔥 connecting to {} SMTP relay server...", provider);
    let (relay_host, relay_port, relay_user, relay_pass) = crate::app::env::relay_credentials();
    let relay_addr = format!("{}:{}", relay_host, relay_port);

    let relay_tcp_stream = TcpStream::connect(&relay_addr).await?;
    let mut relay_reader = BufReader::new(relay_tcp_stream);

    // Read greeting from SMTP relay
    let lines = read_smtp_response_lines(&mut relay_reader).await?;
    log!("👉 {} Greeting: {:?}", provider, lines);

    // Send EHLO
    relay_reader.get_mut().write_all(b"EHLO notify-api\r\n").await?;
    relay_reader.get_mut().flush().await?;
    let lines = read_smtp_response_lines(&mut relay_reader).await?;
    log!("👉 {} EHLO: {:?}", provider, lines);

    // Send STARTTLS
    relay_reader.get_mut().write_all(b"STARTTLS\r\n").await?;
    relay_reader.get_mut().flush().await?;
    let lines = read_smtp_response_lines(&mut relay_reader).await?;
    log!("👉 {} STARTTLS Response: {:?}", provider, lines);

    // Upgrade connection to TLS
    let mut root_cert_store = rmod::rustls::RootCertStore::empty();
    root_cert_store.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
    let config = rmod::rustls::ClientConfig::builder().with_root_certificates(root_cert_store).with_no_client_auth();
    let connector = tokio_rustls::TlsConnector::from(Arc::new(config));
    let server_name = rustls_pki_types::ServerName::try_from(relay_host.as_str())
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidInput, e))?
        .to_owned();

    let relay_tcp_raw = relay_reader.into_inner();
    let mut relay_tls_stream = connector.connect(server_name, relay_tcp_raw).await?;

    // Perform EHLO inside the TLS tunnel
    relay_tls_stream.write_all(b"EHLO notify-api\r\n").await?;
    relay_tls_stream.flush().await?;

    let mut relay_tls_reader = BufReader::new(relay_tls_stream);
    let lines = read_smtp_response_lines(&mut relay_tls_reader).await?;
    log!("👉 {} TLS EHLO: {:?}", provider, lines);

    // Authenticate with SMTP relay using credentials
    relay_tls_reader.get_mut().write_all(b"AUTH LOGIN\r\n").await?;
    relay_tls_reader.get_mut().flush().await?;
    let lines = read_smtp_response_lines(&mut relay_tls_reader).await?;
    log!("👉 {} AUTH LOGIN Response: {:?}", provider, lines);

    // Send Username in base64
    let relay_user_b64 = STANDARD.encode(&relay_user);
    relay_tls_reader.get_mut().write_all(format!("{}\r\n", relay_user_b64).as_bytes()).await?;
    relay_tls_reader.get_mut().flush().await?;
    let lines = read_smtp_response_lines(&mut relay_tls_reader).await?;
    log!("👉 {} Username Response: {:?}", provider, lines);

    // Send Password in base64
    let relay_pass_b64 = STANDARD.encode(&relay_pass);
    relay_tls_reader.get_mut().write_all(format!("{}\r\n", relay_pass_b64).as_bytes()).await?;
    relay_tls_reader.get_mut().flush().await?;
    let lines = read_smtp_response_lines(&mut relay_tls_reader).await?;
    log!("👉 {} Password Response: {:?}", provider, lines);

    // Check if SMTP relay authentication succeeded
    let auth_success = lines.first().map(|line| line.starts_with("235")).unwrap_or(false);

    if !auth_success {
        log!("❌ {} authentication failed!", provider);
        client_reader.get_mut().write_all(b"535 Relay authentication failed\r\n").await?;
        client_reader.get_mut().flush().await?;
        return Err("Relay authentication failed".into());
    }

    log!("🔥 {} authentication successful! Relay is ready.", provider);

    // Inform the client that authentication was successful
    client_reader.get_mut().write_all(b"235 Authentication successful\r\n").await?;
    client_reader.get_mut().flush().await?;

    // Now copy bidirectionally between client and SMTP relay.
    let mut client_raw = client_reader.into_inner();
    let mut relay_raw = relay_tls_reader.into_inner();

    rmod::tokio::io::copy_bidirectional(&mut client_raw, &mut relay_raw).await?;

    Ok(())
}

async fn read_smtp_response_lines<S>(reader: &mut BufReader<S>) -> Result<Vec<String>, std::io::Error>
where
    S: rmod::tokio::io::AsyncRead + Unpin,
{
    let mut lines = Vec::new();
    loop {
        let mut line = String::new();
        let bytes_read = reader.read_line(&mut line).await?;
        if bytes_read == 0 {
            return Err(std::io::Error::new(std::io::ErrorKind::UnexpectedEof, "EOF reached when reading SMTP response"));
        }
        let trimmed = line.trim_end_matches("\r\n");
        lines.push(trimmed.to_string());
        if trimmed.len() >= 4 {
            let sep = trimmed.chars().nth(3).unwrap_or(' ');
            if sep == ' ' {
                break;
            }
        } else {
            break;
        }
    }
    Ok(lines)
}
