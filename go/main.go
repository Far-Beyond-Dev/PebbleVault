package main

/*
#include <stdlib.h>
*/

import "C"
import (
	"encoding/json"
	"fmt"
	"log"
	"strings"
	"unsafe"

	"github.com/tidwall/buntdb"
)

// Structure to hold our object data
type SpatialObject struct {
	Type string  `json:"type"`
	UUID string  `json:"uuid"`
	X    float64 `json:"x"`
	Y    float64 `json:"y"`
	Z    float64 `json:"z"`
}

//export create_in_memory_db
func create_in_memory_db() uintptr {
	db, err := buntdb.Open(":memory:")
	if err != nil {
		log.Fatal(err)
	}
	return uintptr(unsafe.Pointer(db))
	// return uintptr(uintptr(unsafe.Pointer(db)))
}

//export close_in_memory_db
func close_in_memory_db(db uintptr) {
	// Close the database when done.
	(*buntdb.DB)(unsafe.Pointer(db)).Close()
}

//export free_in_memory_pointer_db
func free_in_memory_pointer_db(db uintptr) {
	(*buntdb.DB)(unsafe.Pointer(db)).Close()
}

//export set_object
func set_object(db uintptr, key *C.char, value *C.char) {
	// Read-write transactions
	(*buntdb.DB)(unsafe.Pointer(db)).Update(func(tx *buntdb.Tx) error {
		_, _, err := tx.Set(C.GoString(key), C.GoString(value), nil)
		return err
	})
}

//export get_object
func get_object(db uintptr, key *C.char) *C.char {
	// Read-only transactions
	// Getting non-existent values will cause an ErrNotFound error.
	var result string
	(*buntdb.DB)(unsafe.Pointer(db)).View(func(tx *buntdb.Tx) error {
		val, err := tx.Get(C.GoString(key))
		if err != nil {
			return err
		}
		result = val
		return nil
	})
	return C.CString(result)
}

//export delete_object
func delete_object(db uintptr, key *C.char) {
	// Read-write transactions
	(*buntdb.DB)(unsafe.Pointer(db)).Update(func(tx *buntdb.Tx) error {
		_, err := tx.Delete(C.GoString(key))
		return err
	})
}

//export get_all_objects
func get_all_objects(db uintptr) *C.char {
	var result string
	(*buntdb.DB)(unsafe.Pointer(db)).View(func(tx *buntdb.Tx) error {
		tx.Ascend("", func(key, val string) bool {
			result += key + ":" + val + ","
			return true
		})
		return nil
	})
	return C.CString(result)
}

//export set_custom_index_objects
func set_custom_index_objects(db uintptr, indexName *C.char, indexKey *C.char) {
	(*buntdb.DB)(unsafe.Pointer(db)).CreateIndex(C.GoString(indexName), C.GoString(indexKey), buntdb.IndexString)
}

//export add_object_to_custom_index
func add_object_to_custom_index(db uintptr, key *C.char, value *C.char) {
	(*buntdb.DB)(unsafe.Pointer(db)).Update(func(tx *buntdb.Tx) error {
		_, _, err := tx.Set(C.GoString(key), C.GoString(value), nil)
		return err
	})
}

//export iterate_over_custom_index
func iterate_over_custom_index(db uintptr, indexName *C.char) *C.char {
	var result string
	(*buntdb.DB)(unsafe.Pointer(db)).View(func(tx *buntdb.Tx) error {
		tx.Ascend(C.GoString(indexName), func(key, val string) bool {
			result += key + ":" + val + ","
			return true
		})
		return nil
	})
	return C.CString(result)
}

//export create_spatial_index
func create_spatial_index(db uintptr, indexName *C.char) {
	// Create a spatial index for 3D data
	// Parameters:
	// - db: pointer to the BuntDB database
	// - indexName: name of the index (e.g., "spatial3d")
	(*buntdb.DB)(unsafe.Pointer(db)).CreateSpatialIndex(C.GoString(indexName), "*", index3D)
}

//export add_obejct_to_spatial_index
func add_obejct_to_spatial_index(db uintptr, jsonData *C.char) {
	// Add an object to the spatial index
	// Parameters:
	// - db: pointer to the BuntDB database
	// - jsonData: JSON string containing type, uuid, x, y, z fields
	//   e.g., {"type": "car", "uuid": "abc-123", "x": 1.0, "y": 2.0, "z": 3.0}
	(*buntdb.DB)(unsafe.Pointer(db)).Update(func(tx *buntdb.Tx) error {
		var obj SpatialObject
		if err := json.Unmarshal([]byte(C.GoString(jsonData)), &obj); err != nil {
			return err
		}
		_, _, err := tx.Set(obj.UUID, C.GoString(jsonData), nil)
		return err
	})
}

//export query_spatial_index_by_area
func query_spatial_index_by_area(db uintptr, indexName *C.char, minX, minY, minZ, maxX, maxY, maxZ float64) *C.char {
	// Query the 3D spatial index within a given 3D bounding box
	// Parameters:
	// - db: pointer to the BuntDB database
	// - indexName: name of the index to query
	// - minX, minY, minZ: minimum coordinates of the bounding box
	// - maxX, maxY, maxZ: maximum coordinates of the bounding box
	// Returns: C string containing comma-separated JSON objects of matching items
	var result strings.Builder
	(*buntdb.DB)(unsafe.Pointer(db)).View(func(tx *buntdb.Tx) error {
		tx.Intersects(C.GoString(indexName), fmt.Sprintf("[%f %f %f],[%f %f %f]", minX, minY, minZ, maxX, maxY, maxZ), func(key, val string) bool {
			result.WriteString(val)
			result.WriteString(",")
			return true
		})
		return nil
	})
	return C.CString(strings.TrimRight(result.String(), ","))
}

//export get_object_by_uuid
func get_object_by_uuid(db uintptr, uuid *C.char) *C.char {
	// Retrieve an object by its UUID
	// Parameters:
	// - db: pointer to the BuntDB database
	// - uuid: UUID of the object to retrieve
	// Returns: JSON string of the object if found, empty string if not found
	var result string
	(*buntdb.DB)(unsafe.Pointer(db)).View(func(tx *buntdb.Tx) error {
		val, err := tx.Get(C.GoString(uuid))
		if err != nil {
			return err
		}
		result = val
		return nil
	})
	return C.CString(result)
}

//export delete_object_by_uuid
func delete_object_by_uuid(db uintptr, uuid *C.char) {
	// Delete an object by its UUID
	// Parameters:
	// - db: pointer to the BuntDB database
	// - uuid: UUID of the object to delete
	(*buntdb.DB)(unsafe.Pointer(db)).Update(func(tx *buntdb.Tx) error {
		_, err := tx.Delete(C.GoString(uuid))
		return err
	})
}

//export update_object_by_uuid
func update_object_by_uuid(db uintptr, uuid *C.char, jsonData *C.char) {
	// Update an object by its UUID
	// Parameters:
	// - db: pointer to the BuntDB database
	// - uuid: UUID of the object to update
	// - jsonData: JSON string containing updated object data
	(*buntdb.DB)(unsafe.Pointer(db)).Update(func(tx *buntdb.Tx) error {
		_, _, err := tx.Set(C.GoString(uuid), C.GoString(jsonData), nil)
		return err
	})
}

//export query_objects_by_type
func query_objects_by_type(db uintptr, objectType *C.char) *C.char {
	// Retrieve all objects of a specific type
	// Parameters:
	// - db: pointer to the BuntDB database
	// - objectType: type of objects to retrieve
	// Returns: C string containing comma-separated JSON objects of matching items
	var result strings.Builder
	(*buntdb.DB)(unsafe.Pointer(db)).View(func(tx *buntdb.Tx) error {
		tx.Ascend("", func(key, value string) bool {
			var obj SpatialObject
			if err := json.Unmarshal([]byte(value), &obj); err == nil {
				if obj.Type == C.GoString(objectType) {
					result.WriteString(value)
					result.WriteString(",")
				}
			}
			return true
		})
		return nil
	})
	return C.CString(strings.TrimRight(result.String(), ","))
}

func index3D(s string) (min, max []float64) {
	// Custom indexing function for 3D data
	// Parses the JSON string and extracts X, Y, Z coordinates
	var obj SpatialObject
	if err := json.Unmarshal([]byte(s), &obj); err != nil {
		return nil, nil
	}
	return []float64{obj.X, obj.Y, obj.Z}, []float64{obj.X, obj.Y, obj.Z}
}

func main() {}
