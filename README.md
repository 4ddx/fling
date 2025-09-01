# fling — Peer-to-Peer File Transfer with Bluetooth Discovery + Encrypted Wi-Fi Tunnels

> Because waiting for OBEX is pain.  
> `fling` lets you discover devices over Bluetooth, then securely transfer(fling) files over fast, encrypted P2P tunnels.  

---

## 🚀 What is fling?

`fling` is a Rust-based CLI tool for Linux that:
- Uses **Bluetooth** to discover nearby devices
- Establishes a direct **Wi-Fi (TCP) connection** between peers
- Secures all traffic with **Encryption**
- Lets you transfer **any file, any size** — not limited by Bluetooth speeds
- Works fully peer-to-peer — no servers, no trackers

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

## 🗺️ Roadmap

- [x] Project initialization + FSM design
- [x] Bluetooth discovery with BlueZ D-Bus
- [x] TCP tunnel with encrypted handshake
- [x] CLI interface for send/receive modes
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

Special thanks to the OSS community.

---

## 📰 Follow our updates:

- Star the repo to stay updated :D

