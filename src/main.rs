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

#[tokio::main]
async fn main() {
    env_logger::init();
    let cert_file = File::open("./tls-certs/ca.cert.pem").expect("cannot open cert file");
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

    let client_cert = CertificateDer::pem_file_iter("./tls-certs/client.cert.pem")
        .unwrap()
        .map(Result::unwrap)
        .collect();
    let client_key = PrivateKeyDer::from_pem_file("./tls-certs/client.key.pem").unwrap();
    
    let fido = FidoClient::new(
        FidoMode::Authentication,
        None,
        None,
        None,
        "1234".to_string()
    );

    let config = ClientConfig::builder()
        .with_root_certificates(root_store)
        .with_client_auth_fido(client_cert, client_key, fido)
        .unwrap();

    let tls_config = TlsConfiguration::Rustls(Arc::new(config));
    let transport = Transport::tls_with_config(tls_config);

    let mut mqttoptions = MqttOptions::new("rumqtt-async", "localhost", 1883);
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
