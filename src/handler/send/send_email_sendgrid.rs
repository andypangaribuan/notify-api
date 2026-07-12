/*
 * Copyright (c) 2026.
 * Created by Andy Pangaribuan (andypangaribuan@treasury.id)
 *
 * All Rights Reserved.
 */

use super::model::SendEmailRequest;
use rmod::serde::Serialize;
use std::collections::HashMap;

#[derive(Serialize, Debug)]
#[serde(crate = "rmod::serde")]
struct SendGridEmail {
    email: String,
}

#[derive(Serialize, Debug)]
#[serde(crate = "rmod::serde")]
struct SendGridPersonalization {
    to: Vec<SendGridEmail>,
    #[serde(skip_serializing_if = "Option::is_none")]
    cc: Option<Vec<SendGridEmail>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    bcc: Option<Vec<SendGridEmail>>,
}

#[derive(Serialize, Debug)]
#[serde(crate = "rmod::serde")]
struct SendGridContent {
    #[serde(rename = "type")]
    type_: String,
    value: String,
}

#[derive(Serialize, Debug)]
#[serde(crate = "rmod::serde")]
struct SendGridAttachment {
    content: String,
    filename: String,
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    type_: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    disposition: Option<String>,
}

#[derive(Serialize, Debug)]
#[serde(crate = "rmod::serde")]
struct SendGridPayload {
    personalizations: Vec<SendGridPersonalization>,
    from: SendGridEmail,
    #[serde(skip_serializing_if = "Option::is_none")]
    reply_to: Option<SendGridEmail>,
    subject: String,
    content: Vec<SendGridContent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    attachments: Option<Vec<SendGridAttachment>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    headers: Option<HashMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    custom_args: Option<HashMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    categories: Option<Vec<String>>,
}

pub async fn send_email_sendgrid(req: SendEmailRequest) -> Result<(), String> {
    // 1. Fetch rules from database to match app_name and tag
    let rules = crate::db::repo::email_rules::fetch_all("deleted_at IS NULL", rmod::db::args![])
        .await
        .map_err(|e| format!("database error fetching rules: {:?}", e))?;

    let mut matched_registry = None;

    for rule in rules {
        let app_list: Vec<&str> = rule.allowed_apps.split(',').map(|s| s.trim()).collect();
        let tag_list: Vec<&str> = rule.tags.split(',').map(|s| s.trim()).collect();

        if app_list.contains(&req.app_name.as_str()) && tag_list.contains(&req.purpose_tag.as_str()) {
            if let Ok(Some(registry)) =
                crate::db::repo::email_registry::fetch("uid = $1 AND deleted_at IS NULL", rmod::db::args![rule.email_registry_uid]).await
            {
                matched_registry = Some(registry);
                break;
            }
        }
    }

    // 2. Resolve SendGrid config (registry match or fallback to environment variables)
    let mut api_key = crate::app::env::sendgrid_api_key();
    let mut from_email = crate::app::env::sendgrid_mail_from();
    let mut reply_to_email = req.reply_to.clone().or_else(crate::app::env::sendgrid_reply_to);

    if let Some(ref registry) = matched_registry {
        if let Some(conf) = registry.email_conf.as_object() {
            if let Some(key) = conf.get("api_key").and_then(|k| k.as_str()) {
                api_key = Some(key.to_string());
            }
            if let Some(from) = conf.get("from").and_then(|f| f.as_str()) {
                from_email = Some(from.to_string());
            } else if !registry.sender_email.is_empty() {
                from_email = Some(registry.sender_email.clone());
            }
            if let Some(reply) = conf.get("reply_to").and_then(|r| r.as_str()) {
                if reply_to_email.is_none() {
                    reply_to_email = Some(reply.to_string());
                }
            }
        } else if !registry.sender_email.is_empty() {
            from_email = Some(registry.sender_email.clone());
        }
    }

    let Some(api_key) = api_key else {
        return Err("SendGrid API key is not configured".to_string());
    };

    let Some(from_email) = from_email else {
        return Err("Sender email address (from) is not configured".to_string());
    };

    // 3. Build SendGrid API payload
    let to_emails: Vec<SendGridEmail> = req.send_to.iter().map(|e| SendGridEmail { email: e.clone() }).collect();
    let cc_emails: Option<Vec<SendGridEmail>> = req.cc_to.map(|cc| cc.iter().map(|e| SendGridEmail { email: e.clone() }).collect());
    let bcc_emails: Option<Vec<SendGridEmail>> = req.bcc_to.map(|bcc| bcc.iter().map(|e| SendGridEmail { email: e.clone() }).collect());

    let personalization = SendGridPersonalization { to: to_emails, cc: cc_emails, bcc: bcc_emails };

    let content_type = match req.body_type.to_lowercase().as_str() {
        "html" | "text/html" => "text/html".to_string(),
        _ => "text/plain".to_string(),
    };

    let content = vec![SendGridContent { type_: content_type, value: req.body.clone() }];

    let sg_reply_to = reply_to_email.map(|email| SendGridEmail { email });

    let attachments = req.attachment.map(|att_list| {
        att_list
            .into_iter()
            .map(|att| SendGridAttachment {
                content: att.content,
                filename: att.filename,
                type_: att.type_,
                disposition: Some("attachment".to_string()),
            })
            .collect()
    });

    let mut custom_args = HashMap::new();
    custom_args.insert("env_name".to_string(), req.env_name.clone());
    custom_args.insert("app_name".to_string(), req.app_name.clone());
    if let Some(ref meta) = req.metadata {
        for (k, v) in meta {
            custom_args.insert(k.clone(), v.clone());
        }
    }

    let mut categories = vec![req.env_name.clone(), req.app_name.clone(), req.purpose_tag.clone()];
    if let Some(ref tag_list) = req.tags {
        for t in tag_list {
            categories.push(t.clone());
        }
    }

    let payload = SendGridPayload {
        personalizations: vec![personalization],
        from: SendGridEmail { email: from_email },
        reply_to: sg_reply_to,
        subject: req.subject.clone(),
        content,
        attachments,
        headers: req.headers,
        custom_args: Some(custom_args),
        categories: Some(categories),
    };

    // 4. Send API POST request
    let mut headers = HashMap::new();
    headers.insert("Authorization".to_string(), format!("Bearer {}", api_key));
    headers.insert("Content-Type".to_string(), "application/json".to_string());

    let res = rmod::http::post("https://api.sendgrid.com/v3/mail/send", Some(headers), None, Some(payload)).await;

    match res {
        Ok(response) => {
            if response.status().is_success() {
                rmod::log!("📈 Email sent successfully via SendGrid Web API");
                Ok(())
            } else {
                let status = response.status();
                let body = response.text().await.unwrap_or_default();
                rmod::log!("❌ SendGrid API failed ({}): {}", status, body);
                Err(format!("SendGrid API failed with status {}: {}", status, body))
            }
        }
        Err(e) => {
            rmod::log!("❌ SendGrid API connection error: {:?}", e);
            Err(format!("connection error: {:?}", e))
        }
    }
}
