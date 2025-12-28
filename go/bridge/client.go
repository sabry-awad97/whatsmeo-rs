package main

import (
	"context"
	"fmt"
	"sync"

	_ "github.com/mattn/go-sqlite3"
	"go.mau.fi/whatsmeow"
	"go.mau.fi/whatsmeow/store/sqlstore"
	waLog "go.mau.fi/whatsmeow/util/log"
)

// Client wraps WhatsMeow with an event queue for FFI
type Client struct {
	mu         sync.RWMutex
	client     *whatsmeow.Client
	store      *sqlstore.Container
	eventQueue chan []byte
	ctx        context.Context
	cancel     context.CancelFunc
	connected  bool
	lastError  string
}

// NewClient creates a new WhatsApp client
func NewClient(dbPath string) (*Client, error) {
	ctx := context.Background()

	// Initialize database (new API requires context)
	store, err := sqlstore.New(ctx, "sqlite3",
		fmt.Sprintf("file:%s?_foreign_keys=on", dbPath),
		waLog.Noop)
	if err != nil {
		return nil, fmt.Errorf("failed to open store: %w", err)
	}

	// Get or create device (new API requires context)
	device, err := store.GetFirstDevice(ctx)
	if err != nil {
		return nil, fmt.Errorf("failed to get device: %w", err)
	}

	client := whatsmeow.NewClient(device, waLog.Noop)
	clientCtx, cancel := context.WithCancel(context.Background())

	c := &Client{
		client:     client,
		store:      store,
		eventQueue: make(chan []byte, 1024),
		ctx:        clientCtx,
		cancel:     cancel,
	}

	// Register event handler
	client.AddEventHandler(c.handleEvent)

	return c, nil
}

// Connect initiates the WhatsApp connection
func (c *Client) Connect() error {
	c.mu.Lock()
	defer c.mu.Unlock()

	if c.client.Store.ID == nil {
		// Need QR code login
		qrChan, _ := c.client.GetQRChannel(c.ctx)
		err := c.client.Connect()
		if err != nil {
			c.lastError = err.Error()
			return fmt.Errorf("connect failed: %w", err)
		}

		// Forward QR codes to event queue
		go func() {
			for evt := range qrChan {
				data, err := MarshalEvent(evt)
				if err == nil {
					c.eventQueue <- data
				}
			}
		}()
	} else {
		// Already logged in
		err := c.client.Connect()
		if err != nil {
			c.lastError = err.Error()
			return fmt.Errorf("connect failed: %w", err)
		}
	}

	c.connected = true
	return nil
}

// handleEvent processes any WhatsMeow event
func (c *Client) handleEvent(evt interface{}) {
	data, err := MarshalEvent(evt)
	if err != nil {
		return
	}

	select {
	case c.eventQueue <- data:
	default:
		// Queue full, drop oldest
		select {
		case <-c.eventQueue:
		default:
		}
		c.eventQueue <- data
	}
}

// PollEvent retrieves the next event (non-blocking)
func (c *Client) PollEvent() []byte {
	select {
	case evt := <-c.eventQueue:
		return evt
	default:
		return nil
	}
}

// SendMessage sends a text message
func (c *Client) SendMessage(jid, text string) error {
	// TODO: Implement full message sending
	return nil
}

// Disconnect closes the connection
func (c *Client) Disconnect() {
	c.mu.Lock()
	defer c.mu.Unlock()

	c.client.Disconnect()
	c.connected = false
}

// Destroy cleans up all resources
func (c *Client) Destroy() {
	c.cancel()
	c.Disconnect()
	if c.store != nil {
		c.store.Close()
	}
}

// LastError returns the last error message
func (c *Client) LastError() string {
	c.mu.RLock()
	defer c.mu.RUnlock()
	return c.lastError
}
