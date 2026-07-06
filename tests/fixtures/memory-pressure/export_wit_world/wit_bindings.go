package export_wit_world

import (
	"fmt"
	"wit_component/wit_world"
)

func Run() {
	// Build heap pressure so GC fires during the next host call
	for i := 0; i < 10_000; i++ {
		_ = make([]byte, 1024)
	}

	v := wit_world.GetStr()
	fmt.Println(v)
}
