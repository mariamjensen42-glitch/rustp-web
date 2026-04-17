use actix_web::{dev::{Service, ServiceRequest, ServiceResponse, Transform}, Error, HttpResponse, web};
use std::sync::Arc;
use std::collections::HashMap;
use std::time::{Duration, Instant};
use futures_util::future::LocalBoxFuture;

#[derive(Clone)]
pub struct RateLimiter {
    store: Arc<tokio::sync::Mutex<HashMap<String, (usize, Instant)>>>,
    max_requests: usize,
    window: Duration,
}

impl RateLimiter {
    pub fn new(max_requests: usize, window: Duration) -> Self {
        Self {
            store: Arc::new(tokio::sync::Mutex::new(HashMap::new())),
            max_requests,
            window,
        }
    }
}

impl<S, B> Transform<S, ServiceRequest> for RateLimiter
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = RateLimitMiddleware<S>;
    type InitError = ();
    type Future = LocalBoxFuture<'static, Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        let store = self.store.clone();
        let max_requests = self.max_requests;
        let window = self.window;

        Box::pin(async move {
            Ok(RateLimitMiddleware {
                service: Arc::new(service),
                store,
                max_requests,
                window,
            })
        })
    }
}

pub struct RateLimitMiddleware<S> {
    service: Arc<S>,
    store: Arc<tokio::sync::Mutex<HashMap<String, (usize, Instant)>>>,
    max_requests: usize,
    window: Duration,
}

impl<S, B> Service<ServiceRequest> for RateLimitMiddleware<S>
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
        let store = self.store.clone();
        let max_requests = self.max_requests;
        let window = self.window;
        let service = self.service.clone();

        Box::pin(async move {
            // 先获取路径，再获取IP地址，避免借用冲突
            let path = req.path().to_string();
            let ip = req.connection_info().peer_addr().unwrap_or("unknown").to_string();
            let key = format!("{}:{}", ip, path);

            let mut store = store.lock().await;
            let now = Instant::now();

            if let Some((count, start)) = store.get_mut(&key) {
                if now.duration_since(*start) > window {
                    *count = 1;
                    *start = now;
                } else {
                    *count += 1;
                    if *count > max_requests {
                        // 直接返回错误，不使用into_response
                        return Err(actix_web::error::ErrorTooManyRequests("Too many requests"));
                    }
                }
            } else {
                store.insert(key, (1, now));
            }

            drop(store);
            service.call(req).await
        })
    }
}
