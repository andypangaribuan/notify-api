/*
 * Copyright (c) 2026.
 * Created by Andy Pangaribuan (andypangaribuan@treasury.id)
 *
 * All Rights Reserved.
 */

use super::model;
use std::collections::HashMap;

pub async fn send_email_sendgrid(req: model::SendEmailRequest, api_key: &str, from_email: &str) -> Result<(), String> {
    let req_env_name = req.env_name.clone().unwrap_or_default();
    let req_app_name = req.app_name.clone().unwrap_or_default();
    let req_purpose_tag = req.purpose_tag.clone().unwrap_or_default();
    let req_send_to = req.send_to.clone().unwrap_or_default();
    let req_subject = req.subject.clone().unwrap_or_default();
    let req_body = req.body.clone().unwrap_or_default();
    let req_body_type = req.body_type.clone().unwrap_or_default();
    let req_reply_to = req.reply_to.clone();

    let to_emails: Vec<model::SendGridEmail> = req_send_to.iter().map(|e| model::SendGridEmail { email: e.clone() }).collect();
    let cc_emails: Option<Vec<model::SendGridEmail>> =
        req.cc_to.map(|cc| cc.iter().map(|e| model::SendGridEmail { email: e.clone() }).collect());
    let bcc_emails: Option<Vec<model::SendGridEmail>> =
        req.bcc_to.map(|bcc| bcc.iter().map(|e| model::SendGridEmail { email: e.clone() }).collect());
    let personalization = model::SendGridPersonalization { to: to_emails, cc: cc_emails, bcc: bcc_emails };

    let content_type = match req_body_type.to_lowercase().as_str() {
        "html" | "text/html" => "text/html".to_string(),
        _ => "text/plain".to_string(),
    };
    let content = vec![model::SendGridContent { type_: content_type, value: req_body.clone() }];
    let attachments = req.attachment.map(|att_list| {
        att_list
            .into_iter()
            .map(|att| model::SendGridAttachment {
                content: att.content,
                filename: att.filename,
                type_: att.type_,
                disposition: Some("attachment".to_string()),
            })
            .collect()
    });

    let mut custom_args = HashMap::new();
    custom_args.insert("env_name".to_string(), req_env_name.clone());
    custom_args.insert("app_name".to_string(), req_app_name.clone());
    if let Some(ref meta) = req.metadata {
        for (k, v) in meta {
            custom_args.insert(k.clone(), v.clone());
        }
    }

    let mut categories = vec![req_env_name.clone(), req_app_name.clone(), req_purpose_tag.clone()];
    if let Some(ref tag_list) = req.tags {
        for t in tag_list {
            categories.push(t.clone());
        }
    }

    let payload = model::SendGridPayload {
        personalizations: vec![personalization],
        from: model::SendGridEmail { email: from_email.to_string() },
        reply_to: req_reply_to.map(|email| model::SendGridEmail { email }),
        subject: req_subject.clone(),
        content,
        attachments,
        headers: req.headers,
        custom_args: Some(custom_args),
        categories: Some(categories),
    };

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
