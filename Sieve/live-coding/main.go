package main

import (
	"bufio"
	"fmt"
	"os"

	"github.com/rishitc/fifo"
	"github.com/rishitc/lru"
	"github.com/rishitc/sieve"
)

func main() {
	const fileName = "trace"
	const capacity = 50

	// FIFO algorithm
	fifoChan := make(chan int, 1)
	fifoCache := fifo.NewFIFOCache[string, string](capacity)
	go traceRunner(fileName, fifoChan, fifoCache)

	// LRU algorithm
	lruChan := make(chan int, 1)
	lruCache := lru.NewLRUCache[string, string](capacity)
	go traceRunner(fileName, lruChan, lruCache)

	// SIEVE algorithm
	sieveChan := make(chan int, 1)
	sieveCache := sieve.NewSIEVECache[string, string](capacity)
	go traceRunner(fileName, sieveChan, sieveCache)

	// Wait for all three algorithms to finish the trace
	for count := 0; count < 3; count += 1 {
		select {
		case v := <-fifoChan:
			fmt.Printf("The miss count of FIFO is: %v\n", v)
		case v := <-lruChan:
			fmt.Printf("The miss count of LRU is: %v\n", v)
		case v := <-sieveChan:
			fmt.Printf("The miss count of SIEVE is: %v\n", v)
		}
	}
}

func traceRunner(fileName string, ch chan<- int, cache Cache[string, string]) {
	file, err := os.Open(fileName)
	if err != nil {
		fmt.Println(err)
		return
	}
	defer file.Close()

	// Create an instance of the FIFO cache
	// var cache Cache[string, string] = fifo.NewFIFOCache[string, string](capacity)
	missCount := 0

	scanner := bufio.NewScanner(file)
	scanner.Split(bufio.ScanLines)

	// Returns a boolean based on whether there's a next instance of `\n`
	// character in the IO stream. This step also advances the internal pointer
	// to the next position (after '\n') if it did find that token.

	for read := scanner.Scan(); read; read = scanner.Scan() {
		d := scanner.Text()
		if _, ok := cache.Get(d); !ok {
			missCount += 1
			cache.Set(d, d)
		}
	}

	ch <- missCount
}
