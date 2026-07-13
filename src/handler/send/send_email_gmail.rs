/*
 * Copyright (c) 2026.
 * Created by Andy Pangaribuan (andypangaribuan@treasury.id)
 *
 * This product is protected by copyright and distributed under
 * licenses restricting copying, distribution and decompilation.
 * All Rights Reserved.
 */

use crate::handler::send::model::SendEmailRequest;
use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use rmod::tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use rmod::tokio::net::TcpStream;
use std::sync::Arc;

pub async fn send_email_gmail(req: SendEmailRequest, host: &str, port: u16, username: &str, password: &str) -> Result<(), String> {
    // 1. Build MIME email raw data
    let mut email_data = String::new();
    email_data.push_str(&format!("From: {}\r\n", username));
    if let Some(ref to) = req.send_to {
        email_data.push_str(&format!("To: {}\r\n", to.join(", ")));
    }
    if let Some(ref cc) = req.cc_to {
        email_data.push_str(&format!("Cc: {}\r\n", cc.join(", ")));
    }
    if let Some(ref reply_to) = req.reply_to {
        email_data.push_str(&format!("Reply-To: {}\r\n", reply_to));
    }
    if let Some(ref headers) = req.headers {
        for (k, v) in headers {
            email_data.push_str(&format!("{}: {}\r\n", k, v));
        }
    }
    email_data.push_str(&format!("Subject: {}\r\n", req.subject.clone().unwrap_or_default()));
    email_data.push_str("MIME-Version: 1.0\r\n");

    let boundary = "----=_Part_notify_api_123456789";
    let body_type = req.body_type.clone().unwrap_or_default().to_lowercase();
    let is_html = body_type == "html" || body_type == "text/html";
    let content_type = if is_html { "text/html" } else { "text/plain" };

    if req.attachment.as_ref().map_or(true, |v| v.is_empty()) {
        email_data.push_str(&format!("Content-Type: {}; charset=utf-8\r\n\r\n", content_type));
        email_data.push_str(&req.body.clone().unwrap_or_default());
    } else {
        email_data.push_str(&format!("Content-Type: multipart/mixed; boundary=\"{}\"\r\n\r\n", boundary));

        // Body part
        email_data.push_str(&format!("--{}\r\n", boundary));
        email_data.push_str(&format!("Content-Type: {}; charset=utf-8\r\n\r\n", content_type));
        email_data.push_str(&req.body.clone().unwrap_or_default());
        email_data.push_str("\r\n");

        // Attachments
        if let Some(ref atts) = req.attachment {
            for att in atts {
                email_data.push_str(&format!("--{}\r\n", boundary));
                email_data.push_str(&format!(
                    "Content-Type: {}\r\n",
                    att.type_.clone().unwrap_or_else(|| "application/octet-stream".to_string())
                ));
                email_data.push_str("Content-Transfer-Encoding: base64\r\n");
                email_data.push_str(&format!("Content-Disposition: attachment; filename=\"{}\"\r\n\r\n", att.filename));
                email_data.push_str(&att.content);
                email_data.push_str("\r\n");
            }
        }
        email_data.push_str(&format!("--{}--\r\n", boundary));
    }

    // 2. Connect to SMTP server
    let addr = format!("{}:{}", host, port);
    let tcp_stream = TcpStream::connect(&addr).await.map_err(|e| format!("failed to connect to SMTP server {}: {}", addr, e))?;
    let mut reader = BufReader::new(tcp_stream);

    // Read Greeting
    read_smtp_response_lines(&mut reader).await.map_err(|e| format!("failed to read greeting: {}", e))?;

    // EHLO
    reader.get_mut().write_all(b"EHLO notify-api\r\n").await.map_err(|e| format!("EHLO write failed: {}", e))?;
    reader.get_mut().flush().await.map_err(|e| format!("flush failed: {}", e))?;
    read_smtp_response_lines(&mut reader).await.map_err(|e| format!("failed to read EHLO response: {}", e))?;

    // STARTTLS
    reader.get_mut().write_all(b"STARTTLS\r\n").await.map_err(|e| format!("STARTTLS write failed: {}", e))?;
    reader.get_mut().flush().await.map_err(|e| format!("flush failed: {}", e))?;
    read_smtp_response_lines(&mut reader).await.map_err(|e| format!("failed to read STARTTLS response: {}", e))?;

    // Upgrade connection to TLS
    let mut root_cert_store = rmod::rustls::RootCertStore::empty();
    root_cert_store.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
    let config = rmod::rustls::ClientConfig::builder().with_root_certificates(root_cert_store).with_no_client_auth();
    let connector = tokio_rustls::TlsConnector::from(Arc::new(config));
    let server_name = rustls_pki_types::ServerName::try_from(host).map_err(|e| format!("invalid host name for TLS: {}", e))?.to_owned();

    let raw_stream = reader.into_inner();
    let tls_stream = connector.connect(server_name, raw_stream).await.map_err(|e| format!("TLS handshake failed: {}", e))?;
    let mut tls_reader = BufReader::new(tls_stream);

    // Perform EHLO inside the TLS tunnel
    tls_reader.get_mut().write_all(b"EHLO notify-api\r\n").await.map_err(|e| format!("TLS EHLO write failed: {}", e))?;
    tls_reader.get_mut().flush().await.map_err(|e| format!("flush failed: {}", e))?;
    read_smtp_response_lines(&mut tls_reader).await.map_err(|e| format!("failed to read TLS EHLO response: {}", e))?;

    // Authenticate: AUTH LOGIN
    tls_reader.get_mut().write_all(b"AUTH LOGIN\r\n").await.map_err(|e| format!("AUTH LOGIN write failed: {}", e))?;
    tls_reader.get_mut().flush().await.map_err(|e| format!("flush failed: {}", e))?;
    read_smtp_response_lines(&mut tls_reader).await.map_err(|e| format!("failed to read AUTH LOGIN response: {}", e))?;

    // Send base64 Username
    let user_b64 = STANDARD.encode(username);
    tls_reader.get_mut().write_all(format!("{}\r\n", user_b64).as_bytes()).await.map_err(|e| format!("username write failed: {}", e))?;
    tls_reader.get_mut().flush().await.map_err(|e| format!("flush failed: {}", e))?;
    read_smtp_response_lines(&mut tls_reader).await.map_err(|e| format!("failed to read username response: {}", e))?;

    // Send base64 Password
    let pass_b64 = STANDARD.encode(password);
    tls_reader.get_mut().write_all(format!("{}\r\n", pass_b64).as_bytes()).await.map_err(|e| format!("password write failed: {}", e))?;
    tls_reader.get_mut().flush().await.map_err(|e| format!("flush failed: {}", e))?;
    let pass_resp = read_smtp_response_lines(&mut tls_reader).await.map_err(|e| format!("failed to read password response: {}", e))?;

    if !pass_resp.first().map_or(false, |line| line.starts_with("235")) {
        return Err(format!("SMTP authentication failed: {:?}", pass_resp));
    }

    // MAIL FROM
    tls_reader
        .get_mut()
        .write_all(format!("MAIL FROM:<{}>\r\n", username).as_bytes())
        .await
        .map_err(|e| format!("MAIL FROM write failed: {}", e))?;
    tls_reader.get_mut().flush().await.map_err(|e| format!("flush failed: {}", e))?;
    read_smtp_response_lines(&mut tls_reader).await.map_err(|e| format!("failed to read MAIL FROM response: {}", e))?;

    // RCPT TO (To, CC, BCC)
    let mut recipients = Vec::new();
    if let Some(ref to) = req.send_to {
        recipients.extend(to.clone());
    }
    if let Some(ref cc) = req.cc_to {
        recipients.extend(cc.clone());
    }
    if let Some(ref bcc) = req.bcc_to {
        recipients.extend(bcc.clone());
    }

    for rcpt in recipients {
        tls_reader
            .get_mut()
            .write_all(format!("RCPT TO:<{}>\r\n", rcpt).as_bytes())
            .await
            .map_err(|e| format!("RCPT TO write failed: {}", e))?;
        tls_reader.get_mut().flush().await.map_err(|e| format!("flush failed: {}", e))?;
        read_smtp_response_lines(&mut tls_reader).await.map_err(|e| format!("failed to read RCPT TO response: {}", e))?;
    }

    // DATA
    tls_reader.get_mut().write_all(b"DATA\r\n").await.map_err(|e| format!("DATA write failed: {}", e))?;
    tls_reader.get_mut().flush().await.map_err(|e| format!("flush failed: {}", e))?;
    read_smtp_response_lines(&mut tls_reader).await.map_err(|e| format!("failed to read DATA response: {}", e))?;

    // Send Body
    tls_reader.get_mut().write_all(email_data.as_bytes()).await.map_err(|e| format!("body write failed: {}", e))?;
    // Send trailing dot to finish email
    tls_reader.get_mut().write_all(b"\r\n.\r\n").await.map_err(|e| format!("dot write failed: {}", e))?;
    tls_reader.get_mut().flush().await.map_err(|e| format!("flush failed: {}", e))?;
    read_smtp_response_lines(&mut tls_reader).await.map_err(|e| format!("failed to read body send response: {}", e))?;

    // QUIT
    tls_reader.get_mut().write_all(b"QUIT\r\n").await.map_err(|e| format!("QUIT write failed: {}", e))?;
    tls_reader.get_mut().flush().await.map_err(|e| format!("flush failed: {}", e))?;

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
