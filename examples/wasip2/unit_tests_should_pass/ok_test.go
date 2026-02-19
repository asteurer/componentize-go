package unit_tests_should_pass

import (
	"testing"
)

func test_sum(a, b int) int {
	return a + b
}

func TestSum(t *testing.T) {
	tests := []struct {
		name     string
		a, b     int
		expected int
	}{
		{"positive numbers", 2, 3, 5},
		{"negative numbers", -1, -2, -3},
		{"zeros", 0, 0, 0},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			got := test_sum(tt.a, tt.b)
			if got != tt.expected {
				t.Errorf("test_sum(%d, %d) = %d, expected %d", tt.a, tt.b, got, tt.expected)
			}
		})
	}
}
