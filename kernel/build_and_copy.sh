#!/bin/sh

cargo clean
cargo img3
cargo img4


cp -v out/kernel*.img /Volumes/bootfs  
cp -v img/* /Volumes/bootfs  

diskutil unmount /Volumes/bootfs