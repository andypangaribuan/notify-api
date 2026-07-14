/*
 * Copyright (c) 2026.
 * Created by Andy Pangaribuan (andypangaribuan@treasury.id)
 *
 * All Rights Reserved.
 */

use super::model;
use crate::db::{entity, repo};
use base64::{Engine as _, engine::general_purpose::STANDARD};
use rmod::{
    db, log,
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
            log!("❌ failed to bind smtp proxy to {}: {:?}", addr, e);
            return;
        }
    };

    log!("🔥 smtp proxy listening on {}...", addr);

    loop {
        match listener.accept().await {
            Ok((stream, peer_addr)) => {
                let client_ip = peer_addr.ip().to_string();
                log!("🔌 client connected: {}", peer_addr);
                rmod::tokio::spawn(async move {
                    if let Err(e) = handle_connection(stream, client_ip).await {
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

async fn handle_connection(client_stream: TcpStream, client_ip: String) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut client_reader = BufReader::new(client_stream);

    // 1. Send greeting to client
    client_reader.get_mut().write_all(b"220 notify smtp proxy ready\r\n").await?;
    client_reader.get_mut().flush().await?;

    // Loop for handshake and authentication
    let credential = loop {
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
            client_reader.get_mut().write_all(b"250-notify\r\n250-AUTH LOGIN PLAIN\r\n250 8BITMIME\r\n").await?;
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

            match find_user(&decoded_user, &decoded_pass).await {
                Ok(cred) => break cred,
                Err(_) => {
                    client_reader.get_mut().write_all(b"535 Authentication failed\r\n").await?;
                    client_reader.get_mut().flush().await?;
                }
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
            let mut authenticated = None;
            if parts.len() >= 3 {
                let user_str = String::from_utf8_lossy(parts[1]);
                let pass_str = String::from_utf8_lossy(parts[2]);

                if let Ok(cred) = find_user(&user_str, &pass_str).await {
                    authenticated = Some(cred);
                }
            }

            if let Some(cred) = authenticated {
                break cred;
            } else {
                client_reader.get_mut().write_all(b"535 Authentication failed\r\n").await?;
                client_reader.get_mut().flush().await?;
            }
        } else if upper_cmd == "QUIT" {
            client_reader.get_mut().write_all(b"221 Bye\r\n").await?;
            client_reader.get_mut().flush().await?;
            return Ok(());
        } else {
            client_reader.get_mut().write_all(b"530 5.7.0 Must authenticate first\r\n").await?;
            client_reader.get_mut().flush().await?;
        }
    };

    // Check if the client IP is allowed
    if let Some(allowed_ips) = crate::app::env::smtp_allowed_ips()
        && !allowed_ips.contains(&client_ip)
    {
        log!("🚫 client IP blocked: {}", client_ip);
        client_reader.get_mut().write_all(b"554 5.7.1 Access denied: IP address blocked\r\n").await?;
        client_reader.get_mut().flush().await?;
        return Ok(());
    }

    // 2. Check and reserve rate limit
    let reserve_key = match crate::svc::rate_limit::check_and_reserve().await {
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

    // Get email registry details
    let email_config = match get_email_registry(&credential).await {
        Ok(cfg) => cfg,
        Err(err_msg) => {
            log!("🚫 failed to get email registry: {}", err_msg);
            client_reader.get_mut().write_all(b"554 5.7.1 Transaction failed: email registry lookup failed\r\n").await?;
            client_reader.get_mut().flush().await?;
            return Ok(());
        }
    };

    let smtp_config = match email_config {
        model::EmailConfig::Smtp(cfg) => cfg,
        model::EmailConfig::Api(_) => {
            log!("🚫 api channel is not supported by smtp proxy");
            client_reader.get_mut().write_all(b"554 5.7.1 API channel is not supported by smtp proxy\r\n").await?;
            client_reader.get_mut().flush().await?;
            return Ok(());
        }
    };

    // 3. Relay connection
    if let Err(e) = relay_connection(client_reader, smtp_config).await {
        log!("❌ Relay failed: {:?}", e);
        if let Some(ref key) = reserve_key {
            crate::svc::rate_limit::refund_reserve(key).await;
        }
        return Err(e);
    }

    Ok(())
}

async fn relay_connection(
    mut client_reader: BufReader<TcpStream>,
    smtp_config: model::EmailSmtp,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let provider = smtp_config.provider;
    log!("🔥 connecting to {} smtp relay server...", provider);
    let relay_host = smtp_config.host;
    let relay_port = smtp_config.port;
    let relay_user = smtp_config.user;
    let relay_pass = smtp_config.pass;
    let relay_addr = format!("{}:{}", relay_host, relay_port);

    let relay_tcp_stream = TcpStream::connect(&relay_addr).await?;
    let mut relay_reader = BufReader::new(relay_tcp_stream);

    // Read greeting from smtp relay
    let lines = read_smtp_response_lines(&mut relay_reader).await?;
    log!("👉 {} Greeting: {:?}", provider, lines);

    // Send EHLO
    relay_reader.get_mut().write_all(b"EHLO notify\r\n").await?;
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
    relay_tls_stream.write_all(b"EHLO notify\r\n").await?;
    relay_tls_stream.flush().await?;

    let mut relay_tls_reader = BufReader::new(relay_tls_stream);
    let lines = read_smtp_response_lines(&mut relay_tls_reader).await?;
    log!("👉 {} TLS EHLO: {:?}", provider, lines);

    // Authenticate with smtp relay using credentials
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

    // Check if smtp relay authentication succeeded
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

    // Now copy bidirectionally between client and smtp relay.
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
            return Err(std::io::Error::new(std::io::ErrorKind::UnexpectedEof, "EOF reached when reading smtp response"));
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

async fn find_user(decoded_user: &str, decoded_pass: &str) -> Result<entity::EmailSmtpCredential, String> {
    match repo::email_smtp_credential::fetch(
        "username = $1 AND password = $2",
        db::args![decoded_user.to_string(), decoded_pass.to_string()],
    )
    .await
    {
        Ok(Some(credential)) => Ok(credential),
        Ok(None) => Err("authentication failed: invalid credentials".to_string()),
        Err(err) => {
            log!("❌ database error during smtp credential fetch: {:?}", err);
            Err("database query error".to_string())
        }
    }
}

async fn get_email_registry(credential: &entity::EmailSmtpCredential) -> Result<model::EmailConfig, String> {
    let rules = match repo::email_rules::fetch_all(
        "allowed_apps = $1 OR $2 = ANY(regexp_split_to_array(allowed_apps, '\\s*,\\s*')) OR $3 = ANY(regexp_split_to_array(allowed_apps, '\\s*,\\s*'))",
        db::args![
            "*:*".to_string(),
            format!("*:{}", credential.app_name),
            format!("{}:{}", credential.env_name, credential.app_name)
        ]
    ).await {
        Ok(v) => v,
        Err(e) => {
            log!("❌ database error during fetch email_rules: {:?}", e);
            return Err("database error".to_string());
        }
    };

    if rules.is_empty() {
        return Err("no email rules allowed for this credential".to_string());
    }

    let mut email_registry_uids: Vec<String> = Vec::new();
    for rule in rules {
        if rule.tags.contains(&"#*".to_string()) {
            email_registry_uids.push(rule.email_registry_uid.clone());
        }
    }

    if email_registry_uids.is_empty() {
        return Err("no email registry found for this credential".to_string());
    }

    let registries =
        repo::email_registry::fetch_all("uid = ANY($1) AND is_active = true", db::args![email_registry_uids]).await.map_err(|e| {
            log!("❌ database error during fetch email_registry: {:?}", e);
            "database error".to_string()
        })?;

    if registries.is_empty() {
        return Err("no active email registry found".to_string());
    }

    for registry in registries {
        let conf = registry.email_conf;
        let provider = conf["provider"].as_str();
        let channel = conf["channel"].as_str();

        if let Some(provider) = provider
            && let Some(channel) = channel
        {
            if channel == "smtp" {
                let host = conf["host"].as_str();
                let port = conf["port"].as_u64();
                let user = conf["user"].as_str();
                let pass = conf["pass"].as_str();
                if let Some(host) = host
                    && let Some(port) = port
                    && let Some(user) = user
                    && let Some(pass) = pass
                {
                    return Ok(model::EmailConfig::Smtp(model::EmailSmtp {
                        provider: provider.to_string(),
                        host: host.to_string(),
                        port,
                        user: user.to_string(),
                        pass: pass.to_string(),
                    }));
                }
            } else if channel == "api" {
                let host = conf["host"].as_str();
                let api_key = conf["api-key"].as_str();
                if let Some(host) = host
                    && let Some(api_key) = api_key
                {
                    return Ok(model::EmailConfig::Api(model::EmailApi {
                        provider: provider.to_string(),
                        host: host.to_string(),
                        api_key: api_key.to_string(),
                    }));
                }
            }
        }
    }

    Err("no available registry can be used".to_string())
}
