package std

import (
	"testing"
	"time"
)

func TestPaginate(t *testing.T) {
	cases := []struct {
		page, pageSize int64
		off, lim       int
	}{
		{1, 20, 0, 20},
		{3, 10, 20, 10},
		{0, 20, 0, 20},   // page<1 归 1
		{2, 0, 20, 20},   // pageSize<=0 用默认 20
		{2, -5, 20, 20},  // pageSize<=0 用默认 20
		{2, 5000, 1000, 1000}, // pageSize>1000 截断 1000，offset 按截断后 1000 计
	}
	for _, c := range cases {
		off, lim := Paginate(c.page, c.pageSize)
		if off != c.off || lim != c.lim {
			t.Fatalf("Paginate(%d,%d)=%d,%d want %d,%d", c.page, c.pageSize, off, lim, c.off, c.lim)
		}
	}
}

func TestTimeStr(t *testing.T) {
	if got := TimeStr(nil); got != "" {
		t.Fatalf("TimeStr(nil)=%q want empty", got)
	}
	now := time.Date(2026, 9, 9, 10, 30, 0, 0, time.UTC)
	got := TimeStr(&now)
	if got == "" {
		t.Fatal("TimeStr should not be empty for valid time")
	}
	// RFC3339 可被解析回
	if _, err := time.Parse(time.RFC3339, got); err != nil {
		t.Fatalf("TimeStr output not RFC3339: %q (%v)", got, err)
	}
}

func TestStrBool(t *testing.T) {
	if got := Str(nil); got != "" {
		t.Fatalf("Str(nil)=%q", got)
	}
	s := "abc"
	if got := Str(&s); got != "abc" {
		t.Fatalf("Str(&s)=%q", got)
	}
	if got := Bool(nil); got {
		t.Fatal("Bool(nil) should be false")
	}
	tr := true
	if !Bool(&tr) {
		t.Fatal("Bool(&true) should be true")
	}
}

func TestInt64(t *testing.T) {
	if Int64(nil) != 0 {
		t.Fatal("Int64(nil)!=0")
	}
	v64 := int64(42)
	if Int64(&v64) != 42 {
		t.Fatal("Int64(*int64) wrong")
	}
	u32 := uint32(7)
	if Int64(&u32) != 7 {
		t.Fatal("Int64(*uint32) wrong")
	}
	s := "99"
	if Int64(&s) != 99 {
		t.Fatal("Int64(*string) parse wrong")
	}
	bad := "notnum"
	if Int64(&bad) != 0 {
		t.Fatal("Int64(bad string) should be 0")
	}
	i32 := int32(5)
	if Int64(&i32) != 0 {
		t.Fatal("Int64(*int32) unsupported should be 0")
	}
}