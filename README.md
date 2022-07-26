# Firefly

An optimized tweaked key-value pair database. It is a simple, fast, and secure.
At [Xiler](https://www.xiler.net) it gets used to store and manage client
sessions throughout the platform.

## Inner working

### Serialization

Firefly uses a custom minimalist serialization format. Since all values are
always present, we can just add one delimiter byte to the start of each
partition.

The delimiter byte is a `NULL` byte, this byte is not allowed to be within the
content of the message.

### ASCII optimization

Because we limit all our data to be valid ASCII, and ASCII only uses 7 bits,
we can remove the bit from each byte, this results in a drastically smaller
data size.

```
0011 0001 -> 0110 0010
0011 0010 -> 1100 1001
0011 0011 -> 1001 1011
0011 0100 -> 0100 ....
```
