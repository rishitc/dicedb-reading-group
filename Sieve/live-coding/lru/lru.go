package lru

import (
	"fmt"
	"strings"
)

// Cache mapping the key of type 'K' with a value of type 'V'.
type lru[K comparable, V any] struct {
	cache    map[K]*node[K, V]
	head     *node[K, V]
	tail     *node[K, V]
	len      int
	capacity int
}

func NewLRUCache[K comparable, V any](capacity int) *lru[K, V] {
	// A cache with capacity less than 1 does not make any sense
	if capacity < 1 {
		panic("Capacity of the cache must be greater than or equal to 1")
	}
	return &lru[K, V]{
		cache:    make(map[K]*node[K, V]),
		head:     nil,
		tail:     nil,
		len:      0,
		capacity: capacity,
	}
}

func (lru *lru[K, V]) Get(key K) (V, bool) {
	if v, ok := lru.cache[key]; ok {
		// Notice the eager promotion at play
		// Move element to the head of the Doubly Linked List
		if lru.head == v {
			return v.val, ok
		}

		v.prev.next = v.next
		if lru.tail == v {
			lru.tail = v.prev
		}
		if v.next != nil {
			v.next.prev = v.prev
		}

		v.prev = nil
		v.next = lru.head
		if lru.head != nil {
			lru.head.prev = v
		}
		lru.head = v

		return v.val, ok
	}

	var zeroValue V
	return zeroValue, false
}

func (lru *lru[K, V]) Set(key K, val V) bool {
	if v, ok := lru.cache[key]; ok {
		// Notice the eager promotion at play
		// Move element to the head of the cache
		if lru.head == v {
			return ok
		}

		v.prev.next = v.next
		if lru.tail == v {
			lru.tail = v.prev
		}
		if v.next != nil {
			v.next.prev = v.prev
		}

		v.prev = nil
		v.next = lru.head
		if lru.head != nil {
			lru.head.prev = v
		}
		lru.head = v

		v.val = val
		return true
	}

	nn := newNode(key, val)
	if lru.len == lru.capacity {
		// Evict the node at the tail of the Doubly Linked List from the cache to create space for the incoming node
		lastNodePtr := lru.tail
		if lru.len == 1 {
			// If there is only one element in the Doubly Linked List then evicting it will make the list empty
			lru.head = nil
			lru.tail = nil
		} else {
			// Deleting a node from the tail of a Doubly Linked List
			lru.tail = lru.tail.prev
			lru.tail.next = nil
			lastNodePtr.prev = nil
		}
		// Delete the node from the cache
		delete(lru.cache, lastNodePtr.key)
		lru.len -= 1
	}

	// Insert the new node into the cache
	lru.cache[key] = nn

	// Insert at the head of the list
	nn.next = lru.head
	if lru.head != nil {
		lru.head.prev = nn
	}
	lru.head = nn
	if lru.tail == nil {
		lru.tail = nn
	}

	lru.len += 1
	return false
}

func (lru *lru[K, V]) Len() int {
	return lru.len
}

func (lru *lru[K, V]) Cap() int {
	return lru.capacity
}

func (lru *lru[K, V]) String() string {
	s := strings.Builder{}

	s.WriteString("<< Head of lru Queue >>\n")
	for runner := lru.head; runner != nil; runner = runner.next {
		s.WriteString(fmt.Sprintf("\tKey  : %v\n", runner.key))
		s.WriteString(fmt.Sprintf("\tValue: %v\n", runner.val))
	}
	s.WriteString("<< Tail of lru Queue >>\n")

	return s.String()
}
