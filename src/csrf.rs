use actix_web::{dev::{Service, ServiceRequest, ServiceResponse, Transform}, Error, HttpResponse, web, HttpRequest};
use std::sync::Arc;
use futures_util::future::LocalBoxFuture;
use rand::Rng;
use actix_session::{Session, SessionExt};

#[derive(Clone)]
pub struct CsrfProtection {
    secret: String,
}

impl CsrfProtection {
    pub fn new(secret: &str) -> Self {
        Self {
            secret: secret.to_string(),
        }
    }

    pub fn generate_token(&self) -> String {
        let mut rng = rand::thread_rng();
        let random: Vec<u8> = (0..32).map(|_| rng.r#gen()).collect();
        base64::encode(random)
    }

    pub fn validate_token(&self, token: &str, session: &Session) -> bool {
        if let Ok(Some(session_token)) = session.get::<String>("csrf_token") {
            token == session_token
        } else {
            false
        }
    }
}

impl<S, B> Transform<S, ServiceRequest> for CsrfProtection
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = CsrfMiddleware<S>;
    type InitError = ();
    type Future = LocalBoxFuture<'static, Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        let secret = self.secret.clone();
        Box::pin(async move {
            Ok(CsrfMiddleware {
                service: Arc::new(service),
                secret,
            })
        })
    }
}

pub struct CsrfMiddleware<S> {
    service: Arc<S>,
    secret: String,
}

impl<S, B> Service<ServiceRequest> for CsrfMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(
        &self,
        ctx: &mut ::core::task::Context<'_>,
    ) -> ::core::task::Poll<Result<(), Self::Error>> {
        self.service.poll_ready(ctx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let service = self.service.clone();
        let _secret = self.secret.clone();

        Box::pin(async move {
            let method = req.method();
            let path = req.path();

            // 跳过不需要CSRF保护的路径
            if path.starts_with("/api/auth/") || method == &actix_web::http::Method::GET {
                return service.call(req).await;
            }

            // 获取session
            let session = req.get_session();
            // 检查CSRF令牌
            if let Some(token) = req.headers().get("X-CSRF-Token").and_then(|h| h.to_str().ok()) {
                if let Ok(Some(session_token)) = session.get::<String>("csrf_token") {
                    if token == session_token {
                        return service.call(req).await;
                    }
                }
            }

            // 令牌无效或不存在
            Err(actix_web::error::ErrorForbidden("CSRF token invalid"))
        })
    }
}

// 辅助函数：获取或生成CSRF令牌
pub fn get_csrf_token(session: &Session) -> String {
    if let Ok(Some(token)) = session.get::<String>("csrf_token") {
        token
    } else {
        let mut rng = rand::thread_rng();
        let random: Vec<u8> = (0..32).map(|_| rng.r#gen()).collect();
        let token = base64::encode(random);
        session.insert("csrf_token", &token).unwrap();
        token
    }
}
