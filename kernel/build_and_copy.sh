#!/bin/sh

cargo img3
cargo img4


cp -v out/kernel*.img /Volumes/bootfs  
#cp -v config.txt /Volumes/bootfs  

diskutil unmount /Volumes/bootfs