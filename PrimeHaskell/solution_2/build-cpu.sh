#!/bin/bash

# get cpu list with 
#   llc --march=x86-64 --mcpu=help

ARCH="x86-64"
CPU="$1"
echo "CPU is $CPU"

rm -rf target tmp
mkdir -p target tmp
ghc -v -o target/Primes -outputdir target \
    -keep-tmp-files -tmpdir ./tmp \
    -O2 -fllvm -v \
    -optlc -O3 -optlc --march=$ARCH -optlc --mcpu=$CPU \
    -optlo -O3 -optlo --march=$ARCH -optlo --mcpu=$CPU \
    Primes.hs
