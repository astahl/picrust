#!/bin/sh

cargo imgpi4_64
cargo imgpi3_64


cp -v kernel*.img /Volumes/bootfs  
cp -v config.txt /Volumes/bootfs  

diskutil unmount /Volumes/bootfs