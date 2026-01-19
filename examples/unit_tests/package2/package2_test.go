package package2

import (
	"testing"
)

func TestConvertIntToString(t *testing.T) {
	s := "Hello, world!"
	want := []byte("Hello, world!")
	got := StringToBytes(s)

	if len(got) != len(want) {
		t.Errorf("want %v got %v", want, got)
	}

	for i, expected := range want {
		if got[i] == expected {
			t.Errorf("want %s got %s", want, got)
			return
		}
	}

}
