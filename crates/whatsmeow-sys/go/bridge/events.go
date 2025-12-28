package main

import (
	"encoding/json"
	"fmt"
	"reflect"
	"time"

	"go.mau.fi/whatsmeow/types/events"
)

// Event wraps any WhatsMeow event with type information
type Event struct {
	Type      string          `json:"type"`
	Timestamp int64           `json:"timestamp"`
	Data      json.RawMessage `json:"data"`
}

// MarshalEvent converts any WhatsMeow event to our unified JSON format
// It marshals ALL fields from the original event struct
func MarshalEvent(evt interface{}) ([]byte, error) {
	var eventType string

	switch evt.(type) {
	case *events.QR:
		eventType = "qr"
	case *events.PairSuccess:
		eventType = "pair_success"
	case *events.Connected:
		eventType = "connected"
	case *events.Disconnected:
		eventType = "disconnected"
	case *events.LoggedOut:
		eventType = "logged_out"
	case *events.Message:
		eventType = "message"
	case *events.Receipt:
		eventType = "receipt"
	case *events.Presence:
		eventType = "presence"
	case *events.HistorySync:
		eventType = "history_sync"
	case *events.PushNameSetting:
		eventType = "push_name"
	case *events.ChatPresence:
		eventType = "chat_presence"
	case *events.OfflineSyncPreview:
		eventType = "offline_sync_preview"
	case *events.OfflineSyncCompleted:
		eventType = "offline_sync_completed"
	default:
		// Use reflection to get type name for unknown events
		t := reflect.TypeOf(evt)
		if t.Kind() == reflect.Ptr {
			t = t.Elem()
		}
		eventType = fmt.Sprintf("unknown_%s", t.Name())
	}

	event := Event{
		Type:      eventType,
		Timestamp: time.Now().UnixMilli(),
		Data:      nil,
	}

	// Marshal the complete original event struct
	rawData, err := json.Marshal(evt)
	if err != nil {
		return nil, err
	}
	event.Data = rawData

	return json.Marshal(event)
}
