# Node handshake for NEAR blockchain

#### According to NEAR protocol first message after establishing a connection to a node is PeerMessage::Handshake.

#### Only after a successful handshake node starts receiving another PeerMessages, so we can be assured that the handshake is concluded.

#### Another way to verify that is to send PeerMessage::Ping command and receive PeerMessage::Pong from a target node.

#### Node supports bidirectional handshake, so you can run one instance of the app and connect to it using another instance of the app with `--target-peer-info` taken from running node output and specified `--sender-listen-port`

---

## Testnet nodes

https://rpc.testnet.near.org/network_info

--- 

## Mainnet nodes

https://rpc.near.org/network_info

---

## Required arguments

> --network=localnet|testnet|mainnet

> --target-peer-info=ed25519:Kmpx1xn2mtLPchDPLyTr9sgyf4HFfdeKFfKwqw8HJC4@35.233.240.34:24567

---

## Optional arguments with default values

> --sender-listen-port=34567

> --protocol-version=63

> --oldest-supported-version=61

---