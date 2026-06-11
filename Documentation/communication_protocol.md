# Zappy Broadcast Protocol

## 1. Purpose

**Three packet types exist.**

| Packet Type | Prefix    | Purpose                                  |
| ----------- | --------- | ---------------------------------------- |
| Regular     | `L300`    | Carries a strategic message body         |
| Session     | `L300S`   | Initiates or renews a session token      |
| Fragment    | `L300F`   | Carries one fragment of a long body      |

Regular broadcast usage:

```txt
Broadcast L300|<ver36>|<seq36>|<epoch36>|<route36>|<session36>|<frag36>|<body>|<inner_check36>|<outer_check36>
```

Session broadcast usage:

```txt
Broadcast L300S|<ver36>|<seq36>|<epoch36>|<token36>|<sess_check36>|<outer_check36>
```

Fragment broadcast usage:

```txt
Broadcast L300F|<ver36>|<seq36>|<frag_id36>|<frag_index36>|<frag_total36>|<epoch36>|<route36>|<body>|<outer_check36>
```

When received through the Zappy server:

```txt
message K, L300|...
```

`K` is the official Zappy sound direction and is not part of LABYRINTH-300.
Only the part after the comma is parsed by LABYRINTH-300.

Session packets must be exchanged before any Regular packet can be decoded. See §19.
Fragment packets must be fully reassembled before decode can begin. See §18.

---

## 2. Packet Format

### 2.1 Regular Packet Fields

```txt
L300|<ver36>|<seq36>|<epoch36>|<route36>|<session36>|<frag36>|<body>|<inner_check36>|<outer_check36>
```

| Field            | Description                                                                   |
| ---------------- | ----------------------------------------------------------------------------- |
| `L300`           | Protocol prefix                                                                |
| `ver36`          | Protocol version in base36 — must equal `1` for this specification             |
| `seq36`          | Sequence number in base36                                                      |
| `epoch36`        | Current primary epoch in base36                                                |
| `route36`        | Visible route/type in base36 (not the real route — see §9)                    |
| `session36`      | Session token hash in base36 (see §19)                                        |
| `frag36`         | Fragment field: `0` = unfragmented; nonzero = fragment ID (see §18)           |
| `body`           | Encoded message body                                                           |
| `inner_check36`  | Inner packet check in base36 (covers fields 2–8 inclusive)                    |
| `outer_check36`  | Outer packet check in base36 (covers all fields 1–9 inclusive)                |

Regular packet is invalid if it does not contain exactly **ten** `|`-separated fields.

### 2.2 Session Packet Fields

```txt
L300S|<ver36>|<seq36>|<epoch36>|<token36>|<sess_check36>|<outer_check36>
```

Session packet is invalid if it does not contain exactly **seven** fields.

### 2.3 Fragment Packet Fields

Fragment packets are described in full in §18.
Fragment packet is invalid if it does not contain exactly **ten** fields.

### 2.4 Prefix and Field Count Validation

Prefix detection uses prefix-only matching before splitting the full string.
Field count is validated immediately after splitting.
A wrong field count for the detected prefix causes immediate silent discard.

---

## 3. Numeric Rules

All integer arithmetic is **wrapping arithmetic** unless a step specifies otherwise.

```txt
u8  wraps at 255
u16 wraps at 2^16 − 1
u32 wraps at 2^32 − 1
u64 wraps at 2^64 − 1
```

**Default byte order is big-endian** unless a step explicitly specifies little-endian.
Steps using little-endian are: §13.22, §13.50, §13.45.
All other steps use big-endian.

### 3.1 Base36

```txt
Alphabet:  0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ
```

Parsing is case-insensitive.
Generated packets must use uppercase letters.

### 3.2 Custom Base Alphabets

Two custom alphabets are used in post-transformation encoding. See §13.P3 and §13.P4.
These are distinct from base36 and from each other.

### 3.3 GF(256) Arithmetic

GF(256) operations use the irreducible polynomial defined by `GF256_IRREDUCIBLE_POLY`.
See §13.36 for usage.
GF inverse tables must be precomputed at initialization. See §24.3.

### 3.4 Rotation Semantics

`rotate_left(x, r)` and `rotate_right(x, r)` operate on the natural bit width of `x`.
Rotation amount is always reduced modulo bit width before use.

---

## 4. Public Constants

These constants must be named in code exactly as shown.
Do not use raw magic numbers anywhere in protocol logic.

```rust
const LABYRINTH_PREFIX:                 &str    = "L300";
const LABYRINTH_PREFIX_SESSION:         &str    = "L300S";
const LABYRINTH_PREFIX_FRAGMENT:        &str    = "L300F";
const LABYRINTH_SEPARATOR:              char    = '|';
const LABYRINTH_FIELD_COUNT_REGULAR:    usize   = 10;
const LABYRINTH_FIELD_COUNT_SESSION:    usize   = 7;
const LABYRINTH_FIELD_COUNT_FRAGMENT:   usize   = 10;
const PROTOCOL_VERSION:                 u64     = 1;
const BASE36_RADIX:                     u32     = 36;

const INITIAL_PROTOCOL_EPOCH_A:         u64     = 0x4D59_5A41_5050_5901;
const INITIAL_PROTOCOL_EPOCH_B:         u64     = 0x4C41_4259_5249_4E54;
const INITIAL_PROTOCOL_EPOCH_C:         u64     = 0x3030_3000_0001_0000;

const STATE_MULTIPLIER_A:               u64     = 0x9E37_79B9_7F4A_7C15;
const STATE_MULTIPLIER_B:               u64     = 0xBF58_476D_1CE4_E5B9;
const STATE_MULTIPLIER_C:               u64     = 0x94D0_49BB_1331_11EB;
const STATE_MULTIPLIER_D:               u64     = 0x6C62_272E_07BB_0142;
const STATE_MULTIPLIER_E:               u64     = 0xA3BC_9C12_90A2_8B62;
const STATE_MULTIPLIER_F:               u64     = 0xC4CC_E13B_B387_4023;
const STATE_MULTIPLIER_G:               u64     = 0xD6B3_7B0C_D6B3_7B0D;
const STATE_MULTIPLIER_H:               u64     = 0xE8A7_56C0_1723_9587;
// NOTE: STATE_MULTIPLIER_I is intentionally absent — visual ambiguity with digit 1.
const STATE_MULTIPLIER_J:               u64     = 0xF14A_29CD_E503_1977;

const BYTE_BITS:                        u32     = 8;
const NIBBLE_BITS:                      u32     = 4;
const WORD_BITS:                        u32     = 64;
const HALF_WORD_BITS:                   u32     = 32;
const QUARTER_WORD_BITS:                u32     = 16;

const MATRIX_MIN_WIDTH:                 usize   = 5;
const MATRIX_WIDTH_RANGE:               usize   = 17;
const MATRIX_SECONDARY_MIN_WIDTH:       usize   = 7;
const MATRIX_SECONDARY_WIDTH_RANGE:     usize   = 13;
const MATRIX_TERTIARY_MIN_WIDTH:        usize   = 3;
const MATRIX_TERTIARY_WIDTH_RANGE:      usize   = 23;
const MATRIX_QUATERNARY_MIN_WIDTH:      usize   = 11;
const MATRIX_QUATERNARY_WIDTH_RANGE:    usize   = 5;

const FEISTEL_ROUNDS_PRIMARY:           usize   = 13;
const FEISTEL_ROUNDS_SECONDARY:         usize   = 7;
const FEISTEL_ROUNDS_TERTIARY:          usize   = 17;

const CHECK_ROTATION_A:                 u32     = 7;
const CHECK_ROTATION_B:                 u32     = 19;
const CHECK_ROTATION_C:                 u32     = 31;
const CHECK_ROTATION_D:                 u32     = 11;
const CHECK_ROTATION_E:                 u32     = 23;
const CHECK_ROTATION_F:                 u32     = 37;   // used mod 64 on u64
const CHECK_ROTATION_G:                 u32     = 41;
const CHECK_ROTATION_H:                 u32     = 53;

const PRIMARY_SCHEDULE_SIZE:            usize   = 64;
const SECONDARY_SCHEDULE_SIZE:          usize   = 48;
const TERTIARY_SCHEDULE_SIZE:           usize   = 32;
const SHADOW_SCHEDULE_SIZE:             usize   = 16;

const NOISE_CHARS_PRIMARY:              &[u8]   = b"-_.~";
const NOISE_CHARS_SECONDARY:            &[u8]   = b"*+=^";

const ANTI_REPLAY_WINDOW:               usize   = 256;
const FRAGMENT_MAX_BODY_LENGTH:         usize   = 64;
const SESSION_TOKEN_LENGTH:             usize   = 8;
const BLOOM_HASH_FUNCTIONS:             usize   = 5;
const EPOCH_CONSENSUS_THRESHOLD:        usize   = 3;
const ZONE_CYCLE_PERIOD:                u64     = 1009;    // must be prime
const SECONDARY_ZONE_CYCLE_PERIOD:      u64     = 1013;    // must be prime, distinct
const ROUTE_MASK_LAYERS:                usize   = 3;

const POLYNOMIAL_DEGREE_PRIMARY:        u32     = 5;
const POLYNOMIAL_DEGREE_SECONDARY:      u32     = 7;
const GF256_IRREDUCIBLE_POLY:           u8      = 0x1B;    // x^8 + x^4 + x^3 + x + 1

const CHECKSUM_INTERVAL_MIN:            usize   = 3;
const CHECKSUM_INTERVAL_RANGE:          usize   = 7;
const SECONDARY_CHECKSUM_INTERVAL_MIN:  usize   = 5;
const SECONDARY_CHECKSUM_INTERVAL_RANGE:usize   = 11;

const DECOY_INTERVAL_MIN:               usize   = 4;
const DECOY_INTERVAL_RANGE:             usize   = 9;
const SECONDARY_DECOY_INTERVAL_MIN:     usize   = 6;
const SECONDARY_DECOY_INTERVAL_RANGE:   usize   = 7;

const CHUNK_SIZE_MIN:                   usize   = 2;
const CHUNK_SIZE_RANGE:                 usize   = 9;
const SECONDARY_CHUNK_SIZE_MIN:         usize   = 3;
const SECONDARY_CHUNK_SIZE_RANGE:       usize   = 7;
```

---

## 5. Semantic Opcodes

Readable strategy messages are converted into compact semantic opcodes before encoding.

| Meaning              | Text Form                       | Opcode | Arg Format                        |
| -------------------- | ------------------------------- | ------ | --------------------------------- |
| Meet at zone         | `meet <zone>`                   | `0x21` | `[zone_id: u16]`                  |
| Assign role          | `role <role>`                   | `0x32` | `[role_id: u8]`                   |
| Food status          | `food <status>`                 | `0x43` | `[status_id: u8]`                 |
| Incantation ready    | `incant <level> <zone>`         | `0x54` | `[level: u8, zone_id: u16]`       |
| Help request         | `help <zone>`                   | `0x65` | `[zone_id: u16]`                  |
| Enemy seen           | `enemy <zone>`                  | `0x76` | `[zone_id: u16]`                  |
| Resource found       | `res <resource> <zone>`         | `0x87` | `[res_id: u8, zone_id: u16]`      |
| Regroup              | `group <zone>`                  | `0x98` | `[zone_id: u16]`                  |
| Cancel plan          | `cancel <plan>`                 | `0xA9` | `[plan_id: u8]`                   |
| Plan switch          | `plan <plan>`                   | `0xBA` | `[plan_id: u8]`                   |
| Resource shortage    | `short <resource> <zone>`       | `0xCB` | `[res_id: u8, zone_id: u16]`      |
| Zone clear           | `clear <zone>`                  | `0xDC` | `[zone_id: u16]`                  |
| Level report         | `level <player_id> <level>`     | `0xED` | `[player_id: u8, level: u8]`      |
| Fork request         | `fork <zone>`                   | `0xFE` | `[zone_id: u16]`                  |
| Inventory report     | `inv <player_id> <res> <qty>`   | `0x1A` | `[pid: u8, res_id: u8, qty: u8]`  |
| Drop resource        | `drop <res> <zone>`             | `0x2B` | `[res_id: u8, zone_id: u16]`      |
| Take resource        | `take <res> <zone>`             | `0x3C` | `[res_id: u8, zone_id: u16]`      |
| Abort incantation    | `abort <zone>`                  | `0x4D` | `[zone_id: u16]`                  |
| Decoy signal         | `decoy <zone>`                  | `0x5E` | `[zone_id: u16]`                  |
| Ping                 | `ping`                          | `0x6F` | `[]`                              |
| Acknowledge          | `ack <seq>`                     | `0x70` | `[acked_seq: u32]`                |

Zone IDs are 16-bit strategy values, not raw coordinates. See §13.4 and §13.5.

### Resource IDs

| Resource  | ID     | | Role     | ID     | | Food Status | ID     |
| --------- | ------ |-| -------- | ------ |-| ----------- | ------ |
| food      | `0x11` | | scout    | `0x12` | | critical    | `0x01` |
| linemate  | `0x22` | | gatherer | `0x23` | | low         | `0x02` |
| deraumere | `0x33` | | caller   | `0x34` | | ok          | `0x03` |
| sibur     | `0x44` | | feeder   | `0x45` | | full        | `0x04` |
| mendiane  | `0x55` | | incanter | `0x56` | |             |        |
| phiras    | `0x66` | | anchor   | `0x67` | |             |        |
| thystame  | `0x77` | | relay    | `0x78` | |             |        |

### Plan IDs

| Plan    | ID     | | Plan    | ID     |
| ------- | ------ |-| ------- | ------ |
| alpha   | `0x01` | | epsilon | `0x05` |
| beta    | `0x02` | | zeta    | `0x06` |
| gamma   | `0x03` | | eta     | `0x07` |
| delta   | `0x04` | | theta   | `0x08` |

---

## 6. Protocol State

Each AI maintains three independent state machines per allied team channel.

### 6.1 Primary State

```txt
primary_state_value     : u64
primary_last_seq        : u64
primary_last_check      : u64
primary_epoch           : u64
primary_seen_sequences  : bloom filter (see §20)
primary_session_token   : u64
```

### 6.2 Secondary State

```txt
secondary_state_value   : u64
secondary_fold_counter  : u64
secondary_last_route    : u64
secondary_epoch_offset  : u64
```

### 6.3 Shadow State

```txt
shadow_state_value      : u64
shadow_accumulator      : u64
shadow_parity           : u8     (always 0 or 1)
shadow_sequence_parity  : u8     (always 0 or 1)
```

### 6.4 Initial Values

```txt
primary_state_value    = mix64_primary(INITIAL_PROTOCOL_EPOCH_A, map_width, map_height, frequency)
primary_last_seq       = 0
primary_last_check     = INITIAL_PROTOCOL_EPOCH_A
primary_epoch          = INITIAL_PROTOCOL_EPOCH_A
primary_seen_sequences = empty bloom filter
primary_session_token  = 0

secondary_state_value  = mix64_secondary(INITIAL_PROTOCOL_EPOCH_B, map_width, map_height, frequency)
secondary_fold_counter = 0
secondary_last_route   = 0
secondary_epoch_offset = INITIAL_PROTOCOL_EPOCH_C XOR INITIAL_PROTOCOL_EPOCH_B

shadow_state_value     = mix64_primary(INITIAL_PROTOCOL_EPOCH_C, primary_state_value, secondary_state_value, 0)
shadow_accumulator     = 0
shadow_parity          = 0
shadow_sequence_parity = 0
```

### 6.5 State Update After Valid Regular Packet

Apply in this exact order after a valid decode:

```txt
// Phase 1 — primary
primary_state_value = mix64_primary(primary_state_value, seq, outer_check, real_route)
primary_state_value = mix64_primary(primary_state_value, inner_check, decoded_payload_hash, primary_epoch)
primary_last_seq    = seq
primary_last_check  = outer_check

// Phase 2 — epoch
primary_epoch = rotate_left(primary_epoch XOR outer_check, CHECK_ROTATION_A) XOR primary_state_value

// Phase 3 — secondary
secondary_state_value  = mix64_secondary(secondary_state_value, seq, inner_check, secondary_last_route)
secondary_fold_counter = secondary_fold_counter + 1
secondary_last_route   = real_route
secondary_epoch_offset = rotate_right(secondary_epoch_offset XOR inner_check, CHECK_ROTATION_D)
secondary_epoch_offset = secondary_epoch_offset XOR secondary_state_value

// Phase 4 — shadow
shadow_accumulator     = shadow_accumulator XOR low32(primary_state_value) XOR low32(secondary_state_value)
shadow_parity          = low_bit(shadow_accumulator)
shadow_state_value     = mix64_primary(shadow_state_value, shadow_accumulator, secondary_fold_counter, primary_epoch)
shadow_sequence_parity = low_bit(seq)
```

`decoded_payload_hash` is the `frame_hash` computed as defined in §10.4, using the decoded frame bytes.

### 6.6 Reorder Window

A packet whose sequence number is present in the bloom filter is ignored.
A packet whose sequence number is more than `ANTI_REPLAY_WINDOW` behind the highest accepted sequence is ignored even if the bloom filter returns false.
Otherwise the packet is accepted and its sequence number is inserted into the bloom filter.

---

## 7. Mix Functions

The protocol defines four mix functions. Every section that invokes a mix function specifies which one by full name.

### 7.1 mix64_primary

```txt
mix64_primary(a, b, c, d) =
    x = a XOR rotate_left(b, CHECK_ROTATION_A)
    x = x + STATE_MULTIPLIER_A
    x = x XOR rotate_right(c, CHECK_ROTATION_B)
    x = x * STATE_MULTIPLIER_B
    x = x XOR rotate_left(d, CHECK_ROTATION_C)
    x = x * STATE_MULTIPLIER_C
    return x XOR (x >> 33)
```

### 7.2 mix64_secondary

```txt
mix64_secondary(a, b, c, d) =
    x = a XOR rotate_left(b, CHECK_ROTATION_D)
    x = x + STATE_MULTIPLIER_D
    x = x XOR rotate_right(c, CHECK_ROTATION_E)
    x = x * STATE_MULTIPLIER_E
    x = x XOR rotate_left(d, CHECK_ROTATION_F)
    x = x * STATE_MULTIPLIER_F
    return x XOR (x >> 31)
```

### 7.3 mix64_shadow

```txt
mix64_shadow(a, b, c, d) =
    x = a + rotate_right(b, CHECK_ROTATION_G)
    x = x XOR (x >> 17)
    x = x * STATE_MULTIPLIER_G
    x = x + rotate_left(c, CHECK_ROTATION_H)
    x = x XOR (x >> 29)
    x = x * STATE_MULTIPLIER_H
    x = x XOR rotate_right(d, CHECK_ROTATION_A)
    return x XOR (x >> 37)
```

### 7.4 mix64_fold

```txt
mix64_fold(a, b, c, d) =
    p = mix64_primary(a, b, c, d)
    s = mix64_secondary(b, c, d, a)
    h = mix64_shadow(c, d, a, b)
    return p XOR rotate_left(s, CHECK_ROTATION_B) XOR rotate_right(h, CHECK_ROTATION_E)
```

`mix64_fold` is used only in §8.4 (shadow schedule) and §14.2 (outer check).

---

## 8. Schedule Generation

Each packet generates **four independent schedules**.
The shadow schedule depends on the other three and must be computed last.
**All four schedules must be fully computed before any encoding step is applied.**

### 8.1 Primary Schedule

Inputs: `seq`, `primary_epoch`, `visible_route`, `primary_state_value`, `primary_last_check`

```txt
seed = mix64_primary(seq, primary_epoch, visible_route, primary_state_value XOR primary_last_check)
for i in 0..PRIMARY_SCHEDULE_SIZE:
    seed = mix64_primary(seed, i, primary_epoch, primary_state_value)
    primary_schedule[i] = seed
```

Forward pass:

```txt
for i in 1..PRIMARY_SCHEDULE_SIZE:
    primary_schedule[i] = mix64_primary(
        primary_schedule[i], primary_schedule[i-1], seq, primary_epoch)
```

Backward pass:

```txt
for i in (0..PRIMARY_SCHEDULE_SIZE-1).rev():
    primary_schedule[i] = mix64_primary(
        primary_schedule[i], primary_schedule[i+1], visible_route, primary_last_check)
```

Even/odd index pass:

```txt
for i in 0..PRIMARY_SCHEDULE_SIZE:
    if i is even:
        primary_schedule[i] = rotate_left(primary_schedule[i], i modulo WORD_BITS)
    else:
        primary_schedule[i] = rotate_right(primary_schedule[i], i modulo WORD_BITS)
```

### 8.2 Secondary Schedule

Inputs: `seq`, `primary_epoch`, `secondary_state_value`, `secondary_fold_counter`, `secondary_epoch_offset`

```txt
seed = mix64_secondary(
    seq XOR secondary_epoch_offset, primary_epoch, secondary_state_value, secondary_fold_counter)
for i in 0..SECONDARY_SCHEDULE_SIZE:
    seed = mix64_secondary(seed, i + 1, secondary_state_value, secondary_fold_counter XOR seq)
    secondary_schedule[i] = seed
```

Forward pass (uses `mix64_secondary`):

```txt
for i in 1..SECONDARY_SCHEDULE_SIZE:
    secondary_schedule[i] = mix64_secondary(
        secondary_schedule[i], secondary_schedule[i-1], seq, secondary_epoch_offset)
```

Backward pass (uses `mix64_secondary`):

```txt
for i in (0..SECONDARY_SCHEDULE_SIZE-1).rev():
    secondary_schedule[i] = mix64_secondary(
        secondary_schedule[i], secondary_schedule[i+1], secondary_state_value, seq)
```

Odd-index scramble:

```txt
for i in 0..SECONDARY_SCHEDULE_SIZE:
    if i is odd:
        secondary_schedule[i] = secondary_schedule[i] XOR
            rotate_left(secondary_schedule[(i-1) modulo SECONDARY_SCHEDULE_SIZE], CHECK_ROTATION_D)
    else:
        secondary_schedule[i] = secondary_schedule[i] +
            secondary_schedule[(i+1) modulo SECONDARY_SCHEDULE_SIZE]
```

### 8.3 Tertiary Schedule

Inputs: `seq`, `primary_epoch`, `primary_schedule[0]`, `secondary_schedule[0]`

```txt
seed = mix64_secondary(seq, primary_epoch, primary_schedule[0], secondary_schedule[0])
for i in 0..TERTIARY_SCHEDULE_SIZE:
    seed = mix64_secondary(
        seed,
        i * 3 + 1,
        primary_schedule[i modulo PRIMARY_SCHEDULE_SIZE],
        secondary_schedule[i modulo SECONDARY_SCHEDULE_SIZE])
    tertiary_schedule[i] = seed
```

Forward pass (uses `mix64_primary`):

```txt
for i in 1..TERTIARY_SCHEDULE_SIZE:
    tertiary_schedule[i] = mix64_primary(
        tertiary_schedule[i],
        tertiary_schedule[i-1],
        primary_schedule[i modulo PRIMARY_SCHEDULE_SIZE],
        seq)
```

No backward pass and no index pass for the tertiary schedule.

### 8.4 Shadow Schedule

Must be computed **after** §8.1, §8.2, and §8.3 are complete.

```txt
for i in 0..SHADOW_SCHEDULE_SIZE:
    p = primary_schedule[i * 4 modulo PRIMARY_SCHEDULE_SIZE]
    s = secondary_schedule[i * 3 modulo SECONDARY_SCHEDULE_SIZE]
    t = tertiary_schedule[i * 2 modulo TERTIARY_SCHEDULE_SIZE]
    shadow_schedule[i] = mix64_fold(p, s, t, shadow_state_value XOR seq)
```

No additional passes for the shadow schedule.

---

## 9. Route Mutation

Route mutation operates in **three successive layers**.
All three intermediate values must be computed before encoding begins.
All steps that reference the route use `real_route` (the third-layer result), not `visible_route`.

### 9.1 First-Layer Route

```txt
real_route_1 = visible_route XOR low_byte(primary_schedule[seq modulo PRIMARY_SCHEDULE_SIZE])
```

### 9.2 Second-Layer Route

```txt
real_route_2 = real_route_1 XOR low_byte(secondary_schedule[real_route_1 modulo SECONDARY_SCHEDULE_SIZE])
```

### 9.3 Third-Layer Route (Final)

```txt
real_route = real_route_2 XOR low_byte(shadow_schedule[real_route_2 modulo SHADOW_SCHEDULE_SIZE])
```

`real_route_1` and `real_route_2` are intermediate values and must not be used in encoding steps.

---

## 10. Binary Frame Format

Before payload transformation, the semantic message is packed into a binary frame.

```txt
[version:                   1 byte  — low8 of PROTOCOL_VERSION]
[opcode:                    1 byte]
[arg_count:                 1 byte]
[arg_bytes:                 variable]
[zone_cycle_tag:            2 bytes, big-endian — low16(primary_epoch modulo ZONE_CYCLE_PERIOD)]
[secondary_zone_cycle_tag:  2 bytes, big-endian — low16(primary_epoch modulo SECONDARY_ZONE_CYCLE_PERIOD)]
[seq:                       5 bytes, big-endian low 40 bits]
[primary_epoch:             5 bytes, big-endian low 40 bits]
[secondary_epoch_offset:    4 bytes, big-endian low 32 bits]
[state_fingerprint_primary: 4 bytes]
[state_fingerprint_secondary:4 bytes]
[shadow_fingerprint:        2 bytes]
[frame_check:               4 bytes]
```

### 10.1 state_fingerprint_primary

```txt
state_fingerprint_primary = low32(mix64_primary(primary_state_value, seq, primary_epoch, real_route))
```

### 10.2 state_fingerprint_secondary

```txt
state_fingerprint_secondary = low32(mix64_secondary(secondary_state_value, seq, secondary_fold_counter, real_route))
```

### 10.3 shadow_fingerprint

```txt
shadow_fingerprint = low16(mix64_shadow(shadow_state_value, shadow_accumulator, seq, shadow_parity))
```

### 10.4 frame_hash and frame_check

```txt
frame_hash = INITIAL_PROTOCOL_EPOCH_A
for each byte b at index i in the frame (excluding frame_check bytes):
    frame_hash = mix64_primary(frame_hash, b, i, seq)

frame_check = low32(
    mix64_primary(
        frame_hash,
        seq,
        primary_epoch,
        state_fingerprint_primary XOR state_fingerprint_secondary))
```

---

## 11. Encoding Pipeline — Overview

Encoding applies preparation steps (§12) followed by exactly **60 payload transformation steps** (§13) followed by post-transformation steps (§13.P1–§13.P8).

Decoder applies all steps in exact reverse order.
The decoder's transformation reverse-pass runs Steps 60 down to 20.
After Step 20 is reversed, frame validation is performed (§15 steps 55–58).
After frame validation succeeds, the frame is parsed, zones are un-indirected, and the semantic message is recovered.

**Steps 1–19 are preparation. Binary payload transformation begins at Step 20.**

---

## 12. Preparation Steps 1–19

### Step 1: Normalize Message

```txt
trim leading and trailing spaces
collapse any whitespace sequence to a single space
remove trailing newlines
fold all protocol keywords to lowercase
reject empty messages
reject messages exceeding 255 bytes after normalization
```

Example: `"  MEET   12\n"` → `"meet 12"`

### Step 2: Unicode Guard

Reject any byte above `0x7F`. All semantic message text is pure 7-bit ASCII.

### Step 3: Convert to Semantic Opcode

Convert the normalized message to `(opcode, arg_count, arg_bytes)` using §5.
Reject unknown opcodes.
Reject argument counts that do not match the opcode's defined format.
Reject argument values outside their valid ranges.

### Step 4: Primary Zone ID Indirection

All zone arguments must be indirected before frame construction.

```txt
zone_id = zone_table_primary[x][y][primary_epoch modulo ZONE_CYCLE_PERIOD]
```

Zone tables are shared knowledge precomputed at startup from `map_width`, `map_height`, and `INITIAL_PROTOCOL_EPOCH_A`. See §24.3.

### Step 5: Secondary Zone ID Indirection

Apply secondary indirection to the result of Step 4.

```txt
zone_id = zone_table_secondary[zone_id modulo secondary_zone_table_width][secondary_epoch_offset modulo SECONDARY_ZONE_CYCLE_PERIOD]
```

`secondary_zone_table_width = (SECONDARY_ZONE_CYCLE_PERIOD modulo 64) + 32`
The secondary zone table is precomputed at startup from `INITIAL_PROTOCOL_EPOCH_B`.

### Step 6: Generate All Four Schedules

See §8. All four schedules are generated here.
The schedules are inputs to Steps 20–60 and post-steps P1–P8.
No encoding step prior to Step 20 uses the schedules.

### Step 7: Compute Route Mutation

See §9. Compute `real_route_1`, `real_route_2`, and `real_route`.

### Step 8: Build Binary Frame

See §10. Construct the full binary frame including all fingerprints and frame check.

---

## 13. Transformation Steps 20–60

The binary frame is now the byte vector called `payload`.
Steps 20–60 transform this payload in-place.
All references to `schedule` indices are always reduced modulo the respective schedule's size.

---

### Step 20: Byte Expansion (1 byte → 6 bytes)

Each byte `b` at index `i` becomes six consecutive bytes:

```txt
A = high_nibble(b) XOR low_byte(primary_schedule[i modulo PRIMARY_SCHEDULE_SIZE])
B = low_nibble(b)  XOR low_byte(primary_schedule[(i+1) modulo PRIMARY_SCHEDULE_SIZE])
C = rotate_left(b, primary_schedule[(i+2) modulo PRIMARY_SCHEDULE_SIZE] modulo BYTE_BITS)
D = b XOR low_byte(primary_schedule[(i+3) modulo PRIMARY_SCHEDULE_SIZE]) XOR real_route
E = b XOR low_byte(secondary_schedule[i modulo SECONDARY_SCHEDULE_SIZE])
      XOR low_byte(shadow_schedule[i modulo SHADOW_SCHEDULE_SIZE])
F = low_byte(mix64_shadow(b, i, seq, real_route))
```

The expanded index `i` refers to the original pre-expansion byte index.

### Step 21: Byte Expansion Consistency Verification (Encoder Self-Check)

The encoder verifies its own expansion. The decoder uses this to reject corrupt packets.

```txt
reconstructed_b = (A AND 0x0F) << NIBBLE_BITS | (B AND 0x0F)
verify: C == rotate_left(reconstructed_b, primary_schedule[(i+2)%P] modulo BYTE_BITS)
verify: D == reconstructed_b XOR low_byte(primary_schedule[(i+3)%P]) XOR real_route
verify: E == reconstructed_b XOR low_byte(secondary_schedule[i%S]) XOR low_byte(shadow_schedule[i%SH])
verify: F == low_byte(mix64_shadow(reconstructed_b, i, seq, real_route))
```

Any verification failure on decode causes immediate silent discard.

---

### Step 22: Primary Position XOR Lattice

For every byte at index `i` (note: little-endian index interpretation — see §3):

```txt
mask = low_byte(mix64_primary(seq, primary_epoch, i, primary_state_value))
byte = byte XOR mask
```

### Step 23: Secondary Position XOR Lattice

For every byte at index `i`:

```txt
mask = low_byte(mix64_secondary(seq, secondary_state_value, i, secondary_fold_counter))
byte = byte XOR mask
```

Step 22 is applied first; Step 23 is applied second.
Decoder reverses in opposite order: undo Step 23, then undo Step 22.

---

### Step 24: Rolling Stream A

```txt
stream_a = seq XOR primary_epoch XOR primary_state_value XOR STATE_MULTIPLIER_A
for each byte at index i:
    stream_a = stream_a * STATE_MULTIPLIER_A + primary_schedule[i modulo PRIMARY_SCHEDULE_SIZE]
    byte = byte XOR low_byte(stream_a)
```

### Step 25: Rolling Stream B (Feedback)

```txt
stream_b  = primary_last_check XOR primary_epoch XOR STATE_MULTIPLIER_B
previous  = low_byte(primary_state_value)
for each byte at index i:
    stream_b = stream_b * STATE_MULTIPLIER_B + primary_schedule[(i+1) modulo PRIMARY_SCHEDULE_SIZE]
    feedback = rotate_left(previous, primary_schedule[i modulo PRIMARY_SCHEDULE_SIZE] modulo BYTE_BITS)
    byte     = byte XOR low_byte(stream_b) XOR feedback
    previous = byte
```

Decoder must process bytes in forward order and reconstruct `previous` exactly.

### Step 26: Rolling Stream C (Double Feedback)

```txt
stream_c      = secondary_state_value XOR shadow_state_value XOR STATE_MULTIPLIER_C
prev_c        = low_byte(secondary_state_value)
prev2_c       = low_byte(shadow_state_value)
for each byte at index i:
    stream_c  = stream_c * STATE_MULTIPLIER_C + secondary_schedule[i modulo SECONDARY_SCHEDULE_SIZE]
    feedback  = rotate_left(
                    prev_c XOR prev2_c,
                    tertiary_schedule[i modulo TERTIARY_SCHEDULE_SIZE] modulo BYTE_BITS)
    byte      = byte XOR low_byte(stream_c) XOR feedback
    prev2_c   = prev_c
    prev_c    = byte
```

---

### Step 27: Primary Rotating Byte Ring

```txt
for each byte at index i:
    rotation = primary_schedule[i modulo PRIMARY_SCHEDULE_SIZE] modulo BYTE_BITS
    byte = rotate_left(byte, rotation)
```

### Step 28: Secondary Rotating Byte Ring (Reverse Direction)

```txt
for each byte at index i:
    rotation = secondary_schedule[i modulo SECONDARY_SCHEDULE_SIZE] modulo BYTE_BITS
    byte = rotate_right(byte, rotation)
```

Step 27 is applied first; Step 28 second.
Decoder undoes Step 28 before Step 27.

---

### Step 29: Primary Variable Block Reversal

```txt
block_size = MATRIX_MIN_WIDTH + primary_schedule[block_index modulo PRIMARY_SCHEDULE_SIZE] modulo MATRIX_WIDTH_RANGE
```

Reverse each block. `block_index` increments once per block, starting at 0.

### Step 30: Secondary Variable Block Reversal

Applied to the result of Step 29.

```txt
block_size = MATRIX_SECONDARY_MIN_WIDTH + secondary_schedule[block_index modulo SECONDARY_SCHEDULE_SIZE] modulo MATRIX_SECONDARY_WIDTH_RANGE
```

Decoder undoes Step 30 before Step 29.

---

### Step 31: Primary Braid Permutation

Partition all byte indices into three groups:

```txt
group_a: index where (index modulo 3) == low2bits(primary_schedule[index modulo PRIMARY_SCHEDULE_SIZE])
group_b: index where (index modulo 5) == (low3bits(primary_schedule[index modulo PRIMARY_SCHEDULE_SIZE]) modulo 5)
group_c: all remaining indices (indices that satisfy neither condition, or both)
```

Indices satisfying both conditions belong to `group_a`. Indices satisfying neither belong to `group_c`.

Rebuild the byte vector as:

```txt
reverse(group_b) + group_c + group_a
```

### Step 32: Secondary Braid Permutation

Applied to the result of Step 31. Partition all indices into four groups:

```txt
group_p: index where (index modulo 7)  == (low3bits(secondary_schedule[index modulo SECONDARY_SCHEDULE_SIZE]) modulo 7)
group_q: index where (index modulo 11) == (low4bits(secondary_schedule[index modulo SECONDARY_SCHEDULE_SIZE]) modulo 11)
group_r: index where (index modulo 13) == (low4bits(secondary_schedule[(index+1) modulo SECONDARY_SCHEDULE_SIZE]) modulo 13)
group_s: all remaining indices
```

Indices satisfying multiple conditions: priority is `p > q > r > s` (earlier group wins).

Rebuild:

```txt
group_s + reverse(group_r) + group_p + reverse(group_q)
```

Note the asymmetry: `group_s` and `group_p` are not reversed; `group_r` and `group_q` are.
Decoder must reconstruct the original partitions exactly and invert both permutations in reverse order.

---

### Step 33: High Nibble Substitution

```rust
const HIGH_NIBBLE_SBOX: [u8; 16] = [
    0x0C, 0x05, 0x0A, 0x01,
    0x0F, 0x03, 0x0E, 0x08,
    0x00, 0x0D, 0x04, 0x09,
    0x02, 0x07, 0x0B, 0x06,
];
```

```txt
for each byte:
    byte = (HIGH_NIBBLE_SBOX[high_nibble(byte)] << NIBBLE_BITS) | low_nibble(byte)
```

### Step 34: Low Nibble Substitution

```rust
const LOW_NIBBLE_SBOX: [u8; 16] = [
    0x07, 0x0E, 0x01, 0x0B,
    0x04, 0x0C, 0x00, 0x09,
    0x0D, 0x02, 0x0F, 0x05,
    0x0A, 0x03, 0x06, 0x08,
];
```

```txt
for each byte:
    byte = (high_nibble(byte) << NIBBLE_BITS) | LOW_NIBBLE_SBOX[low_nibble(byte)]
```

Decoder must build inverse tables for both S-boxes. Inverse tables are precomputed at initialization.

### Step 35: Full-Byte Schedule-Keyed Substitution

The full-byte S-box is not a compile-time constant.
It is generated at startup via a seeded Fisher–Yates shuffle:

```txt
for i in 0..256:
    sbox_primary[i] = i as u8

seed = mix64_primary(INITIAL_PROTOCOL_EPOCH_A, INITIAL_PROTOCOL_EPOCH_B, INITIAL_PROTOCOL_EPOCH_C, STATE_MULTIPLIER_A)
for i in (1..256).rev():
    seed   = mix64_primary(seed, i as u64, INITIAL_PROTOCOL_EPOCH_A, STATE_MULTIPLIER_C)
    j      = low_byte(seed) modulo (i + 1)
    swap sbox_primary[i] and sbox_primary[j]
```

This produces a bijection. The inverse is computed by building `sbox_primary_inv[sbox_primary[i]] = i`.

For each byte at index `i` during encoding:

```txt
rotation_offset = low_byte(shadow_schedule[seq modulo SHADOW_SCHEDULE_SIZE])
effective_index = (byte + rotation_offset) modulo 256
byte            = sbox_primary[effective_index]
```

Decoder computes the same `rotation_offset`, builds the offset-rotated inverse, and applies it.

---

### Step 36: GF(256) Galois Field Multiplication Pass

```txt
for each byte at index i:
    gf_key = low_byte(tertiary_schedule[i modulo TERTIARY_SCHEDULE_SIZE])
    if gf_key == 0:
        gf_key = 1     // GF multiplication by zero is non-invertible; clamp to 1
    byte = gf_multiply(byte, gf_key)
```

`gf_multiply(a, b)` performs multiplication in GF(2^8) under `GF256_IRREDUCIBLE_POLY`.
Decoder applies `gf_multiply(byte, gf_inverse(gf_key))` using the precomputed GF inverse table.

---

### Step 37: Primary Index Polynomial Drift (Degree 5)

```txt
for each byte at index i:
    i2   = i as u64
    drift = low_byte(i2^5 + seq * i2^3 + primary_epoch * i2^2 + primary_state_value * i2 + real_route)
    byte  = byte + drift
```

All arithmetic wrapping u64. Only low byte of the final polynomial value is used as `drift`.

### Step 38: Secondary Index Polynomial Drift (Degree 7)

Applied after Step 37.

```txt
for each byte at index i:
    i2    = i as u64
    drift2 = low_byte(i2^7 + secondary_state_value * i2^5 + seq * i2^3 + secondary_fold_counter * i2 + shadow_accumulator)
    byte   = byte + drift2
```

Decoder subtracts `drift2` first (Step 38 inverse), then subtracts `drift` (Step 37 inverse).

---

### Step 39: Primary Checksum Byte Injection

Insert primary checksum bytes at dynamic intervals. Maintain a `previous_fold` running XOR of all payload bytes seen so far before each insertion.

```txt
interval     = CHECKSUM_INTERVAL_MIN + primary_schedule[insertion_counter modulo PRIMARY_SCHEDULE_SIZE] modulo CHECKSUM_INTERVAL_RANGE
check_byte   = low_byte(mix64_primary(previous_fold, seq, primary_epoch, insertion_counter))
```

`insertion_counter` increments once per insertion.

Decoder regenerates the same intervals and removes primary checksum bytes, validating each against the running fold.

### Step 40: Secondary Checksum Byte Injection

Maintain a separate `previous_secondary_fold`. Secondary intervals are computed independently:

```txt
interval2    = SECONDARY_CHECKSUM_INTERVAL_MIN + secondary_schedule[j modulo SECONDARY_SCHEDULE_SIZE] modulo SECONDARY_CHECKSUM_INTERVAL_RANGE
check_byte2  = low_byte(mix64_secondary(previous_secondary_fold, seq, secondary_state_value, j))
```

`j` is a separate secondary insertion counter.
Secondary checksum bytes are inserted into the vector that already contains primary checksum bytes.
Decoder removes secondary checksum bytes **before** removing primary checksum bytes.

---

### Step 41: Primary Matrix Transposition

```txt
width = MATRIX_MIN_WIDTH + primary_schedule[0] modulo MATRIX_WIDTH_RANGE
```

Fill rows, read columns (standard column-major transposition).
Padding bytes for incomplete rows:

```txt
pad = low_byte(primary_schedule[padding_index modulo PRIMARY_SCHEDULE_SIZE])
```

Decoder removes padding and undoes the transposition.

### Step 42: Secondary Matrix Transposition

Applied to the result of Step 41.

```txt
width = MATRIX_SECONDARY_MIN_WIDTH + secondary_schedule[0] modulo MATRIX_SECONDARY_WIDTH_RANGE
```

Padding bytes:

```txt
pad = low_byte(secondary_schedule[padding_index modulo SECONDARY_SCHEDULE_SIZE]) XOR real_route
```

Decoder undoes Step 42 before Step 41.

---

### Step 43: Primary Diagonal Matrix Walk

```txt
width = MATRIX_MIN_WIDTH + primary_schedule[1] modulo MATRIX_WIDTH_RANGE
```

Fill row-by-row. Read top-left-to-bottom-right diagonals. Every **even-indexed** diagonal is read in reverse.

### Step 44: Secondary Diagonal Matrix Walk

Applied to the result of Step 43.

```txt
width = MATRIX_SECONDARY_MIN_WIDTH + secondary_schedule[1] modulo MATRIX_SECONDARY_WIDTH_RANGE
```

Fill row-by-row. Read **bottom-right-to-top-left** diagonals. Every **odd-indexed** diagonal is read in reverse.

The asymmetry between primary (top-left to bottom-right, even diagonals reversed) and secondary (bottom-right to top-left, odd diagonals reversed) is intentional.
Decoder undoes Step 44 before Step 43.

---

### Step 45: Spiral Matrix Walk

```txt
width = MATRIX_TERTIARY_MIN_WIDTH + tertiary_schedule[0] modulo MATRIX_TERTIARY_WIDTH_RANGE
```

Fill in row-major order. Read out in **clockwise spiral** order starting from the top-left corner.
This step uses **little-endian** index interpretation for its padding byte formula (see §3).

Padding bytes for incomplete spiral:

```txt
pad = low_byte(shadow_schedule[padding_index modulo SHADOW_SCHEDULE_SIZE])
```

### Step 46: Hilbert Curve Permutation

```txt
hilbert_n = smallest power of 2 such that hilbert_n^2 >= current payload length
```

Fill the `hilbert_n × hilbert_n` matrix in row-major order (schedule-derived padding for remainder).
Read out following the **2D Hilbert curve** traversal order for order `log2(hilbert_n)`.

For `hilbert_n > 64`, use an iterative (non-recursive) Hilbert curve implementation.
The Hilbert curve traversal sequence must be identical on encoder and decoder.

Padding bytes:

```txt
pad = low_byte(mix64_shadow(hilbert_n, padding_index, seq, shadow_state_value))
```

---

### Step 47: Primary Conditional Complement

```txt
for each byte at index i:
    cond = low_bit(mix64_primary(i, seq, primary_epoch, primary_state_value))
    if cond == 1:
        byte = NOT byte
```

### Step 48: Secondary Conditional Complement

```txt
for each byte at index i:
    cond = low_bit(mix64_secondary(i, seq, secondary_state_value, shadow_accumulator))
    if cond == 1:
        byte = NOT byte
```

Primary and secondary complement conditions are independent.
Decoder undoes Step 48, then undoes Step 47.

---

### Step 49: Primary Whole-Array Byte Rotation

```txt
rotation = mix64_primary(seq, primary_epoch, payload_length, primary_state_value) modulo payload_length
```

Rotate the entire byte vector left by `rotation` positions.

### Step 50: Secondary Whole-Array Word Rotation

Treat the byte vector as an array of `u64` words (little-endian, last word zero-padded).

```txt
word_count   = ceil(payload_length / 8)
word_rotation = mix64_secondary(seq, secondary_state_value, word_count, secondary_epoch_offset) modulo word_count
```

Rotate the word array left by `word_rotation` positions. Re-extract bytes little-endian, discard padding.

This step uses **little-endian** word encoding (see §3).
Decoder undoes Step 50 (word rotation) before undoing Step 49 (byte rotation).

---

### Step 51: Primary Deterministic Decoy Insertion

```txt
interval   = DECOY_INTERVAL_MIN + primary_schedule[decoy_counter modulo PRIMARY_SCHEDULE_SIZE] modulo DECOY_INTERVAL_RANGE
decoy_byte = low_byte(mix64_primary(seq, primary_epoch, insert_position, primary_last_check))
```

`decoy_counter` increments once per decoy inserted.

### Step 52: Secondary Deterministic Decoy Insertion

Applied to the vector already containing primary decoys.

```txt
interval2   = SECONDARY_DECOY_INTERVAL_MIN + secondary_schedule[j modulo SECONDARY_SCHEDULE_SIZE] modulo SECONDARY_DECOY_INTERVAL_RANGE
decoy_byte2 = low_byte(mix64_secondary(seq, secondary_state_value, insert_position, shadow_accumulator))
```

Decoder removes secondary decoys first, then primary decoys.

---

### Step 53: Primary Bit Permutation

```rust
const BIT_PERMUTATION_PRIMARY: [u8; 8] = [5, 2, 7, 0, 6, 1, 4, 3];
```

Applied to every byte:

```txt
output_bit[i] = input_bit[BIT_PERMUTATION_PRIMARY[i]]
```

### Step 54: Secondary Bit Permutation (Conditional)

```rust
const BIT_PERMUTATION_SECONDARY: [u8; 8] = [3, 6, 0, 5, 1, 7, 2, 4];
```

Applied only to bytes at indices where:

```txt
low_bit(mix64_shadow(i, seq, shadow_parity, shadow_sequence_parity)) == 1
```

Decoder evaluates the same condition and applies the inverse permutation only to matching bytes.
Decoder undoes Step 54 before Step 53.

---

### Step 55: Primary Pairwise Byte Mixing

For every pair `(byte[2k], byte[2k+1])`:

```txt
rotation = primary_schedule[k modulo PRIMARY_SCHEDULE_SIZE] modulo BYTE_BITS
new_a    = byte[2k]   XOR rotate_left(byte[2k+1], rotation)
new_b    = byte[2k+1] + rotate_right(byte[2k], rotation)
```

If payload length is odd, last byte is mixed with:

```txt
byte[last] = byte[last] XOR low_byte(primary_schedule[last modulo PRIMARY_SCHEDULE_SIZE])
```

### Step 56: Secondary Triplet Byte Mixing

For every triplet `(byte[3k], byte[3k+1], byte[3k+2])`:

```txt
r1    = secondary_schedule[k modulo SECONDARY_SCHEDULE_SIZE] modulo BYTE_BITS
r2    = tertiary_schedule[k modulo TERTIARY_SCHEDULE_SIZE] modulo BYTE_BITS
new_a = byte[3k]   XOR rotate_left(byte[3k+1], r1) XOR rotate_right(byte[3k+2], r2)
new_b = byte[3k+1] + rotate_left(byte[3k], r1)
new_c = byte[3k+2] XOR rotate_right(new_a, r2) + new_b
```

Remaining 1 or 2 bytes after the last complete triplet:

```txt
remaining_mask = low_byte(mix64_secondary(seq, secondary_state_value, k, real_route))
byte[n] = byte[n] XOR remaining_mask
```

Decoder undoes Step 56 before Step 55. Triplet reconstruction must use `new_a` in the `new_c` formula exactly as defined.

---

### Step 57: Primary Feistel Network

Split payload into left and right halves. If odd length, right half receives the extra byte.
Run `FEISTEL_ROUNDS_PRIMARY` rounds.

```txt
for r in 0..FEISTEL_ROUNDS_PRIMARY:
    round_key = primary_schedule[r modulo PRIMARY_SCHEDULE_SIZE]
    new_left  = right
    new_right = left XOR feistel_f_primary(right, round_key, r)
    left      = new_left
    right     = new_right
```

`feistel_f_primary(data, round_key, r)`:

```txt
for each byte at index i:
    mask = low_byte(mix64_primary(round_key, i, r, byte))
    byte = rotate_left(byte XOR mask, mask modulo BYTE_BITS)
```

### Step 58: Secondary Feistel Network

Applied to the result of Step 57.
Round keys are mixed from both schedules:

```txt
for r in 0..FEISTEL_ROUNDS_SECONDARY:
    round_key = secondary_schedule[r modulo SECONDARY_SCHEDULE_SIZE]
                XOR primary_schedule[(r + FEISTEL_ROUNDS_PRIMARY) modulo PRIMARY_SCHEDULE_SIZE]
    new_left  = right
    new_right = left XOR feistel_f_secondary(right, round_key, r)
    left      = new_left
    right     = new_right
```

`feistel_f_secondary(data, round_key, r)`:

```txt
for each byte at index i:
    mask       = low_byte(mix64_secondary(round_key, i, r, byte))
    gf_factor  = mask | 1       // ensure nonzero for GF invertibility
    byte       = gf_multiply(rotate_left(byte XOR mask, mask modulo BYTE_BITS), gf_factor)
```

### Step 59: Tertiary Feistel Network (Conditional)

Applied **only if** the payload length after Step 58 is **odd**.
If even, this step is a no-op on both encoder and decoder.

```txt
for r in 0..FEISTEL_ROUNDS_TERTIARY:
    round_key = tertiary_schedule[r modulo TERTIARY_SCHEDULE_SIZE]
                XOR shadow_schedule[r modulo SHADOW_SCHEDULE_SIZE]
    new_left  = right
    new_right = left XOR feistel_f_tertiary(right, round_key, r)
    left      = new_left
    right     = new_right
```

`feistel_f_tertiary(data, round_key, r)`:

```txt
for each byte at index i:
    mask = low_byte(mix64_shadow(round_key, i, r, shadow_state_value))
    byte = byte XOR mask XOR rotate_left(
               low_byte(shadow_schedule[i modulo SHADOW_SCHEDULE_SIZE]),
               r modulo BYTE_BITS)
```

---

### Step 60: Final Polynomial Mask Pass

```txt
for each byte at index i:
    i2   = i as u64
    mask = low_byte(i2^5 + seq * i2^3 + primary_epoch * i2 + real_route)
    byte = byte + mask
```

---

## 13. Post-Transformation Steps (P1–P8)

These steps convert the transformed binary payload into the final body string.

### Step P1: Primary Shadow Check Injection

```txt
payload_hash = frame_hash computed over all post-Step-60 payload bytes
               using same formula as §10.4 but over the transformed bytes

shadow_check  = low32(mix64_primary(payload_hash, seq, primary_epoch, primary_state_value))
shadow_check  = shadow_check XOR low32(rotate_left(primary_epoch, CHECK_ROTATION_A))
shadow_check  = shadow_check XOR low32(rotate_right(primary_last_check, CHECK_ROTATION_B))
shadow_check  = shadow_check XOR low32(mix64_shadow(shadow_accumulator, shadow_parity, shadow_state_value, seq))
```

Append four bytes of `shadow_check` (big-endian) to the payload.

### Step P2: Secondary Shadow Check Injection

```txt
secondary_shadow = low16(mix64_secondary(payload_hash, seq, secondary_state_value, shadow_check))
secondary_shadow = secondary_shadow XOR low16(rotate_right(secondary_epoch_offset, CHECK_ROTATION_E))
```

Append two bytes of `secondary_shadow` (big-endian) to the payload.

### Step P3: Primary Custom Base Encoding

Encode binary payload using the primary custom alphabet:

```txt
CUSTOM_ALPHABET_PRIMARY =
Qx7ZaP0LmN9bVcK2sD8fGhJ3kT5yUiO1rE4wY6tHuIpASdFgXzCvBnMje
```

Rotate by:

```txt
offset = primary_schedule[2] modulo len(CUSTOM_ALPHABET_PRIMARY)
```

### Step P4: Secondary Custom Base Encoding

Treat the output of P3 as a binary input (ASCII values) and re-encode using the secondary custom alphabet:

```txt
CUSTOM_ALPHABET_SECONDARY =
abcdefghjkmnpqrstuvwxyz23456789ABCDEFGHJKMNPQRSTUVWXYZwz!@#
```

Rotate by:

```txt
offset2 = secondary_schedule[2] modulo len(CUSTOM_ALPHABET_SECONDARY)
```

Decoder decodes secondary pass first, then primary pass.

### Step P5: Primary Text Chunk Permutation

```txt
chunk_size = CHUNK_SIZE_MIN + primary_schedule[3] modulo CHUNK_SIZE_RANGE
```

Split text into chunks of `chunk_size`. Generate permutation from the primary schedule.
Reorder chunks according to the permutation.

### Step P6: Secondary Text Chunk Permutation (Sliding Window)

```txt
window_size = SECONDARY_CHUNK_SIZE_MIN + secondary_schedule[3] modulo SECONDARY_CHUNK_SIZE_RANGE
step        = max(1, window_size / 2)
```

For each window position, permute the characters within the window using the Lehmer code at index:

```txt
perm_index = shadow_schedule[window_index modulo SHADOW_SCHEDULE_SIZE] modulo factorial(window_size)
```

Decoder applies the inverse Lehmer code permutation.
Window positions advance by `step`, not `window_size` — windows overlap.

### Step P7: Primary Text Noise Injection

```txt
insert when: low_byte(mix64_primary(seq, primary_epoch, index, primary_schedule[index modulo PRIMARY_SCHEDULE_SIZE])) modulo 5 == 0
noise_char:  NOISE_CHARS_PRIMARY[primary_schedule[index modulo PRIMARY_SCHEDULE_SIZE] modulo 4]
```

`index` is the current position in the output string being built.

### Step P8: Secondary Text Noise Injection

```txt
insert when: low_byte(mix64_secondary(seq, secondary_state_value, index, secondary_schedule[index modulo SECONDARY_SCHEDULE_SIZE])) modulo 7 == 0
noise_char:  NOISE_CHARS_SECONDARY[secondary_schedule[index modulo SECONDARY_SCHEDULE_SIZE] modulo 4]
```

`index` counts positions in the string **after** primary noise is already present.
Decoder removes secondary noise first (P8 inverse), then primary noise (P7 inverse).

The resulting string after P8 becomes the `body` field of the regular packet.

---

## 14. Final Packet Checks

### 14.1 Inner Check

Covers fields `ver36` through `body` inclusive.

```txt
inner = mix64_primary(seq, primary_epoch, visible_route, primary_last_check)
inner = mix64_primary(inner, body_hash, primary_state_value, body_length)
inner = mix64_secondary(inner, secondary_state_value, secondary_fold_counter, shadow_accumulator)
inner = rotate_left(inner, CHECK_ROTATION_C)
```

`body_hash` is:

```txt
body_hash = INITIAL_PROTOCOL_EPOCH_A
for each character c at index i in the body string:
    body_hash = mix64_primary(body_hash, c as u64, i, seq)
```

### 14.2 Outer Check

Covers all fields 1–9 including `inner_check36`.

```txt
outer = mix64_fold(inner, seq, primary_epoch, shadow_state_value)
outer = mix64_primary(outer, body_length, visible_route, primary_epoch XOR secondary_epoch_offset)
outer = outer XOR low64(mix64_shadow(shadow_parity, shadow_sequence_parity, primary_last_check, outer))
outer = rotate_right(outer, CHECK_ROTATION_H)
```

A receiver must verify the outer check before the inner check.
A receiver must verify the inner check before attempting full payload decode.

---

## 15. Decoding Order

Decoding applies all preparation and transformation steps in exact reverse.

```txt
1.  Parse packet fields
2.  Parse base36 fields (ver36, seq36, epoch36, route36, session36, frag36, inner_check36, outer_check36)
3.  Verify protocol version == PROTOCOL_VERSION
4.  Verify outer check (§14.2)
5.  Check bloom filter (§6.6)
6.  Generate all four schedules (§8.1–§8.4)
7.  Compute route mutation (§9)
8.  Verify inner check (§14.1)
9.  Verify session token (§19.4)
10. Remove secondary text noise [P8 inverse]
11. Remove primary text noise [P7 inverse]
12. Invert secondary sliding-window chunk permutation [P6 inverse]
13. Invert primary chunk permutation [P5 inverse]
14. Decode secondary custom base alphabet [P4 inverse]
15. Decode primary custom base alphabet [P3 inverse]
16. Remove and verify secondary shadow check [P2 inverse]
17. Remove and verify primary shadow check [P1 inverse]
18. Undo Step 60 (polynomial mask pass)
19. Undo Step 59 (tertiary Feistel — only if payload is odd length)
20. Undo Step 58 (secondary Feistel)
21. Undo Step 57 (primary Feistel)
22. Undo Step 56 (secondary triplet mixing)
23. Undo Step 55 (primary pairwise mixing)
24. Undo Step 54 (secondary bit permutation — conditional)
25. Undo Step 53 (primary bit permutation)
26. Remove secondary decoys [Step 52 inverse]
27. Remove primary decoys [Step 51 inverse]
28. Undo Step 50 (secondary word rotation)
29. Undo Step 49 (primary byte rotation)
30. Undo Step 48 (secondary conditional complement)
31. Undo Step 47 (primary conditional complement)
32. Undo Step 46 (Hilbert curve permutation)
33. Undo Step 45 (spiral matrix walk)
34. Undo Step 44 (secondary diagonal matrix walk)
35. Undo Step 43 (primary diagonal matrix walk)
36. Undo Step 42 (secondary matrix transposition)
37. Undo Step 41 (primary matrix transposition)
38. Remove and verify secondary checksum bytes [Step 40 inverse]
39. Remove and verify primary checksum bytes [Step 39 inverse]
40. Undo Step 38 (secondary polynomial drift)
41. Undo Step 37 (primary polynomial drift)
42. Undo Step 36 (GF(256) pass)
43. Undo Step 35 (full-byte S-box)
44. Undo Step 34 (low nibble substitution)
45. Undo Step 33 (high nibble substitution)
46. Undo Step 32 (secondary braid permutation)
47. Undo Step 31 (primary braid permutation)
48. Undo Step 30 (secondary variable block reversal)
49. Undo Step 29 (primary variable block reversal)
50. Undo Step 28 (secondary rotating byte ring)
51. Undo Step 27 (primary rotating byte ring)
52. Undo Step 26 (rolling stream C)
53. Undo Step 25 (rolling stream B with feedback)
54. Undo Step 24 (rolling stream A)
55. Undo Step 23 (secondary XOR lattice)
56. Undo Step 22 (primary XOR lattice)
57. Collapse byte expansion and verify all four consistency fields [Steps 21–20 inverse]
58. Verify frame check
59. Verify shadow_fingerprint
60. Verify state_fingerprint_secondary
61. Verify state_fingerprint_primary
62. Parse binary frame fields
63. Undo secondary zone indirection [Step 5 inverse]
64. Undo primary zone indirection [Step 4 inverse]
65. Reconstruct semantic message from opcode and arguments
66. Update all three state machines (§6.5)
67. Add seq to bloom filter (§20)
```

If any step fails, the message is silently ignored.
State is **not** updated for failed decodes.
`ko` is never sent; this protocol is embedded in `Broadcast` text.

---

## 16. Error Handling

A packet is silently discarded when any of the following occur:

```txt
prefix does not match L300, L300S, or L300F
field count does not match prefix type
protocol version field is not 1
base36 parsing fails for any numeric field
sequence number is in the bloom filter (§20)
sequence number is more than ANTI_REPLAY_WINDOW behind the current maximum
outer check fails
inner check fails
session token validation fails
primary custom base decoding fails
secondary custom base decoding fails
primary shadow check fails
secondary shadow check fails
frame check fails
state_fingerprint_primary fails
state_fingerprint_secondary fails
shadow_fingerprint fails
any byte expansion consistency check fails (Step 21)
any primary checksum byte validation fails (Step 39 inverse)
any secondary checksum byte validation fails (Step 40 inverse)
opcode is unknown or absent from §5
argument count does not match the opcode's defined arg format
any argument value is out of its valid range
zone ID cannot be resolved in either zone table
any Feistel function step returns a length inconsistency
Hilbert curve input length is zero
```

Invalid packets must not stop the AI.
Invalid packets must not update any protocol state.

---

## 17. Strategic Message Layer

Even after decoding, the message must not expose raw strategy directly.

Prefer:

```txt
plan 3
role anchor
meet zone 12
incant 2 zone 8
```

Avoid:

```txt
go to x=4 y=7
start incantation now
need linemate at 3 5
```

Use zones, roles, plans, and status codes from §5 throughout.
The decoded message is a strategy intent, not a direct command.

---

## 18. Fragmentation Protocol

If the encoded body produced after Step P8 exceeds `FRAGMENT_MAX_BODY_LENGTH` characters, the message must be fragmented before broadcast.

### 18.1 Fragment ID Assignment

```txt
frag_id = low64(mix64_primary(seq, primary_epoch, body_length, primary_state_value)) modulo (2^36 - 1)
```

`frag_id` must be nonzero. If zero, increment by 1.

### 18.2 Fragment Splitting

Split the body string into chunks of at most `FRAGMENT_MAX_BODY_LENGTH` characters.
Fragments are numbered starting from 0.
`frag_total` equals the total fragment count.

### 18.3 Fragment Packet Format

```txt
L300F|<ver36>|<seq36>|<frag_id36>|<frag_index36>|<frag_total36>|<epoch36>|<route36>|<fragment_body>|<outer_check36>
```

Fragment outer check:

```txt
frag_outer = mix64_fold(seq, primary_epoch, frag_id, frag_index)
frag_outer = mix64_primary(frag_outer, fragment_body_hash, primary_state_value, frag_total)
frag_outer = rotate_left(frag_outer, CHECK_ROTATION_G)
```

`fragment_body_hash` is computed using the same running-hash formula as `body_hash` (§14.1) applied to the fragment body substring.

### 18.4 Reassembly

Collect fragments by `frag_id`.
A fragment set is complete when `frag_total` fragments for the same `frag_id` have been received.
Fragment packets arriving more than `ANTI_REPLAY_WINDOW` seq steps after the first fragment for a given `frag_id` are discarded.
Reassembled body is processed as a regular packet body starting from Step P7 inverse onward.

---

## 19. Session Negotiation

A session token must be established before any regular packet can be decoded from a given source.
A decoder that receives a regular packet from a source with no established session for that source silently discards it.

### 19.1 Session Token Generation

```txt
token  = mix64_primary(primary_state_value, seq, primary_epoch, INITIAL_PROTOCOL_EPOCH_C)
token  = mix64_secondary(token, secondary_state_value, secondary_fold_counter, INITIAL_PROTOCOL_EPOCH_B)
token  = token XOR mix64_shadow(shadow_state_value, shadow_accumulator, seq, primary_epoch)
session_token = token
```

### 19.2 Session Packet Checks

```txt
sess_check = rotate_right(
                 mix64_primary(seq, primary_epoch, token, primary_state_value),
                 CHECK_ROTATION_D)

sess_outer = rotate_left(
                 mix64_fold(seq, primary_epoch, sess_check, secondary_state_value),
                 CHECK_ROTATION_F)
```

### 19.3 Session Acceptance

When a valid session packet is received and all checks pass:

```txt
primary_session_token = token
// Also apply §6.5 state update using the session packet's seq
```

### 19.4 Regular Packet Session Field Validation

The `session36` field in a regular packet must equal:

```txt
expected = low64(mix64_primary(primary_session_token, seq, primary_epoch, primary_state_value)) modulo (2^36 - 1)
```

Mismatch causes silent discard without state update.

---

## 20. Anti-Replay Bloom Filter

Filter size: `ANTI_REPLAY_WINDOW * BLOOM_HASH_FUNCTIONS * 8` bits.

Insert sequence `s`:

```txt
for k in 0..BLOOM_HASH_FUNCTIONS:
    bit = mix64_primary(s, k, primary_epoch, STATE_MULTIPLIER_A + k as u64) modulo filter_bit_count
    set bit
```

Query sequence `s` (returns `true` if probably seen):

```txt
for k in 0..BLOOM_HASH_FUNCTIONS:
    bit = mix64_primary(s, k, primary_epoch, STATE_MULTIPLIER_A + k as u64) modulo filter_bit_count
    if bit is not set: return false
return true
```

False positives cause valid allied packets to be silently dropped. This is an acceptable rare loss.
False positives must not update any state.

---

## 21. Epoch Consensus

### 21.1 Epoch Advertisement

Each team member includes its `primary_epoch` in its regular packets as `epoch36`.

### 21.2 Epoch Drift Tolerance

A packet whose `epoch36` differs from the receiver's `primary_epoch` by more than 2^32 is considered out-of-epoch and is silently discarded before schedule generation.

### 21.3 Epoch Resynchronization

If `EPOCH_CONSENSUS_THRESHOLD` or more valid packets from the same team within the current `ANTI_REPLAY_WINDOW` carry the same epoch value — and that epoch differs from the receiver's current epoch by at most 2^32 — the receiver adopts that epoch:

```txt
primary_epoch          = received_epoch
secondary_epoch_offset = received_epoch XOR INITIAL_PROTOCOL_EPOCH_B
shadow_state_value     = mix64_primary(shadow_state_value, primary_epoch, secondary_epoch_offset, shadow_accumulator)
```

---
