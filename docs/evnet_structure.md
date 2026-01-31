# WebSocket Compression and Encoding Strategies for a MUD Protocol

> **Question:** How can WebSocket event packets be made much smaller, especially given shared structure and repeated data?
>
> **Short answer:** Yes — very effectively. You can stack compression at multiple layers, from simple transport compression to protocol-level dictionary encoding.

This document explains the *reasoning*, tradeoffs, and a recommended path forward.

---

## 1. Why This Is Worth Thinking About Early

A MUD-style game has properties that make compression unusually effective:

* Extremely **repetitive structure** (same event types, same fields)
* Repeated **strings** (room IDs, image keys, conditions, mob names)
* Mostly **small messages**, sent frequently
* Long-lived connections (perfect for shared dictionaries)

This means you can achieve **very high compression ratios** with relatively modest complexity.

---

## 2. Layer 1: Transport-Level Compression (permessage-deflate)

### What it is

WebSocket supports a standard extension called **`permessage-deflate`**.

* Compresses each message using DEFLATE
* Automatically learns repeated strings and keys
* Requires no changes to your event schema

### Why it works well here

JSON event payloads repeat keys like:

* `type`
* `channel`
* `speak_priority`
* `image_key`

DEFLATE excels at this pattern.

### Pros

* Easiest win
* Widely supported
* No protocol redesign
* Transparent to clients

### Cons

* Some CPU cost
* Needs to be enabled on both sides

### Bottom line

> **Do this first.** Even with plain JSON, it often cuts bandwidth by 70–90%.

---

## 3. Layer 2: Binary Encoding (MessagePack / CBOR / Protobuf)

### Problem with JSON

JSON repeats field names on every message:

```json
{"type":"Text","data":{"channel":"Narration","text":"hi","speak":true}}
```

That verbosity dominates message size.

### Binary encodings solve this

* **MessagePack** / **CBOR**: compact, schema-light
* **Protobuf / FlatBuffers**: very compact, schema-driven

These encodings:

* Replace strings with small integers
* Encode booleans and enums efficiently
* Work well over binary WebSocket frames

### Pros

* Much smaller messages
* Faster to parse than JSON in many cases
* Excellent with compression

### Cons

* Less human-readable
* Harder to debug without tooling

### Practical compromise

* Support **JSON** and **binary** modes
* Negotiate on connect: `format=json | msgpack`

---

## 4. Layer 3: Protocol-Level Dictionary Compression

This is where your *shared dictionary idea* really shines.

### 4.1 Numeric Event and Field IDs

Instead of verbose objects:

```json
{"type":"Text","data":{"channel":"Narration","text":"hi","speak":true,"speak_priority":1}}
```

Use numeric IDs:

```json
[1,[0,"hi",1,1]]
```

Where:

* `1` = TextEvent
* `0` = Narration
* `1/0` = true/false

#### Pros

* Extremely compact
* Works even without compression

#### Cons

* Requires published schema
* Harder to eyeball

---

### 4.2 Dictionary Handshake (Schema-on-Connect)

Instead of hardcoding IDs forever:

1. Server sends a **dictionary** on connect
2. Client caches it
3. Future messages use short IDs

This allows:

* Schema evolution
* Optional fields
* Backward compatibility

---

### 4.3 String Table Interning

Many *values* repeat just as often as keys:

* Room IDs
* Image keys
* Condition names
* NPC names

Instead of sending the string repeatedly:

1. Server sends:

```json
{"type":"StringTableAdd","data":{"id":42,"value":"forest_day_01"}}
```

2. Later messages use:

```json
{"image_key_id":42}
```

This is very effective for room images and state-heavy UI.

---

## 5. Layer 4: Delta / State Compression

Often the *biggest* savings come from simply **not resending unchanged data**.

### Example: Stats

Instead of:

```json
{"hp":27,"hp_max":32,"mp":14,"mp_max":20}
```

Send:

```json
{"hp":27}
```

### Works well for

* Stats
* Buffs / conditions
* Exit lock state
* Combat ticks

Delta events reduce traffic even before compression is applied.

---

## 6. What Actually Dominates Bandwidth in Practice

In MUD-like games, bandwidth is usually dominated by:

1. Combat spam
2. Chat spam
3. Repeated status updates

Therefore, the most impactful optimizations are:

* Channel filtering
* Delta updates
* Compression
* Client-side coalescing (batching lines)

Not raw room/exits data.

---

## 7. Recommended Phased Strategy

### Phase 1 (Simple, Debuggable)

* JSON events
* WebSocket + `permessage-deflate`

### Phase 2 (Efficiency without pain)

* Optional MessagePack / CBOR mode
* Binary WebSocket frames

### Phase 3 (Protocol maturity)

* Delta events for stats/combat
* String table for repeated values

### Phase 4 (Extreme efficiency, optional)

* Numeric field IDs
* Dictionary handshake

Each phase stands on its own; you don’t need to jump ahead.

---

## 8. Design Cautions

* Always version your protocol
* Keep a JSON fallback for debugging
* Make advanced compression **opt-in** for clients
* Avoid stateful compression bugs (reset tables on reconnect)

---

## 9. Final Takeaway

> **You don’t need to choose just one approach.**

Compression works best when layered:

* Transport compression handles repetition
* Binary encoding removes verbosity
* Dictionaries exploit shared structure
* Delta updates avoid redundancy entirely

For a MUD-style game, this combination is especially powerful — and worth designing for early, even if you implement it gradually.
