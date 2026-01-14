package package1

import (
	"testing"
)

func TestConvertIntToString(t *testing.T) {
	num := 12
	want := "12"
	got := ConvertIntToString(num)
	if got != want {
		t.Errorf("want %s got %s", want, got)
	}
}
