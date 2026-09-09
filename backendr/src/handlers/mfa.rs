// mfa 模块 handlers（骨架，业务逻辑待填充）
use axum::extract::State;
use crate::state::AppState;
use crate::error::AppError;

pub async fn mfa_disable_m_f_a(_state: State<AppState>) -> Result<axum::response::Response, AppError> {
    // POST /mfa/disable  禁用/移除已注册 凭证。 注意： 生成器不支持 请求体（ 只 ）， 而 生成器默认把 序列化为 ——为两端一致改用 + 。
    Err(AppError::NotImplemented)
}

pub async fn mfa_confirm_enroll_method(_state: State<AppState>) -> Result<axum::response::Response, AppError> {
    // POST /mfa/enroll/confirm  确认注册 方法（提交首码完成绑定）
    Err(AppError::NotImplemented)
}

pub async fn mfa_start_enroll_method(_state: State<AppState>) -> Result<axum::response::Response, AppError> {
    // POST /mfa/enroll/start  开始注册 方法（返回 /，仅 本轮实现）
    Err(AppError::NotImplemented)
}

pub async fn mfa_list_enrolled_methods(_state: State<AppState>) -> Result<axum::response::Response, AppError> {
    // GET /mfa/methods  列出已注册的 凭证
    Err(AppError::NotImplemented)
}

pub async fn mfa_get_m_f_a_status(_state: State<AppState>) -> Result<axum::response::Response, AppError> {
    // GET /mfa/status  查询当前登录用户 总览
    Err(AppError::NotImplemented)
}

pub async fn mfa_verify_m_f_a_challenge(_state: State<AppState>) -> Result<axum::response::Response, AppError> {
    // POST /mfa/verify  验证登录 挑战。通过则返回 （含真 ）。 免鉴权：登录流程在密码校验通过、待二次验证阶段调用。
    Err(AppError::NotImplemented)
}

pub async fn mfa_revoke_m_f_a_device(_state: State<AppState>) -> Result<axum::response::Response, AppError> {
    // DELETE /mfa/{credentialId}  撤销指定 凭证（按 ）。 当前无前端调用方： 请求体在 生成器（恒 ）与 生成器 （发 ）之间不一致，贸然对接会静默丢参——需要时应改 （参见 ）。
    Err(AppError::NotImplemented)
}
