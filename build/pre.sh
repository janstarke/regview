#!/bin/bash

echo "updating package metadata"
echo "============================================"
if [ -f /etc/alpine-release ]; then
    apk update
else 
    apt-get update
fi
echo "============================================"

echo "installing missing packages"
if [ -f /etc/alpine-release ]; then
    apk add autoconf automake libtool bison
else
    # do nothing
fi
echo "============================================"

NCURSES_PREFIX=/opt/ncurses

TARGET=x86_64-unknown-linux-musl
export CC=musl-gcc
export LD=musl-gcc
curl https://ftp.gnu.org/pub/gnu/ncurses/ncurses-6.3.tar.gz --output - | tar xz
pushd ncurses-6.3
./configure --prefix=$NCURSES_PREFIX --enable-static --host=$TARGET --target=$TARGET --without-debug --enable-widec --without-ada --with-shared --with-normal && make && make install
popd

pushd $NCURSES_PREFIX/lib
ln -s libncurses.a libcurses.a
popd

echo $NCURSES_PREFIX >> /etc/ld-musl-x86_64.d/x86_64-linux-musl.path
ld-musl-config
export CFLAGS="-I$NCURSES_PREFIX/include -I$NCURSES_PREFIX/include/ncursesw"
export LDFLAGS="-L$NCURSES_PREFIX/lib"
for F in $NCURSES_PREFIX/lib/*; do ln -s $F /usr/lib/x86_64-linux-musl/; done
