package btree

import (
	"slices"
)

type btree struct {
	root *node
}

func NewBTree() btree {
	return btree{}
}

func (t *btree) Find(key string) (Student, bool) {
	for node := t.root; node != nil; {
		idx, ok := slices.BinarySearchFunc(t.root.items[:node.numItems], key, func(ele *item, targetKey string) int {
			if ele.key < targetKey {
				return -1
			} else if ele.key == targetKey {
				return 0
			} else {
				return 1
			}
		})
		if ok {
			return node.items[idx].value, true
		}
		node = node.children[idx]
	}
	return Student{}, false
}

func (t *btree) Insert(key string, value Student) {
	i := item{key, value}
	if t.root == nil {
		t.root = &node{}
	}

	if t.root.numItems >= maxItems {
		t.splitRoot()
	}

	t.root.insert(&i)
}

func (t *btree) splitRoot() {
	newRoot := node{}
	midItem, newNode := t.root.split()
	newRoot.insertItemAt(0, midItem)
	newRoot.insertChildAt(0, t.root)
	newRoot.insertChildAt(1, newNode)
	t.root = &newRoot
}
