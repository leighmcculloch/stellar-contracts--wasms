#!/bin/bash
# Mock stellar-core script for testing XDR processing
# This script outputs some sample base64-encoded data to simulate stellar-core output

echo "Mock stellar-core starting with metadata mode..."
echo "Simulating XDR Frame<LedgerCloseMeta> output..."

# Output some mock base64 data (this won't be valid XDR, but will test the base64 decoding)
sleep 1
echo "dGVzdCBkYXRh"  # base64 for "test data"
sleep 1
echo "YW5vdGhlciB0ZXN0"  # base64 for "another test"
sleep 1
echo "bGFzdCB0ZXN0"  # base64 for "last test"

echo "Mock stellar-core finished"