#!/bin/bash
# Simple BLE advertiser using hcitool
# Advertises the poem as manufacturer data in the BLE advertisement

POEM="Do you see it too? A signal in the darkness, waiting to be found."
DEVICE="hci0"

# Convert poem to hex
POEM_HEX=$(echo -n "$POEM" | xxd -p)

# Enable advertising
hcitool -i $DEVICE cmd 0x08 0x0008 1e 02 01 06 1a ff 4c 00 $(echo -n "$POEM_HEX" | fold -w2 | tr '\n' ' ')

echo "✓ Broadcasting poem as manufacturer data..."
echo "  Poem: $POEM"
