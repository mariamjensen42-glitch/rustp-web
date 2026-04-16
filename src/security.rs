use actix_web::{dev::ServiceRequest, dev::ServiceResponse, Error, web::Data, dev::Transform, web}; 
use std::sync::{Mutex, Arc}; 
use std::collections::HashMap; 
use std::time::{Instant, Duration}; 
use chrono::Utc; 
use futures_util::future::{ok, FutureExt, LocalBoxFuture}; 

// 访问频率限制配置
const RATE_LIMIT_WINDOW: Duration = Duration::from_secs(60); // 1分钟窗口
const RATE_LIMIT_MAX_REQUESTS: usize = 100; // 每分钟最大请求数

// 恶意请求检测配置
const MALICIOUS_PATTERNS: &[&str] = &[
    "../", // 路径遍历
    "DROP TABLE", // SQL注入
    "' OR 1=1", // SQL注入
    "<script>", // XSS
    "javascript:", // XSS
]; 

// 访问记录结构
#[derive(Debug)] 
struct RateLimitRecord {
    requests: Vec<Instant>,
} 

// 安全日志结构
#[derive(Debug, Clone, serde::Serialize)] 
pub struct SecurityLog {
    pub ip: String,
    pub timestamp: chrono::DateTime<Utc>,
    pub method: String,
    pub path: String,
    pub status: u16,
    pub action: String,
    pub details: String,
} 

// 扩展 AppState 以包含安全相关数据
pub struct SecurityState {
    pub rate_limit: Mutex<HashMap<String, RateLimitRecord>>,
    pub security_logs: Mutex<Vec<SecurityLog>>,
} 

impl SecurityState {
    pub fn new() -> Self {
        Self {
            rate_limit: Mutex::new(HashMap::new()),
            security_logs: Mutex::new(Vec::new()),
        }
    }

    // 检查并更新访问频率限制
    pub fn check_rate_limit(&self, ip: &str) -> bool {
        let mut rate_limit = self.rate_limit.lock().unwrap();
        let now = Instant::now();

        // 获取或创建IP的访问记录
        let record = rate_limit.entry(ip.to_string()).or_insert(RateLimitRecord {
            requests: Vec::new(),
        });

        // 清理过期的请求记录
        record.requests.retain(|&time| now.duration_since(time) < RATE_LIMIT_WINDOW);

        // 检查是否超过限制
        if record.requests.len() >= RATE_LIMIT_MAX_REQUESTS {
            false
        } else {
            // 添加当前请求
            record.requests.push(now);
            true
        }
    }

    // 检测恶意请求
    pub fn detect_malicious_request(&self, method: &str, path: &str, body: Option<&str>) -> Option<String> {
        // 检查路径中的恶意模式
        for pattern in MALICIOUS_PATTERNS {
            if path.contains(pattern) {
                return Some(format!("Malicious pattern '{}' detected in path", pattern));
            }
        }

        // 检查请求体中的恶意模式
        if let Some(body) = body {
            for pattern in MALICIOUS_PATTERNS {
                if body.contains(pattern) {
                    return Some(format!("Malicious pattern '{}' detected in body", pattern));
                }
            }
        }

        None
    }

    // 记录安全日志
    pub fn log_security_event(&self, ip: &str, method: &str, path: &str, status: u16, action: &str, details: &str) {
        let mut logs = self.security_logs.lock().unwrap();
        logs.push(SecurityLog {
            ip: ip.to_string(),
            timestamp: Utc::now(),
            method: method.to_string(),
            path: path.to_string(),
            status,
            action: action.to_string(),
            details: details.to_string(),
        });

        // 限制日志数量，防止内存溢出
        let log_count = logs.len();
        if log_count > 10000 {
            logs.drain(0..log_count - 10000);
        }
    }
}
// 安全中间件
pub struct SecurityMiddleware;

impl<S, B> Transform<S, ServiceRequest> for SecurityMiddleware
where
    S: actix_web::dev::Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = SecurityMiddlewareService<S>;
    type Future = LocalBoxFuture<'static, Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(SecurityMiddlewareService { service: Arc::new(service) }).boxed_local()
    }
}

pub struct SecurityMiddlewareService<S> {
    service: Arc<S>,
}

impl<S, B> actix_web::dev::Service<ServiceRequest> for SecurityMiddlewareService<S>
where
    S: actix_web::dev::Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&self, cx: &mut std::task::Context<'_>) -> std::task::Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let ip = req.connection_info().realip_remote_addr().unwrap_or("unknown").to_string();
        let method = req.method().as_str().to_string();
        let path = req.path().to_string();
        let service = self.service.clone();

        // 先获取安全状态，避免借用冲突
        let security_state = req.app_data::<Data<SecurityState>>().unwrap();
        let security_state_clone = security_state.clone();

        async move {
            // 检查访问频率限制
            if !security_state_clone.check_rate_limit(&ip) {
                security_state_clone.log_security_event(
                    &ip, 
                    &method, 
                    &path, 
                    429, 
                    "Rate limit exceeded", 
                    &format!("IP {} exceeded rate limit", ip)
                );
                return Err(actix_web::error::ErrorTooManyRequests("Too many requests"));
            }

            // 检测恶意请求
            if let Some(details) = security_state_clone.detect_malicious_request(&method, &path, None) {
                security_state_clone.log_security_event(
                    &ip, 
                    &method, 
                    &path, 
                    400, 
                    "Malicious request detected", 
                    &details
                );
                return Err(actix_web::error::ErrorBadRequest("Malicious request detected"));
            }

            // 继续处理请求
            let response: ServiceResponse<B> = service.call(req).await?;

            // 记录正常请求
            security_state_clone.log_security_event(
                &ip, 
                &method, 
                &path, 
                response.status().as_u16(), 
                "Request processed", 
                &format!("Status: {}", response.status())
            );

            Ok(response)
        }
        .boxed_local()
    }
}

// 获取安全日志的处理函数
pub async fn get_security_logs(state: web::Data<SecurityState>) -> impl actix_web::Responder {
    let logs = state.security_logs.lock().unwrap();
    actix_web::HttpResponse::Ok().json(logs.clone())
}
