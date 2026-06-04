# fling — Peer-to-Peer File Transfer with Bluetooth Discovery + Encrypted Wi-Fi Tunnels

> Because waiting for OBEX is pain.  
> `fling` lets you discover devices over Bluetooth, then securely transfer(fling) files over fast, encrypted P2P tunnels.  

---

## 🚀 What is fling?

`fling` is a Rust-based CLI tool for Linux that:
- Uses **Bluetooth** to discover nearby devices
- Establishes a direct **Wi-Fi (TCP) connection** between peers
- Lets you transfer **any file, any size** — not limited by Bluetooth speeds
- Works fully peer-to-peer — no servers, no trackers

---
# Technical Architecture

`fling` is a peer-to-peer file transfer system designed around a split control-plane / data-plane architecture.

Bluetooth Low Energy (BLE) is used exclusively as a discovery and coordination channel, while bulk data transfer occurs over a high-throughput encrypted TCP connection established over a direct Wi-Fi link.

## Design Goals

* Zero cloud dependencies
* Local-first architecture
* No centralized coordination service
* Multi-gigabyte file transfers
* Strong transport confidentiality
* Low-latency device discovery
* Cross-platform protocol design
* Minimal user interaction

---

# System Architecture

```text
                   Control Plane
            (Discovery + Coordination)

+------------+                           +------------+
|  Sender    |<------ BLE GATT --------->| Receiver   |
+------------+                           +------------+
       |                                        |
       | Exchange Metadata                      |
       | Negotiate Transfer                     |
       | Exchange Session Parameters            |
       v                                        v

==================================================
                Data Plane Boundary
==================================================

+------------+                           +------------+
|  Sender    |<---- Encrypted TCP -----> | Receiver   |
+------------+                           +------------+

      Compression
            ↓
      Streaming I/O
            ↓
      Chunked Transfer
            ↓
      File Reconstruction
```

The architecture intentionally separates discovery concerns from transport concerns.

BLE provides an efficient mechanism for local peer discovery while Wi-Fi provides the throughput characteristics necessary for transferring large datasets.

---

# Bluetooth Discovery Layer

Device discovery is implemented through bluer and bluest(MacOS) APIs.

The receiver exposes a custom GATT service advertisement which contains:

* Device identifier
* Protocol version
* Transfer capability flags
* Session bootstrap metadata

The sender continuously scans for compatible advertisements and establishes a BLE connection when a matching service UUID is discovered.

BLE is never used for bulk data transfer.

The discovery channel exists solely to:

* locate peers
* exchange bootstrap information
* negotiate connection parameters
* initiate transport setup

This minimizes BLE airtime and avoids bandwidth bottlenecks.

---

# Transport Initialization

Once discovery completes, peers transition into transport establishment.

The receiver provisions a temporary network endpoint and publishes connection metadata through the BLE control channel.

The sender retrieves:

* IP address
* TCP port
* protocol version
* cryptographic parameters

and performs a transport bootstrap sequence.

Connection establishment is modeled as a finite state machine to ensure deterministic transitions between:

```text
Idle
  ↓
Discovering
  ↓
Connecting
  ↓
Negotiating
  ↓
Authenticated
  ↓
Transferring
  ↓
Completed
```

This prevents invalid state transitions and simplifies recovery logic.

---

# Encryption Layer

All payload traffic traverses an encrypted tunnel.

The transport layer is designed around authenticated session establishment before file transmission begins.

Security objectives:

* confidentiality
* integrity
* replay protection
* session isolation

No file data is transmitted until successful cryptographic negotiation has completed.

Session keys are ephemeral and scoped to individual transfers.

Future protocol revisions will support stronger forward secrecy guarantees and key rotation mechanisms.

---

# Compression Pipeline

Large files are compressed prior to transmission to reduce:

* network utilization
* transfer duration
* storage overhead

Compression operates as a streaming stage within the transfer pipeline.

```text
File Input
    ↓
Compression
    ↓
Chunk Generation
    ↓
Encryption
    ↓
TCP Stream
```

The pipeline is designed to avoid unnecessary intermediate disk writes wherever possible.

Compression and network transmission can execute concurrently, enabling overlap between CPU-bound and network-bound workloads.

---

# Platform Abstraction

The networking stack is designed around platform-specific adapters.

Linux implementation currently uses:

* Bluer
* Native networking interfaces

The long-term goal is to support:

* Linux
* macOS
* Windows

while preserving a consistent protocol layer.

Platform-specific functionality is isolated behind trait-based abstractions to minimize OS-dependent logic leakage into higher protocol layers.

---

# Performance Characteristics

Current benchmark:

Dataset:
Gitlab CE source archive

Original Size:
3.81 GB

Compressed Size:
1.6 GB

Observed Throughput:
200+ Mbps

Transfer Mode:
Direct encrypted peer-to-peer connection

The architecture is optimized around minimizing:

* network round trips
* memory copies
* protocol overhead
* unnecessary disk I/O

Future work includes:

* parallel transfer streams
* resumable transfers
* adaptive chunk sizing
* transport-level congestion awareness
* zero-copy buffer optimizations
* SIMD-accelerated compression paths

---

# Engineering Challenges

Some of the most interesting technical challenges encountered during development:

* Designing a reliable BLE-to-WiFi bootstrap workflow
* Managing asynchronous state transitions across multiple transport layers
* Balancing compression efficiency against latency
* Building a transport protocol capable of handling multi-gigabyte transfers
* Abstracting OS-specific networking functionality behind portable interfaces
* Maintaining predictable behavior under network interruptions
* Coordinating multiple concurrent subsystems without introducing blocking execution paths

The project serves as an exploration of systems programming, networking, protocol design, asynchronous Rust, and peer-to-peer communication architectures.


---
⚠️ macOS Support Status

Currently, sending from macOS is not supported due to Apple’s platform restrictions:

    No unrestricted GATT advertising — required for fling to announce itself as discoverable over Bluetooth.

    No programmatic P2P Wi-Fi hotspot creation — needed for automatic encrypted tunnel setup between peers.

These APIs are locked behind Apple’s CoreBluetooth / NEHotspotConfiguration entitlements, which are only available with a $99/year Apple Developer Program account and a signed, notarized app.

💡 Receiving on macOS works fine! — fling can detect nearby Linux senders and receive over the encrypted Wi-Fi tunnel.

Sending from MacOS is currently Work In Progress. If I get a developer account with apple, I can implement the entire logic in Swift, using CoreBluetooth's APIs to have even faster transitions. 

I am also planning of writing a Menu Bar GUI Icon for MacOS which just stays on the Menu Bar and can be clicked in one go to invoke fling.

---
## 💻 Authors

- Ishan Kumar (https://github.com/4ddx)
- Akhil Jose (https://github.com/AkZuza)

Special thanks to the OSS community.
---

## 📰 Follow our updates:

- Star the repo to stay updated :D

