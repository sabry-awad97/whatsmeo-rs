package main

/*
#include <stdlib.h>
#include <string.h>
*/
import "C"

import (
	"sync"
	"unsafe"
)

// Error codes matching Rust side
const (
	WM_OK                   = 0
	WM_ERR_INIT             = -1
	WM_ERR_CONNECT          = -2
	WM_ERR_DISCONNECTED     = -3
	WM_ERR_INVALID_HANDLE   = -4
	WM_ERR_BUFFER_TOO_SMALL = -5
)

// Global client registry
var (
	clientsMu sync.RWMutex
	clients           = make(map[uintptr]*Client)
	nextID    uintptr = 1
)

//export wm_client_new
func wm_client_new(dbPath *C.char, deviceName *C.char) C.uintptr_t {
	config := ClientConfig{
		DbPath:     C.GoString(dbPath),
		DeviceName: C.GoString(deviceName),
	}

	client, err := NewClient(config)
	if err != nil {
		return 0
	}

	clientsMu.Lock()
	defer clientsMu.Unlock()

	id := nextID
	nextID++
	clients[id] = client

	return C.uintptr_t(id)
}

//export wm_client_connect
func wm_client_connect(handle C.uintptr_t) C.int {
	client := getClient(uintptr(handle))
	if client == nil {
		return WM_ERR_INVALID_HANDLE
	}

	err := client.Connect()
	if err != nil {
		return WM_ERR_CONNECT
	}

	return WM_OK
}

//export wm_client_disconnect
func wm_client_disconnect(handle C.uintptr_t) C.int {
	client := getClient(uintptr(handle))
	if client == nil {
		return WM_ERR_INVALID_HANDLE
	}

	client.Disconnect()
	return WM_OK
}

//export wm_client_destroy
func wm_client_destroy(handle C.uintptr_t) {
	clientsMu.Lock()
	defer clientsMu.Unlock()

	if client, ok := clients[uintptr(handle)]; ok {
		client.Destroy()
		delete(clients, uintptr(handle))
	}
}

//export wm_poll_event
func wm_poll_event(handle C.uintptr_t, buf *C.char, bufLen C.int) C.int {
	client := getClient(uintptr(handle))
	if client == nil {
		return WM_ERR_INVALID_HANDLE
	}

	data := client.PollEvent()
	if data == nil {
		return 0 // No event
	}

	if len(data) > int(bufLen) {
		return WM_ERR_BUFFER_TOO_SMALL
	}

	// Copy to buffer
	C.memcpy(unsafe.Pointer(buf), unsafe.Pointer(&data[0]), C.size_t(len(data)))
	return C.int(len(data))
}

//export wm_send_message
func wm_send_message(handle C.uintptr_t, jid *C.char, text *C.char) C.int {
	client := getClient(uintptr(handle))
	if client == nil {
		return WM_ERR_INVALID_HANDLE
	}

	err := client.SendMessage(C.GoString(jid), C.GoString(text))
	if err != nil {
		return WM_ERR_CONNECT
	}

	return WM_OK
}

//export wm_last_error
func wm_last_error(handle C.uintptr_t, buf *C.char, bufLen C.int) C.int {
	client := getClient(uintptr(handle))
	if client == nil {
		return 0
	}

	msg := client.LastError()
	if msg == "" {
		return 0
	}

	if len(msg) > int(bufLen)-1 {
		msg = msg[:bufLen-1]
	}

	cstr := C.CString(msg)
	defer C.free(unsafe.Pointer(cstr))
	C.strcpy(buf, cstr)

	return C.int(len(msg))
}

func getClient(handle uintptr) *Client {
	clientsMu.RLock()
	defer clientsMu.RUnlock()
	return clients[handle]
}

func main() {} // Required for CGO build
