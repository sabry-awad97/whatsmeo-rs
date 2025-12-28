package bridge

import (
	"encoding/json"
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
func MarshalEvent(evt interface{}) ([]byte, error) {
	var eventType string
	var data interface{}

	switch v := evt.(type) {
	case *events.QR:
		eventType = "qr"
		data = map[string]interface{}{
			"codes": v.Codes,
		}

	case *events.PairSuccess:
		eventType = "pair_success"
		data = map[string]interface{}{
			"id":            v.ID.String(),
			"business_name": v.BusinessName,
			"platform":      v.Platform,
		}

	case *events.Connected:
		eventType = "connected"
		data = nil

	case *events.Disconnected:
		eventType = "disconnected"
		data = nil

	case *events.LoggedOut:
		eventType = "logged_out"
		data = map[string]interface{}{
			"on_connect": v.OnConnect,
			"reason":     v.Reason.String(),
		}

	case *events.Message:
		eventType = "message"

		text := ""
		if v.Message.GetConversation() != "" {
			text = v.Message.GetConversation()
		} else if v.Message.GetExtendedTextMessage() != nil {
			text = v.Message.GetExtendedTextMessage().GetText()
		}

		data = map[string]interface{}{
			"id":        v.Info.ID,
			"from":      v.Info.Sender.String(),
			"chat":      v.Info.Chat.String(),
			"text":      text,
			"timestamp": v.Info.Timestamp.UnixMilli(),
			"is_group":  v.Info.IsGroup,
			"push_name": v.Info.PushName,
		}

	case *events.Receipt:
		eventType = "receipt"
		data = map[string]interface{}{
			"message_ids": v.MessageIDs,
			"chat":        v.Chat.String(),
			"sender":      v.Sender.String(),
			"type":        v.Type.String(),
			"timestamp":   v.Timestamp.UnixMilli(),
		}

	case *events.Presence:
		eventType = "presence"
		data = map[string]interface{}{
			"from":        v.From.String(),
			"unavailable": v.Unavailable,
			"last_seen":   v.LastSeen.UnixMilli(),
		}

	case *events.HistorySync:
		eventType = "history_sync"
		data = map[string]interface{}{
			"progress": v.Progress,
		}

	case *events.PushNameSetting:
		eventType = "push_name"
		data = map[string]interface{}{
			"action": v.Action.String(),
		}

	default:
		// Unknown event - serialize full struct
		eventType = "unknown"
		data = evt
	}

	event := Event{
		Type:      eventType,
		Timestamp: time.Now().UnixMilli(),
		Data:      nil,
	}

	if data != nil {
		rawData, err := json.Marshal(data)
		if err != nil {
			return nil, err
		}
		event.Data = rawData
	}

	return json.Marshal(event)
}
