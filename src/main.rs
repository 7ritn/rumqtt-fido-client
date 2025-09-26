use std::fs::File;
use std::io::BufReader;
use std::sync::Arc;
use rumqttc::{MqttOptions, AsyncClient, QoS, Transport, TlsConfiguration};
use tokio::{task, time};
use std::time::Duration;
use rumqttc::tokio_rustls::rustls::{ClientConfig, RootCertStore};
use rumqttc::tokio_rustls::rustls::pki_types::{CertificateDer, PrivateKeyDer};
use rumqttc::tokio_rustls::rustls::pki_types::pem::PemObject;
use rumqttc::tokio_rustls::rustls::rustls_fido::enums::FidoMode;
use rumqttc::tokio_rustls::rustls::rustls_fido::client::FidoClient;
use rustls_pemfile::certs;

macro_rules! env_var_or_default {
    ($name:expr, $default:expr) => {
        std::env::var($name).unwrap_or_else(|_| $default.to_string())
    };
}

#[tokio::main]
async fn main() {
    // Read environment variables
    let ca_cert_path = env_var_or_default!("CA_CERT_PATH", "./tls-certs/ca.cert.pem");
    let client_cert_path = env_var_or_default!("CLIENT_CERT_PATH", "./tls-certs/client.cert.pem");
    let client_key_path = env_var_or_default!("CLIENT_KEY_PATH", "./tls-certs/client.key.pem");
    let host = env_var_or_default!("SERVER_HOST", "localhost");
    let port = env_var_or_default!("SERVER_PORT", "1883").parse().unwrap();
    let fido_pin = env_var_or_default!("FIDO_DEVICE_PIN", "1234");

    let cert_file = File::open(ca_cert_path).expect("cannot open cert file");
    let mut reader = BufReader::new(cert_file);

    // Parse the certificate(s)
    let certs = certs(&mut reader)
        .collect::<Result<Vec<_>, _>>()
        .unwrap();

    // Create a root store and add the certs
    let mut root_store = RootCertStore::empty();
    for cert in certs {
        root_store
            .add(cert)
            .expect("failed to add cert to root store");
    }

    let client_cert = CertificateDer::pem_file_iter(client_cert_path)
        .unwrap()
        .map(Result::unwrap)
        .collect();
    let client_key = PrivateKeyDer::from_pem_file(client_key_path).unwrap();
    
    let fido = FidoClient::new(
        FidoMode::Authentication,
        None,
        None,
        None,
        fido_pin
    );

    let config = ClientConfig::builder()
        .with_root_certificates(root_store)
        .with_client_auth_fido(client_cert, client_key, fido)
        .unwrap();

    let tls_config = TlsConfiguration::Rustls(Arc::new(config));
    let transport = Transport::tls_with_config(tls_config);

    let mut mqttoptions = MqttOptions::new("rumqtt-async", host, port);
    mqttoptions.set_transport(transport);
    mqttoptions.set_keep_alive(Duration::from_secs(5));

    let (client, mut eventloop) = AsyncClient::new(mqttoptions, 10);
    client.subscribe("hello/rumqtt", QoS::AtMostOnce).await.unwrap();

    task::spawn(async move {
        for i in 0..10 {
            client.publish("hello/rumqtt", QoS::AtLeastOnce, false, vec![i; i as usize]).await.unwrap();
            time::sleep(Duration::from_millis(100)).await;
        }
    });

    while let Ok(notification) = eventloop.poll().await {
        println!("Received = {:?}", notification);
    }
}
