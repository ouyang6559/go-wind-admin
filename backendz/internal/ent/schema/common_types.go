package schema

// 本文件定义 ent JSON 字段所用的共享结构体类型。
// 原版 backend 使用 go-wind-admin 私有 proto 生成的类型（auditV1.GeoLocation 等）；
// backendz 不依赖后端 proto 包，故内联为普通 Go 结构体。
// json tag 与 proto json_name 保持一致，保证与既有数据库存储的 JSON 双向兼容。

// GeoLocation 地理位置（对应 backend audit.GeoLocation）
type GeoLocation struct {
	CountryCode   string `json:"countryCode,omitempty"`
	Province      string `json:"province,omitempty"`
	City          string `json:"city,omitempty"`
	ISP           string `json:"isp,omitempty"`
	AddressRemark string `json:"addressRemark,omitempty"`
	Latitude      *int64 `json:"latitude,omitempty"`
	Longitude     *int64 `json:"longitude,omitempty"`
}

// DeviceType 设备类型（对应 backend audit.DeviceInfo.DeviceType）
type DeviceType int32

const (
	DeviceTypeUnspecified DeviceType = 0 // 未指定
	DeviceDesktop         DeviceType = 1 // 台式/笔记本
	DeviceMobile          DeviceType = 2 // 手机
	DeviceTablet          DeviceType = 3 // 平板
	DeviceBot             DeviceType = 4 // 机器人/爬虫
	DeviceOther           DeviceType = 5 // 其他
)

// DeviceInfo 设备信息（对应 backend audit.DeviceInfo）
type DeviceInfo struct {
	ClientID          string      `json:"clientId,omitempty"`
	ClientName        string      `json:"clientName,omitempty"`
	OSName            string      `json:"osName,omitempty"`
	OSVersion         string      `json:"osVersion,omitempty"`
	DeviceID          string      `json:"deviceId,omitempty"`
	DeviceType        *DeviceType `json:"deviceType,omitempty"`
	Manufacturer      string      `json:"manufacturer,omitempty"`
	Model             string      `json:"model,omitempty"`
	Platform          string      `json:"platform,omitempty"`
	OSBuild           string      `json:"osBuild,omitempty"`
	AppName           string      `json:"appName,omitempty"`
	AppVersion        string      `json:"appVersion,omitempty"`
	ScreenWidth       *uint32     `json:"screenWidth,omitempty"`
	ScreenHeight      *uint32     `json:"screenHeight,omitempty"`
	Locale            string      `json:"locale,omitempty"`
	Timezone          string      `json:"timeZone,omitempty"`
	NetworkType       string      `json:"networkType,omitempty"`
	Carrier           string      `json:"carrier,omitempty"`
	DeviceFingerprint string      `json:"deviceFingerprint,omitempty"`
	UserAgent         string      `json:"userAgent,omitempty"`
	BrowserName       string      `json:"browserName,omitempty"`
	BrowserVersion    string      `json:"browserVersion,omitempty"`
	BrowserEngine     string      `json:"browserEngine,omitempty"`
	EngineVersion     string      `json:"engineVersion,omitempty"`
}

// MenuMeta 路由元信息（对应 backend permission.MenuMeta）
type MenuMeta struct {
	ActiveIcon                string   `json:"activeIcon,omitempty"`
	ActivePath                string   `json:"activePath,omitempty"`
	AffixTab                  *bool    `json:"affixTab,omitempty"`
	AffixTabOrder             *int32   `json:"affixTabOrder,omitempty"`
	Authority                 []string `json:"authority,omitempty"`
	Badge                     string   `json:"badge,omitempty"`
	BadgeType                 string   `json:"badgeType,omitempty"`
	BadgeVariants             string   `json:"badgeVariants,omitempty"`
	HideChildrenInMenu        *bool    `json:"hideChildrenInMenu,omitempty"`
	HideInBreadcrumb          *bool    `json:"hideInBreadcrumb,omitempty"`
	HideInMenu                *bool    `json:"hideInMenu,omitempty"`
	HideInTab                 *bool    `json:"hideInTab,omitempty"`
	Icon                      string   `json:"icon,omitempty"`
	IframeSrc                 string   `json:"iframeSrc,omitempty"`
	IgnoreAccess              *bool    `json:"ignoreAccess,omitempty"`
	KeepAlive                 *bool    `json:"keepAlive,omitempty"`
	Link                      string   `json:"link,omitempty"`
	Loaded                    *bool    `json:"loaded,omitempty"`
	MaxNumOfOpenTab           *int32   `json:"maxNumOfOpenTab,omitempty"`
	MenuVisibleWithForbidden  *bool    `json:"menuVisibleWithForbidden,omitempty"`
	OpenInNewWindow           *bool    `json:"openInNewWindow,omitempty"`
	Order                     *int32   `json:"order,omitempty"`
	Title                     string   `json:"title,omitempty"`
}

// PermissionDelta 权限点覆盖（对应 backend permission.RoleOverride.PermissionDelta）
type PermissionDelta struct {
	AddedPermissions   []string `json:"addedPermissions,omitempty"`
	RemovedPermissions []string `json:"removedPermissions,omitempty"`
}

// RoleSecurityPolicy 角色安全策略覆盖
type RoleSecurityPolicy struct {
	ForceMFA    bool     `json:"forceMfa,omitempty"`
	IPAllowList []string `json:"ipAllowList,omitempty"`
}

// RoleOverride 角色模板覆盖（对应 backend permission.RoleOverride）
type RoleOverride struct {
	Permissions      *PermissionDelta    `json:"permissions,omitempty"`
	DisplayName      string              `json:"displayName,omitempty"`
	Description      string              `json:"description,omitempty"`
	ExtendedSettings map[string]string   `json:"extendedSettings,omitempty"`
	SecurityPolicy   *RoleSecurityPolicy `json:"securityPolicy,omitempty"`
}

// TaskOption 任务选项（对应 backend task.TaskOption）。
// proto 中的 Duration 序列化为 "10s" 格式字符串、Timestamp 序列化为 RFC3339 字符串，故用 string 表示。
type TaskOption struct {
	MaxRetry   *uint32            `json:"maxRetry,omitempty"`
	Timeout    string             `json:"timeout,omitempty"`
	Deadline   string             `json:"deadline,omitempty"`
	ProcessIn  string             `json:"processIn,omitempty"`
	ProcessAt  string             `json:"processAt,omitempty"`
	UniqueTTL  string             `json:"uniqueTTL,omitempty"`
	Retention  string             `json:"retention,omitempty"`
	Group      string             `json:"group,omitempty"`
	TaskID     string             `json:"taskID,omitempty"`
}