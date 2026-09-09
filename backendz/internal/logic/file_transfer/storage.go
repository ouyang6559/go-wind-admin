package file_transfer

import (
	"crypto/sha256"
	"encoding/hex"
	"encoding/base64"
	"os"
	"path/filepath"
	"strings"
)

// uploadBaseDir 本地临时文件存储根目录。
// backendz 网关暂未接入对象存储（OSS/MinIO），上传内容作为最小可用兜底写入本地临时目录，
// 并仅落库文件元数据；如需生产级存储，请在此处替换为 OSS/MinIO 客户端。
func uploadBaseDir() string {
	return filepath.Join(os.TempDir(), "gowind-uploads")
}

// decodeFileContent 解码请求体中的文件内容字符串（base64 编码）。空串返回空字节。
func decodeFileContent(s string) ([]byte, error) {
	if s == "" {
		return nil, nil
	}
	return base64.StdEncoding.DecodeString(s)
}

// storeLocalFile 将文件内容写入本地临时目录，返回相对存储路径（/目录/文件名）。
// dir 为空时落到 uploadBaseDir 根；objectName 需安全（去除路径穿越）。
func storeLocalFile(dir, objectName string, content []byte) (string, error) {
	if dir == "" {
		dir = "default"
	}
	dir = strings.Trim(strings.ReplaceAll(filepath.ToSlash(dir), "..", ""), "/")
	objectName = strings.TrimLeft(objectName, "/")

	fullDir := filepath.Join(uploadBaseDir(), filepath.FromSlash(dir))
	if err := os.MkdirAll(fullDir, 0o755); err != nil {
		return "", err
	}
	fullPath := filepath.Join(fullDir, objectName)
	if err := os.WriteFile(fullPath, content, 0o644); err != nil {
		return "", err
	}
	return filepath.ToSlash(filepath.Join(dir, objectName)), nil
}

// readLocalFile 读取相对存储路径对应的本地文件内容，返回 base64 编码字符串。
func readLocalFile(relPath string) (string, error) {
	fullPath := filepath.Join(uploadBaseDir(), filepath.FromSlash(strings.Trim(relPath, "/")))
	data, err := os.ReadFile(fullPath)
	if err != nil {
		return "", err
	}
	return base64.StdEncoding.EncodeToString(data), nil
}

// contentHash 计算字节内容的 SHA256 十六进制摘要。
func contentHash(data []byte) string {
	sum := sha256.Sum256(data)
	return hex.EncodeToString(sum[:])
}