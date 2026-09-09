// @generated DTO 实体（来自 openapi components/schemas，serde 使用 camelCase）
//! 全量实体类型，业务进行中按需迁移

pub mod types {
    #![allow(non_snake_case)]

    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
    pub struct ListResponse<T: serde::Serialize> {
        pub items: Vec<T>,
        pub total: String,
    }

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ActionDistributionResponse {
    pub items: Vec<crate::dto::types::DistributionItem>,
}
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Api {
    pub id: i64,
    pub operation: String,
    pub path: String,
    pub method: String,
    pub module: String,
    pub moduleDescription: String,
    pub businessModule: String,
    pub description: String,
    pub scope: String,
    pub status: String,
    pub createdBy: i64,
    pub updatedBy: i64,
    pub deletedBy: i64,
    pub createdAt: String,
    pub updatedAt: String,
    pub deletedAt: String,
}
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ApiAuditLog {
    pub id: i64,
    pub tenantId: i64,
    pub tenantName: String,
    pub userId: i64,
    pub username: String,
    pub ipAddress: String,
    pub geoLocation: crate::dto::types::GeoLocation,
    pub deviceInfo: crate::dto::types::DeviceInfo,
    pub referer: String,
    pub appVersion: String,
    pub httpMethod: String,
    pub path: String,
    pub requestUri: String,
    pub apiModule: String,
    pub apiOperation: String,
    pub apiDescription: String,
    pub requestId: String,
    pub traceId: String,
    pub spanId: String,
    pub latencyMs: i64,
    pub success: bool,
    pub statusCode: i64,
    pub reason: String,
    pub requestHeader: String,
    pub requestBody: String,
    pub response: String,
    pub logHash: String,
    pub signature: String,
    pub createdAt: String,
}
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BatchCreateLanguagesRequest {
    pub items: Vec<crate::dto::types::Language>,
}
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BindContactRequest {
    pub phone: crate::dto::types::BindPhoneRequest,
    pub email: crate::dto::types::BindEmailRequest,
}
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BindEmailRequest {
    pub email: String,
    pub verificationCode: String,
}
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BindPhoneRequest {
    pub phone: String,
    pub code: String,
}
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ChangePasswordRequest {
    pub oldPassword: String,
    pub newPassword: String,
}
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CleanupTenantDataRequest {
    pub id: i64,
}
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ConfirmEnrollMethodRequest {
    pub method: String,
    pub operationId: String,
    pub totpCode: String,
    pub sms: crate::dto::types::SMSVerification,
    pub webauthn: crate::dto::types::WebAuthnAssertion,
    pub backupCode: String,
    pub display: String,
}
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ConfirmEnrollMethodResponse {
    pub success: bool,
    pub credentialId: String,
}
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ControlTaskRequest {
    pub controlType: String,
    pub typeName: String,
}
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CreateApiRequest {
    pub data: crate::dto::types::Api,
}
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CreateDictEntryRequest {
    pub data: crate::dto::types::DictEntry,
}
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CreateDictTypeRequest {
    pub data: crate::dto::types::DictType,
}
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CreateFileRequest {
    pub data: crate::dto::types::File,
}
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CreateInternalMessageCategoryRequest {
    pub data: crate::dto::types::InternalMessageCategory,
}
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CreateLanguageRequest {
    pub data: crate::dto::types::Language,
}
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CreateLoginPolicyRequest {
    pub data: crate::dto::types::LoginPolicy,
}
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CreateMenuRequest {
    pub data: crate::dto::types::Menu,
}
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CreateOrgUnitRequest {
    pub data: crate::dto::types::OrgUnit,
}
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CreatePermissionGroupRequest {
    pub data: crate::dto::types::PermissionGroup,
}
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CreatePermissionRequest {
    pub data: crate::dto::types::Permission,
}
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CreatePlanModuleRequest {
    pub data: crate::dto::types::PlanModule,
}
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CreatePlanQuotaRequest {
    pub data: crate::dto::types::PlanQuota,
}
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CreatePlanRequest {
    pub data: crate::dto::types::Plan,
}
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CreatePositionRequest {
    pub data: crate::dto::types::Position,
}
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CreateRoleRequest {
    pub data: crate::dto::types::Role,
}
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CreateTaskRequest {
    pub data: crate::dto::types::Task,
}
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CreateTenantRequest {
    pub data: crate::dto::types::Tenant,
}
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CreateTenantWithAdminUserRequest {
    pub tenant: crate::dto::types::Tenant,
    pub user: crate::dto::types::User,
    pub password: String,
}
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CreateUserRequest {
    pub data: crate::dto::types::User,
    pub password: String,
}
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DashboardOverviewResponse {
    pub userCount: i64,
    pub roleCount: i64,
    pub todayLoginCount: i64,
    pub todayOperationCount: i64,
}
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DataAccessAuditLog {
    pub id: i64,
    pub tenantId: i64,
    pub tenantName: String,
    pub userId: i64,
    pub username: String,
    pub ipAddress: String,
    pub requestId: String,
    pub dataSource: String,
    pub tableName: String,
    pub dataId: String,
    pub accessType: String,
    pub sqlDigest: String,
    pub sqlText: String,
    pub affectedRows: i64,
    pub latencyMs: i64,
    pub success: bool,
    pub sensitiveLevel: String,
    pub dataMasked: bool,
    pub maskingRules: String,
    pub businessPurpose: String,
    pub dataCategory: String,
    pub dbUser: String,
    pub logHash: String,
    pub signature: String,
    pub createdAt: String,
}
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DeleteNotificationFromInboxRequest {
    pub userId: i64,
    pub recipientIds: Vec<i64>,
}
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DeviceInfo {
    pub clientId: String,
    pub clientName: String,
    pub osName: String,
    pub osVersion: String,
    pub deviceId: String,
    pub deviceType: String,
    pub manufacturer: String,
    pub model: String,
    pub platform: String,
    pub osBuild: String,
    pub appName: String,
    pub appVersion: String,
    pub screenWidth: i64,
    pub screenHeight: i64,
    pub locale: String,
    pub timeZone: String,
    pub networkType: String,
    pub carrier: String,
    pub deviceFingerprint: String,
    pub userAgent: String,
    pub browserName: String,
    pub browserVersion: String,
    pub browserEngine: String,
    pub engineVersion: String,
}
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DictEntry {
    pub id: i64,
    pub typeId: i64,
    pub entryValue: String,
    pub numericValue: i64,
    pub isEnabled: bool,
    pub sortOrder: i64,
    pub i18n: serde_json::Value,
    pub currentI18n: crate::dto::types::DictEntryI18n,
    pub tenantId: i64,
    pub tenantName: String,
    pub createdBy: i64,
    pub updatedBy: i64,
    pub deletedBy: i64,
    pub createdAt: String,
    pub updatedAt: String,
    pub deletedAt: String,
}
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DictEntryI18n {
    pub entryLabel: String,
    pub description: String,
    pub languageCode: String,
    pub languageName: String,
}
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DictType {
    pub id: i64,
    pub typeCode: String,
    pub typeName: String,
    pub isEnabled: bool,
    pub sortOrder: i64,
    pub tenantId: i64,
    pub tenantName: String,
    pub createdBy: i64,
    pub updatedBy: i64,
    pub deletedBy: i64,
    pub createdAt: String,
    pub updatedAt: String,
    pub deletedAt: String,
}
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DisableMFARequest {
    pub credentialId: String,
    pub method: String,
    pub password: String,
    pub totpCode: String,
    pub sms: crate::dto::types::SMSVerification,
    pub webauthn: crate::dto::types::WebAuthnAssertion,
    pub reason: String,
    pub userId: i64,
}
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DistributionItem {
    pub label: String,
    pub count: i64,
}
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DownloadFileResponse {
    pub file: String,
    pub downloadUrl: String,
    pub sourceFileName: String,
    pub mime: String,
    pub size: String,
    pub checksum: String,
    pub storagePath: String,
    pub updatedAt: String,
}
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EditUserPasswordRequest {
    pub userId: i64,
    pub newPassword: String,
}
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EmailVerification {
    pub email: String,
    pub code: String,
}
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EnrolledMethod {
    pub id: String,
    pub method: String,
    pub display: String,
    pub enabled: bool,
    pub createdAt: String,
    pub lastUsedAt: String,
}
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct File {
    pub id: i64,
    pub provider: String,
    pub bucketName: String,
    pub fileDirectory: String,
    pub fileGuid: String,
    pub saveFileName: String,
    pub fileName: String,
    pub extension: String,
    pub size: String,
    pub sizeFormat: String,
    pub linkUrl: String,
    pub contentHash: String,
    pub tenantId: i64,
    pub tenantName: String,
    pub createdBy: i64,
    pub updatedBy: i64,
    pub deletedBy: i64,
    pub createdAt: String,
    pub updatedAt: String,
    pub deletedAt: String,
}
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GenerateCaptchaResponse {
    pub captchaId: String,
    pub imageBase64: String,
}
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GeoLocation {
    pub countryCode: String,
    pub province: String,
    pub city: String,
    pub isp: String,
    pub addressRemark: String,
    pub latitude: String,
    pub longitude: String,
}
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GetMFAStatusResponse {
    pub enabled: bool,
    pub enrolled: Vec<crate::dto::types::EnrolledMethod>,
    pub enforcement: String,
}
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct InfoEntry {
    pub key: String,
    pub value: String,
}
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct InfoSection {
    pub name: String,
    pub entries: Vec<crate::dto::types::InfoEntry>,
}
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct InitialContextResponse {
    pub menus: Vec<crate::dto::types::MenuRouteItem>,
    pub permissions: Vec<String>,
}
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct InternalMessage {
    pub id: i64,
    pub title: String,
    pub content: String,
    pub status: String,
    pub r#type: String,
    pub senderId: i64,
    pub senderName: String,
    pub categoryId: i64,
    pub categoryName: String,
    pub tenantId: i64,
    pub tenantName: String,
    pub createdBy: i64,
    pub updatedBy: i64,
    pub deletedBy: i64,
    pub createdAt: String,
    pub updatedAt: String,
    pub deletedAt: String,
}
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct InternalMessageCategory {
    pub id: i64,
    pub name: String,
    pub code: String,
    pub iconUrl: String,
    pub sortOrder: i64,
    pub isEnabled: bool,
    pub tenantId: i64,
    pub tenantName: String,
    pub createdBy: i64,
    pub updatedBy: i64,
    pub deletedBy: i64,
    pub createdAt: String,
    pub updatedAt: String,
    pub deletedAt: String,
}
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct InternalMessageRecipient {
    pub id: i64,
    pub messageId: i64,
    pub recipientUserId: i64,
    pub status: String,
    pub receivedAt: String,
    pub readAt: String,
    pub title: String,
    pub content: String,
    pub tenantId: i64,
    pub tenantName: String,
    pub createdBy: i64,
    pub updatedBy: i64,
    pub deletedBy: i64,
    pub createdAt: String,
    pub updatedAt: String,
    pub deletedAt: String,
}
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct KratosStatus {
    pub code: f64,
    pub message: String,
    pub reason: String,
    pub metadata: serde_json::Value,
}
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Language {
    pub id: i64,
    pub languageCode: String,
    pub languageName: String,
    pub nativeName: String,
    pub isDefault: bool,
    pub isEnabled: bool,
    pub sortOrder: i64,
    pub createdBy: i64,
    pub updatedBy: i64,
    pub deletedBy: i64,
    pub createdAt: String,
    pub updatedAt: String,
    pub deletedAt: String,
}
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ListApiAuditLogResponse {
    pub items: Vec<crate::dto::types::ApiAuditLog>,
    pub total: String,
}
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ListApiResponse {
    pub items: Vec<crate::dto::types::Api>,
    pub total: String,
}
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ListDataAccessAuditLogResponse {
    pub items: Vec<crate::dto::types::DataAccessAuditLog>,
    pub total: String,
}
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ListDictEntryByTypeCodeResponse {
    pub items: Vec<crate::dto::types::DictEntry>,
}
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ListDictEntryResponse {
    pub items: Vec<crate::dto::types::DictEntry>,
    pub total: String,
}
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ListDictTypeResponse {
    pub items: Vec<crate::dto::types::DictType>,
    pub total: String,
}
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ListEnrolledMethodsResponse {
    pub items: Vec<crate::dto::types::EnrolledMethod>,
}
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ListFileResponse {
    pub items: Vec<crate::dto::types::File>,
    pub total: String,
}
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ListInternalMessageCategoryResponse {
    pub items: Vec<crate::dto::types::InternalMessageCategory>,
    pub total: String,
}
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ListInternalMessageResponse {
    pub items: Vec<crate::dto::types::InternalMessage>,
    pub total: String,
}
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ListLanguageResponse {
    pub items: Vec<crate::dto::types::Language>,
    pub total: String,
}
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ListLoginAuditLogResponse {
    pub items: Vec<crate::dto::types::LoginAuditLog>,
    pub total: String,
}
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ListLoginPolicyResponse {
    pub items: Vec<crate::dto::types::LoginPolicy>,
    pub total: String,
}
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ListMenuResponse {
    pub items: Vec<crate::dto::types::Menu>,
    pub total: String,
}
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ListOperationAuditLogResponse {
    pub items: Vec<crate::dto::types::OperationAuditLog>,
    pub total: String,
}
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ListOrgUnitResponse {
    pub items: Vec<crate::dto::types::OrgUnit>,
    pub total: String,
}
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ListPermissionAuditLogResponse {
    pub items: Vec<crate::dto::types::PermissionAuditLog>,
    pub total: String,
}
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ListPermissionCodeResponse {
    pub codes: Vec<String>,
}
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ListPermissionGroupResponse {
    pub items: Vec<crate::dto::types::PermissionGroup>,
    pub total: String,
}
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ListPermissionResponse {
    pub items: Vec<crate::dto::types::Permission>,
    pub total: String,
}
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ListPlanModuleResponse {
    pub items: Vec<crate::dto::types::PlanModule>,
    pub total: String,
}
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ListPlanQuotaResponse {
    pub items: Vec<crate::dto::types::PlanQuota>,
    pub total: String,
}
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ListPlanResponse {
    pub items: Vec<crate::dto::types::Plan>,
    pub total: String,
}
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ListPolicyEvaluationLogResponse {
    pub items: Vec<crate::dto::types::PolicyEvaluationLog>,
    pub total: String,
}
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ListPositionResponse {
    pub items: Vec<crate::dto::types::Position>,
    pub total: String,
}
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ListRoleResponse {
    pub items: Vec<crate::dto::types::Role>,
    pub total: String,
}
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ListRouteResponse {
    pub items: Vec<crate::dto::types::MenuRouteItem>,
}
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ListTaskResponse {
    pub items: Vec<crate::dto::types::Task>,
    pub total: String,
}
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ListTaskTypeNameResponse {
    pub typeNames: Vec<String>,
}
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ListTenantResponse {
    pub items: Vec<crate::dto::types::Tenant>,
    pub total: String,
}
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ListUserInboxResponse {
    pub items: Vec<crate::dto::types::InternalMessageRecipient>,
    pub total: String,
}
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ListUserResponse {
    pub items: Vec<crate::dto::types::User>,
    pub total: String,
}
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LoginAuditLog {
    pub id: i64,
    pub tenantId: i64,
    pub tenantName: String,
    pub userId: i64,
    pub username: String,
    pub ipAddress: String,
    pub geoLocation: crate::dto::types::GeoLocation,
    pub sessionId: String,
    pub deviceInfo: crate::dto::types::DeviceInfo,
    pub requestId: String,
    pub traceId: String,
    pub actionType: String,
    pub status: String,
    pub failureReason: String,
    pub mfaStatus: String,
    pub loginMethod: String,
    pub riskScore: i64,
    pub riskLevel: String,
    pub riskFactors: Vec<String>,
    pub logHash: String,
    pub signature: String,
    pub createdAt: String,
}
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LoginPolicy {
    pub id: i64,
    pub targetId: i64,
    pub r#type: String,
    pub method: String,
    pub value: String,
    pub reason: String,
    pub tenantId: i64,
    pub tenantName: String,
    pub createdBy: i64,
    pub updatedBy: i64,
    pub deletedBy: i64,
    pub createdAt: String,
    pub updatedAt: String,
    pub deletedAt: String,
}
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LoginRequest {
    pub granttype: String,
    pub clientid: String,
    pub clientsecret: String,
    pub scope: String,
    pub redirecturi: String,
    pub userid: i64,
    pub username: String,
    pub email: String,
    pub mobile: String,
    pub password: String,
    pub refreshtoken: String,
    pub code: String,
    pub clienttype: String,
    pub deviceid: String,
    pub jti: String,
    pub tenantcode: String,
}
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LoginResponse {
    pub tokentype: String,
    pub accesstoken: String,
    pub expiresin: String,
    pub refreshtoken: String,
    pub scope: String,
    pub refreshexpiresin: String,
    pub idtoken: String,
    pub mfaoperationid: String,
}
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LoginTrendResponse {
    pub points: Vec<crate::dto::types::TrendPoint>,
}
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MarkNotificationAsReadRequest {
    pub userId: i64,
    pub recipientIds: Vec<i64>,
}
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Menu {
    pub id: i64,
    pub status: String,
    pub r#type: String,
    pub path: String,
    pub redirect: String,
    pub alias: String,
    pub name: String,
    pub component: String,
    pub meta: crate::dto::types::MenuMeta,
    pub module: String,
    pub parentId: i64,
    pub children: Vec<crate::dto::types::Menu>,
    pub createdBy: i64,
    pub updatedBy: i64,
    pub deletedBy: i64,
    pub createdAt: String,
    pub updatedAt: String,
    pub deletedAt: String,
}
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MenuMeta {
    pub activeIcon: String,
    pub activePath: String,
    pub affixTab: bool,
    pub affixTabOrder: i64,
    pub authority: Vec<String>,
    pub badge: String,
    pub badgeType: String,
    pub badgeVariants: String,
    pub hideChildrenInMenu: bool,
    pub hideInBreadcrumb: bool,
    pub hideInMenu: bool,
    pub hideInTab: bool,
    pub icon: String,
    pub iframeSrc: String,
    pub ignoreAccess: bool,
    pub keepAlive: bool,
    pub link: String,
    pub loaded: bool,
    pub maxNumOfOpenTab: i64,
    pub menuVisibleWithForbidden: bool,
    pub openInNewWindow: bool,
    pub order: i64,
    pub title: String,
}
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MenuRouteItem {
    pub children: Vec<crate::dto::types::MenuRouteItem>,
    pub path: String,
    pub redirect: String,
    pub alias: String,
    pub name: String,
    pub component: String,
    pub meta: crate::dto::types::MenuMeta,
}
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct OperationAuditLog {
    pub id: i64,
    pub tenantId: i64,
    pub tenantName: String,
    pub userId: i64,
    pub username: String,
    pub resourceType: String,
    pub resourceId: String,
    pub action: String,
    pub beforeData: String,
    pub afterData: String,
    pub sensitiveLevel: String,
    pub requestId: String,
    pub traceId: String,
    pub success: bool,
    pub failureReason: String,
    pub ipAddress: String,
    pub geoLocation: crate::dto::types::GeoLocation,
    pub logHash: String,
    pub signature: String,
    pub createdAt: String,
}
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct OrgUnit {
    pub id: i64,
    pub name: String,
    pub code: String,
    pub r#type: String,
    pub path: String,
    pub status: String,
    pub sortOrder: i64,
    pub leaderId: i64,
    pub leaderName: String,
    pub tenantId: i64,
    pub tenantName: String,
    pub remark: String,
    pub description: String,
    pub businessScopes: Vec<String>,
    pub externalId: String,
    pub isLegalEntity: bool,
    pub registrationNumber: String,
    pub taxId: String,
    pub legalEntityOrgId: i64,
    pub address: String,
    pub phone: String,
    pub email: String,
    pub timezone: String,
    pub country: String,
    pub latitude: f64,
    pub longitude: f64,
    pub startAt: String,
    pub endAt: String,
    pub attributes: serde_json::Value,
    pub permissionTags: Vec<String>,
    pub contactUserId: i64,
    pub contactUserName: String,
    pub parentId: i64,
    pub children: Vec<crate::dto::types::OrgUnit>,
    pub createdBy: i64,
    pub updatedBy: i64,
    pub deletedBy: i64,
    pub createdAt: String,
    pub updatedAt: String,
    pub deletedAt: String,
}
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Permission {
    pub id: i64,
    pub name: String,
    pub code: String,
    pub description: String,
    pub status: String,
    pub groupId: i64,
    pub groupName: String,
    pub menuIds: Vec<i64>,
    pub apiIds: Vec<i64>,
    pub createdBy: i64,
    pub updatedBy: i64,
    pub deletedBy: i64,
    pub createdAt: String,
    pub updatedAt: String,
    pub deletedAt: String,
}
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PermissionAuditLog {
    pub id: i64,
    pub tenantId: i64,
    pub operatorId: i64,
    pub operatorName: String,
    pub targetType: String,
    pub targetId: String,
    pub targetName: String,
    pub action: String,
    pub oldValue: String,
    pub newValue: String,
    pub ipAddress: String,
    pub requestId: String,
    pub reason: String,
    pub logHash: String,
    pub signature: String,
    pub createdAt: String,
}
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PermissionGroup {
    pub id: i64,
    pub name: String,
    pub path: String,
    pub module: String,
    pub sortOrder: i64,
    pub status: String,
    pub description: String,
    pub parentId: i64,
    pub children: Vec<crate::dto::types::PermissionGroup>,
    pub createdBy: i64,
    pub updatedBy: i64,
    pub deletedBy: i64,
    pub createdAt: String,
    pub updatedAt: String,
    pub deletedAt: String,
}
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PhoneVerification {
    pub phone: String,
    pub code: String,
}
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Plan {
    pub id: i64,
    pub name: String,
    pub version: String,
    pub expiryPolicy: String,
    pub dataRetentionDays: i64,
    pub description: String,
    pub remark: String,
    pub createdBy: i64,
    pub updatedBy: i64,
    pub deletedBy: i64,
    pub createdAt: String,
    pub updatedAt: String,
    pub deletedAt: String,
}
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PlanModule {
    pub id: i64,
    pub planId: i64,
    pub module: String,
    pub createdBy: i64,
    pub updatedBy: i64,
    pub deletedBy: i64,
    pub createdAt: String,
    pub updatedAt: String,
    pub deletedAt: String,
}
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PlanQuota {
    pub id: i64,
    pub planId: i64,
    pub quotaType: String,
    pub quotaValue: String,
    pub createdBy: i64,
    pub updatedBy: i64,
    pub deletedBy: i64,
    pub createdAt: String,
    pub updatedAt: String,
    pub deletedAt: String,
}
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PolicyEvaluationLog {
    pub id: i64,
    pub tenantId: i64,
    pub userId: i64,
    pub membershipId: i64,
    pub permissionId: i64,
    pub policyId: i64,
    pub requestPath: String,
    pub requestMethod: String,
    pub result: bool,
    pub effectDetails: String,
    pub scopeSql: String,
    pub ipAddress: String,
    pub traceId: String,
    pub evaluationContext: String,
    pub logHash: String,
    pub signature: String,
    pub createdAt: String,
}
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Position {
    pub id: i64,
    pub name: String,
    pub code: String,
    pub headcount: i64,
    pub sortOrder: i64,
    pub status: String,
    pub r#type: String,
    pub remark: String,
    pub description: String,
    pub jobFamily: String,
    pub jobGrade: String,
    pub level: i64,
    pub isKeyPosition: bool,
    pub tenantId: i64,
    pub tenantName: String,
    pub orgUnitId: i64,
    pub orgUnitName: String,
    pub reportsToPositionId: i64,
    pub reportsToPositionName: String,
    pub startAt: String,
    pub endAt: String,
    pub createdBy: i64,
    pub updatedBy: i64,
    pub deletedBy: i64,
    pub createdAt: String,
    pub updatedAt: String,
    pub deletedAt: String,
}
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PresignOption {
    pub method: String,
    pub expireSeconds: i64,
    pub contentType: String,
}
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct QuotaUsage {
    pub quotaType: String,
    pub quotaValue: String,
}
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RedisCacheMonitorInfo {
    pub sections: Vec<crate::dto::types::InfoSection>,
    pub dbSize: String,
    pub slowlog: Vec<crate::dto::types::SlowLogEntry>,
}
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RegisterUserRequest {
    pub username: String,
    pub password: String,
    pub tenantCode: String,
    pub email: String,
    pub clienttype: String,
}
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RegisterUserResponse {
    pub userId: i64,
}
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RestartAllTaskResponse {
    pub count: i64,
}
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RevokeMessageRequest {
    pub messageId: i64,
    pub userId: i64,
}
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Role {
    pub id: i64,
    pub name: String,
    pub code: String,
    pub sortOrder: i64,
    pub status: String,
    pub description: String,
    pub isProtected: bool,
    pub r#type: String,
    pub permissions: Vec<i64>,
    pub tenantId: i64,
    pub tenantName: String,
    pub createdBy: i64,
    pub updatedBy: i64,
    pub deletedBy: i64,
    pub createdAt: String,
    pub updatedAt: String,
    pub deletedAt: String,
}
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SMSResult {
    pub verificationId: String,
    pub smsSent: bool,
    pub maskedPhone: String,
}
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SMSVerification {
    pub verificationId: String,
    pub code: String,
}
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SendMessageRequest {
    pub r#type: String,
    pub recipientUserId: i64,
    pub conversationId: i64,
    pub categoryId: i64,
    pub targetUserIds: Vec<i64>,
    pub targetAll: bool,
    pub title: String,
    pub content: String,
}
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SendMessageResponse {
    pub messageId: i64,
}
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SlowLogEntry {
    pub id: String,
    pub createdAt: String,
    pub durationUsec: String,
    pub args: Vec<String>,
    pub clientAddr: String,
    pub clientName: String,
}
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct StartEnrollMethodRequest {
    pub method: String,
    pub phone: String,
    pub email: String,
}
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct StartEnrollMethodResponse {
    pub totp: crate::dto::types::TOTPResult,
    pub sms: crate::dto::types::SMSResult,
    pub webauthn: crate::dto::types::WebAuthnResult,
    pub expiresAt: String,
    pub operationId: String,
}
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct StatusDistributionResponse {
    pub items: Vec<crate::dto::types::DistributionItem>,
}
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct StorageObject {
    pub bucketName: String,
    pub fileDirectory: String,
    pub objectName: String,
}
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SyncMenusRequest {
    pub items: Vec<crate::dto::types::Menu>,
}
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TOTPResult {
    pub secret: String,
    pub otpAuthUrl: String,
    pub qrCodeDataUri: String,
}
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Task {
    pub id: i64,
    pub r#type: String,
    pub typeName: String,
    pub taskPayload: String,
    pub cronSpec: String,
    pub taskOptions: crate::dto::types::TaskOption,
    pub enable: bool,
    pub remark: String,
    pub tenantId: i64,
    pub createdBy: i64,
    pub updatedBy: i64,
    pub deletedBy: i64,
    pub createdAt: String,
    pub updatedAt: String,
    pub deletedAt: String,
}
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TaskOption {
    pub maxRetry: i64,
    pub timeout: String,
    pub deadline: String,
    pub processIn: String,
    pub processAt: String,
    pub uniqueTTL: String,
    pub retention: String,
    pub group: String,
    pub taskID: String,
}
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Tenant {
    pub id: i64,
    pub name: String,
    pub code: String,
    pub domain: String,
    pub logoUrl: String,
    pub industry: String,
    pub r#type: String,
    pub remark: String,
    pub adminUserId: i64,
    pub adminUserName: String,
    pub subscriptionAt: String,
    pub unsubscribeAt: String,
    pub expiredAt: String,
    pub subscriptionPlan: String,
    pub planId: i64,
    pub memberCount: i64,
    pub status: String,
    pub auditStatus: String,
    pub createdBy: i64,
    pub updatedBy: i64,
    pub deletedBy: i64,
    pub createdAt: String,
    pub updatedAt: String,
    pub deletedAt: String,
}
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TenantExistsResponse {
    pub exist: bool,
}
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TenantUsage {
    pub tenantId: i64,
    pub userCount: String,
    pub storageUsedBytes: String,
    pub apiCallCount: String,
    pub planId: i64,
    pub planName: String,
    pub quotas: Vec<crate::dto::types::QuotaUsage>,
}
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TrendPoint {
    pub date: String,
    pub count: i64,
}
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct UpdateApiRequest {
    pub id: i64,
    pub data: crate::dto::types::Api,
    pub updateMask: String,
    pub allowMissing: bool,
}
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct UpdateDictEntryRequest {
    pub id: i64,
    pub data: crate::dto::types::DictEntry,
    pub updateMask: String,
    pub allowMissing: bool,
}
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct UpdateDictTypeRequest {
    pub id: i64,
    pub data: crate::dto::types::DictType,
    pub updateMask: String,
    pub allowMissing: bool,
}
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct UpdateFileRequest {
    pub id: i64,
    pub data: crate::dto::types::File,
    pub updateMask: String,
    pub allowMissing: bool,
}
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct UpdateInternalMessageCategoryRequest {
    pub id: i64,
    pub data: crate::dto::types::InternalMessageCategory,
    pub updateMask: String,
    pub allowMissing: bool,
}
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct UpdateInternalMessageRequest {
    pub id: i64,
    pub data: crate::dto::types::InternalMessage,
    pub updateMask: String,
    pub allowMissing: bool,
}
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct UpdateLanguageRequest {
    pub id: i64,
    pub data: crate::dto::types::Language,
    pub updateMask: String,
    pub allowMissing: bool,
}
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct UpdateLoginPolicyRequest {
    pub id: i64,
    pub data: crate::dto::types::LoginPolicy,
    pub updateMask: String,
    pub allowMissing: bool,
}
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct UpdateMenuRequest {
    pub id: i64,
    pub data: crate::dto::types::Menu,
    pub updateMask: String,
    pub allowMissing: bool,
}
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct UpdateOrgUnitRequest {
    pub id: i64,
    pub data: crate::dto::types::OrgUnit,
    pub updateMask: String,
    pub allowMissing: bool,
}
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct UpdatePermissionGroupRequest {
    pub id: i64,
    pub data: crate::dto::types::PermissionGroup,
    pub updateMask: String,
    pub allowMissing: bool,
}
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct UpdatePermissionRequest {
    pub id: i64,
    pub data: crate::dto::types::Permission,
    pub updateMask: String,
    pub allowMissing: bool,
}
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct UpdatePlanModuleRequest {
    pub id: i64,
    pub data: crate::dto::types::PlanModule,
    pub updateMask: String,
    pub allowMissing: bool,
}
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct UpdatePlanQuotaRequest {
    pub id: i64,
    pub data: crate::dto::types::PlanQuota,
    pub updateMask: String,
    pub allowMissing: bool,
}
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct UpdatePlanRequest {
    pub id: i64,
    pub data: crate::dto::types::Plan,
    pub updateMask: String,
    pub allowMissing: bool,
}
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct UpdatePositionRequest {
    pub id: i64,
    pub data: crate::dto::types::Position,
    pub updateMask: String,
    pub allowMissing: bool,
}
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct UpdateRoleRequest {
    pub id: i64,
    pub data: crate::dto::types::Role,
    pub updateMask: String,
    pub allowMissing: bool,
}
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct UpdateTaskRequest {
    pub id: i64,
    pub data: crate::dto::types::Task,
    pub updateMask: String,
    pub allowMissing: bool,
}
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct UpdateTenantRequest {
    pub id: i64,
    pub data: crate::dto::types::Tenant,
    pub updateMask: String,
    pub allowMissing: bool,
}
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct UpdateUserRequest {
    pub id: i64,
    pub data: crate::dto::types::User,
    pub password: String,
    pub updateMask: String,
    pub allowMissing: bool,
}
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct UploadAvatarRequest {
    pub imageBase64: String,
    pub imageUrl: String,
}
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct UploadAvatarResponse {
    pub url: String,
}
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct UploadFileRequest {
    pub storageObject: crate::dto::types::StorageObject,
    pub file: String,
    pub presign: crate::dto::types::PresignOption,
    pub sourceFileName: String,
    pub mime: String,
    pub size: String,
}
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct UploadFileResponse {
    pub objectName: String,
    pub presignedUrl: String,
}
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct User {
    pub id: i64,
    pub tenantId: i64,
    pub tenantName: String,
    pub orgUnitId: i64,
    pub orgUnitIds: Vec<i64>,
    pub orgUnitName: String,
    pub orgUnitNames: Vec<String>,
    pub positionId: i64,
    pub positionIds: Vec<i64>,
    pub positionName: String,
    pub positionNames: Vec<String>,
    pub roleId: i64,
    pub roleIds: Vec<i64>,
    pub roles: Vec<String>,
    pub roleNames: Vec<String>,
    pub username: String,
    pub nickname: String,
    pub realname: String,
    pub avatar: String,
    pub email: String,
    pub mobile: String,
    pub telephone: String,
    pub gender: String,
    pub address: String,
    pub region: String,
    pub description: String,
    pub remark: String,
    pub lastLoginAt: String,
    pub lastLoginIp: String,
    pub status: String,
    pub lockedUntil: String,
    pub createdBy: i64,
    pub updatedBy: i64,
    pub deletedBy: i64,
    pub createdAt: String,
    pub updatedAt: String,
    pub deletedAt: String,
}
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct UserExistsResponse {
    pub exist: bool,
}
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct VerifyCaptchaRequest {
    pub captchaId: String,
    pub userInput: String,
}
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct VerifyCaptchaResponse {
    pub valid: bool,
}
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct VerifyContactRequest {
    pub phone: crate::dto::types::PhoneVerification,
    pub email: crate::dto::types::EmailVerification,
    pub verificationId: String,
}
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct VerifyMFAChallengeRequest {
    pub operationId: String,
    pub totpCode: String,
    pub sms: crate::dto::types::SMSVerification,
    pub webauthn: crate::dto::types::WebAuthnAssertion,
    pub backupCode: String,
}
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WebAuthnAssertion {
    pub id: String,
    pub clientDataJson: String,
    pub authenticatorData: String,
    pub signature: String,
    pub userHandle: String,
}
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WebAuthnResult {
    pub challenge: String,
    pub optionsJson: String,
    pub rpId: String,
}
}
