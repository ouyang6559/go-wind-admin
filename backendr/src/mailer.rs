//! SMTP 邮件发送（对齐 Go `pkg/mailer/smtp.go`）。
//!
//! SMTP 配置完全来自 `sys_notification_channels` 渠道行（无全局配置）：
//! TLS 模式 `SSL` → 隐式 TLS（465）；`NONE`/`START_TLS`/空 → 明文连接，
//! 服务器支持时升级 STARTTLS；`from` 缺省回落 `smtp_username`。

use crate::error::AppError;

/// 渠道行中的 SMTP 配置（调用方从 DB 读出后传入）
#[derive(Debug, Clone)]
pub struct SmtpConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub from: String,
    /// NONE | START_TLS | SSL
    pub tls: String,
}

/// 发送纯文本邮件。`from` 为空时回落 username。
pub async fn send_mail(cfg: &SmtpConfig, to: &str, subject: &str, body: &str) -> Result<(), AppError> {
    // lettre 的 Transport::send 是阻塞 IO，放到 blocking 线程执行避免拖慢 reactor
    let cfg = cfg.clone();
    let to = to.to_string();
    let subject = subject.to_string();
    let body = body.to_string();
    tokio::task::spawn_blocking(move || send_mail_blocking(&cfg, &to, &subject, &body))
        .await
        .map_err(|e| AppError::Internal {
            context: "send mail task join failed".into(),
            source: Some(Box::new(e)),
        })?
}

fn send_mail_blocking(cfg: &SmtpConfig, to: &str, subject: &str, body: &str) -> Result<(), AppError> {
    use lettre::transport::smtp::authentication::Credentials;
    use lettre::transport::smtp::client::Tls;
    use lettre::{Message, SmtpTransport, Transport};

    let email = Message::builder()
        .from(lettre::message::Mailbox::new(
            None,
            cfg.from_or_username()
                .parse()
                .map_err(|e| AppError::Validation(format!("invalid from address: {e}")))?,
        ))
        .to(lettre::message::Mailbox::new(
            None,
            to.parse()
                .map_err(|e| AppError::Validation(format!("invalid recipient address: {e}")))?,
        ))
        .subject(subject)
        .body(body.to_string())
        .map_err(|e| AppError::Internal {
            context: "build mail failed".into(),
            source: Some(Box::new(e)),
        })?;

    let mut builder = if cfg.tls.eq_ignore_ascii_case("SSL") {
        SmtpTransport::relay(&cfg.host)
            .map_err(|e| AppError::Internal {
                context: "smtp relay builder failed".into(),
                source: Some(Box::new(e)),
            })?
            .port(cfg.port)
    } else {
        let mut b = SmtpTransport::builder_dangerous(&cfg.host).port(cfg.port);
        if cfg.tls.eq_ignore_ascii_case("START_TLS") {
            let params = lettre::transport::smtp::client::TlsParameters::new(cfg.host.clone())
                .map_err(|e| AppError::Internal {
                    context: "build tls parameters failed".into(),
                    source: Some(Box::new(e)),
                })?;
            b = b.tls(Tls::Required(params));
        } else {
            b = b.tls(Tls::None);
        }
        b
    };

    if !cfg.username.is_empty() {
        builder = builder.credentials(Credentials::new(
            cfg.username.clone(),
            cfg.password.clone(),
        ));
    }

    let mailer = builder.build();
    mailer
        .send(&email)
        .map_err(|e| AppError::Validation(format!("send mail failed: {e}")))?;
    Ok(())
}

impl SmtpConfig {
    fn from_or_username(&self) -> String {
        if self.from.trim().is_empty() {
            self.username.clone()
        } else {
            self.from.clone()
        }
    }
}
