package export_wasi_http_handler

import (
	. "wit_component/wasi_http_types"

	. "go.bytecodealliance.org/pkg/wit/types"
)

// Handle the specified `Request`, returning a `Response`
func Handle(request *Request) Result[*Response, ErrorCode] {
	method := request.GetMethod().Tag()
	path := request.GetPathWithQuery().SomeOr("/")

	if method == MethodGet && path == "/hello" {
		// Say hello!

		tx, rx := MakeStreamU8()

		go func() {
			defer tx.Drop()
			tx.WriteAll([]uint8("Hello, world!"))
		}()

		response, send := ResponseNew(
			FieldsFromList([]Tuple2[string, []byte]{
				{F0: "content-type", F1: []byte("text/plain")},
			}).Ok(),
			Some(rx),
			trailersFuture(),
		)
		send.Drop()

		return Ok[*Response, ErrorCode](response)

	} else {
		// Bad request

		response, send := ResponseNew(
			MakeFields(),
			None[*StreamReader[uint8]](),
			trailersFuture(),
		)
		send.Drop()
		response.SetStatusCode(400).Ok()

		return Ok[*Response, ErrorCode](response)

	}
}

func trailersFuture() *FutureReader[Result[Option[*Fields], ErrorCode]] {
	tx, rx := MakeFutureResultOptionFieldsErrorCode()
	go tx.Write(Ok[Option[*Fields], ErrorCode](None[*Fields]()))
	return rx
}

func unitFuture() *FutureReader[Result[Unit, ErrorCode]] {
	tx, rx := MakeFutureResultUnitErrorCode()
	go tx.Write(Ok[Unit, ErrorCode](Unit{}))
	return rx
}
