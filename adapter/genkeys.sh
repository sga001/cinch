#!/bin/bash

# generate all the keys
for i in ca client server; do
    openssl genrsa -out ${i}_key.pem 2048
done

# CA key is self-signed
openssl req -x509 -new -sha256 -nodes -key ca_key.pem -days 3650 -out ca_cert.pem -subj "/C=US/ST=NY/O=cinch/CN=cinch_ca"

# client and server signing requests
for i in client server; do
    openssl req -new -sha256 -key ${i}_key.pem -out ${i}_csr.pem -subj "/C=US/ST=NY/O=cinch/CN=cinch_"${i}
done

# sign the keys
for i in client server; do
    openssl x509 -req -in ${i}_csr.pem -CA ca_cert.pem -CAkey ca_key.pem -CAcreateserial -out ${i}_cert.pem -days 3650
done

# get rid of everything we don't need
rm ca_key.pem
rm *_csr.pem
rm ca_cert.srl
