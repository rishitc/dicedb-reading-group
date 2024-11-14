package btree

const (
	degree = 5

	minChildren = degree
	maxChildren = 2 * degree

	minItems = minChildren - 1
	maxItems = maxChildren - 1
)
