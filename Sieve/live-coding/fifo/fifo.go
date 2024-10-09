package fifo

import (
	"fmt"
	"strings"
)

// Cache mapping the key of type 'K' with a value of type 'V'.
type fifo[K comparable, V any] struct {
	cache    map[K]*node[K, V]
	head     *node[K, V]
	tail     *node[K, V]
	len      int
	capacity int
}

func NewFIFOCache[K comparable, V any](capacity int) *fifo[K, V] {
	// A cache with capacity less than 1 does not make any sense
	if capacity < 1 {
		panic("Capacity of the cache must be greater than or equal to 1")
	}

	return &fifo[K, V]{
		cache:    make(map[K]*node[K, V]),
		head:     nil,
		tail:     nil,
		len:      0,
		capacity: capacity,
	}
}

func (fifo *fifo[K, V]) Get(key K) (V, bool) {
	if v, ok := fifo.cache[key]; ok {
		// Notice that a cache hit does not affect the FIFO policy
		return v.val, ok
	}

	var zeroValue V
	return zeroValue, false
}

func (fifo *fifo[K, V]) Set(key K, val V) bool {
	if v, ok := fifo.cache[key]; ok {
		v.val = val
		return true
	}

	nn := newNode(key, val)
	if fifo.len == fifo.capacity {
		// Evict the node at the tail of the Doubly Linked List from the cache to create space for the incoming node
		lastNodePtr := fifo.tail
		if fifo.len == 1 {
			// If there is only one element in the Doubly Linked List then evicting it will make the list empty
			fifo.head = nil
			fifo.tail = nil
		} else {
			// Deleting a node from the tail of a Doubly Linked List
			fifo.tail = fifo.tail.prev
			fifo.tail.next = nil
			lastNodePtr.prev = nil
		}
		// Delete the node from the cache
		delete(fifo.cache, lastNodePtr.key)
		fifo.len -= 1
	}

	// Insert the new node into the cache
	fifo.cache[key] = nn

	// Insert at the head of the Doubly Linked List
	nn.next = fifo.head
	if fifo.head != nil {
		fifo.head.prev = nn
	}
	fifo.head = nn
	if fifo.tail == nil {
		fifo.tail = nn
	}

	fifo.len += 1
	return false
}

func (fifo *fifo[K, V]) Len() int {
	return fifo.len
}

func (fifo *fifo[K, V]) Cap() int {
	return fifo.capacity
}

func (fifo *fifo[K, V]) String() string {
	s := strings.Builder{}

	s.WriteString("<< Head of fifo Queue >>\n")
	for runner := fifo.head; runner != nil; runner = runner.next {
		s.WriteString(fmt.Sprintf("\tKey  : %v\n", runner.key))
		s.WriteString(fmt.Sprintf("\tValue: %v\n", runner.val))
	}
	s.WriteString("<< Tail of fifo Queue >>\n")

	return s.String()
}
