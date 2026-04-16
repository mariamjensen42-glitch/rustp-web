use actix_web::{dev::{Service, ServiceRequest, ServiceResponse, Transform}, Error, HttpResponse, web, HttpRequest};
use lru::LruCache;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::sync::Mutex;
use std::time::{Duration, Instant};
use std::num::NonZeroUsize;
use futures_util::FutureExt;
use futures_util::future;

#[derive(Debug, Clone)]
pub struct CachedResponse {
    body: Vec<u8>,
    status: u16,
    headers: Vec<(String, String)>,
    timestamp: Instant,
}

pub struct CacheState {
    pub cache: Mutex<LruCache<u64, CachedResponse>>,
    pub ttl: Duration,
}

impl CacheState {
    pub fn new() -> Self {
        Self {
            cache: Mutex::new(LruCache::new(NonZeroUsize::new(1000).unwrap())),
            ttl: Duration::from_secs(300), // 5分钟缓存
        }
    }
}

pub struct CacheMiddleware;

impl<S, B> Transform<S, ServiceRequest> for CacheMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = CacheService<S>;
    type Future = future::Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        future::ok(CacheService { service })
    }
}

pub struct CacheService<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for CacheService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = S::Future;

    fn poll_ready(&self, ctx: &mut std::task::Context<'_>) -> std::task::Poll<Result<(), Self::Error>> {
        self.service.poll_ready(ctx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        // 简化实现，暂时不做缓存
        // 在实际项目中，应该实现完整的缓存逻辑
        self.service.call(req)
    }
}

fn generate_cache_key(req: &actix_web::HttpRequest) -> u64 {
    let mut hasher = DefaultHasher::new();
    req.method().hash(&mut hasher);
    req.uri().path().hash(&mut hasher);
    req.uri().query().hash(&mut hasher);
    hasher.finish()
}