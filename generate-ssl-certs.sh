#!/bin/bash
# Generate self-signed SSL certificates for development

mkdir -p nginx/ssl

# Generate private key
openssl genrsa -out nginx/ssl/key.pem 4096

# Generate certificate signing request
openssl req -new -key nginx/ssl/key.pem -out nginx/ssl/cert.csr -subj "/C=US/ST=CA/L=SF/O=TerragonLabs/CN=localhost"

# Generate self-signed certificate
openssl x509 -req -days 365 -in nginx/ssl/cert.csr -signkey nginx/ssl/key.pem -out nginx/ssl/cert.pem

# Set appropriate permissions
chmod 600 nginx/ssl/key.pem
chmod 644 nginx/ssl/cert.pem

# Clean up CSR file
rm nginx/ssl/cert.csr

echo "SSL certificates generated in nginx/ssl/"
echo "Key: nginx/ssl/key.pem"
echo "Certificate: nginx/ssl/cert.pem"