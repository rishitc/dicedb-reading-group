package btree

import "slices"

type item struct {
	key   string
	value Student
}

type node struct {
	items       [maxItems]*item
	children    [maxChildren]*node
	numItems    int
	numChildren int
}

func (n *node) isLeaf() bool {
	return n.numChildren == 0
}

func (n *node) insertItemAt(idx int, item *item) {
	if idx < n.numItems {
		copy(n.items[idx+1:], n.items[idx:])
	}
	n.items[idx] = item
	n.numItems++
}

func (n *node) insertChildAt(idx int, child *node) {
	if idx < n.numChildren {
		copy(n.children[idx+1:], n.children[idx:])
	}
	n.children[idx] = child
	n.numChildren++
}

func (n *node) split() (*item, *node) {
	midIdx := minItems
	midItem := n.items[midIdx]

	newNode := &node{}
	copy(newNode.items[:], n.items[midIdx+1:])
	newNode.numItems = minItems

	if !n.isLeaf() {
		copy(newNode.children[:], n.children[midIdx+1:])
		newNode.numChildren = minItems + 1
	}

	for i, l := midIdx, n.numItems; i < l; i++ {
		n.items[i] = nil
		n.numItems--

		if !n.isLeaf() {
			n.children[i+1] = nil
			n.numChildren--
		}
	}

	return midItem, newNode
}

func (n *node) insert(i *item) bool {
	idx, ok := slices.BinarySearchFunc(n.items[:n.numItems], i.key, func(i *item, targetKey string) int {
		if i.key < targetKey {
			return -1
		} else if i.key == targetKey {
			return 0
		} else {
			return 1
		}
	})

	if ok {
		n.items[idx] = i
		return false
	}

	if n.isLeaf() {
		n.insertItemAt(idx, i)
		return true
	}

	if n.children[idx].numItems >= maxItems {
		midItem, newNode := n.children[idx].split()
		n.insertItemAt(idx, midItem)
		n.insertChildAt(idx+1, newNode)
		if i.key > midItem.key {
			idx++
		} else if i.key == midItem.key {
			n.items[idx] = i
			return false
		}
	}

	return n.children[idx].insert(i)
}
