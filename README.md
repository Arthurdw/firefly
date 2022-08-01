# Firefly

An optimized tweaked key-value pair database. It is a simple, fast, and secure.
At [Xiler](https://www.xiler.net) it gets used to store and manage client
sessions throughout the platform.

## Query Language

Firefly only has three data operators, `NEW`, `GET`, `DROP`. We'll walk over
each one.

### Important note on values

ALL values must be valid ASCII, if not the server will reject the query.

### Defining query types

The server may only accept one query type per TCP connection type. The default
type is a string query. But this can be changed by using the `QUERY TYPE`
keyword.

```ffly
QUERY TYPE [STRING | BITWISE];
```

### String queries

The simplest way to query something is by querying it using a string query. But
because the parsing, and bandwidth of this is more than that is necessary we
also provide bitwise queries.

#### Create

You can create a new record by using the `NEW` keyword, the arguments should be
wrapped within quotes. The first argument _(after `NEW`)_ is the key, this
should be unique throughout the db. If no TLL value is provided, the server
will use 0 _(aka no expiry)_.

```ffly
NEW '{key}'
VALUE '{value}'
[ WITH TTL {ttl}];
```

##### Create examples

```ffly
NEW 's2.8eqYursP2McHeQvHB2bauyE6n3vptOj8M96PxmGAQMDfimeZ31WAzP3hSw5Ixv5Y'
VALUE '86ebe1a0-11bf-11ed-aa8e-13602e2ad46b';
```

```ffly
NEW 's2.8eqYursP2McHeQvHB2bauyE6n3vptOj8M96PxmGAQMDfimeZ31WAzP3hSw5Ixv5Y'
VALUE '86ebe1a0-11bf-11ed-aa8e-13602e2ad46b'
WITH TTL 604800;
```

#### Fetch

The `GET` keyword returns the value and TTL by default. But if you only want
one of the two, you can specify this. You can only search by key!

```ffly
GET [VALUE | TTL] '{key}';
```

##### Fetch examples

```ffly
GET 's2.8eqYursP2McHeQvHB2bauyE6n3vptOj8M96PxmGAQMDfimeZ31WAzP3hSw5Ixv5Y';
```

```ffly
GET TTL 's2.8eqYursP2McHeQvHB2bauyE6n3vptOj8M96PxmGAQMDfimeZ31WAzP3hSw5Ixv5Y';
```

#### Delete

Deleting one record is as straightforward as fetching one. You can only delete
whole records. If required all records that have a value can also be deleted,
but this action is very expensive and generally not recommended.

```ffly
DROP '{key}';
DROPALL '{value}';
```

### Bitwise queries

Because string queries can consume more resources than what is required, there
is a more efficient _(less friendly)_ way to interact with Firefly. This is by
sending the bits in a specific format. This section just describes the formats,
if you want more information about the queries itself, then you can find it in
the `String queries` section.

Please keep in mind that all the bitwise queries are very strict, and if a
query is unrecognized it will discard it.

#### General notes

-   All values are delimited by a NUL character. _(`0x0`)_
-   The end of the query is assumed to be the last byte.
-   Queries start with their type, this is a numeric value
    -   0: `NEW`
    -   1: `GET`
    -   2: `GET VALUE`
    -   3: `GET TTL`
    -   4: `DROP`
    -   5: `DROPALL`
-   The query type does not need to be delimited

#### Bitwise create

A with TTL must always be provided. If you don't want a TTL set this to 0.

`0{key}0x0{value}0x0{ttl}`

These are the two same create examples from the string queries:
`0s2.8eqYursP2McHeQvHB2bauyE6n3vptOj8M96PxmGAQMDfimeZ31WAzP3hSw5Ixv5Y0x086ebe1a0-11bf-11ed-aa8e-13602e2ad46b0x0`
`0s2.8eqYursP2McHeQvHB2bauyE6n3vptOj8M96PxmGAQMDfimeZ31WAzP3hSw5Ixv5Y0x086ebe1a0-11bf-11ed-aa8e-13602e2ad46b0x0602800`

#### Bitwise fetch

`1s2.8eqYursP2McHeQvHB2bauyE6n3vptOj8M96PxmGAQMDfimeZ31WAzP3hSw5Ixv5Y`
`2s2.8eqYursP2McHeQvHB2bauyE6n3vptOj8M96PxmGAQMDfimeZ31WAzP3hSw5Ixv5Y`
`3s2.8eqYursP2McHeQvHB2bauyE6n3vptOj8M96PxmGAQMDfimeZ31WAzP3hSw5Ixv5Y`

#### Bitwise delete

`4s2.8eqYursP2McHeQvHB2bauyE6n3vptOj8M96PxmGAQMDfimeZ31WAzP3hSw5Ixv5Y`
`5s2.8eqYursP2McHeQvHB2bauyE6n3vptOj8M96PxmGAQMDfimeZ31WAzP3hSw5Ixv5Y`

## Inner working

### Serialization

Firefly uses a custom minimalist serialization format. Since all values are
always present, we can just add one delimiter byte to the start of each
partition.

The delimiter byte is a `NUL` byte, this byte is not allowed to be within the
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
