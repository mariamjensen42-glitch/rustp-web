use std::env;
use std::sync::Arc;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message, SmtpTransport, Transport};
use handlebars::Handlebars;
use serde::Serialize;

#[derive(Clone)]
pub struct EmailService {
    smtp_client: Arc<SmtpTransport>,
    template_engine: Handlebars<'static>,
    from_email: String,
}

impl EmailService {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let smtp_server = env::var("SMTP_SERVER").unwrap_or_else(|_| "smtp.gmail.com".to_string());
        let smtp_port = env::var("SMTP_PORT").unwrap_or_else(|_| "587".to_string()).parse()?;
        let smtp_username = env::var("SMTP_USERNAME").expect("SMTP_USERNAME is required");
        let smtp_password = env::var("SMTP_PASSWORD").expect("SMTP_PASSWORD is required");
        let from_email = env::var("FROM_EMAIL").unwrap_or_else(|_| smtp_username.clone());

        let credentials = Credentials::new(smtp_username, smtp_password);
        let smtp_client = SmtpTransport::relay(&smtp_server)?
            .port(smtp_port)
            .credentials(credentials)
            .build();

        let mut template_engine = Handlebars::new();
        template_engine.register_template_string("comment_notification", include_str!("../templates/comment_notification.hbs"))?;
        template_engine.register_template_string("post_status_change", include_str!("../templates/post_status_change.hbs"))?;

        Ok(Self {
            smtp_client: Arc::new(smtp_client),
            template_engine,
            from_email,
        })
    }

    pub async fn send_comment_notification(
        &self,
        to_email: &str,
        post_title: &str,
        comment_author: &str,
        comment_content: &str,
        post_url: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        #[derive(Serialize)]
        struct CommentNotificationData {
            post_title: String,
            comment_author: String,
            comment_content: String,
            post_url: String,
        }

        let data = CommentNotificationData {
            post_title: post_title.to_string(),
            comment_author: comment_author.to_string(),
            comment_content: comment_content.to_string(),
            post_url: post_url.to_string(),
        };

        let html_content = self.template_engine.render("comment_notification", &data)?;

        let email = Message::builder()
            .from(self.from_email.parse()?)
            .to(to_email.parse()?)
            .subject("New Comment on Your Post")
            .body(html_content)?;

        self.smtp_client.send(&email)?;
        Ok(())
    }

    pub async fn send_post_status_change_notification(
        &self,
        to_email: &str,
        post_title: &str,
        old_status: &str,
        new_status: &str,
        post_url: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        #[derive(Serialize)]
        struct PostStatusChangeData {
            post_title: String,
            old_status: String,
            new_status: String,
            post_url: String,
        }

        let data = PostStatusChangeData {
            post_title: post_title.to_string(),
            old_status: old_status.to_string(),
            new_status: new_status.to_string(),
            post_url: post_url.to_string(),
        };

        let html_content = self.template_engine.render("post_status_change", &data)?;

        let email = Message::builder()
            .from(self.from_email.parse()?)
            .to(to_email.parse()?)
            .subject("Post Status Changed")
            .body(html_content)?;

        self.smtp_client.send(&email)?;
        Ok(())
    }

    pub async fn send_test_email(&self, to_email: &str) -> Result<(), Box<dyn std::error::Error>> {
        let email = Message::builder()
            .from(self.from_email.parse()?)
            .to(to_email.parse()?)
            .subject("Test Email")
            .body("<p>This is a test email from the blog API.</p>".to_string())?;

        self.smtp_client.send(&email)?;
        Ok(())
    }
}