# rumqtt-fido-client

This repository contains a Rumqtt Client application, that authenticates itself using a FIDO2 token against a compatible MQTT broker.
For this client to work, the broker must support the TLS FIDO extension. A capable rumqtt daemon can be found [here](https://github.com/7ritn/rumqtt).
This client only supports authentication, but no registration. To register your token please use the helper register application found at [rustls-fido](https://github.com/7ritn/rustls-fido).

This client can be configured using the following environment variables:

| Variable Name            | Description                                   | Default Value                 | Example Value                     |
|--------------------------|-----------------------------------------------|-------------------------------|-----------------------------------|
| `CA_CERT_PATH`           | Path to the CA certificate file               | `./tls-certs/ca.cert.pem`     | `/etc/ssl/certs/ca.cert.pem`      |
| `CLIENT_CERT_PATH`       | Path to the client certificate file           | `./tls-certs/client.cert.pem` | `/etc/ssl/certs/client.cert.pem`  |
| `CLIENT_KEY_PATH`        | Path to the client private key file           | `./tls-certs/client.key.pem`  | `/etc/ssl/private/client.key.pem` |
| `SERVER_HOST`            | Address of the server to connect to           | `localhost`                   | `example.com`                     |
| `SERVER_PORT`            | Port of the server to connect to              | `1883`                        | `2222`                            |
| `FIDO_DEVICE_PIN`        | FIDO PIN for authentication                   | `1234`                        | `5678`                            |

Running the client is simple with `cargo run`

## Example for a complete setup
```bash
mkdir rumqtt-demo
cd rumqtt-demo

git clone https://github.com/7ritn/rustls-fido.git
git clone https://github.com/7ritn/rustls.git
git clone https://github.com/7ritn/rumqtt.git
git clone https://github.com/7ritn/rumqtt-fido-client.git

export FIDO_DB_PATH="$(pwd)/fido.db3"
export CA_CERT_PATH="$(pwd)/rustls/tls-certs/ca.cert.pem"
export CLIENT_CERT_PATH="$(pwd)/rustls/tls-certs/client.cert.pem"
export CLIENT_KEY_PATH="$(pwd)/rustls/tls-certs/client.key.pem"
export FIDO_DEVICE_PIN="1234"

cd rustls-fido
cargo run --features build-binary --bin register

cd ..
cd rumqtt
cargo build --package rumqttd --bin rumqttd --config rumqttd/rumqttd.toml
cargo run --package rumqttd --bin rumqttd --config rumqttd/rumqttd.toml &

cd ..
cd rumqtt-fido-client
cargo run
```
