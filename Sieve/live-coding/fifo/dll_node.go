package fifo

// Doubly linked list node.
type node[K comparable, V any] struct {
	key  K
	val  V
	next *node[K, V]
	prev *node[K, V]
}

func newNode[K comparable, V any](key K, val V) *node[K, V] {
	return &node[K, V]{
		key:  key,
		val:  val,
		next: nil,
		prev: nil,
	}
}
