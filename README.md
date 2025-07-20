# fling — Peer-to-Peer File Transfer with Bluetooth Discovery + Encrypted Wi-Fi Tunnels

> Because waiting for OBEX is pain.  
> `fling` lets you discover devices over Bluetooth, then securely transfer(fling) files over fast, encrypted P2P tunnels.  

---

## 🚀 What is fling?

`fling` is a Rust-based CLI tool for Linux that:
- Uses **Bluetooth** to discover nearby devices
- Establishes a direct **Wi-Fi (TCP) connection** between peers
- Secures all traffic with **Noise Protocol + ChaCha20-Poly1305 encryption**
- Lets you transfer **any file, any size** — not limited by Bluetooth speeds
- Works fully peer-to-peer — no servers, no trackers

---

## 📦 Project Goals

- ✅ Clean CLI experience: `fling send file.txt --to laptop`
- ✅ Asynchronous Rust — minimal latency
- ✅ Fully documented + MIT/Apache licensed
- ✅ Zero external servers, fully peer-controlled
- ✅ Memorable branding

---

## 🗺️ Roadmap

- [x] Project initialization + FSM design
- [ ] Bluetooth discovery with BlueZ D-Bus
- [ ] TCP tunnel with encrypted handshake
- [ ] Encrypted file transfer logic (chunked streaming)
- [ ] CLI interface for send/receive modes
- [ ] Add terminal ASCII splash after transfers
- [ ] Docs, manpage, and packaging for AUR
- [ ] v1.0 OSS release + Hacker News launch 🚀

---

## 💬 Why fling exists

> Because Linux doesn’t have a good, fast, secure AirDrop alternative.   
> Because hacking protocols, wrapping Bluetooth, and doing a P2P transfer from your terminal is cool.  

---

## 💻 Authors

- Ishan Kumar (https://github.com/4ddx)
- Akhil Jose (https://github.com/AkZuza)

Special thanks to the OSS community — we stand on the shoulders of giants.

---

## 📰 Follow our updates:

- Star the repo to stay updated :D

