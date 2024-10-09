package sieve

// Doubly linked list node.
type node[K comparable, V any] struct {
	key     K
	val     V
	visited bool // <- Notice the addition of an additional field to track the visited status!
	next    *node[K, V]
	prev    *node[K, V]
}

func newNode[K comparable, V any](key K, val V) *node[K, V] {
	return &node[K, V]{
		key:     key,
		val:     val,
		visited: false, // <- Notice that when we create a node the initial value of `visited` is `false`
		next:    nil,
		prev:    nil,
	}
}
