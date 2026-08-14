# Supply Bucket

A supply bucket is a deterministic set of string dimensions that partitions supply for the same target.

For example, a provider can use dimensions to distinguish calculated capacity classes or time windows without inventing a different target type. Keys are unique, and both keys and values must be non-empty.

An empty bucket represents supply that needs no additional partition.
