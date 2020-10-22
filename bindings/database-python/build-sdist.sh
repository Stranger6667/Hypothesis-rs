#!/bin/bash
set -ex

# Create a symlink for database
ln -sf ../../crates/database database
# Modify Cargo.toml to include this symlink
cp Cargo.toml Cargo.toml.orig
sed -i 's/\.\.\/\.\.\/\.\./crates/database/\.\/database/' Cargo.toml
# Build the source distribution
python setup.py sdist
# Undo changes
rm database
mv Cargo.toml.orig Cargo.toml