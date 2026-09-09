package password

import (
	"strings"
	"testing"
)

func TestHashAndVerify(t *testing.T) {
	h, err := Hash("admin")
	if err != nil {
		t.Fatalf("Hash err: %v", err)
	}
	if h == "" {
		t.Fatal("empty hash")
	}
	// bcrypt 哈希以 $2 开头且为 60 字符
	if !strings.HasPrefix(h, "$2") {
		t.Fatalf("unexpected hash prefix: %q", h)
	}
	if len(h) != 60 {
		t.Fatalf("expected 60-char bcrypt, got %d", len(h))
	}
	if !Verify(h, "admin") {
		t.Fatal("Verify(hash, \"admin\") should be true")
	}
	if Verify(h, "wrong") {
		t.Fatal("Verify(hash, \"wrong\") should be false")
	}
	if Verify(h, "") {
		t.Fatal("Verify(hash, \"\") should be false")
	}
}

func TestVerifyInvalidHash(t *testing.T) {
	if Verify("not-a-bcrypt-hash", "admin") {
		t.Fatal("Verify with malformed hash must return false, not panic")
	}
	if Verify("", "admin") {
		t.Fatal("Verify with empty hash must return false")
	}
}

func TestHashSaltRandomized(t *testing.T) {
	h1, err1 := Hash("same")
	h2, err2 := Hash("same")
	if err1 != nil || err2 != nil {
		t.Fatalf("hash err: %v / %v", err1, err2)
	}
	if h1 == h2 {
		t.Fatal("bcrypt must use random salt, two hashes of same plaintext should differ")
	}
	if !Verify(h1, "same") || !Verify(h2, "same") {
		t.Fatal("both hashes must verify the plaintext")
	}
}