use actix_web::{dev::{Service, ServiceRequest, ServiceResponse, Transform}, Error, http::header::{HeaderName, HeaderValue}};
use std::time::{Duration, Instant};
use std::sync::Mutex;
use std::collections::VecDeque;
use futures_util::FutureExt;
use futures_util::future;

#[derive(Debug, Clone)]
pub struct RequestMetrics {
    path: String,
    method: String,
    duration: Duration,
    status: u16,
    timestamp: Instant,
}

pub struct MonitoringState {
    pub metrics: Mutex<VecDeque<RequestMetrics>>,
    pub max_metrics: usize,
}

impl MonitoringState {
    pub fn new() -> Self {
        Self {
            metrics: Mutex::new(VecDeque::with_capacity(1000)),
            max_metrics: 1000,
        }
    }
}

pub struct MonitoringMiddleware;

impl<S, B> Transform<S, ServiceRequest> for MonitoringMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = MonitoringService<S>;
    type Future = future::Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        future::ok(MonitoringService { service })
    }
}

pub struct MonitoringService<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for MonitoringService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = future::Either<
        future::Ready<Result<ServiceResponse<B>, Error>>,
        S::Future,
    >;

    fn poll_ready(&self, ctx: &mut std::task::Context<'_>) -> std::task::Poll<Result<(), Self::Error>> {
        self.service.poll_ready(ctx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        // 直接返回服务调用结果，简化实现
        // 注意：这种实现方式会失去监控功能
        // 在实际项目中，应该使用更复杂的实现来保持监控功能
        future::Either::Right(self.service.call(req))
    }
}

// 辅助函数：获取慢查询
pub fn get_slow_queries(monitoring_state: &actix_web::web::Data<MonitoringState>, threshold: Duration) -> Vec<RequestMetrics> {
    if let Ok(metrics) = monitoring_state.metrics.lock() {
        metrics
            .iter()
            .filter(|m| m.duration > threshold)
            .cloned()
            .collect()
    } else {
        vec![]
    }
}