package main

import (
	"fmt"

	"example.com/btree/btree"
)

func main() {
	inputItems := []struct {
		key   string
		value btree.Student
	}{
		{
			"abc",
			btree.Student{
				Name:  "Troy",
				Age:   10,
				Score: 90,
			},
		},
		{
			"def",
			btree.Student{
				Name:  "Alex",
				Age:   15,
				Score: 80,
			},
		},
		{
			"aaa",
			btree.Student{
				Name:  "Nina",
				Age:   17,
				Score: 97,
			},
		},
		{
			"ikj",
			btree.Student{
				Name:  "Dan",
				Age:   4,
				Score: 86,
			},
		},
		{
			"lmn",
			btree.Student{
				Name:  "Sam",
				Age:   12,
				Score: 94,
			},
		},
		{
			"opq",
			btree.Student{
				Name:  "Jack",
				Age:   14,
				Score: 96,
			},
		},
	}

	// * Create a new B-Tree
	bt := btree.NewBTree()

	// * Insert elements into the B-Tree
	for _, item := range inputItems {
		bt.Insert(item.key, item.value)
	}

	// * Search the B-Tree for certain keys
	keys := []string{
		"abc",
		"xyz",
		"lmn",
	}
	for _, key := range keys {
		if value, ok := bt.Find(key); ok {
			fmt.Printf("The key %q is found in the b-tree with value %+v!\n", key, value)
		} else {
			fmt.Printf("The key %q is NOT found in the b-tree with value %+v!\n", key, value)
		}
	}
}
