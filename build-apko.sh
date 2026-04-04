#!/usr/bin/env bash
set -euo pipefail

IMAGE_NAME="md-exporter:apko"
IMAGE_TAR="image.tar"
PACKAGES_DIR="packages"
KEY="melange.rsa"

echo "==> Generating signing keypair..."
melange keygen "$KEY"

echo "==> Building md-export apk with Melange..."
melange build melange.yaml \
  --signing-key "$KEY" \
  --out-dir "$PACKAGES_DIR" \
  --arch host \
  --source-dir .

echo "==> Building OCI image with apko..."
apko build apko.yaml "$IMAGE_NAME" "$IMAGE_TAR" \
  --keyring-append "${KEY}.pub" \
  --repository-append "$PACKAGES_DIR" \
  --arch arm64

echo "==> Loading image into Docker..."
docker load < "$IMAGE_TAR"

echo ""
echo "Done! Run with:"
echo "  docker run --rm -p 8080:8080 $IMAGE_NAME"
