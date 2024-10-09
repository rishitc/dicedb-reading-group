package main

import "fmt"

type Cache[K comparable, V any] interface {
	Get(key K) (V, bool)
	Set(key K, val V) bool
	Len() int
	Cap() int
	fmt.Stringer
}
