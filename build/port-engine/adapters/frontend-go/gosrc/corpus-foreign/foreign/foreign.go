package foreign

import (
	"bytes"
	"io"
)

// Sink accepts anything that can be written to.
func Sink(w io.Writer) {
	_, _ = w.Write([]byte("x"))
}

// Source accepts anything that can be read from.
func Source(r io.Reader) error {
	_, err := r.Read(make([]byte, 1))
	return err
}

// Drive makes one FOREIGN type satisfy two interfaces.
//
// `bytes.Buffer` belongs to neither this corpus nor the crate the engine emits, so there is
// nowhere to put an impl for it — both facts are recorded as foreign satisfactions and deferred.
// They share a name and are not the same fact, which is the shape that used to reject a whole
// snapshot: two entries named `bytes.Buffer` read as two declarations of one package-scope name.
func Drive() error {
	var buf bytes.Buffer
	Sink(&buf)
	return Source(&buf)
}
