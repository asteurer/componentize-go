module github.com/bytecodealliance/componentize-go

go 1.27.1

require (
	github.com/apparentlymart/go-userdirs v0.0.0-20200915174352-b0c018a67c13
	github.com/gofrs/flock v0.13.0
)

require golang.org/x/sys v0.37.0 // indirect

retract v0.5.0 // manifest versions not bumped; use v0.5.1
