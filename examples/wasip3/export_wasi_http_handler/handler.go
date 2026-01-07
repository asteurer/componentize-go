package export_wasi_http_handler

import (
	"fmt"
	"sync"
	. "wit_component/wasi_http_types"

	. "github.com/bytecodealliance/wit-bindgen/wit_types"
)

func Handle(request *Request) Result[*Response, ErrorCode] {
	tx, rx := MakeStreamU8()

	go func() {
		defer tx.Drop()
		for i := range 100 {
			tx.Write([]byte(fmt.Sprintf("%d", i)))
		}
	}()

	var wg sync.WaitGroup
	for i := range 1 {
		wg.Add(1)
		go func(id int) {
			defer wg.Done()
			buffer := make([]uint8, 16*1024)
			fmt.Println("START READ")
			// ERROR: READ CONCURRENTLY
			count := rx.Read(buffer)
			if count == 0 {
				panic("Read buffer is empty")
			}

			fmt.Println("END READ")
			fmt.Printf("output: %s", string(buffer))
		}(i)
	}

	wg.Wait()

	response, send := ResponseNew(
		FieldsFromList([]Tuple2[string, []uint8]{
			Tuple2[string, []uint8]{"content-type", []uint8("text/plain")},
		}).Ok(),
		None[*StreamReader[uint8]](),
		trailersFuture(),
	)
	send.Drop()

	return Ok[*Response, ErrorCode](response)

}

func trailersFuture() *FutureReader[Result[Option[*Fields], ErrorCode]] {
	tx, rx := MakeFutureResultOptionFieldsErrorCode()
	go tx.Write(Ok[Option[*Fields], ErrorCode](None[*Fields]()))
	return rx
}
