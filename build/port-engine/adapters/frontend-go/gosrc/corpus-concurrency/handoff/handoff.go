// Package handoff is the CSP surface, reduced to what one reader can hold at once.
//
// The source's channel is ONE value carrying both ends. The target splits it: a sender that clones
// and a receiver that does not. So a channel does not become a type here, it becomes a PAIR, and
// which end each site holds is a fact about where the channel is used rather than about the channel.
//
// Nothing in this file is a library call. `time`, `net` and the rest are the next question and a
// separate one; what is here is the scheduler surface alone, so a rule that answers it can be
// judged without a foreign type standing in the way.
package handoff

// Counter accumulates values a producer sends it.
type Counter struct {
	total int64
}

// Add folds one value in.
func (c *Counter) Add(n int64) {
	c.total = c.total + n
}

// Total reports what has been folded in.
func (c *Counter) Total() int64 {
	return c.total
}

// Produce sends every value down the channel and closes nothing: the source's send blocks until a
// receiver takes it, which is the rendezvous the target's bounded sender has too.
func Produce(out chan int64, values []int64) {
	for _, value := range values {
		out <- value
	}
}

// Consume folds every value the channel yields until it is drained.
//
// The parameter is not named `in`: that is a keyword in the target, and a first proof of the
// scheduler surface should not also be a proof of keyword escaping.
func Consume(source chan int64, c *Counter) {
	for {
		select {
		case value := <-source:
			c.Add(value)
		}
	}
}

// Start runs the producer CONCURRENTLY with its caller, which is the whole of what `go` says.
func Start(out chan int64, values []int64) {
	go Produce(out, values)
}
