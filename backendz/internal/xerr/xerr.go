// Package xerr 统一业务错误定义。
// 错误类型使用 github.com/zeromicro/x/errors.CodeMsg，配合 handler 模板中
// xhttp.JsonBaseResponseCtx 的 {code,msg,data} 封装格式输出。
package xerr

import (
	"fmt"

	"github.com/zeromicro/x/errors"
)

// 通用业务错误码（10000 起，各业务模块可自行追加）。
const (
	OK                    = 0
	ServerError           = 10001 // 服务器内部错误
	BadRequest            = 10002 // 请求参数错误
	Unauthorized          = 10003 // 未认证
	Forbidden             = 10004 // 无权限
	NotFound              = 10005 // 资源不存在
	TooManyRequests       = 10006 // 请求过于频繁
	InvalidPassword       = 10007 // 用户名或密码错误
	InvalidGrantType      = 10008 // 非法授权类型
	IncorrectRefreshToken = 10009 // refresh token 无效
	InvalidCaptcha        = 10010 // 验证码错误
)

// New 返回带业务码的错误。
func New(code int, msg string) error {
	return errors.New(code, msg)
}

func Newf(code int, format string, a ...any) error {
	return errors.New(code, fmt.Sprintf(format, a...))
}

func ServerErrorMsg(msg string) error { return New(ServerError, msg) }
func BadRequestMsg(msg string) error   { return New(BadRequest, msg) }
func UnauthorizedMsg(msg string) error { return New(Unauthorized, msg) }
func ForbiddenMsg(msg string) error    { return New(Forbidden, msg) }
func NotFoundMsg(msg string) error     { return New(NotFound, msg) }

func InvalidPasswordMsg() error             { return New(InvalidPassword, "invalid username or password") }
func InvalidGrantTypeMsg() error            { return New(InvalidGrantType, "invalid grant type") }
func IncorrectRefreshTokenMsg() error       { return New(IncorrectRefreshToken, "invalid refresh token") }
func InvalidCaptchaMsg() error              { return New(InvalidCaptcha, "invalid or missing captcha") }