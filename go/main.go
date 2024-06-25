package main

/*
#include <stdlib.h>
*/
import "C"

import (
	"fmt"
	"log"
	"unsafe"

	"github.com/tidwall/buntdb"
)

//export Greet
func Greet(name *C.char) *C.char {
	return C.CString(fmt.Sprintf("Hello from Go, %s!", C.GoString(name)))
}

//export GoFree
func GoFree(ptr *C.char) {
	C.free(unsafe.Pointer(ptr))
}

//export CreateDB
func CreateDB() uintptr {
	// Open the data.db file. It will be created if it doesn't exist.
	db, err := buntdb.Open("data.db")
	if err != nil {
		log.Fatal(err)
	}
	return uintptr(unsafe.Pointer(db))
	// return uintptr(uintptr(unsafe.Pointer(db)))
}

//export CloseDB
func CloseDB(db uintptr) {
	// Close the database when done.
	(*buntdb.DB)(unsafe.Pointer(db)).Close()
}

func main() {}
