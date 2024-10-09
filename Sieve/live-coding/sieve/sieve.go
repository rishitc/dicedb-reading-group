package sieve

import (
	"fmt"
	"strings"
)

// Cache mapping the key of type 'K' with a value of type 'V'.
type sieve[K comparable, V any] struct {
	cache    map[K]*node[K, V]
	head     *node[K, V]
	hand     *node[K, V] // <- Notice the addition of an additional pointer for the Hand!
	tail     *node[K, V]
	len      int
	capacity int
}

func NewSIEVECache[K comparable, V any](capacity int) *sieve[K, V] {
	// A cache with capacity less than 1 does not make any sense.
	if capacity < 1 {
		panic("Capacity must be >= 1")
	}

	return &sieve[K, V]{
		cache:    make(map[K]*node[K, V]),
		head:     nil,
		hand:     nil,
		tail:     nil,
		len:      0,
		capacity: capacity,
	}
}

// Algorithm:
//
//  1. Check if the `key` is present in the `cache`
//     1.1 If the `key` is present in the `cache`, then
//     1.1.1 Mark the `visited` value as `true` and return the `value` and `true`
//     1.1.2 Return the `value` and `true`
//
//     1.2 If the `key` is absent in the `cache`, then
//     1.2.1 Return the `zeroValue` and `false`
func (sieve *sieve[K, V]) Get(key K) (V, bool) {
	if v, ok := sieve.cache[key]; ok {
		v.visited = true
		return v.val, true
	}
	var zeroValue V
	return zeroValue, false
}

// Algorithm:
//
//  1. Check if the `key` is present in the `cache`
//     1.1 If the `key` is present in the `cache`, then
//     1.1.1 Mark the `visited` value as `true`
//     1.1.2 Update the `val` and return `true`
//
//     1.2 If the `key` is absent in the `cache`, then
//     1.2.1 Check if the cache has reached capacity
//     1.2.1.1 If the cache has reached capacity then evict a node using the SIEVE algorithm (see the image `algorithm.png` for the algorithm)
//     1.2.2 Insert a new node containing the new key-value pair to the head of the Doubly Linked List
func (sieve *sieve[K, V]) Set(key K, val V) bool {
	if v, ok := sieve.cache[key]; ok {
		v.visited = true
		v.val = val
		return true
	}

	if sieve.len == sieve.capacity {
		hand := sieve.hand
		if hand == nil {
			hand = sieve.tail
		}
		for hand.visited == true {
			hand.visited = false
			hand = hand.prev
			if hand == nil {
				hand = sieve.tail
			}
		}
		sieve.hand = hand.prev

		if hand.next != nil {
			hand.next.prev = hand.prev
		} else {
			sieve.tail = hand.prev
		}
		if hand.prev != nil {
			hand.prev.next = hand.next
		} else {
			sieve.hand = hand.next
		}

		delete(sieve.cache, hand.key)
		sieve.len -= 1
	}

	nn := newNode(key, val)
	sieve.cache[key] = nn

	nn.next = sieve.head
	if sieve.head != nil {
		sieve.head.prev = nn
	}
	sieve.head = nn
	if sieve.tail == nil {
		sieve.tail = nn
	}
	sieve.len += 1
	return false
}

func (sieve *sieve[K, V]) Len() int {
	return sieve.len
}

func (sieve *sieve[K, V]) Cap() int {
	return sieve.capacity
}

func (sieve *sieve[K, V]) String() string {
	s := strings.Builder{}

	s.WriteString("<< Head of sieve Queue >>\n")
	for runner := sieve.head; runner != nil; runner = runner.next {
		s.WriteString(fmt.Sprintf("\tKey  : %v\n", runner.key))
		s.WriteString(fmt.Sprintf("\tValue: %v\n", runner.val))
		if sieve.hand == runner {
			s.WriteString(fmt.Sprintf("\tThe hand is here!\n"))
		}
	}
	s.WriteString("<< Tail of sieve Queue >>\n")

	return s.String()
}
